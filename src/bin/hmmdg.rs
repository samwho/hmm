use chrono::{prelude::*, Duration};
use hmmcli::{entry::Entry, Result};
use human_panic::setup_panic;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hmmdg", about = "Generate valid .hmm files for benchmarking.")]
struct Opt {
    /// Path to write the generated .hmm file to. Specifically does not default to the
    /// usual .hmm file location, and will refuse to run if the file already exists.
    #[structopt(long = "path")]
    path: PathBuf,

    /// How many simulated entries per day you would like to write.
    #[structopt(long = "entries-per-day", default_value = "1440")]
    entries_per_day: u64,

    /// How many days you would like to simulate entries for.
    #[structopt(long = "num-days", default_value = "3650")]
    num_days: u64,

    /// You can optionally supply a fixed message to write for every entry. If this is not
    /// supplied, a random message is generated for you.
    #[structopt(long = "message")]
    message: Option<String>,
}

fn main() {
    setup_panic!();

    if let Err(e) = app(&Opt::from_args()) {
        eprintln!("{}", e);
        exit(1);
    }
}

fn app(opt: &Opt) -> Result<()> {
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
    let start = now
        .checked_sub_signed(Duration::days(opt.num_days as i64))
        .unwrap();
    let step = Duration::seconds((24 * 60 * 60) / opt.entries_per_day as i64);

    let sty = indicatif::ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {wide_bar:.cyan/blue} {pos}/{len} {percent}% {eta_precise}")
        .unwrap()
        .progress_chars("##-");
    let pb = indicatif::ProgressBar::new(opt.entries_per_day * opt.num_days);
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb.set_style(sty);

    for i in 0..(opt.entries_per_day * opt.num_days) {
        let t = start.checked_add_signed(step * i as i32).unwrap();
        Entry::new(
            t,
            opt.message
                .clone()
                .unwrap_or_else(|| lipsum::lipsum_words(20)),
        )
        .write(&mut w)?;
        pb.inc(1);
    }

    pb.finish();

    Ok(())
}
