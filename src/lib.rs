pub mod entries;
pub mod entry;
pub mod error;
pub mod format;
pub mod seek;

pub type Result<T> = std::result::Result<T, error::Error>;
