use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use hmm::{
    bsearch::{seek, seek_start_of_current_line, seek_start_of_prev_line, SeekType},
    entry::Entry,
    error::Error,
    format::Format,
    Result,
};
use rand::distributions::{Distribution, Uniform};
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
    /// Path to your hmm file, defaults to your default configuration directory,
    /// ~/.config on *nix systems, %APPDATA% on Windows.
    #[structopt(long = "path")]
    path: Option<PathBuf>,

    /// How to format entry output. hmm uses Handlebars as a template format, see
    /// https://handlebarsjs.com/guide/ for information on how to use them. The values
    /// "datetime" and "message" are passed in.
    #[structopt(
        long = "format",
        default_value = "{{ color \"blue\" (strftime datetime \"%Y-%m-%d %H:%M:%S\") }}\n{{ indent message }}\n"
    )]
    format: String,

    /// By default, entries are printed in ascending chronological order. This
    /// flag prints in reverse chronological order.
    #[structopt(long = "descending")]
    descending: bool,

    /// Print a random entry. Specifying this flag means the other flags will be
    /// ignored.
    #[structopt(long = "random")]
    random: bool,

    /// The number of entries to print. If a start and end date have been specified,
    /// this will print the first N of that range. In ascending order, this is the first
    /// N entries chronologically, and in descending order it will be the last N entries.
    #[structopt(short = "n")]
    num_entries: Option<usize>,

    /// Date to start printing from, inclusive. The date will be read in your
    /// local time, and can be specified using any subset of an RFC3339 date,
    /// e.g. 2012, 2012-01, 2012-01-29, 2012-01-29T14, 2012-01-29T14:30,
    /// 2012-01-29T14:30:11.
    #[structopt(short = "s", long = "start")]
    start: Option<String>,

    /// Date to stop printing at, exclusive. Like --start, this can be any subset of an
    /// RFC3339 date. See --start for details.
    #[structopt(short = "e", long = "end")]
    end: Option<String>,

    /// Only print entries that contain this substring exactly.
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
    let formatter = Format::with_template(&opt.format)?;
    let path = opt.path.unwrap_or_else(|| dirs::home_dir().unwrap().join(".hmm"));

    if opt.random {
        print_random_entry(&path, &formatter)?;
        return Ok(());
    }

    let mut f = BufReader::new(File::open(&path)?);
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

            if let Some(n) = opt.num_entries {
                count += 1;
                if count > n {
                    break;
                }
            }

            println!("{}", formatter.format_entry(&entry)?);

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

            if let Some(n) = opt.num_entries {
                count += 1;
                if count > n {
                    break;
                }
            }

            println!("{}", formatter.format_entry(&entry)?);
        }
    }

    Ok(())
}

fn print_random_entry(path: &PathBuf, formatter: &Format) -> Result<()> {
    let mut f = File::open(path)?;

    let mut reader_builder = csv::ReaderBuilder::new();
    reader_builder.has_headers(false);

    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, f.metadata()?.len());
    f.seek(SeekFrom::Start(range.sample(&mut rng)))?;
    seek_start_of_current_line(&mut f)?;

    let mut buf = String::new();
    let mut br = BufReader::new(f);
    br.read_line(&mut buf)?;

    let mut r = reader_builder.from_reader(buf.as_bytes());
    let mut record = csv::StringRecord::new();
    if !r.read_record(&mut record)? {
        return Err(Error::StringError(format!(
            "failed to parse \"{}\" as CSV row",
            buf
        )));
    }

    let entry: Entry = (&record).try_into()?;
    println!("{}", formatter.format_entry(&entry)?);
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
