use std::env::args;
use std::error::Error;
use std::fs::OpenOptions;
use std::result::Result;
use std::io::{Read, Write, BufReader, BufWriter};

use colored::*;

fn main() -> Result<(), Box<dyn Error>> {
    let arg = itertools::join(args().skip(1), " ");
    let home = dirs::home_dir().unwrap();
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(home.join(".hmm"))?;

    if arg.is_empty() {
        print_entries(BufReader::new(f))?;
    } else {
        write_entry(BufWriter::new(f), arg)?;
    }

    Ok(())
}

fn write_entry(w: impl Write, msg: String) -> Result<(), Box<dyn Error>> {
    let now = chrono::Utc::now();
    let mut writer = csv::Writer::from_writer(w);
    Ok(writer.write_record(&[now.to_rfc3339(), msg])?)
}

fn print_entries(r: impl Read) -> Result<(), Box<dyn Error>> {
    for record in csv::Reader::from_reader(r).into_records() {
        match record {
            Ok(e) => print_entry(e)?,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

fn print_entry(sr: csv::StringRecord) -> Result<(), Box<dyn Error>> {
    let date = sr.get(0).unwrap();
    let msg = sr.get(1).unwrap();

    let datetime = chrono::DateTime::parse_from_rfc3339(date)?;

    let wrapper = textwrap::Wrapper::with_termwidth().initial_indent("| ").subsequent_indent("| ");

    println!("{}", datetime.format("%Y-%m-%d %H:%M").to_string().blue());
    println!("{}\n", wrapper.fill(msg));
    Ok(())
}