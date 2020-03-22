use chrono::prelude::*;
use hmmcli::{entries::Entries, format::Format, Result};
use human_panic::setup_panic;
use std::fs::File;
use std::io::{BufReader, Read};
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
    /// https://handlebarsjs.com/guide/ for information on how to use them. The
    /// values "datetime" and "message" are passed in.
    #[structopt(
        long = "format",
        default_value = "╭ {{ color \"blue\" (strftime \"%Y-%m-%d %H:%M\" datetime) }}\n{{ indent (markdown message) }}╰─────────────────"
    )]
    format: String,

    /// Path to a file containing a Handlebar template to use as --format. If both
    /// --format-file and --format are supplied, --format-file takes precedence.
    #[structopt(long = "format-file")]
    format_file: Option<PathBuf>,

    /// Print a random entry. Specifying this flag means the other flags will be
    /// ignored.
    #[structopt(long = "random")]
    random: bool,

    /// Print the number of matched entries instead of the content of the entries.
    /// If you specify --format alongside this flag, it will not do anything. Same
    /// with --raw.
    #[structopt(short = "c", long = "count")]
    count: bool,

    /// Prints out entries in their raw CSV format. Anything set in --format is
    /// ignored if you specify this flag.
    #[structopt(long = "raw")]
    raw: bool,

    /// Print out the first N entries only. Cannot be used alongside --last.
    #[structopt(long = "first")]
    first: Option<i64>,

    /// Print out the last N entries only. Cannot be used alongside --first.
    #[structopt(long = "last")]
    last: Option<i64>,

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

    /// Only print entries that contain this substring exactly. Cannot be used
    /// with --regex.
    #[structopt(long = "contains")]
    contains: Option<String>,

    /// Only print entries that match this regular expression. Cannot be used with
    /// --contains.
    #[structopt(long = "regex")]
    regex: Option<String>,
}

fn main() {
    setup_panic!();

    if let Err(e) = app(Opt::from_args()) {
        eprintln!("{}", e);
        exit(1);
    }
}

fn app(opt: Opt) -> Result<()> {
    let mut formatter = if let Some(path) = opt.format_file {
        let mut f = File::open(path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        Format::with_template(&contents)?
    } else {
        Format::with_template(&opt.format)?
    };

    let path = opt
        .path
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".hmm"));

    let mut fopts = std::fs::OpenOptions::new();
    fopts.create(true);
    fopts.read(true);
    fopts.write(true);

    let f = fopts.open(&path).map_err(|e| {
        format!(
            "Couldn't open or create file at {}: {}",
            path.to_string_lossy(),
            e
        )
    })?;
    let mut entries = Entries::new(BufReader::new(f));

    if opt.random {
        if let Some(entry) = entries.rand_entry()? {
            println!("{}", formatter.format_entry(&entry)?);
        }
        return Ok(());
    }

    if opt.regex.is_some() && opt.contains.is_some() {
        return Err("You can only specify one of --contains and --regex".into());
    }

    let regex = match opt.regex {
        None => None,
        Some(s) => Some(regex::Regex::new(&s)?),
    };

    if opt.first.is_some() && opt.last.is_some() {
        return Err("cannot specify --first and --last at the same time".into());
    }

    if let Some(first) = opt.first {
        if first < 1 {
            return Err("--first must be greater than 0".into());
        }
    }

    if let Some(last) = opt.last {
        if last < 1 {
            return Err("--last must be greater than 0".into());
        }
    }

    if let Some(ref start_date) = opt.start {
        entries.seek_to_first(start_date)?;
    }

    if let Some(last) = opt.last {
        match opt.end {
            Some(ref end_date) => {
                // Because --end is exclusive, all we need to do is seek to the
                // first occurrence of a given time and then work backward from
                // there.
                entries.seek_to_first(end_date)?;
            }
            None => {
                // We read the last entry to get to the end of the file. We'll
                // end up reading the entry again later, so it's definitely not
                // the most optimal way to achieve this but it is the simplest.
                let len = entries.len()?;
                entries.at(len)?;
            }
        }

        // Seek back --last number of lines so the loop begins where we want it
        // to.
        for _ in 0..last {
            entries.seek_to_prev()?;
        }
    }

    let mut count = 0;
    loop {
        if opt.first.is_some() && count >= opt.first.unwrap() {
            break;
        }

        match entries.next_entry()? {
            None => break,
            Some(entry) => {
                // If we've found an entry that occurs on or after our given end
                // date, break out and stop printing.
                if opt.end.is_some() && opt.end.as_ref().unwrap() <= entry.datetime() {
                    break;
                }

                // If we've found an entry that does not contain the specified
                // string to search for, move to the next loop iteration.
                if opt.contains.is_some()
                    && !entry.message().contains(opt.contains.as_ref().unwrap())
                {
                    continue;
                }

                if regex.is_some() && !regex.as_ref().unwrap().is_match(entry.message()) {
                    continue;
                }

                if !opt.count {
                    if opt.raw {
                        print!("{}", entry.to_csv_row()?);
                    } else {
                        println!("{}", formatter.format_entry(&entry)?);
                    }
                }
                count += 1;
            }
        };
    }

    if opt.count {
        println!("{}", count);
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

    Err(format!("unrecognised date format: \"{}\", accepted formats include things like:\n  - 2012\n  - 2012-01\n  - 2012-01-24\n  - 2012-01-24T16\n  - 2012-01-24T16:20\n  - 2012-01-24T16:20:30", s).into())
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
    use assert_cmd::{assert::Assert, prelude::*};
    use escargot::{CargoBuild, CargoRun};
    use lazy_static::lazy_static;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use test_case::test_case;

    lazy_static! {
        static ref HMMQ: CargoRun = CargoBuild::new()
            .bin("hmmq")
            .current_release()
            .current_target()
            .run()
            .unwrap();
    }

    fn run_with_path(path: &PathBuf, args: Vec<&str>) -> Assert {
        HMMQ.command()
            .arg("--path")
            .arg(path.as_os_str())
            .args(args)
            .assert()
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

    #[test_case(vec!["--first", "1", "--raw"] => "2020-01-01T00:01:00.899849209+00:00,\"\"\"1\"\"\"\n")]
    #[test_case(vec!["--first", "2", "--format", "{{ message }}"] => "1\n2\n" ; "get first two lines")]
    #[test_case(vec!["--first", "1", "--start", "2020-02", "--format", "{{ message }}"] => "2\n")]
    #[test_case(vec!["--last", "1", "--raw"] => "2020-06-13T10:12:53.353050231+00:00,\"\"\"6\"\"\"\n")]
    #[test_case(vec!["--last", "2", "--format", "{{ message }}"] => "5\n6\n" ; "get last two lines")]
    #[test_case(vec!["--start", "2021", "--end", "2020"] => "")]
    #[test_case(vec!["--first", "1", "--format", "{{ indent message }}"] => "│ 1\n")]
    #[test_case(vec!["--first", "1", "--format", "{{ strftime \"%Y-%m-%d\" datetime }}"] => "2020-01-01\n")]
    #[test_case(vec!["--start", "2020-01-01T00:01:00", "--end", "2020-03-12T00:00:00", "--format", "{{ message }}"] => "1\n2\n")]
    #[test_case(vec!["--last", "1", "--end", "2020-03-12T00:00:00", "--format", "{{ message }}"] => "2\n")]
    #[test_case(vec!["--start", "2020-06-13", "--end", "2020-06-14", "--format", "{{ message }}"] => "6\n")]
    #[test_case(vec!["--contains", "1", "--format", "{{ message }}"] => "1\n")]
    #[test_case(vec!["--regex", "(1|2)", "--format", "{{ message }}"] => "1\n2\n")]
    #[test_case(vec!["--raw"] => TESTDATA)]
    #[test_case(vec!["--count"] => "6\n")]
    #[test_case(vec!["--first", "1", "--count"] => "1\n")]
    #[test_case(vec!["--contains", "4", "--count"] => "1\n")]
    #[test_case(vec!["--contains", "nope", "--count"] => "0\n")]
    fn test_hmmq(args: Vec<&str>) -> String {
        let path = new_tempfile(TESTDATA);

        let assert = run_with_path(&path, args);
        String::from_utf8(assert.get_output().stdout.clone()).unwrap()
    }

    #[test_case(vec!["--path", "/this/path/does/not/exist"],        "Couldn't open or create file at")]
    #[test_case(vec!["--path", "something", "--path", "something"], "The argument '--path <path>' was provided more than once")]
    #[test_case(vec!["--nonexistent"],                              "Found argument '--nonexistent' which wasn't expected")]
    #[test_case(vec!["--contains", "a", "--regex", "b"],            "You can only specify one of --contains and --regex")]
    #[test_case(vec!["--regex", "("],                               "regex parse error")]
    #[test_case(vec!["--path", new_tempfile("").to_str().unwrap(),  "--first=-1"],                  "--first must be greater than 0")]
    #[test_case(vec!["--path", new_tempfile("").to_str().unwrap(),  "--first", "0"],                "--first must be greater than 0")]
    #[test_case(vec!["--path", new_tempfile("").to_str().unwrap(),  "--last=-1"],                   "--last must be greater than 0")]
    #[test_case(vec!["--path", new_tempfile("").to_str().unwrap(),  "--last", "0"],                 "--last must be greater than 0")]
    #[test_case(vec!["--path", new_tempfile("").to_str().unwrap(),  "--start", "nope"],             "unrecognised date format")]
    #[test_case(vec!["--path", new_tempfile("").to_str().unwrap(),  "--end", "nope"],               "unrecognised date format")]
    #[test_case(vec!["--path", new_tempfile("").to_str().unwrap(),  "--format", "{{"],              "invalid handlebars syntax")]
    fn test_hmmq_errors(args: Vec<&str>, error: &str) {
        let assert = HMMQ.command().args(args).assert();
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
