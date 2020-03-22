use chrono::prelude::*;
use fs2::FileExt;
use hmmcli::{entries::Entries, entry::Entry, Result};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::path::PathBuf;
use std::process::{exit, Command};
use structopt::StructOpt;
use tempfile::NamedTempFile;

#[derive(Debug, StructOpt)]
#[structopt(name = "hmm", about = "Command line note taking")]
struct Opt {
    /// Path to your hmm file, defaults to your default configuration directory,
    /// ~/.config/.hmm on *nix systems, %APPDATA%\.hmm on Windows.
    #[structopt(long = "path")]
    path: Option<PathBuf>,

    /// If you call hmm with no arguments, it will attempt to open an editor to
    /// compose an entry. It will use this value, or the EDITOR environment
    /// variable.
    #[structopt(long = "editor", env)]
    editor: Option<String>,

    /// Message to add to your hmm journal. Feel free to use quotes or not, but
    /// be wary of how your shell interprets strings. For example, # is often the
    /// beginning of a comment, so anything after it is likely to be ignored.
    /// Additionally anything beginning with a dash or two dashed may be
    /// interpreted as an argument, causing the command to error.
    message: Vec<String>,
}

fn main() {
    if let Err(e) = app(Opt::from_args()) {
        eprintln!("{}", e);
        exit(1);
    }
}

fn app(opt: Opt) -> Result<()> {
    let path = opt
        .path
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".hmm"));

    let mut fopts = std::fs::OpenOptions::new();
    fopts.create(true);
    fopts.read(true);
    fopts.write(true);
    fopts.append(true);

    let mut f = match fopts.open(&path) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!(
                "Couldn't open or create file at {}: {}",
                path.to_string_lossy(),
                e
            )
            .into())
        }
    };

    let mut msg = itertools::join(opt.message, " ");
    if msg.is_empty() {
        if opt.editor.is_none() {
            return Err("Unable to find an editor, set your EDITOR environment variable".into());
        }
        msg = compose_entry(&opt.editor.unwrap())?;
    }

    f.lock_exclusive()?;

    let mut entries = Entries::new(BufReader::new(&mut f));

    if entries.len()? > 0 {
        entries.seek_to_end()?;
        let entry = entries.prev_entry()?.unwrap();

        if entry.datetime() > &Utc::now() {
            return Err("clock skew detected, writing an entry now would break the ordering of your hmm file, please try again in a moment".into());
        }

        entries.seek_to_end()?;

        // Because the seek_to_end function seeks in such a way that reading the previous
        // entry reads the last entry, if we attempted to write to the underlying file at
        // this point there would be a single null byte in between the last entry and the
        // new entry. For this reason, we need to read the previous entry to make sure we
        // aren't leaving any gaps.
        entries.prev_entry()?;
    }

    let res = Entry::with_message(&msg).write(BufWriter::new(&f));
    f.unlock()?;
    res
}

fn compose_entry(editor: &str) -> Result<String> {
    let f = NamedTempFile::new()?;
    let path = f.into_temp_path();

    let status = Command::new(editor).arg(&path).status()?;

    if !status.success() {
        return Err("something went wrong composing entry, please try again".into());
    }

    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::{assert::Assert, prelude::*};
    use hmmcli::entries::Entries;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use test_case::test_case;
    use lazy_static::lazy_static;
    use escargot::{CargoRun, CargoBuild};

    lazy_static! {
        static ref HMM: CargoRun = CargoBuild::new()
            .bin("hmm")
            .current_release()
            .current_target()
            .run()
            .unwrap();
    }

    fn run_with_path(path: &PathBuf, args: Vec<&str>) -> Assert {
        HMM.command()
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

        let mut entries = Entries::new(BufReader::new(File::open(&path).unwrap()));
        entries.next_entry().unwrap().unwrap().message().to_owned()
    }

    #[test_case(vec!["1", "2"]           => vec!["1", "2"]           ; "two invocations")]
    #[test_case(vec!["1", "2", "3"]      => vec!["1", "2", "3"]      ; "three invocations")]
    #[test_case(vec!["1", "2", "3", "4"] => vec!["1", "2", "3", "4"] ; "four invocations")]
    fn test_hmm_multiple_invocations(messages: Vec<&str>) -> Vec<String> {
        let path = new_tempfile_path();
        for message in &messages {
            let assert = run_with_path(&path, vec![message]);
            assert_eq!(
                assert.get_output().stdout.is_empty(),
                true,
                "expected stdout to be empty"
            );
            assert.success();
        }

        // Start with the earliest possible date, as we're going to compare the dates we find
        // in the resulting file with this value to make sure they always increase.
        let mut date: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339("1970-01-01T00:00:00+00:00").unwrap();

        let r = BufReader::new(File::open(&path).unwrap());
        let entries = Entries::new(r);
        let mut messages: Vec<String> = Vec::with_capacity(messages.len());
        for result in entries {
            let entry = result.unwrap();

            messages.push(entry.message().to_owned());

            assert_eq!(true, &date <= entry.datetime());
            date = entry.datetime().to_owned();
        }

        messages
    }

    #[test_case(vec!["--path", "/this/path/does/not/exist"],        "Couldn't open or create file at")]
    #[test_case(vec!["--path", "something", "--path", "something"], "The argument '--path <path>' was provided more than once")]
    #[test_case(vec!["--nonexistent"], "Found argument '--nonexistent' which wasn't expected")]
    fn test_hmm_errors(args: Vec<&str>, error: &str) {
        let assert = HMM.command().args(args).assert();
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        assert.failure();
        assert_eq!(
            stderr.contains(error),
            true,
            "could not find \"{}\" in \"{}\"",
            error,
            stderr
        );
    }
}
