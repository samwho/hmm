pub mod seek;
pub mod entry;
pub mod error;
pub mod format;
pub mod entries;

pub type Result<T> = std::result::Result<T, error::Error>;
