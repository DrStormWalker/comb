mod device;
mod monitor;

use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub use monitor::watch;
use serde::{Deserialize, Serialize};
use xdg::BaseDirectoriesError;

use self::device::Device;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub devices: Vec<Device>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    BaseDirectoriesError(#[from] BaseDirectoriesError),
    // #[error(transparent)]
    // TomlError(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn load() -> Result<(PathBuf, Config)> {
    let config_path = match get_config_file_path()? {
        Some(config_path) => config_path,
        None => {
            println!("Unable to find config file. Generating default.");

            create_config_file()?
        }
    };

    let mut config_file = File::open(&config_path)?;

    let mut config = String::new();

    config_file.read_to_string(&mut config)?;

    let config = toml::from_str(&config).unwrap_or_else(|err| {
        println!("Failed to load config file. Using default");
        println!("{}", err);

        Config::default()
    });

    Ok((config_path, config))
}

pub fn reload(config_path: impl AsRef<Path>) -> Result<Option<Config>> {
    let mut config_file = File::open(config_path)?;

    let mut config = String::new();

    config_file.read_to_string(&mut config)?;

    let config = match toml::from_str(&config) {
        Ok(config) => config,
        Err(err) => {
            println!("Failed to load config file, using previous version.");
            println!("{}", err);

            return Ok(None);
        }
    };

    Ok(Some(config))
}

fn get_config_file_path() -> Result<Option<PathBuf>> {
    xdg::BaseDirectories::with_prefix("comb")
        .ok()
        .and_then(|xdg| xdg.find_config_file("config.toml").map(|file| Ok(file)))
        .transpose()
}

fn create_config_file() -> Result<PathBuf> {
    xdg::BaseDirectories::with_prefix("comb")
        .map_err(|err| err.into())
        .and_then(|xdg| {
            let config_path = xdg.place_config_file("config.toml")?;

            Ok(config_path)
        })
        .and_then(|config_path| {
            // File::create_new is unstable
            // (issue #105135 https://github.com/rust-lang/rust/issues/105135)
            File::create_new(&config_path)?;

            Ok(config_path)
        })
}
