use super::{error::Error, Result};
use chrono::{DateTime, Utc};
use csv::StringRecord;
use std::convert::TryFrom;

pub struct Entry {
    datetime: DateTime<Utc>,
    message: String,
}

impl Entry {
    pub fn datetime(&self) -> &DateTime<Utc> {
        &self.datetime
    }

    pub fn message(&self) -> &str {
        &self.message
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
