use super::error::Error;
use super::Result;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub path: Option<PathBuf>,
    pub date_format: Option<String>,
    pub editor: Option<String>,
}

impl Config {
    pub fn read() -> Result<Config> {
        let path = directories::ProjectDirs::from("sh", "hmm", "hmm")
            .unwrap()
            .config_dir()
            .join("config.toml");
        Self::read_from(&path)
    }

    pub fn read_from(path: &PathBuf) -> Result<Config> {
        if !path.exists() {
            return Ok(Config {
                ..Default::default()
            });
        }

        let mut s = String::new();
        File::open(path)?.read_to_string(&mut s)?;
        let config: Config = toml::from_str(&s)?;
        Ok(config)
    }

    pub fn path(&self) -> Result<PathBuf> {
        let p = match &self.path {
            Some(ref path) => path.to_owned(),
            None => {
                let dirs = directories::UserDirs::new().unwrap();
                let path = dirs.home_dir().to_path_buf().join(".hmm");

                if path.is_dir() {
                    return Err(Error::StringError(format!(
                        "\"{}\" is a directory and can't be used as the file hmm writes to",
                        path.to_string_lossy()
                    )));
                }

                path
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
            Err(Error::StringError(format!("unable to find an editor, set your EDITOR environment variable or add a line like `editor = \"nano\"` to your config at {}", self.path()?.to_str().unwrap())))
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
