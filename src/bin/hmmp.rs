use hmmcli::{entry::Entry, format::Format, Result};
use std::convert::TryInto;
use std::io::{stdin, BufRead};
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hmmp", about = "Pipe hmm entries to this binary to format them")]
struct Opt {
    /// How to format entry output. hmm uses Handlebars as a template format, see
    /// https://handlebarsjs.com/guide/ for information on how to use them. The
    /// values "datetime" and "message" are passed in.
    #[structopt(
        long = "format",
        default_value = "{{ color \"blue\" (strftime \"%Y-%m-%d %H:%M:%S\" datetime) }}\n{{ indent message }}"
    )]
    format: String,
}

fn main() {
    if let Err(e) = app(Opt::from_args(), stdin().lock()) {
        eprintln!("{}", e);
        exit(1);
    }
}

fn app(opt: Opt, stdin: impl BufRead) -> Result<()> {
    let mut formatter = Format::with_template(&opt.format)?;

    for line in stdin.lines() {
        let entry: Entry = line?.try_into()?;
        println!("{}", formatter.format_entry(&entry)?);
    }

    Ok(())
}
