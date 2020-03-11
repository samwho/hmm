use super::{error::Error, Result};
use chrono::{DateTime, Utc};
use csv::StringRecord;
use std::convert::{TryFrom, TryInto};
use std::io::Write;

pub struct Entry {
    datetime: DateTime<Utc>,
    message: String,
}

impl Entry {
    pub fn with_message(message: &str) -> Self {
        Entry {
            datetime: Utc::now(),
            message: message.trim().to_owned(),
        }
    }

    pub fn datetime(&self) -> &DateTime<Utc> {
        &self.datetime
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn contains(&self, s: &str) -> bool {
        self.message.contains(s)
    }

    pub fn write(&self, w: impl Write) -> Result<()> {
        let mut writer = csv::Writer::from_writer(w);
        Ok(writer.write_record(&[
            self.datetime.to_rfc3339(),
            serde_json::to_string(&self.message)?,
        ])?)
    }
}

impl TryFrom<&StringRecord> for Entry {
    type Error = Error;

    fn try_from(sr: &StringRecord) -> Result<Self> {
        let date = sr
            .get(0)
            .ok_or_else(|| Error::StringError("malformed CSV".to_owned()))?;
        let msg = sr
            .get(1)
            .ok_or_else(|| Error::StringError("malformed CSV".to_owned()))?;

        Ok(Entry {
            datetime: chrono::DateTime::parse_from_rfc3339(date)?.into(),
            message: serde_json::from_str(&msg)?,
        })
    }
}

impl TryFrom<&str> for Entry {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        let mut record = csv::StringRecord::new();
        let mut reader_builder = csv::ReaderBuilder::new();
        reader_builder.has_headers(false);

        let mut r = reader_builder.from_reader(s.as_bytes());
        if !r.read_record(&mut record)? {
            return Err(Error::StringError("error parsing CSV row".to_owned()));
        }

        (&record).try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("2012-01-01T00:00:00+00:00,\"\"\"hello world\"\"\""   => ("2012-01-01T00:00:00+00:00".to_owned(), "hello world".to_owned()) ; "basic entry")]
    #[test_case("2012-01-01T00:00:00+00:00,\"\"\"hello\\nworld\"\"\"" => ("2012-01-01T00:00:00+00:00".to_owned(), "hello\nworld".to_owned()) ; "entry with newline")]
    #[test_case("2012-01-01T01:00:00+01:00,\"\"\"hello world\"\"\""   => ("2012-01-01T00:00:00+00:00".to_owned(), "hello world".to_owned()) ; "entry with non-UTC timezone")]
    #[test_case("2012-01-01T00:00:00+00:00,\"\"\"\"\"\""              => ("2012-01-01T00:00:00+00:00".to_owned(), "".to_owned()) ; "empty entry")]
    fn test_from_str(s: &str) -> (String, String) {
        let entry: Entry = s.try_into().unwrap();
        (entry.datetime().to_rfc3339(), entry.message().to_owned())
    }
}
