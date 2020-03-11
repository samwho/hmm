use super::Result;
use std::cmp::Ordering;
use std::io::{ErrorKind, Read, Seek, SeekFrom};

pub enum SeekType {
    // Seek to the first occurrence of a given prefix. If the start of file is
    // reached before finding an exact match, return start of file.
    First,
    // Seek to the last occurrence of a given prefix. If end of file is reached
    // before finding an exact match, return end of file.
    Last,
}

// seek_first takes a Seek + Read and a string prefix and will seek to either
// the first exact match of the prefix or the match lexicographically closest.
// Its use is intended to seek a file to the line closest matching a given
// ISO8601 datetime string, for printing out hmm entries.
//
// If this function gets to the end of the file and can't find a match, it will
// return Ok(None).assert_eq! If it gets to the start of the file, it will
// return Ok(0) as the rest of the file is lexicographically later than the
// given prefix.
pub fn seek<T: Seek + Read>(f: &mut T, prefix: &str, seek_type: SeekType) -> Result<Option<u64>> {
    let mut end = f.seek(SeekFrom::End(0))?;
    let mut start = f.seek(SeekFrom::Start(0))?;
    let mut buf = vec![0; prefix.len()];
    let bytes = prefix.as_bytes();

    loop {
        if end <= start {
            break;
        }

        let cur = start + (end - start) / 2;

        f.seek(SeekFrom::Start(cur))?;
        let mut line_start = seek_start_of_current_line(f)?;

        match f.read_exact(&mut buf) {
            Ok(_) => (),
            Err(e) => {
                // We read past the end of the file, which means we can't
                // possibly find a match in this file, so we return Ok(None) to
                // signal that the match happens after the content of the file.
                if e.kind() == ErrorKind::UnexpectedEof {
                    return Ok(None);
                } else {
                    return Err(e.into());
                }
            }
        }

        match bytes.cmp(&buf) {
            Ordering::Less => {
                if cur == 0 {
                    // We've been given a prefix that occurs before the first
                    // line of the file, so we return the start of the file.
                    // This is expected behaviour when searching for all entries
                    // between, say, 2018 and 2019 but you only have entries in
                    // 2019.
                    return Ok(Some(0));
                }
                end = cur - 1;
            }
            Ordering::Equal => loop {
                // First line of the file has matched exactly, so we seek to the
                // start and return.
                if line_start == 0 {
                    f.seek(SeekFrom::Start(line_start))?;
                    return Ok(Some(line_start));
                }

                match seek_type {
                    // We've matched exactly but because we've been jumping
                    // through the file in a binary search fashion, we may not
                    // be at the earliest possible match. Scan through the file
                    // firward or backward until we get to a non-exact match.
                    SeekType::First => {
                        let new_start = seek_start_of_prev_line(f)?;
                        if new_start.is_none() {
                            f.seek(SeekFrom::Start(0))?;
                            return Ok(Some(0));
                        }

                        f.read_exact(&mut buf)?;
                        if let Ordering::Greater = bytes.cmp(&buf) {
                            f.seek(SeekFrom::Start(line_start))?;
                            return Ok(Some(line_start));
                        } else {
                            line_start = new_start.unwrap();
                        }
                    }
                    SeekType::Last => {
                        let new_start = seek_start_of_next_line(f)?;
                        if new_start.is_none() {
                            f.seek(SeekFrom::Start(line_start))?;
                            return Ok(Some(line_start));
                        }

                        f.read_exact(&mut buf)?;
                        if let Ordering::Less = bytes.cmp(&buf) {
                            f.seek(SeekFrom::Start(line_start))?;
                            return Ok(Some(line_start));
                        } else {
                            line_start = new_start.unwrap();
                        }
                    }
                }
            },
            Ordering::Greater => {
                start = cur + 1;
            }
        }
    }

    Ok(None)
}

pub fn seek_start_of_next_line<T: Seek + Read>(f: &mut T) -> Result<Option<u64>> {
    let mut buf = [0; 1];
    let mut pos = f.seek(SeekFrom::Current(0))?;

    loop {
        pos += 1;
        if let Err(e) = f.read_exact(&mut buf) {
            if e.kind() == ErrorKind::UnexpectedEof {
                return Ok(None);
            } else {
                return Err(e.into());
            }
        }

        if buf[0] == 0x0a {
            return Ok(Some(pos));
        }
    }
}

pub fn seek_start_of_prev_line<T: Seek + Read>(f: &mut T) -> Result<Option<u64>> {
    seek_start_of_current_line(f)?;

    let mut buf = [0; 1];
    let mut pos = f.seek(SeekFrom::Current(0))?;

    if pos == 0 {
        return Ok(None);
    }

    pos -= 1;
    f.seek(SeekFrom::Start(pos))?;

    loop {
        if pos == 0 {
            f.seek(SeekFrom::Start(0))?;
            return Ok(Some(0));
        }

        pos -= 1;
        f.seek(SeekFrom::Start(pos))?;
        f.read_exact(&mut buf)?;

        if buf[0] == 0x0a {
            return Ok(Some(pos + 1));
        }
    }
}

fn seek_start_of_current_line<T: Seek + Read>(f: &mut T) -> Result<u64> {
    let mut buf = [0; 1];
    let mut pos = f.seek(SeekFrom::Current(0))?;

    if let Err(e) = f.read_exact(&mut buf) {
        // If we try to read past the end of the file, which is what
        // ErrorKind::UnexpectedEof represents, it's not really a problem. We
        // just quietly drop in to the loop below and start backtracking. If
        // not, we raise the error.
        if e.kind() != ErrorKind::UnexpectedEof {
            return Err(e.into());
        }
    }

    if buf[0] == 0x0a {
        if pos == 0 {
            f.seek(SeekFrom::Start(0))?;
            return Ok(0);
        }
        f.seek(SeekFrom::Start(pos - 1))?;
        pos -= 1;
    } else {
        f.seek(SeekFrom::Start(pos))?;
    }

    loop {
        // If we're at the start we are by definition at the start of the line,
        // so just rewind the single-byte read we just did and return a 0
        // position.
        if pos == 0 {
            f.seek(SeekFrom::Start(0))?;
            return Ok(pos);
        }

        if let Err(e) = f.read_exact(&mut buf) {
            if e.kind() != ErrorKind::UnexpectedEof {
                return Err(e.into());
            }
        } else {
            // If we've read a newline character (0x0a), we've reached the start
            // of the line and can return the position we just read.
            if buf[0] == 0x0a {
                return Ok(pos + 1);
            }
        }

        // We haven't reached the start of the line, so we go back a byte and
        // start the loop again.
        pos -= 1;
        f.seek(SeekFrom::Start(pos))?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, Cursor, Seek, SeekFrom};
    use test_case::test_case;

    fn str_reader(s: &str) -> Cursor<&[u8]> {
        Cursor::new(s.as_bytes())
    }

    fn read_line(r: &mut impl BufRead) -> Result<String> {
        let mut buf = String::new();
        r.read_line(&mut buf)?;
        Ok(buf)
    }

    #[test_case("",                       0  => ""         ; "empty file")]
    #[test_case("line 1\nline 2\nline 3", 0  => "line 1\n" ; "start of first line")]
    #[test_case("line 1\nline 2\nline 3", 3  => "line 1\n" ; "middle of first line")]
    #[test_case("line 1\nline 2\nline 3", 6  => "line 1\n" ; "end of first line")]
    #[test_case("line 1\nline 2\nline 3", 7  => "line 2\n" ; "start of second line")]
    #[test_case("line 1\nline 2\nline 3", 12 => "line 2\n" ; "middle of second line")]
    #[test_case("line 1\nline 2\nline 3", 13 => "line 2\n" ; "end of second line")]
    #[test_case("line 1\nline 2\nline 3", 14 => "line 3"   ; "start of third line")]
    #[test_case("line 1\nline 2\nline 3", 15 => "line 3"   ; "middle of third line")]
    #[test_case("line 1\nline 2\nline 3", 19 => "line 3"   ; "end of third line")]
    #[test_case("line 1\nline 2\nline 3", 26 => "line 3"   ; "past eof")]
    fn test_seek_start_of_current_line(s: &str, pos: u64) -> String {
        let mut r = str_reader(s);
        r.seek(SeekFrom::Start(pos)).unwrap();
        seek_start_of_current_line(&mut r).unwrap();
        read_line(&mut r).unwrap()
    }

    #[test_case("line 1\nline 2\nline 3", 0  => Some(7)  ; "start of first line")]
    #[test_case("line 1\nline 2\nline 3", 2  => Some(7)  ; "middle of first line")]
    #[test_case("line 1\nline 2\nline 3", 6  => Some(7)  ; "end of first line")]
    #[test_case("line 1\nline 2\nline 3", 7  => Some(14) ; "start of second line")]
    #[test_case("line 1\nline 2\nline 3", 9  => Some(14) ; "middle of second line")]
    #[test_case("line 1\nline 2\nline 3", 13 => Some(14) ; "end of second line")]
    #[test_case("line 1\nline 2\nline 3", 14 => None     ; "start of last line")]
    #[test_case("line 1\nline 2\nline 3", 16 => None     ; "middle of last line")]
    #[test_case("line 1\nline 2\nline 3", 19 => None     ; "end of last line")]
    fn test_seek_start_of_next_line(s: &str, pos: u64) -> Option<u64> {
        let mut r = str_reader(s);
        r.seek(SeekFrom::Start(pos)).unwrap();
        seek_start_of_next_line(&mut r).unwrap()
    }

    #[test_case("line 1\nline 2\nline 3", 0  => None     ; "start of first line")]
    #[test_case("line 1\nline 2\nline 3", 2  => None     ; "middle of first line")]
    #[test_case("line 1\nline 2\nline 3", 1  => None     ; "second letter of first line")]
    #[test_case("line 1\nline 2\nline 3", 6  => None     ; "end of first line")]
    #[test_case("line 1\nline 2\nline 3", 7  => Some(0)  ; "start of second line")]
    #[test_case("line 1\nline 2\nline 3", 9  => Some(0)  ; "middle of second line")]
    #[test_case("line 1\nline 2\nline 3", 13 => Some(0)  ; "end of second line")]
    #[test_case("line 1\nline 2\nline 3", 14 => Some(7)  ; "start of last line")]
    #[test_case("line 1\nline 2\nline 3", 16 => Some(7)  ; "middle of last line")]
    #[test_case("line 1\nline 2\nline 3", 19 => Some(7)  ; "end of last line")]
    fn test_seek_start_of_prev_line(s: &str, pos: u64) -> Option<u64> {
        let mut r = str_reader(s);
        r.seek(SeekFrom::Start(pos)).unwrap();
        seek_start_of_prev_line(&mut r).unwrap()
    }

    #[test_case("a\nb\nc\nd\ne\nf\ng", "b", SeekType::First => Some(2)  ; "SeekType first: find line in middle of file")]
    #[test_case("a\nb\nc\nd\ne\nf\ng", "a", SeekType::First => Some(0)  ; "SeekType first: find first line")]
    #[test_case("a\nb\nc\nd\ne\nf\ng", "g", SeekType::First => Some(12) ; "SeekType first: find last line")]
    #[test_case("a\nb\nc\nd\ne\nf\ng", "h", SeekType::First => None     ; "SeekType first: seek past end of file")]
    #[test_case("a\nb\nc\nd\ne\nf\ng", "A", SeekType::First => Some(0)  ; "SeekType first: find prefix before first line")]
    #[test_case("a\nb\nb\nb\nb\nb\nc", "b", SeekType::First => Some(2)  ; "SeekType first: make sure we seek to the first occurrence")]
    #[test_case("b\nb\nb\nb\nb\nb\nc", "b", SeekType::First => Some(0)  ; "SeekType first: even if the first occurence is at the start of the file")]
    #[test_case("a\nb\nc\nd\ne\nf\ng", "b", SeekType::Last  => Some(2)  ; "SeekType last: find line in middle of file")]
    #[test_case("a\nb\nc\nd\ne\nf\ng", "a", SeekType::Last  => Some(0)  ; "SeekType last: find first line")]
    #[test_case("a\nb\nc\nd\ne\nf\ng", "g", SeekType::Last  => Some(12) ; "SeekType last: find last line")]
    #[test_case("a\nb\nc\nd\ne\nf\ng", "h", SeekType::Last  => None     ; "SeekType last: seek past end of file")]
    #[test_case("a\nb\nc\nd\ne\nf\ng", "A", SeekType::Last  => Some(0)  ; "SeekType last: find prefix before first line")]
    #[test_case("a\nb\nb\nb\nb\nb\nc", "b", SeekType::Last  => Some(10) ; "SeekType last: make sure we seek to the last occurrence")]
    #[test_case("b\nb\nb\nb\nb\nb\nb", "b", SeekType::Last  => Some(12) ; "SeekType last: even if the last occurence is at the start of the file")]
    fn test_seek(s: &str, prefix: &str, seek_type: SeekType) -> Option<u64> {
        let mut r = str_reader(s);
        seek(&mut r, prefix, seek_type).unwrap()
    }
}
