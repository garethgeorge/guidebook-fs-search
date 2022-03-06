#![allow(dead_code)]

pub mod config;
pub mod error;
pub mod index;

use crate::index::tantivy_backend::*;
use crate::index::*;
use crate::config::Config;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let config = Config::from_file(&Path::new("./config.yml")).expect("failed to load config");
    println!("Loaded config: {:?}", config);

    fs::create_dir_all("./test_index").expect("failed to create directory for the index");
    let mut index =
        TantivyIndex::create(Path::new("./test_index")).expect("failed to create the TantivyIndex");

    

    let mut writer = index
        .get_document_writer()
        .expect("Failed to get a document writer");
    let mut keywords: Vec<String> = Vec::new();
    keywords.push("my key words".to_string());
    writer.add_document(&Document {
        metadata: DocumentMetadata {
            path: PathBuf::from("/test/hello/world"),
            size: 100,
        },
        title: "world".to_string(),
        preview_text: None,
        preview_img_path: None,
    }, &keywords);
    writer
        .commit()
        .expect("Failed to commit newly indexed documents. Uh oh.");

    println!("Hello world.")
}
