mod error;
mod index;

use crate::index::*;
use crate::index::tantivy_backend::*;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    fs::create_dir_all("./test_index").expect("failed to create directory for the index");
    let mut index =
        TantivyIndex::create(Path::new("./test_index")).expect("failed to create the TantivyIndex");

    // TODO: implement an indexer I suppose?

    let mut writer = index
        .get_document_writer()
        .expect("Failed to get a document writer");
    let mut keywords: Vec<String> = Vec::new();
    keywords.push("my key words".to_string());
    writer.add_document(&Document {
        keywords: keywords,
        metadata: DocumentMetadata {
            path: PathBuf::from("/test/hello/world"),
            size: 100,
        },
    });
    writer
        .commit()
        .expect("Failed to commit newly indexed documents. Uh oh.");

    println!("Hello world.")
}
