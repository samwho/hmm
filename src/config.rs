use std::default::Default;
use std::path::PathBuf;
use crate::error::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            path: dirs::home_dir().expect("no home directory found, can't run without one").join(".hmm")
        }
    }
}

pub fn get() -> Result<Config, Error> {
    Ok(confy::load("hmm")?)
}