use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use colored::*;
use hmm::{
    bsearch::{seek, seek_start_of_prev_line, SeekType},
    config::Config,
    entry::Entry,
    error::Error,
    Result,
};
use std::cmp::Ordering;
use std::convert::TryInto;
use std::fs::File;
use std::io::{stderr, BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hmmq", about = "Query your hmm file")]
struct Opt {
    #[structopt(short = "c", long = "config")]
    config: Option<PathBuf>,

    #[structopt(long = "descending")]
    descending: bool,

    #[structopt(long = "random")]
    random: bool,

    #[structopt(short = "n")]
    num_entries: Option<usize>,

    #[structopt(short = "s", long = "start")]
    start: Option<String>,

    #[structopt(short = "e", long = "end")]
    end: Option<String>,

    #[structopt(long = "contains")]
    contains: Option<String>,
}

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = app(opt) {
        if let Err(write_e) = writeln!(&mut stderr(), "{}", e) {
            panic!(write_e);
        }
        exit(1);
    }
}

fn app(opt: Opt) -> Result<()> {
    let config = opt
        .config
        .map(|c| Config::read_from(&c))
        .unwrap_or_else(Config::read)?;

    let mut f = BufReader::new(File::open(config.path()?)?);
    let mut record = csv::StringRecord::new();
    let mut buf = String::new();

    let mut reader_builder = csv::ReaderBuilder::new();
    reader_builder.has_headers(false);

    let ed = match opt.end {
        Some(ref end) => parse_date_arg(end)?.to_rfc3339(),
        None => "".to_owned(),
    };

    let sd = match opt.start {
        Some(ref start) => parse_date_arg(start)?.to_rfc3339(),
        None => "".to_owned(),
    };

    let mut count = 0;

    if opt.descending {
        // print in descending order
        if opt.end.is_some() {
            if seek(&mut f, &ed, SeekType::Last)?.is_none() {
                return Ok(());
            }
        } else {
            f.seek(SeekFrom::End(0))?;
            seek_start_of_prev_line(&mut f)?;
        }

        loop {
            if let Some(n) = opt.num_entries {
                count += 1;
                if count > n {
                    break;
                }
            }

            buf.clear();
            f.read_line(&mut buf)?;

            if opt.start.is_some() {
                match buf.as_bytes().cmp(sd.as_bytes()) {
                    Ordering::Equal | Ordering::Less => break,
                    _ => (),
                }
            }

            let mut r = reader_builder.from_reader(buf.as_bytes());
            if !r.read_record(&mut record)? {
                break;
            }

            let entry: Entry = (&record).try_into()?;

            if let Some(ref contains) = opt.contains {
                if !entry.message().contains(contains) {
                    continue;
                }
            }

            print_entry(&config, &entry)?;

            seek_start_of_prev_line(&mut f)?;

            if seek_start_of_prev_line(&mut f)?.is_none() {
                break;
            }
        }
    } else {
        // print in ascending order
        if seek(&mut f, &sd, SeekType::First)?.is_none() {
            return Ok(());
        }

        loop {
            if let Some(n) = opt.num_entries {
                count += 1;
                if count > n {
                    break;
                }
            }

            buf.clear();
            f.read_line(&mut buf)?;

            if opt.end.is_some() {
                match buf.as_bytes().cmp(ed.as_bytes()) {
                    Ordering::Equal | Ordering::Greater => break,
                    _ => (),
                }
            }

            let mut r = reader_builder.from_reader(buf.as_bytes());
            if !r.read_record(&mut record)? {
                break;
            }

            let entry: Entry = (&record).try_into()?;

            if let Some(ref contains) = opt.contains {
                if !entry.message().contains(contains) {
                    continue;
                }
            }

            print_entry(&config, &entry)?;
        }
    }

    Ok(())
}

fn print_entry(config: &Config, entry: &Entry) -> Result<()> {
    let wrapper = textwrap::Wrapper::with_termwidth()
        .initial_indent("| ")
        .subsequent_indent("| ");

    println!(
        "{}",
        entry
            .datetime()
            .with_timezone(&Local)
            .format(&config.date_format())
            .to_string()
            .blue()
    );
    println!("{}\n", wrapper.fill(entry.message()));
    Ok(())
}

fn parse_date_arg(s: &str) -> Result<DateTime<Utc>> {
    if let Ok(d) = parse_local_datetime_str(&format!("{}-01-01T00:00:00", s), "%Y-%m-%dT%H:%M:%S") {
        return Ok(d);
    }
    if let Ok(d) = parse_local_datetime_str(&format!("{}-01T00:00:00", s), "%Y-%m-%dT%H:%M:%S") {
        return Ok(d);
    }
    if let Ok(d) = parse_local_datetime_str(&format!("{}T00:00:00", s), "%Y-%m-%dT%H:%M:%S") {
        return Ok(d);
    }
    if let Ok(d) = parse_local_datetime_str(&format!("{}:00:00", s), "%Y-%m-%dT%H:%M:%S") {
        return Ok(d);
    }
    if let Ok(d) = parse_local_datetime_str(&format!("{}:00", s), "%Y-%m-%dT%H:%M:%S") {
        return Ok(d);
    }
    if let Ok(d) = parse_local_datetime_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(d);
    }

    Err(Error::StringError(format!("unrecognised date format: \"{}\", accepted formats include things like:\n  - 2012\n  - 2012-01\n  - 2012-01-24\n  - 2012-01-24T16\n  - 2012-01-24T16:20\n  - 2012-01-24T16:20:30", s)))
}

fn parse_local_datetime_str(s: &str, format: &str) -> Result<DateTime<Utc>> {
    let d = NaiveDateTime::parse_from_str(s, format)?;
    let local_result = Utc.from_local_datetime(&d);
    Ok(local_result.earliest().unwrap_or_else(|| {
        local_result
            .latest()
            .unwrap_or_else(|| local_result.unwrap())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("2012"                => "2012-01-01T00:00:00+00:00" ; "y")]
    #[test_case("2012-02"             => "2012-02-01T00:00:00+00:00" ; "ym")]
    #[test_case("2012-02-02"          => "2012-02-02T00:00:00+00:00" ; "ymd")]
    #[test_case("2012-02-02T02"       => "2012-02-02T02:00:00+00:00" ; "ymdh")]
    #[test_case("2012-02-02T02:02"    => "2012-02-02T02:02:00+00:00" ; "ymdhm")]
    #[test_case("2012-02-02T02:02:02" => "2012-02-02T02:02:02+00:00" ; "ymdhms")]
    fn test_parse_date_arg(s: &str) -> String {
        parse_date_arg(s).unwrap().to_rfc3339()
    }
}
