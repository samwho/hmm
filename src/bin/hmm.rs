use fs2::FileExt;
use hmm::{config::Config, entry::Entry, error::Error, Result};
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
    if msg.is_empty() {
        msg = compose_entry(&config.editor()?)?;
    }

    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(config.path()?)?;

    f.lock_exclusive()?;
    let res = Entry::with_message(&msg).write(BufWriter::new(&f));
    f.unlock()?;
    res
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
