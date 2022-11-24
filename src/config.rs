use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/**
 * Guidebook's configuration
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub database_location: String,
    pub indexed_directories: Vec<IndexedDirectory>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexedDirectory {
    pub path: String,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Config> {
        println!("config path: {:?}", path);
        let extension = path.extension();
        if extension.is_none() || extension.unwrap_or_default() != "yml" {
            return Err(Error::msg(format!(
                "Config file must be a .yml file, found: {:?}",
                extension
            )));
        }

        let config: Config = serde_yaml::from_str(&fs::read_to_string(&path)?)?;
        return Ok(config);
    }
}
