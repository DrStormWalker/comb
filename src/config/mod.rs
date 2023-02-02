mod monitor;

pub use monitor::watch;

use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub imports: Option<Vec<PathBuf>>,

    #[serde(default)]
    pub errors: Errors,

    #[serde(default)]
    pub devices: Vec<Device>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    #[serde(flatten)]
    pub accessor: DeviceAccessor,

    pub alias: Option<Alias>,

    #[serde(rename = "virtual", skip_serializing_if = "skip_bool_if_false")]
    pub is_virtual: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Alias {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceAccessor {
    Path(PathBuf),
    Name(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    Error,
    Warning,
    Ignore,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Errors {
    unsupported_option: ErrorType,
}
impl Default for Errors {
    fn default() -> Self {
        Self {
            unsupported_option: ErrorType::Error,
        }
    }
}

fn skip_bool_if_false(b: &bool) -> bool {
    !b
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error(transparent)]
    TomlError(#[from] toml::de::Error),
}

pub fn load_config_from_path(p: impl AsRef<Path>) -> Result<Config, Error> {
    let mut f = File::open(p)?;

    load_config_from_file(&mut f)
}

pub fn load_config_from_file(f: &mut File) -> Result<Config, Error> {
    let mut s = String::new();

    f.read_to_string(&mut s)?;

    load_config_from_str(&s)
}

pub fn load_config_from_str(s: &str) -> Result<Config, Error> {
    let config: Config = toml::from_str(s)?;

    Ok(config)
}
