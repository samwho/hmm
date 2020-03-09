use chrono::{DateTime, Local};
use colored::*;
use easy_reader::EasyReader;
use hmm::{config::Config, Result};
use std::fs::File;
use std::io::{stderr, BufReader, Read, Write};
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hmmq", about = "Query your hmm file")]
struct Opt {
    #[structopt(short = "n", default_value = "10")]
    num_entries: usize,
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

    let mut r = EasyReader::new(File::open(config.path()?)?)?;
    r.eof();

    for _ in 0..=opt.num_entries {
        if let None = r.prev_line()? {
            // Without this the first call to next_line will return the second line.
            r.bof();
            break;
        }
    }

    let mut record = csv::StringRecord::new();

    while let Ok(Some(line)) = r.next_line() {
        if line.is_empty() {
            continue;
        }

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(line.as_bytes());

        if !rdr.read_record(&mut record)? {
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
