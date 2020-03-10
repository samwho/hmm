use std::io::{Seek, SeekFrom, Read, ErrorKind};
use super::Result;
use std::cmp::Ordering;

pub fn seek_first<T: Seek + Read>(f: &mut T, prefix: &str) -> Result<Option<u64>> {
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
        let mut line_start = seek_start_of_line(f)?;

        match f.read_exact(&mut buf) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    return Ok(None);
                } else {
                    return Err(e.into());
                }
            },
        }

        match bytes.cmp(&buf) {
            Ordering::Less => {
                end = cur - 1;
            },
            Ordering::Equal => {
                loop {
                    if line_start == 0 {
                        f.seek(SeekFrom::Start(line_start))?;
                        return Ok(Some(line_start));
                    }

                    f.seek(SeekFrom::Start(line_start - 1))?;
                    let new_start = seek_start_of_line(f)?;
                    f.read_exact(&mut buf)?;
                    if let Ordering::Greater = bytes.cmp(&buf) {
                        f.seek(SeekFrom::Start(line_start))?;
                        return Ok(Some(line_start));
                    } else {
                        line_start = new_start;
                    }
                }
            },
            Ordering::Greater => {
                start = cur + 1;
            },
        }
    }

    Ok(None)
}

fn seek_start_of_line<T: Seek + Read>(f: &mut T) -> Result<u64> {
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
    use std::io::{BufRead, Seek, SeekFrom, Cursor};
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
    fn test_seek_start(s: &str, pos: u64) -> String {
        let mut r = str_reader(s);
        r.seek(SeekFrom::Start(pos)).unwrap();
        seek_start_of_line(&mut r).unwrap();
        read_line(&mut r).unwrap()
    }
}