use std::io;
use std::time::SystemTimeError;
use tantivy::error::TantivyError;
use thiserror::Error;

pub type Result<T, E = GuidebookError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum GuidebookError {
    #[error("An IO error occurred: '{0}'")]
    IoError(#[from] io::Error),

    #[error("A TantivyError error occurred: '{0}'")]
    TantivyError(#[from] TantivyError),

    #[error("An IO error occurred: '{0}'")]
    SystemTimeError(#[from] SystemTimeError),
}
