use std::fs;
use toml;
use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct Config {
    pub origin: String,
    pub base_link: String,
}

impl Config {
    pub fn load(config_filepath: &str) -> Config {
        let toml_content = fs::read_to_string(config_filepath).expect("Failed to read config. Check for matching filepath.");
        
        let config: Self = toml::from_str(&toml_content).unwrap();

        config
    }
}