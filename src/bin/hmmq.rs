use chrono::{DateTime, Local};
use colored::*;
use hmm::{bsearch::seek_first, config::Config, Result};
use std::fs::File;
use std::io::{stderr, BufRead, BufReader, Write};
use std::process::exit;
use structopt::StructOpt;
use std::cmp::Ordering;

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

    if let Some(ref prefix) = opt.start {
        if seek_first(&mut f, prefix)?.is_none() {
            return Ok(());
        }
    }

    loop {
        buf.clear();
        f.read_line(&mut buf)?;

        if let Some(ref end) = opt.end {
            if let Ordering::Greater = buf.as_bytes().cmp(end.as_bytes()) {
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
