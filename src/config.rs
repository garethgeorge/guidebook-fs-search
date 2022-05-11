use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path};
use crate::error::{GuidebookError, ValidationError};

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
    pub fn from_file(path: &Path) -> Result<Config, GuidebookError> {
        println!("config path: {:?}", path);
        let extension = path.extension();
        if extension.is_none() || extension.unwrap_or_default() != "yml" {
            return Err(GuidebookError::from(ValidationError("config filepath must end with .yml".to_string())));
        }

        let config: Config = serde_yaml::from_str(&fs::read_to_string(&path)?)?;
        return Ok(config);
    }
}