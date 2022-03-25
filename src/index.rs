pub mod tantivy_backend;

use crate::error::GuidebookError;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    vec,
};

pub trait WritableIndex {
    fn begin_add_documents(&mut self) -> Result<Box<dyn IndexWriter + '_>, GuidebookError>;
}

pub trait SearchableIndex {
    fn search<'a>(
        &'a mut self,
        query: &str,
        result_limit: usize
    ) -> Result<Vec<Document>, GuidebookError>;
}

pub trait IndexWriter {
    fn should_add_document(&mut self, metadata: &DocumentMetadata) -> bool;
    fn add_document(&mut self, doc: &Document, keywords: &Vec<String>) -> Result<(), GuidebookError>;
    fn commit(&mut self) -> Result<(), GuidebookError>;
}

/**
 * Represents only the metadata for a given document.
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DocumentMetadata {
    pub path: PathBuf,
    pub size: u64,
}

impl DocumentMetadata {
    pub fn from_path(path: &Path) -> Result<DocumentMetadata, GuidebookError> {
        let metadata = fs::metadata(path)?;

        return Ok(DocumentMetadata {
            path: PathBuf::from(path),
            size: metadata.len(),
        });
    }
}

/**
 * Represents an entire document.
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Document {
    pub metadata: DocumentMetadata,
    pub title: String,
    pub preview_text: Option<String>, // a preview text to show for the document, recommended to be less than 200 chars.
    pub preview_img_path: Option<PathBuf>, // a preview image to show for the document.
}
