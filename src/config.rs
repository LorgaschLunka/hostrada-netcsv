use std::{fs};
use anyhow::Context;
use toml;
use serde::Deserialize;
use dirs;

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
    /// If a config file is found, will check and then use the one that exists.
    pub fn create_dir() -> anyhow::Result<()> {
        let config_dir = dirs::config_local_dir()
            .ok_or(anyhow::anyhow!("Could not determine local user config directory"))?
            .join(DEFAULT_CONFIG_DIR_NAME);
        
        fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create directory: {}", config_dir.display()))?;

        let file_path = config_dir.join("config.toml");

        if !fs::exists(&file_path)? {
            println!("No config file found.\nCreating default configuration file {}", &file_path.display());
            fs::write(&file_path, DEFAULT_CONFIG)?;
        } else {
            // if file exists: check if config options are correct (no check for correct link is implemented here, only origin)
            let temp = Self::load()?;
            let _ = chrono::DateTime::parse_from_rfc3339(&temp.origin)
                .with_context(|| format!("Invalid config: {} is not valid rfc3330", temp.origin))?;
        }

        Ok(())
        
    }

    /// Load config. Assumes that the config exists in the os user local config dir, will not create new
    pub fn load() -> anyhow::Result<Self> {
        let file_path = dirs::config_local_dir()
            .ok_or(anyhow::anyhow!("Could not determine local user config directory"))?
            .join(DEFAULT_CONFIG_DIR_NAME)
            .join("config.toml");

        let toml_content = fs::read_to_string(&file_path)
            .with_context(|| format!("Could not read contents of config file {}", &file_path.display()))?;
        
        let config: Self = toml::from_str(&toml_content)
            .with_context(|| format!("Failed to serialize toml content {toml_content}"))?;

        Ok(config)
    }

}