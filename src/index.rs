pub mod tantivy_backend;

use crate::error::GuidebookError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub trait WritableIndex {
    fn begin_add_documents(&mut self) -> Box<dyn IndexWriter>;
}

pub trait IndexWriter {
    fn should_add_document(&mut self, metadata: &DocumentMetadata) -> bool;
    fn add_document(&mut self, doc: &Document, keywords: &Vec<String>) -> ();
    fn commit(&mut self) -> Result<(), GuidebookError>;
}

/**
 * Represents only the metadata for a given document.
 */
#[derive(Serialize, Deserialize, PartialEq)]
pub struct DocumentMetadata {
    pub path: PathBuf,
    pub size: i64,
}

/**
 * Represents an entire document.
 */
#[derive(Serialize, Deserialize)]
pub struct Document {
    pub metadata: DocumentMetadata,
    pub title: String,
    pub preview_text: Option<String>, // a preview text to show for the document, recommended to be less than 200 chars.
    pub preview_img_path: Option<PathBuf>, // a preview image to show for the document.
}
