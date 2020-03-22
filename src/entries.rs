use super::{entry::Entry, seek, Result};
use chrono::prelude::*;
use rand::distributions::{Distribution, Uniform};
use std::convert::TryInto;
use std::io::{BufRead, Read, Seek, SeekFrom};

pub struct Entries<T: Seek + Read + BufRead> {
    f: T,
    buf: String,
}

impl<T: Seek + Read + BufRead> Entries<T> {
    pub fn new(f: T) -> Self {
        Entries {
            f,
            buf: String::with_capacity(4096),
        }
    }

    pub fn len(&mut self) -> Result<u64> {
        let prev = self.f.seek(SeekFrom::Current(0))?;
        let len = self.f.seek(SeekFrom::End(0))?;
        self.f.seek(SeekFrom::Start(prev))?;
        Ok(len)
    }

    pub fn is_empty(&mut self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn at(&mut self, pos: u64) -> Result<Option<Entry>> {
        if pos > self.len()? {
            return Ok(None);
        }

        self.f.seek(SeekFrom::Start(pos))?;
        seek::start_of_current_line(&mut self.f)?;
        self.next_entry()
    }

    pub fn seek_to_end(&mut self) -> Result<()> {
        let len = self.len()?;
        self.at(len)?;
        Ok(())
    }

    pub fn seek_to_next(&mut self) -> Result<Option<u64>> {
        seek::start_of_next_line(&mut self.f)
    }

    pub fn seek_to_prev(&mut self) -> Result<Option<u64>> {
        seek::start_of_prev_line(&mut self.f)
    }

    pub fn next_entry(&mut self) -> Result<Option<Entry>> {
        self.buf.clear();
        self.f.read_line(&mut self.buf)?;

        // read_line will leave the buffer empty if it was attempting to read
        // past the end of the file. We set the file cursor to past the end of
        // the file so that we can check later on when trying to come back and
        // read a previous line we can read the last line instead of skipping
        // over it, because prev_line() by default skips the line that was just
        // read.
        if self.buf.is_empty() {
            self.f.seek(SeekFrom::End(1))?;
            return Ok(None);
        }

        let row = quick_csv::Csv::from_reader(self.buf.as_bytes())
            .next()
            .unwrap()?;
        Ok(Some(row.try_into()?))
    }

    pub fn rand_entry(&mut self) -> Result<Option<Entry>> {
        let mut rng = rand::thread_rng();
        let range = Uniform::new(0, self.len()?);
        self.at(range.sample(&mut rng))
    }

    pub fn prev_entry(&mut self) -> Result<Option<Entry>> {
        // This seek takes us to the start of the line that was just read. It
        // will sometimes be None if we're already at the start of the file but
        // that's fine. We don't do this seek if we've previously read past the
        // end of the file, so that when we do read past the end of the file we
        // can again go back and read the last line.
        if self.f.seek(SeekFrom::Current(0))? <= self.len()? {
            self.seek_to_prev()?;
        }

        // This seek takes us to the actual previous entry. If this one returns None
        // it means we're trying to go past the start of the file, and there is no
        // previous entry.
        if self.seek_to_prev()?.is_none() {
            return Ok(None);
        }

        self.next_entry()
    }

    pub fn seek_to_first(&mut self, date: &chrono::DateTime<FixedOffset>) -> Result<()> {
        let file_size = self.len()?;
        let mut end = file_size;
        let mut start = self.f.seek(SeekFrom::Start(0))?;

        while start < end {
            let cur = start + (end - start) / 2;

            let entry = match self.at(cur)? {
                Some(entry) => entry,
                // If we get none back from at() it means we've tried to seek past
                // the end of the file. We break out of the loop in this case and
                // ultimately return to the caller with the file cursor at end of
                // file. This allows people to seek backwards from the end if they
                // want to.
                None => break,
            };

            if entry.datetime() >= date {
                end = cur - 1;
            } else {
                start = cur + 1;
            }
        }

        // When we exit the binary search loop we know that we're in one of the following
        // states:
        //
        //   - We're at the very start of the file.
        //   - We're at or past the end of the file.
        //   - We're somewhere in the middle, potentially on the row before the row we
        //     want to return.
        //
        // We need to navigate to the line that is exactly after the line before us that
        // is less than the given time.

        // If we're at the end of the file, it means that there are no lines in the file
        // that can be less than the given date, so we return with the file cursor at the
        // end of the file.
        if end >= file_size {
            return Ok(());
        }

        // We have to move forward one line at first, as we could have exited the binary
        // search loop on the entry before the one that we need to return.
        self.next_entry()?;

        loop {
            match self.prev_entry()? {
                None => break,
                Some(entry) => {
                    if entry.datetime() < date {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

impl<T: Seek + Read + BufRead> Iterator for Entries<T> {
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_entry() {
            Ok(opt) => match opt {
                Some(entry) => Some(Ok(entry)),
                None => None,
            },
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use test_case::test_case;

    // Each TESTDATA line is 43 characters long, 44 if you count the newline.
    const TESTDATA: &str = "2020-01-01T00:01:00.899849209+00:00,\"\"\"1\"\"\"
2020-02-12T23:08:40.987613062+00:00,\"\"\"2\"\"\"
2020-03-12T00:00:00.000000000+00:00,\"\"\"3\"\"\"
2020-04-12T23:28:45.726598931+00:00,\"\"\"4\"\"\"
2020-05-12T23:28:48.495151445+00:00,\"\"\"5\"\"\"
2020-06-13T10:12:53.353050231+00:00,\"\"\"6\"\"\"
";

    // Clippy isn't a big fan of mathematics that can be represented simpler
    // or evaluates to zero, but in these tests it helps make clear that we're
    // searching in to offsets of each line, so we allow it.
    #[allow(clippy::identity_op, clippy::erasing_op)]
    #[test_case(44 * 0 + 0  => Some("1".to_owned()))]
    #[test_case(44 * 0 + 10 => Some("1".to_owned()))]
    #[test_case(44 * 0 + 43 => Some("1".to_owned()))]
    #[test_case(44 * 1 + 0  => Some("2".to_owned()))]
    #[test_case(44 * 1 + 10 => Some("2".to_owned()))]
    #[test_case(44 * 1 + 43 => Some("2".to_owned()))]
    #[test_case(44 * 2 + 0  => Some("3".to_owned()))]
    #[test_case(44 * 2 + 10 => Some("3".to_owned()))]
    #[test_case(44 * 2 + 43 => Some("3".to_owned()))]
    #[test_case(44 * 3 + 0  => Some("4".to_owned()))]
    #[test_case(44 * 3 + 10 => Some("4".to_owned()))]
    #[test_case(44 * 3 + 43 => Some("4".to_owned()))]
    #[test_case(44 * 4 + 0  => Some("5".to_owned()))]
    #[test_case(44 * 4 + 10 => Some("5".to_owned()))]
    #[test_case(44 * 4 + 43 => Some("5".to_owned()))]
    #[test_case(44 * 5 + 0  => Some("6".to_owned()))]
    #[test_case(44 * 5 + 10 => Some("6".to_owned()))]
    #[test_case(44 * 5 + 43 => Some("6".to_owned()))]
    #[test_case(44 * 6 + 0  => None)]
    #[test_case(44 * 7 + 0  => None)]
    #[test_case(44 * 8 + 0  => None)]
    fn test_entry_at(pos: u64) -> Option<String> {
        let r = Cursor::new(Vec::from(TESTDATA.as_bytes()));
        Entries::new(r)
            .at(pos)
            .unwrap()
            .map(|e| e.message().to_owned())
    }

    // Test cases for exact date matches on each line.
    #[test_case("2020-01-01T00:01:00.899849209+00:00" => Some("1".to_owned()))]
    #[test_case("2020-02-12T23:08:40.987613062+00:00" => Some("2".to_owned()))]
    #[test_case("2020-03-12T00:00:00.000000000+00:00" => Some("3".to_owned()))]
    #[test_case("2020-04-12T23:28:45.726598931+00:00" => Some("4".to_owned()))]
    #[test_case("2020-05-12T23:28:48.495151445+00:00" => Some("5".to_owned()))]
    #[test_case("2020-06-13T10:12:53.353050231+00:00" => Some("6".to_owned()))]
    // Testing dates before and after the dates in the file.
    #[test_case("2000-01-01T00:01:00.000000000+00:00" => Some("1".to_owned()))]
    #[test_case("2021-01-01T00:00:00.000000000+00:00" => None)]
    // Testing dates that aren't exact matches but land us in the middle of the
    // file.
    #[test_case("2020-02-12T23:08:00+00:00" => Some("2".to_owned()))]
    #[test_case("2020-02-12T23:59:00+00:00" => Some("3".to_owned()))]
    #[test_case("2020-04-12T23:27:00+00:00" => Some("4".to_owned()))]
    #[test_case("2020-05-12T23:27:00+00:00" => Some("5".to_owned()))]
    #[test_case("2020-06-13T10:00:00+00:00" => Some("6".to_owned()))]
    fn test_seek_to_first(date_str: &str) -> Option<String> {
        let date = DateTime::parse_from_rfc3339(date_str).unwrap();
        let r = Cursor::new(Vec::from(TESTDATA.as_bytes()));
        let mut entries = Entries::new(r);
        entries.seek_to_first(&date).unwrap();
        entries
            .next_entry()
            .unwrap()
            .map(|e| e.message().to_owned())
    }

    #[test]
    fn test_navigating_entries() -> Result<()> {
        let r = Cursor::new(Vec::from(TESTDATA.as_bytes()));
        let mut entries = Entries::new(r);

        assert_eq!(entries.next_entry()?.unwrap().message(), "1");
        assert_eq!(entries.next_entry()?.unwrap().message(), "2");
        assert_eq!(entries.next_entry()?.unwrap().message(), "3");
        assert_eq!(entries.next_entry()?.unwrap().message(), "4");
        assert_eq!(entries.next_entry()?.unwrap().message(), "5");
        assert_eq!(entries.next_entry()?.unwrap().message(), "6");
        assert_eq!(entries.next_entry()?.is_none(), true);
        assert_eq!(entries.prev_entry()?.unwrap().message(), "6");
        assert_eq!(entries.prev_entry()?.unwrap().message(), "5");
        assert_eq!(entries.prev_entry()?.unwrap().message(), "4");
        assert_eq!(entries.prev_entry()?.unwrap().message(), "3");
        assert_eq!(entries.prev_entry()?.unwrap().message(), "2");
        assert_eq!(entries.prev_entry()?.unwrap().message(), "1");
        assert_eq!(entries.prev_entry()?.is_none(), true);
        assert_eq!(entries.prev_entry()?.is_none(), true);
        assert_eq!(entries.prev_entry()?.is_none(), true);
        assert_eq!(entries.next_entry()?.unwrap().message(), "1");
        assert_eq!(entries.next_entry()?.unwrap().message(), "2");
        assert_eq!(entries.next_entry()?.unwrap().message(), "3");
        assert_eq!(entries.next_entry()?.unwrap().message(), "4");
        assert_eq!(entries.next_entry()?.unwrap().message(), "5");
        assert_eq!(entries.next_entry()?.unwrap().message(), "6");
        assert_eq!(entries.next_entry()?.is_none(), true);
        Ok(())
    }

    #[test]
    fn test_seek_to_end() -> Result<()> {
        let r = Cursor::new(Vec::from(TESTDATA.as_bytes()));
        let mut entries = Entries::new(r);

        entries.seek_to_end()?;
        assert_eq!(entries.prev_entry()?.unwrap().message(), "6");
        Ok(())
    }

    #[test]
    fn test_iterator() -> Result<()> {
        let r = Cursor::new(Vec::from(TESTDATA.as_bytes()));
        let mut entries = Entries::new(r);

        assert_eq!(entries.next().unwrap().unwrap().message(), "1");
        assert_eq!(entries.next().unwrap().unwrap().message(), "2");
        assert_eq!(entries.next().unwrap().unwrap().message(), "3");
        assert_eq!(entries.next().unwrap().unwrap().message(), "4");
        assert_eq!(entries.next().unwrap().unwrap().message(), "5");
        assert_eq!(entries.next().unwrap().unwrap().message(), "6");
        assert_eq!(entries.next().is_none(), true);
        Ok(())
    }
}
