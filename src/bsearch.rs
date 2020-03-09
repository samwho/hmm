use std::fs::File;
use std::io::{Seek, SeekFrom, Read, ErrorKind};
use super::Result;
use std::cmp::Ordering;

pub fn seek_first<T>(f: &mut T, prefix: &str) -> Result<Option<u64>> 
where
  T: Seek + Read 
{
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

fn seek_start_of_line<T>(f: &mut T) -> Result<u64>
where
  T: Seek + Read 
{
    let mut buf = [0; 1];
    let mut pos = f.seek(SeekFrom::Current(0))?;

    f.read_exact(&mut buf)?;
    if buf[0] == 0x0a {
        if pos == 0 {
            return Ok(0);
        }
        f.seek(SeekFrom::Start(pos - 1))?;
        pos -= 1;
    }

    loop {
        if pos == 0 {
            return Ok(pos);
        }

        f.read_exact(&mut buf)?;
        if buf[0] == 0x0a {
            return Ok(pos + 1);
        }

        pos -= 1;
        f.seek(SeekFrom::Start(pos))?;
    }
}