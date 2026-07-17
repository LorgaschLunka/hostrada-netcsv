use std::{fs};
use toml;
use serde::Deserialize;
use dirs;

use crate::error::ConfigError;

const DEFAULT_CONFIG: &str = r##"
# days since origin is the time unit of hostrada data. As of now, the origin in hostrada netcdf files is 1949-12-01
# check with hostrada-netcsv origin
origin = "1949-12-01T00:00:00+00:00"
# the base link for the hourly data
base_link = "https://opendata.dwd.de/climate_environment/CDC/grids_germany/hourly/hostrada/"
"##;

const DEFAULT_CONFIG_DIR_NAME: &str = "hostrada-netcsv";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub origin: String,
    pub base_link: String,
}

impl Config {
    /// Load config. If no config file is found, this will create a new config with default values
    /// If a config file is found, will use the one that exists.
    pub fn create_dir() -> Result<(), ConfigError> {
        let config_dir = dirs::config_local_dir()
            .ok_or(ConfigError::DirNotFound)?
            .join(DEFAULT_CONFIG_DIR_NAME);
        
        fs::create_dir_all(&config_dir)?;

        let file_path = config_dir.join("config.toml");

        if !fs::exists(&file_path)? {
            println!("No config file found.\nCreating default configuration file {}", &file_path.display());
            fs::write(&file_path, DEFAULT_CONFIG)?;
        }

        Ok(())
        
    }

    /// Load config. Assumes that the config exists in the os user local config dir, will not create new
    pub fn load() -> Result<Self, ConfigError> {
        let file_path = dirs::config_local_dir()
            .ok_or(ConfigError::DirNotFound)?
            .join(DEFAULT_CONFIG_DIR_NAME);

        let toml_content = fs::read_to_string(file_path)?;
        
        let config: Self = toml::from_str(&toml_content)?;

        Ok(config)
    }
}