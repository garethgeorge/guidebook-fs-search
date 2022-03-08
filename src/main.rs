#![allow(dead_code)]

pub mod config;
pub mod error;
pub mod index;
pub mod indexer_worker;

use crate::config::Config;
use crate::index::tantivy_backend::*;
use crate::index::*;
use crate::indexer_worker::{
    metadata_providers::DefaultMetadataProvider, IndexerWorker, MetadataProvider,
};
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

    let mut paths: Vec<PathBuf> = Vec::new();
    paths.push(PathBuf::from(
        "/Users/garethgeorge/Documents/workspace/projects/guidebook-fs-search",
    ));

    let mut providers: Vec<Box<dyn MetadataProvider>> = Vec::new();
    providers.push(Box::new(DefaultMetadataProvider::create()));

    let mut worker = IndexerWorker::create(&paths, providers);
    worker.index(writer.as_mut());

    writer
        .commit()
        .expect("Failed to commit newly indexed documents. Uh oh.");

    println!("Hello world.")
}
