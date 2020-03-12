use fs2::FileExt;
use hmm::{config::Config, entry::Entry, error::Error, Result};
use std::fs::OpenOptions;
use std::io::{stderr, BufWriter, Read, Write};
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

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use hmm::{config::Config, entry::Entry, Result};
    use std::convert::TryInto;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
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

    #[test_case(vec!["hello world"]      => "hello world"   ; "single argument, single line entry")]
    #[test_case(vec!["hello", "world"]   => "hello world"   ; "multiple argument, single line entry")]
    #[test_case(vec!["hello\nworld"]     => "hello\nworld"  ; "single argument, multiple line entry")]
    #[test_case(vec!["hello\n", "world"] => "hello\n world" ; "multiple argument, multiple line entry")]
    fn test_hmm_single_invocation(args: Vec<&str>) -> String {
        let t = NamedTempFile::new().unwrap();
        let path = t.keep().unwrap().1;

        let config = Config {
            path: Some(path),
            ..Default::default()
        };
        let config_path = mkconfig(&config).unwrap();

        let assert = Command::cargo_bin("hmm")
            .unwrap()
            .arg("--config")
            .arg(config_path.path())
            .args(args)
            .assert();
        assert.success();

        let mut buf = String::new();

        BufReader::new(File::open(config.path().unwrap()).unwrap())
            .read_line(&mut buf)
            .unwrap();

        let entry: Entry = buf.as_str().try_into().unwrap();

        entry.message().to_owned()
    }
}
