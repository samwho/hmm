use hmm::{config::Config, error::Error, Result};
use std::env;
use std::fs::OpenOptions;
use std::io::{stderr, BufWriter, Read, Write};
use std::process::{exit, Command};
use tempfile::NamedTempFile;

fn main() {
    if let Err(e) = app() {
        if let Err(write_e) = writeln!(&mut stderr(), "{}", e) {
            panic!(write_e);
        }
        exit(1);
    }
}

fn app() -> Result<()> {
    let config = Config::default();

    let mut msg = itertools::join(env::args().skip(1), " ");
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(config.path()?)?;

    if msg.is_empty() {
        msg = compose_entry(&config.editor()?)?;
    }

    write_entry(BufWriter::new(f), msg.trim())?;
    Ok(())
}

fn compose_entry(editor: &str) -> Result<String> {
    let mut f = NamedTempFile::new()?;
    let path = f.path().as_os_str();

    let status = Command::new(editor).arg(path).status()?;

    if !status.success() {
        return Err(Error::StringError(
            "something went wrong composing entry, please try again".to_owned(),
        ));
    }

    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

fn write_entry(w: impl Write, msg: &str) -> Result<()> {
    let now = chrono::Utc::now();
    let mut writer = csv::Writer::from_writer(w);

    Ok(writer.write_record(&[now.to_rfc3339(), serde_json::to_string(&msg)?])?)
}
