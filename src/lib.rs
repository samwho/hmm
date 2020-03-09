pub mod error;
pub mod config;
pub mod bsearch;

pub type Result<T> = std::result::Result<T, error::Error>;