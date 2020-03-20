
use chrono::{prelude::*, Duration};
use hmmcli::{entry::Entry, Result};
use std::io::{BufWriter};
use std::path::PathBuf;
use std::process::{exit};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hmm", about = "Command line note taking")]
struct Opt {
    #[structopt(long = "path")]
    path: PathBuf,

    #[structopt(long = "entries-per-day", default_value = "1440")]
    entries_per_day: u64,

    #[structopt(long = "num-days", default_value = "3650")]
    num_days: u64,
}

fn main() {
    if let Err(e) = app(Opt::from_args()) {
        eprintln!("{}", e);
        exit(1);
    }
}

fn app(opt: Opt) -> Result<()> {
    let mut fopts = std::fs::OpenOptions::new();
    fopts.create_new(true);
    fopts.write(true);

    let f = match fopts.open(&opt.path) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!(
                "error creating file at {}: {}",
                opt.path.to_string_lossy(),
                e
            )
            .into())
        }
    };

    let mut w = BufWriter::new(f);
    let now: DateTime<FixedOffset> = Utc::now().into();
    let start = now.checked_sub_signed(Duration::days(opt.num_days as i64)).unwrap();
    let step = Duration::seconds((24 * 60 * 60) / opt.entries_per_day as i64);

    let sty = indicatif::ProgressStyle::default_bar()
      .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {percent}% {eta_precise}")
      .progress_chars("##-");
    let pb = indicatif::ProgressBar::new(opt.entries_per_day * opt.num_days);
    pb.set_style(sty);

    for i in 0..(opt.entries_per_day * opt.num_days) {
      let t = start.checked_add_signed(step * i as i32).unwrap();
      Entry::new(t, lipsum::lipsum_words(20)).write(&mut w)?;
      pb.inc(1);
    }

    pb.finish();

    Ok(())
}