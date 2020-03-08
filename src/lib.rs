pub mod error;
pub mod config;

pub type Result<T> = std::result::Result<T, error::Error>;