use super::{
    error::{self, Error},
    Result,
};
use chrono::prelude::*;
use csv::StringRecord;
use std::convert::{TryFrom, TryInto};
use std::io::Write;

pub struct Entry {
    datetime: DateTime<FixedOffset>,
    message: String,
}

impl Entry {
    pub fn new(datetime: DateTime<FixedOffset>, message: String) -> Self {
        Entry { datetime, message }
    }

    pub fn with_message(message: &str) -> Self {
        Self::new(Utc::now().into(), message.trim().to_owned())
    }

    pub fn datetime(&self) -> &DateTime<FixedOffset> {
        &self.datetime
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn contains(&self, s: &str) -> bool {
        self.message.contains(s)
    }

    pub fn write(&self, mut w: impl Write) -> Result<()> {
        Ok(w.write_all(self.to_csv_row()?.as_bytes())?)
    }

    pub fn to_csv_row(&self) -> Result<String> {
        let mut buf = Vec::new();
        {
            let mut writer = csv::Writer::from_writer(&mut buf);
            writer.write_record(&[
                self.datetime.to_rfc3339(),
                serde_json::to_string(&self.message)?,
            ])?;
        }
        Ok(String::from_utf8(buf)?)
    }
}

impl TryFrom<quick_csv::Row> for Entry {
    type Error = Error;

    fn try_from(r: quick_csv::Row) -> Result<Self> {
        let mut cols = r.columns()?;

        let date = cols
            .next()
            .ok_or_else(|| error::from_str("malformed CSV"))?;
        let msg = cols
            .next()
            .ok_or_else(|| error::from_str("malformed CSV"))?;

        Ok(Entry {
            datetime: chrono::DateTime::parse_from_rfc3339(date)?,
            message: serde_json::from_str(&msg)?,
        })
    }
}

impl TryFrom<&StringRecord> for Entry {
    type Error = Error;

    fn try_from(sr: &StringRecord) -> Result<Self> {
        let date = sr.get(0).ok_or_else(|| error::from_str("malformed CSV"))?;
        let msg = sr.get(1).ok_or_else(|| error::from_str("malformed CSV"))?;

        Ok(Entry {
            datetime: chrono::DateTime::parse_from_rfc3339(date)?,
            message: serde_json::from_str(&msg)?,
        })
    }
}

impl TryFrom<&str> for Entry {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        quick_csv::Csv::from_string(s).next().unwrap()?.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("2012-01-01T00:00:00+00:00,\"\"\"hello world\"\"\""   => ("2012-01-01T00:00:00+00:00".to_owned(), "hello world".to_owned()) ; "basic entry")]
    #[test_case("2012-01-01T00:00:00+00:00,\"\"\"hello\\nworld\"\"\"" => ("2012-01-01T00:00:00+00:00".to_owned(), "hello\nworld".to_owned()) ; "entry with newline")]
    #[test_case("2012-01-01T01:00:00+01:00,\"\"\"hello world\"\"\""   => ("2012-01-01T01:00:00+01:00".to_owned(), "hello world".to_owned()) ; "entry with non-UTC timezone")]
    #[test_case("2012-01-01T00:00:00+00:00,\"\"\"\"\"\""              => ("2012-01-01T00:00:00+00:00".to_owned(), "".to_owned()) ; "empty entry")]
    fn test_from_str(s: &str) -> (String, String) {
        let entry: Entry = s.try_into().unwrap();
        (entry.datetime().to_rfc3339(), entry.message().to_owned())
    }
}
