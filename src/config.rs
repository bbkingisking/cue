use std::path::PathBuf;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use dirs::config_dir;
use thiserror::Error;

const CONFIG_FILENAME: &str = "config.toml";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub artists: Vec<Artist>
}

#[derive(Serialize, Deserialize)]
pub struct Artist {
    pub mbid: String,
    #[serde(default = "Artist::all_release_types")]
    pub release_types: Vec<ReleaseType>,
}

impl Artist {
    pub fn all_release_types() -> Vec<ReleaseType> {
        vec![
            ReleaseType::Album,
            ReleaseType::Single,
            ReleaseType::EP,
            ReleaseType::Live,
            ReleaseType::Compilation,
        ]
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum ReleaseType {
    Album,
    Single,
    #[serde(rename = "EP")]
    EP,
    Live,
    Compilation,
}

impl ReleaseType {
    pub fn as_str(&self) -> &str {
        match self {
            ReleaseType::Album => "Album",
            ReleaseType::Single => "Single",
            ReleaseType::EP => "EP",
            ReleaseType::Live => "Live",
            ReleaseType::Compilation => "Compilation",
        }
    }
}

impl Config {
    fn path() -> Result<PathBuf, ConfigError> {
        let xdg_conf = config_dir().ok_or(ConfigError::ConfigDirNotFound)?;
        let app_conf = xdg_conf.join(env!("CARGO_PKG_NAME"));
        Ok(app_conf)
    }

    pub fn load() -> Result<Config, ConfigError> {
        let path = Self::path()?.join(CONFIG_FILENAME);
        if !path.try_exists()? {
            return Err(ConfigError::ConfigNotFound)
        }
        let raw = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&raw)?)
    }

    pub fn create() -> Result<PathBuf, ConfigError> {
        let path = Self::path()?;
        std::fs::create_dir_all(&path)?;

        let sample_config = Config {
            artists: vec![Artist {
                mbid: "b9545342-1e6d-4dae-84ac-013374ad8d7c".to_string(),
                release_types: vec![ReleaseType::Album, ReleaseType::EP],
            }]
        };

        let sample_config_serialized = toml::to_string_pretty(&sample_config)?;

        std::fs::write(path.join(CONFIG_FILENAME), &sample_config_serialized)?;

        Ok(path)
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not find config dir.")]
    ConfigDirNotFound,
    #[error("Could not access config file.")]
    ConfigInaccessible(#[from] std::io::Error),
    #[error("Could not deserialize config.")]
    ConfigCouldNotDeserialize(#[from] toml::de::Error),
    #[error("Could not serialize config.")]
    ConfigCouldNotSerialize(#[from] toml::ser::Error),
    #[error("Config not found.")]
    ConfigNotFound,
}
