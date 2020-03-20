use std::{error, fmt, io};

pub fn from_str(s: &str) -> Error {
    s.to_owned().into()
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Csv(csv::Error),
    QuickCsv(quick_csv::error::Error),
    ChronoParse(chrono::format::ParseError),
    SerdeJson(serde_json::error::Error),
    Template(handlebars::TemplateError),
    TemplateRender(handlebars::TemplateRenderError),
    Render(handlebars::RenderError),
    Utf8(std::string::FromUtf8Error),
    Regex(regex::Error),
    String(String),
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Csv(ref err) => Some(err),
            Error::QuickCsv(ref err) => Some(err),
            Error::ChronoParse(ref err) => Some(err),
            Error::SerdeJson(ref err) => Some(err),
            Error::Template(ref err) => Some(err),
            Error::TemplateRender(ref err) => Some(err),
            Error::Render(ref err) => Some(err),
            Error::Utf8(ref err) => Some(err),
            Error::Regex(ref err) => Some(err),
            Error::String(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::Csv(ref err) => err.fmt(f),
            Error::QuickCsv(ref err) => err.fmt(f),
            Error::ChronoParse(ref err) => err.fmt(f),
            Error::SerdeJson(ref err) => err.fmt(f),
            Error::Template(ref err) => err.fmt(f),
            Error::TemplateRender(ref err) => err.fmt(f),
            Error::Render(ref err) => err.fmt(f),
            Error::Utf8(ref err) => err.fmt(f),
            Error::Regex(ref err) => err.fmt(f),
            Error::String(ref s) => f.write_str(s),
        }
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Error {
        Error::String(s.to_owned())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::String(s)
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Error {
        Error::Regex(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Error {
        Error::Utf8(err)
    }
}

impl From<handlebars::RenderError> for Error {
    fn from(err: handlebars::RenderError) -> Error {
        Error::Render(err)
    }
}

impl From<handlebars::TemplateRenderError> for Error {
    fn from(err: handlebars::TemplateRenderError) -> Error {
        Error::TemplateRender(err)
    }
}

impl From<handlebars::TemplateError> for Error {
    fn from(err: handlebars::TemplateError) -> Error {
        Error::Template(err)
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

impl From<quick_csv::error::Error> for Error {
    fn from(err: quick_csv::error::Error) -> Error {
        Error::QuickCsv(err)
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
