use std::default::Default;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use super::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub path: PathBuf,
    pub editor: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let dirs = directories::UserDirs::new()
            .expect("no home directory found, make sure your $HOME environment variable is set");

        Config {
            path: dirs.home_dir().to_path_buf(),
            editor: None,
        }
    }
}

pub fn get() -> Result<Config> {
    Ok(confy::load("hmm")?)
}

pub fn path() -> PathBuf {
    directories::ProjectDirs::from("rs", "hmm", "hmm").unwrap().config_dir().join("hmm.toml")
}