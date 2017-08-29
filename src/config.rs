//! Config Module
//!
//! This module houses functionality for parsing repo_config.toml
//! and matches this config against the incoming POST for issuing cmds.

use std::{error, io};
use std::collections::BTreeMap;
use std::convert::AsRef;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use toml;


//struct for deserializing "repo_config.toml" from repo root
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub repositories: BTreeMap<String, String>,
}

//I don't believe it's proper to panic! as below. Seeking input regarding the best approach.
impl Config {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
        let mut f = match File::open(path.as_ref()) {
            Err(e) => return Err(ConfigError::IO(e)),
            Ok(file) => file,
        };
        let mut buf = String::new();
        match f.read_to_string(&mut buf) {
            Err(e) => return Err(ConfigError::IO(e)),
            Ok(buf) => buf,
        };
        let config: Config = match toml::from_str(&buf) {
            Ok(btree) => btree,
            Err(e) => return Err(ConfigError::BadConfig),
        };
        Ok(Config { repositories: config.repositories })
    }
}
#[derive(Debug)]
pub enum ConfigError {
    IO(io::Error),
    BadConfig,
}

impl error::Error for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::IO(ref e) => "I/O Error!",
            ConfigError::BadConfig => "Problem within repo_config.toml!",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ConfigError::IO(ref e) => Some(e as &error::Error),
            _ => None,
        }
    }
}
impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ConfigError::IO(ref e) => write!(f, "{}", &e),
            ConfigError::BadConfig => write!(f, "Bad config!"),
        }
    }
}
