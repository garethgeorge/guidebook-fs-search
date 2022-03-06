use std::time::SystemTimeError;
use std::{fmt, io};
use tantivy::error::TantivyError;
use thiserror::Error;

pub type Result<T, E = GuidebookError> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct ValidationError(pub String);

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ValidationError {}

/**
 * Common error type for Guidebook.
 */
#[derive(Debug, Error)]
pub enum GuidebookError {
    #[error("An IO error occurred: '{0}'")]
    IoError(#[from] io::Error),

    #[error("A TantivyError error occurred: '{0}'")]
    TantivyError(#[from] TantivyError),

    #[error("An IO error occurred: '{0}'")]
    SystemTimeError(#[from] SystemTimeError),

    #[error("Serde YML parsing error: '{0}'")]
    SerdeYmlError(#[from] serde_yaml::Error),

    #[error("Validation error: '{0}'")]
    ValidationError(#[from] ValidationError),
}
