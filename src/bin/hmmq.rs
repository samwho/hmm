use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use colored::*;
use hmm::{bsearch::seek_first, config::Config, error::Error, Result};
use std::cmp::Ordering;
use std::fs::File;
use std::io::{stderr, BufRead, BufReader, Write};
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hmmq", about = "Query your hmm file")]
struct Opt {
    #[structopt(short = "n", default_value = "10")]
    num_entries: usize,

    #[structopt(short = "s", long = "start")]
    start: Option<String>,

    #[structopt(short = "e", long = "end")]
    end: Option<String>,
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
    let config = Config::default();

    let mut f = BufReader::new(File::open(config.path()?)?);
    let mut record = csv::StringRecord::new();
    let mut buf = String::new();

    let mut reader_builder = csv::ReaderBuilder::new();
    reader_builder.has_headers(false);

    let ed = match opt.end {
        Some(ref end) => parse_date_arg(end)?.to_rfc3339(),
        None => "".to_owned(),
    };

    if let Some(ref start) = opt.start {
        let sd = parse_date_arg(start)?;
        if seek_first(&mut f, &sd.to_rfc3339())?.is_none() {
            return Ok(());
        }
    }

    loop {
        buf.clear();
        f.read_line(&mut buf)?;

        if opt.end.is_some() {
            if let Ordering::Greater = buf.as_bytes().cmp(ed.as_bytes()) {
                break;
            }
        }

        let mut r = reader_builder.from_reader(buf.as_bytes());
        if !r.read_record(&mut record)? {
            break;
        }

        print_entry(&config, &record)?;
    }

    Ok(())
}

fn print_entry(config: &Config, sr: &csv::StringRecord) -> Result<()> {
    let date = sr.get(0).unwrap();
    let msg = sr.get(1).unwrap();

    let datetime: DateTime<Local> = DateTime::from(chrono::DateTime::parse_from_rfc3339(date)?);

    let wrapper = textwrap::Wrapper::with_termwidth()
        .initial_indent("| ")
        .subsequent_indent("| ");

    println!(
        "{}",
        datetime.format(&config.date_format()).to_string().blue()
    );
    let decoded: String = serde_json::from_str(&msg)?;
    println!("{}\n", wrapper.fill(&decoded));
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
