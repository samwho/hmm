use chrono::prelude::*;
use fs2::FileExt;
use hmm::{seek, entry::Entry, error::Error, Result};
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::{stderr, BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::{exit, Command};
use structopt::StructOpt;
use tempfile::NamedTempFile;

#[derive(Debug, StructOpt)]
#[structopt(name = "hmm", about = "Command line note taking")]
struct Opt {
    /// Path to your hmm file, defaults to your default configuration directory,
    /// ~/.config on *nix systems, %APPDATA% on Windows.
    #[structopt(long = "path")]
    path: Option<PathBuf>,

    /// Message to add to your hmm journal. Feel free to use quotes or not, but
    /// be wary of how your shell interprets strings. For example, # is often the
    /// beginning of a comment, so anything after it is likely to be ignored.
    message: Vec<String>,
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
    let mut msg = itertools::join(opt.message, " ");
    if msg.is_empty() {
        msg = compose_entry(&editor()?)?;
    }

    let path = opt
        .path
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".hmm"));

    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(path)?;

    f.lock_exclusive()?;

    let meta = f.metadata();
    if meta.is_ok() && meta.unwrap().len() > 0 {
        let return_pos = f.seek(SeekFrom::Current(0))?;
        let last_line = read_last_line(&mut f)?;
        let last_entry: Entry = last_line.as_str().try_into()?;

        if last_entry.datetime() > &Utc::now() {
            return Err(Error::StringError("clock skew detected, writing an entry now would break the ordering of your hmm file, please try again in a moment".to_owned()));
        }

        f.seek(SeekFrom::Start(return_pos))?;
    }

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

fn editor() -> Result<String> {
    if let Ok(editor) = std::env::var("EDITOR") {
        Ok(editor)
    } else {
        Err(Error::StringError(
            "unable to find an editor, set your EDITOR environment variable".to_owned(),
        ))
    }
}

fn read_last_line(f: &mut (impl Seek + Read)) -> Result<String> {
    f.seek(SeekFrom::End(-1))?;
    seek::start_of_current_line(f)?;
    let mut buf = String::new();
    BufReader::new(f).read_line(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::{assert::Assert, Command};
    use hmm::entry::Entry;
    use std::convert::TryInto;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Cursor};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use test_case::test_case;

    fn run_with_path(path: &PathBuf, args: Vec<&str>) -> Assert {
        Command::cargo_bin("hmm")
            .unwrap()
            .arg("--path")
            .arg(path.as_os_str())
            .args(args)
            .assert()
    }

    fn new_tempfile_path() -> PathBuf {
        NamedTempFile::new().unwrap().keep().unwrap().1
    }

    #[test_case(vec!["hello world"]      => "hello world"   ; "single argument, single line entry")]
    #[test_case(vec!["hello", "world"]   => "hello world"   ; "multiple argument, single line entry")]
    #[test_case(vec!["hello\nworld"]     => "hello\nworld"  ; "single argument, multiple line entry")]
    #[test_case(vec!["hello\n", "world"] => "hello\n world" ; "multiple argument, multiple line entry")]
    fn test_hmm_single_invocation(args: Vec<&str>) -> String {
        let path = new_tempfile_path();
        let assert = run_with_path(&path, args);
        assert.success();

        let mut buf = String::new();
        BufReader::new(File::open(&path).unwrap())
            .read_line(&mut buf)
            .unwrap();

        let entry: Entry = buf.as_str().try_into().unwrap();
        entry.message().to_owned()
    }

    #[test_case(vec!["1", "2"]           => vec!["1", "2"]           ; "two invocations")]
    #[test_case(vec!["1", "2", "3"]      => vec!["1", "2", "3"]      ; "three invocations")]
    #[test_case(vec!["1", "2", "3", "4"] => vec!["1", "2", "3", "4"] ; "four invocations")]
    fn test_hmm_multiple_invocations(messages: Vec<&str>) -> Vec<String> {
        let path = new_tempfile_path();
        for message in &messages {
            let assert = run_with_path(&path, vec![message]);
            assert.success();
        }

        // Start with the earliest possible date, as we're going to compare the dates we find
        // in the resulting file with this value to make sure they always increase.
        let mut date: DateTime<FixedOffset> = DateTime::parse_from_rfc3339("1970-01-01T00:00:00+00:00").unwrap();

        let mut messages: Vec<String> = Vec::with_capacity(messages.len());
        let r = BufReader::new(File::open(&path).unwrap());
        for line in r.lines() {
            let entry: Entry = line.unwrap().as_str().try_into().unwrap();
            messages.push(entry.message().to_owned());

            assert_eq!(true, &date <= entry.datetime());
            date = entry.datetime().to_owned();
        }

        messages
    }

    #[test_case("line 1\nline 2\nline 3\n" => "line 3\n" ; "line ending in new line")]
    fn test_read_last_line(s: &str) -> String {
        let mut r = Cursor::new(s.as_bytes());
        read_last_line(&mut r).unwrap()
    }
}
