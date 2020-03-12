use chrono::Utc;
use fs2::FileExt;
use hmm::{
    bsearch::seek_start_of_current_line, config::Config, entry::Entry, error::Error, Result,
};
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
    #[structopt(short = "c", long = "config")]
    config: Option<PathBuf>,

    rest: Vec<String>,
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
    let config = opt
        .config
        .map(|c| Config::read_from(&c))
        .unwrap_or_else(Config::read)?;

    let mut msg = itertools::join(opt.rest, " ");
    if msg.is_empty() {
        msg = compose_entry(&config.editor()?)?;
    }

    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(config.path()?)?;

    f.lock_exclusive()?;

    let last_line = read_last_line(&mut f)?;
    let last_entry: Entry = last_line.as_str().try_into()?;

    if last_entry.datetime() < &Utc::now() {
        return Err(Error::StringError("clock skew detected, writing an entry now would break the ordering of your hmm file, please try again in a moment".to_owned()));
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

fn read_last_line(f: &mut (impl Seek + Read)) -> Result<String> {
    f.seek(SeekFrom::End(0))?;
    seek_start_of_current_line(f)?;
    let mut buf = String::new();
    BufReader::new(f).read_line(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use assert_cmd::{assert::Assert, Command};
    use chrono::{DateTime, Utc};
    use hmm::{config::Config, entry::Entry, Result};
    use std::convert::TryInto;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use test_case::test_case;

    fn mkconfig(config: &Config) -> Result<NamedTempFile> {
        let mut f = NamedTempFile::new()?;
        let t = toml::to_string(config)?;
        f.write_all(t.as_bytes())?;
        f.flush()?;
        f.seek(SeekFrom::Start(0))?;
        Ok(f)
    }

    fn run_with_config(config: &Config, args: Vec<&str>) -> Assert {
        let config_path = mkconfig(&config).unwrap();
        Command::cargo_bin("hmm")
            .unwrap()
            .arg("--config")
            .arg(config_path.path())
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
        let config = Config {
            path: Some(new_tempfile_path()),
            ..Default::default()
        };

        let assert = run_with_config(&config, args);
        assert.success();

        let mut buf = String::new();
        BufReader::new(File::open(config.path().unwrap()).unwrap())
            .read_line(&mut buf)
            .unwrap();

        let entry: Entry = buf.as_str().try_into().unwrap();
        entry.message().to_owned()
    }

    #[test_case(vec!["1", "2"]           => vec!["1", "2"]           ; "two invocations")]
    #[test_case(vec!["1", "2", "3"]      => vec!["1", "2", "3"]      ; "three invocations")]
    #[test_case(vec!["1", "2", "3", "4"] => vec!["1", "2", "3", "4"] ; "four invocations")]
    fn test_hmm_multiple_invocations(messages: Vec<&str>) -> Vec<String> {
        let config = Config {
            path: Some(new_tempfile_path()),
            ..Default::default()
        };

        for message in &messages {
            let assert = run_with_config(&config, vec![message]);
            assert.success();
        }

        // Start with the earliest possible date, as we're going to compare the dates we find
        // in the resulting file with this value to make sure they always increase.
        let mut date: DateTime<Utc> = DateTime::parse_from_rfc3339("1970-01-01T00:00:00+00:00")
            .unwrap()
            .into();

        let mut messages: Vec<String> = Vec::with_capacity(messages.len());
        let r = BufReader::new(File::open(config.path().unwrap()).unwrap());
        for line in r.lines() {
            let entry: Entry = line.unwrap().as_str().try_into().unwrap();
            messages.push(entry.message().to_owned());

            assert_eq!(true, &date <= entry.datetime());
            date = entry.datetime().to_owned();
        }

        messages
    }
}
