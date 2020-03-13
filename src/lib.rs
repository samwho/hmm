pub mod bsearch;
pub mod config;
pub mod entry;
pub mod error;
pub mod format;

pub type Result<T> = std::result::Result<T, error::Error>;
