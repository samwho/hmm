use super::Result;
use std::io::{ErrorKind, Read, Seek, SeekFrom};

pub fn start_of_next_line<T: Seek + Read>(f: &mut T) -> Result<Option<u64>> {
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

pub fn start_of_prev_line<T: Seek + Read>(f: &mut T) -> Result<Option<u64>> {
    start_of_current_line(f)?;

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

pub fn start_of_current_line<T: Seek + Read>(f: &mut T) -> Result<u64> {
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

    fn read_line(r: &mut impl BufRead) -> Result<String> {
        let mut buf = String::new();
        r.read_line(&mut buf)?;
        Ok(buf)
    }

    #[test_case("",                         0  => ""         ; "empty file")]
    #[test_case("line 1\nline 2\nline 3",   0  => "line 1\n" ; "start of first line")]
    #[test_case("line 1\nline 2\nline 3",   3  => "line 1\n" ; "middle of first line")]
    #[test_case("line 1\nline 2\nline 3",   6  => "line 1\n" ; "end of first line")]
    #[test_case("line 1\nline 2\nline 3",   7  => "line 2\n" ; "start of second line")]
    #[test_case("line 1\nline 2\nline 3",   12 => "line 2\n" ; "middle of second line")]
    #[test_case("line 1\nline 2\nline 3",   13 => "line 2\n" ; "end of second line")]
    #[test_case("line 1\nline 2\nline 3",   14 => "line 3"   ; "start of third line")]
    #[test_case("line 1\nline 2\nline 3",   15 => "line 3"   ; "middle of third line")]
    #[test_case("line 1\nline 2\nline 3",   19 => "line 3"   ; "end of third line")]
    #[test_case("line 1\nline 2\nline 3",   26 => "line 3"   ; "past eof")]
    #[test_case("line 1\nline 2\nline 3\n", 20 => "line 3\n" ; "last line when line ends with eof")]
    fn test_start_of_current_line(s: &str, pos: u64) -> String {
        let mut r = Cursor::new(s.as_bytes());
        r.seek(SeekFrom::Start(pos)).unwrap();
        start_of_current_line(&mut r).unwrap();
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
    fn test_start_of_next_line(s: &str, pos: u64) -> Option<u64> {
        let mut r = Cursor::new(s.as_bytes());
        r.seek(SeekFrom::Start(pos)).unwrap();
        start_of_next_line(&mut r).unwrap()
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
    fn test_start_of_prev_line(s: &str, pos: u64) -> Option<u64> {
        let mut r = Cursor::new(s.as_bytes());
        r.seek(SeekFrom::Start(pos)).unwrap();
        start_of_prev_line(&mut r).unwrap()
    }
}
