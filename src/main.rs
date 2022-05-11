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
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::{fs, io};

fn main() {
    let config = Config::from_file(&Path::new("./config.yml")).expect("failed to load config");
    println!("Loaded config: {:?}", config);

    fs::create_dir_all("./test_index").expect("failed to create directory for the index");
    let mut index =
        TantivyIndex::create(Path::new("./test_index")).expect("failed to create the index");

    {
        let mut writer = &mut index
            .begin_add_documents()
            .expect("Failed to get a document writer");

        let mut paths: Vec<PathBuf> = Vec::new();
        for indexed_dir in config.indexed_directories {
            paths.push(PathBuf::from(indexed_dir.path));
        }

        let mut providers: Vec<Box<dyn MetadataProvider>> = Vec::new();
        providers.push(Box::new(DefaultMetadataProvider::create()));

        let mut worker = IndexerWorker::create(&paths, providers);
        worker.index(writer.as_mut()).expect("failed to index");

        writer
            .commit()
            .expect("Failed to commit newly indexed documents. Uh oh.");
    }

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();

        println!("searching...");
        let documents = index
            .search(&line, 10)
            .expect("failed to execute the query.");

        for document in documents {
            println!("{:?}", document);
        }
    }

    println!("Hello world.")
}
