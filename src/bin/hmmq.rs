use chrono::prelude::*;
use hmmcli::{entries::Entries, error::Error, format::Format, Result};
use rand::distributions::{Distribution, Uniform};
use std::cmp::Ordering;
use std::io::{stderr, BufReader, Write};
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hmmq", about = "Query your hmm file")]
struct Opt {
    /// Path to your hmm file, defaults to your default configuration directory,
    /// ~/.config on *nix systems, %APPDATA% on Windows.
    #[structopt(long = "path")]
    path: Option<PathBuf>,

    /// How to format entry output. hmm uses Handlebars as a template format, see
    /// https://handlebarsjs.com/guide/ for information on how to use them. The values
    /// "datetime" and "message" are passed in.
    #[structopt(
        long = "format",
        default_value = "{{ color \"blue\" (strftime \"%Y-%m-%d %H:%M:%S\" datetime) }}\n{{ indent message }}\n"
    )]
    format: String,

    /// By default, entries are printed in ascending chronological order. This
    /// flag prints in reverse chronological order.
    #[structopt(long = "descending")]
    descending: bool,

    /// Print a random entry. Specifying this flag means the other flags will be
    /// ignored.
    #[structopt(long = "random")]
    random: bool,

    /// The number of entries to print. If a start and end date have been specified,
    /// this will print the first N of that range. In ascending order, this is the first
    /// N entries chronologically, and in descending order it will be the last N entries.
    #[structopt(short = "n")]
    num_entries: Option<i64>,

    /// Date to start printing from, inclusive. The date will be read in your
    /// local time, and can be specified using any subset of an RFC3339 date,
    /// e.g. 2012, 2012-01, 2012-01-29, 2012-01-29T14, 2012-01-29T14:30,
    /// 2012-01-29T14:30:11.
    #[structopt(short = "s", long = "start", parse(try_from_str = parse_date_arg))]
    start: Option<DateTime<FixedOffset>>,

    /// Date to stop printing at, exclusive. Like --start, this can be any subset of an
    /// RFC3339 date. See --start for details.
    #[structopt(short = "e", long = "end", parse(try_from_str = parse_date_arg))]
    end: Option<DateTime<FixedOffset>>,

    /// Only print entries that contain this substring exactly.
    #[structopt(long = "contains")]
    contains: Option<String>,
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
    let formatter = Format::with_template(&opt.format)?;
    let path = opt
        .path
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".hmm"));

    let mut fopts = std::fs::OpenOptions::new();
    fopts.create(true);
    fopts.read(true);
    fopts.write(true);

    let f = match fopts.open(&path) {
        Ok(f) => f,
        Err(e) => {
            return Err(Error::StringError(format!(
                "couldn't open or create file at {}: {}",
                path.to_string_lossy(),
                e
            )))
        }
    };

    let mut entries = Entries::new(BufReader::new(f));

    if opt.random {
        let mut rng = rand::thread_rng();
        let range = Uniform::new(0, entries.len()?);
        let entry = entries.at(range.sample(&mut rng))?.unwrap();
        println!("{}", formatter.format_entry(&entry)?);
        return Ok(());
    }

    if opt.num_entries.is_some() && opt.num_entries.unwrap() < 1 {
        return Err(Error::StringError(
            "-n must be greater than or equal to 1".to_owned(),
        ));
    }

    let mut entries_printed = 0;

    if opt.descending {
        match opt.end {
            Some(ref end_date) => {
                // Because we want to print in descending order from some end
                // date, we have to seek to the first occurrence of the end date
                // and then scan forward incase there are multiple entries with
                // the same date, exiting the scan when we find an entry with a
                // later date. In practice this will almost always be a single
                // scan forward, but it's best to be correct.
                entries.seek_to_first(end_date)?;
                loop {
                    match entries.next_entry()? {
                        None => break,
                        Some(entry) => {
                            if let Ordering::Greater = entry.datetime().cmp(end_date) {
                                break;
                            }
                        }
                    }
                }
            }
            None => {
                // We read the last entry to get to the end of the file. We'll
                // end up reading the entry again later, so it's definitely not
                // the most optimal way to achieve this but it is the simplest.
                let len = entries.len()?;
                entries.at(len)?;
            }
        }

        loop {
            if opt.num_entries.is_some() && entries_printed >= opt.num_entries.unwrap() {
                break;
            }

            match entries.prev_entry()? {
                None => break,
                Some(entry) => {
                    // If we've found an entry that occurs before our given start
                    // date, break out and stop printing.
                    if opt.start.is_some()
                        && opt.start.unwrap().cmp(entry.datetime()) == Ordering::Greater
                    {
                        break;
                    }

                    // If we've found an entry that does not contain the specified
                    // string to search for, move to the next loop iteration.
                    if opt.contains.is_some()
                        && !entry.message().contains(opt.contains.as_ref().unwrap())
                    {
                        continue;
                    }

                    println!("{}", formatter.format_entry(&entry)?);
                    entries_printed += 1;
                }
            };
        }
    } else {
        if let Some(ref start_date) = opt.start {
            entries.seek_to_first(start_date)?;
        }

        loop {
            if opt.num_entries.is_some() && entries_printed >= opt.num_entries.unwrap() {
                break;
            }

            match entries.next_entry()? {
                None => break,
                Some(entry) => {
                    // If we've found an entry that occurs after our given end
                    // date, break out and stop printing.
                    if opt.end.is_some() && opt.end.unwrap().cmp(entry.datetime()) == Ordering::Less
                    {
                        break;
                    }

                    // If we've found an entry that does not contain the specified
                    // string to search for, move to the next loop iteration.
                    if opt.contains.is_some()
                        && !entry.message().contains(opt.contains.as_ref().unwrap())
                    {
                        continue;
                    }

                    println!("{}", formatter.format_entry(&entry)?);
                    entries_printed += 1;
                }
            };
        }
    }

    Ok(())
}

fn parse_date_arg(s: &str) -> Result<DateTime<FixedOffset>> {
    if let Ok(d) = parse_local_datetime_str(&format!("{}-01-01T00:00:00", s), "%Y-%m-%dT%H:%M:%S") {
        return Ok(d.into());
    }
    if let Ok(d) = parse_local_datetime_str(&format!("{}-01T00:00:00", s), "%Y-%m-%dT%H:%M:%S") {
        return Ok(d.into());
    }
    if let Ok(d) = parse_local_datetime_str(&format!("{}T00:00:00", s), "%Y-%m-%dT%H:%M:%S") {
        return Ok(d.into());
    }
    if let Ok(d) = parse_local_datetime_str(&format!("{}:00:00", s), "%Y-%m-%dT%H:%M:%S") {
        return Ok(d.into());
    }
    if let Ok(d) = parse_local_datetime_str(&format!("{}:00", s), "%Y-%m-%dT%H:%M:%S") {
        return Ok(d.into());
    }
    if let Ok(d) = parse_local_datetime_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(d.into());
    }

    Err(Error::StringError(format!("unrecognised date format: \"{}\", accepted formats include things like:\n  - 2012\n  - 2012-01\n  - 2012-01-24\n  - 2012-01-24T16\n  - 2012-01-24T16:20\n  - 2012-01-24T16:20:30", s)))
}

fn parse_local_datetime_str(s: &str, format: &str) -> Result<DateTime<Utc>> {
    let d = NaiveDateTime::parse_from_str(s, format)?;
    let local_result = Utc.from_local_datetime(&d);
    Ok(local_result.earliest().unwrap_or_else(|| {
        local_result
            .latest()
            .unwrap_or_else(|| local_result.unwrap())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::{prelude::*, assert::Assert};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use test_case::test_case;
    use std::process::Command;

    fn cmd(name: &str) -> Command {
        escargot::CargoBuild::new().bin(name).current_release().current_target().run().unwrap().command()
    }

    fn run_with_path(path: &PathBuf, args: Vec<&str>) -> Assert {
        cmd("hmmq").arg("--path").arg(path.as_os_str()).args(args).assert()
    }

    fn new_tempfile(content: &str) -> PathBuf {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.keep().unwrap().1
    }

    #[test_case("2012"                => "2012-01-01T00:00:00+00:00" ; "y")]
    #[test_case("2012-02"             => "2012-02-01T00:00:00+00:00" ; "ym")]
    #[test_case("2012-02-02"          => "2012-02-02T00:00:00+00:00" ; "ymd")]
    #[test_case("2012-02-02T02"       => "2012-02-02T02:00:00+00:00" ; "ymdh")]
    #[test_case("2012-02-02T02:02"    => "2012-02-02T02:02:00+00:00" ; "ymdhm")]
    #[test_case("2012-02-02T02:02:02" => "2012-02-02T02:02:02+00:00" ; "ymdhms")]
    fn test_parse_date_arg(s: &str) -> String {
        parse_date_arg(s).unwrap().to_rfc3339()
    }

    const TESTDATA: &str = "2020-01-01T00:01:00.899849209+00:00,\"\"\"1\"\"\"
2020-02-12T23:08:40.987613062+00:00,\"\"\"2\"\"\"
2020-03-12T00:00:00+00:00,\"\"\"3\"\"\"
2020-04-12T23:28:45.726598931+00:00,\"\"\"4\"\"\"
2020-05-12T23:28:48.495151445+00:00,\"\"\"5\"\"\"
2020-06-13T10:12:53.353050231+00:00,\"\"\"6\"\"\"
";

    #[test_case(vec!["-n", "1", "--format", "{{ raw }}"] => "2020-01-01T00:01:00.899849209+00:00,\"\"\"1\"\"\"\n")]
    #[test_case(vec!["-n", "2", "--format", "{{ message }}"] => "1\n2\n" ; "get first two lines")]
    #[test_case(vec!["-n", "2", "--descending", "--format", "{{ message }}"] => "6\n5\n" ; "get last two lines")]
    #[test_case(vec!["-n", "2", "--descending", "--end", "2020-05-12T23:28:49", "--format", "{{ message }}"] => "5\n4\n")]
    #[test_case(vec!["--descending", "--start", "2021", "--end", "2020"] => "")]
    #[test_case(vec!["--start", "2021", "--end", "2020"] => "")]
    #[test_case(vec!["-n", "1", "--format", "{{ indent message }}"] => "| 1\n")]
    #[test_case(vec!["-n", "1", "--format", "{{ strftime \"%Y-%m-%d\" datetime }}"] => "2020-01-01\n")]
    #[test_case(vec!["--start", "2020-06-13", "--end", "2020-06-14", "--format", "{{ message }}"] => "6\n")]
    #[test_case(vec!["--format", "{{ raw }}"] => TESTDATA)]
    fn test_hmmq(args: Vec<&str>) -> String {
        let path = new_tempfile(TESTDATA);

        let assert = run_with_path(&path, args);
        String::from_utf8(assert.get_output().stdout.clone()).unwrap()
    }

    #[test_case(vec!["--path", "/this/path/does/not/exist"],        "No such file or directory")] // lame error?
    #[test_case(vec!["--path", "something", "--path", "something"], "The argument '--path <path>' was provided more than once")]
    #[test_case(vec!["--nonexistent"],                              "Found argument '--nonexistent' which wasn't expected")]
    #[test_case(vec!["--path", new_tempfile("").to_str().unwrap(),  "-n=-1"],                       "-n must be greater than or equal to 1")]
    #[test_case(vec!["--path", new_tempfile("").to_str().unwrap(),  "-n", "0"],                     "-n must be greater than or equal to 1")]
    #[test_case(vec!["--path", new_tempfile("").to_str().unwrap(),  "--start", "nope"],             "unrecognised date format")]
    #[test_case(vec!["--path", new_tempfile("").to_str().unwrap(),  "--end", "nope"],               "unrecognised date format")]
    #[test_case(vec!["--path", new_tempfile("").to_str().unwrap(),  "--format", "{{"],              "invalid handlebars syntax")]
    fn test_hmmq_errors(args: Vec<&str>, error: &str) {
        let assert = cmd("hmmq").args(args).assert();
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
