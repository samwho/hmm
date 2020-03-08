use hmm::{config, Result};
use colored::*;
use std::io::{stderr, BufReader, Write, Read};
use std::fs::File;
use std::process::{exit};

fn main() {
    if let Err(e) = app() {
        if let Err(write_e) = writeln!(&mut stderr(), "{}", e) {
            panic!(write_e);
        }
        exit(1);
    }
}

fn app() -> Result<()> {
    let config = config::get()?;
    print_entries(BufReader::new(File::open(config.path)?))?;
    Ok(())
}

fn print_entries(r: impl Read) -> Result<()> {
    for record in csv::Reader::from_reader(r).into_records() {
        match record {
            Ok(e) => print_entry(e)?,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

fn print_entry(sr: csv::StringRecord) -> Result<()> {
    let date = sr.get(0).unwrap();
    let msg = sr.get(1).unwrap();

    let datetime = chrono::DateTime::parse_from_rfc3339(date)?;

    let wrapper = textwrap::Wrapper::with_termwidth()
        .initial_indent("| ")
        .subsequent_indent("| ");

    println!("{}", datetime.format("%Y-%m-%d %H:%M").to_string().blue());
    println!("{}\n", wrapper.fill(msg));
    Ok(())
}