use std::{error, fmt, io};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Csv(csv::Error),
    ChronoParse(chrono::format::ParseError),
    SerdeJson(serde_json::error::Error),
    TomlDeserialize(toml::de::Error),
    TomlSerialize(toml::ser::Error),
    StringError(String),
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Csv(ref err) => Some(err),
            Error::ChronoParse(ref err) => Some(err),
            Error::SerdeJson(ref err) => Some(err),
            Error::TomlDeserialize(ref err) => Some(err),
            Error::TomlSerialize(ref err) => Some(err),
            Error::StringError(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::Csv(ref err) => err.fmt(f),
            Error::ChronoParse(ref err) => err.fmt(f),
            Error::SerdeJson(ref err) => err.fmt(f),
            Error::TomlDeserialize(ref err) => err.fmt(f),
            Error::TomlSerialize(ref err) => err.fmt(f),
            Error::StringError(ref s) => f.write_str(s),
        }
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Error {
        Error::TomlSerialize(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::TomlDeserialize(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Error {
        Error::Csv(err)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Error {
        Error::SerdeJson(err)
    }
}

impl From<chrono::format::ParseError> for Error {
    fn from(err: chrono::format::ParseError) -> Error {
        Error::ChronoParse(err)
    }
}
