use super::error::Error;
use super::Result;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    path: Option<PathBuf>,
    date_format: Option<String>,
    editor: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        confy::load("hmm").unwrap()
    }
}

impl Config {
    pub fn path(&self) -> Result<PathBuf> {
        let p = match &self.path {
            Some(ref path) => path.to_owned(),
            None => {
                let dirs = directories::UserDirs::new().unwrap();
                dirs.home_dir().to_path_buf()
            }
        };
        Ok(p)
    }

    pub fn editor(&self) -> Result<String> {
        if let Some(editor) = &self.editor {
            Ok(editor.to_owned())
        } else if let Ok(editor) = env::var("EDITOR") {
            Ok(editor)
        } else {
            Err(Error::StringError(format!("unable to find an editor, set your EDITOR environment variable or add a line like `editor = \"nano\"` to your config at {}", path().to_str().unwrap())))
        }
    }

    pub fn date_format(&self) -> String {
        if let Some(date_format) = &self.date_format {
            date_format.to_owned()
        } else {
            "%Y-%m-%d %H:%M".to_owned()
        }
    }
}

pub fn get() -> Result<Config> {
    Ok(confy::load("hmm")?)
}

pub fn path() -> PathBuf {
    directories::ProjectDirs::from("rs", "hmm", "hmm")
        .unwrap()
        .config_dir()
        .join("hmm.toml")
}
