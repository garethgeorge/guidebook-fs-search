pub mod tantivy_backend;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    vec,
};

pub trait WritableIndex {
    fn begin_add_documents(&mut self) -> Result<Box<dyn IndexWriter + '_>>;
}

// SearchableIndex represents an open index that can be searched, implementation
// must be safe to implement Sync + Send
pub trait SearchableIndex: Sync + Send {
    fn search(
        &self,
        query: &str,
        result_limit: usize,
        result_offset: usize,
    ) -> Result<Vec<Document>>;
}

pub trait Index: WritableIndex + SearchableIndex {
    fn as_writable(&mut self) -> &mut dyn WritableIndex;
    fn as_searchable(&self) -> &dyn SearchableIndex;
}
impl<T> Index for T
where
    T: WritableIndex + SearchableIndex,
{
    fn as_writable(&mut self) -> &mut dyn WritableIndex {
        self
    }
    fn as_searchable(&self) -> &dyn SearchableIndex {
        self
    }
}

pub trait IndexWriter {
    fn should_add_document(&mut self, path: &Path) -> bool;
    fn add_document(&mut self, doc: &Document, keywords: &Vec<String>) -> Result<()>;
    fn commit(&mut self) -> Result<()>;
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
    pub fn from_path(path: &Path) -> Result<DocumentMetadata> {
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
    // TODO: add keywords to the document.
    pub preview_text: Option<String>, // a preview text to show for the document, recommended to be less than 200 chars.
    pub preview_img_path: Option<PathBuf>, // a preview image to show for the document.
}
