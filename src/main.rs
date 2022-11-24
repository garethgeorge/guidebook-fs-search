#![allow(dead_code)]

pub mod config;
pub mod index;
pub mod indexer_worker;

use crate::config::Config;
use crate::index::tantivy_backend::*;
use crate::index::*;
use crate::indexer_worker::{
    metadata_providers::BasicAttributesMetadataProvider, IndexerWorker, MetadataProvider,
};
use anyhow::Context;
use clap::{App, Arg, SubCommand};
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::SystemTime;
use std::{fs, io};

fn main() {
    let default_config_path = format!(
        "{}/.config/guidebook.yml",
        std::env::var("HOME").unwrap().as_str()
    );

    let m = App::new("guidebook")
        .version("0.1.0")
        .author("Gareth George")
        .about("Guidebook fs search indexes your filesystem over time and makes it searchable!")
        .arg(
            clap::arg!([config] "Path to the config file to use.")
                .default_value(&default_config_path.as_str()),
        )
        .subcommand(
            SubCommand::with_name("startweb")
                .about("starts the web ui")
                .arg(
                    Arg::with_name("path")
                        .help("Path to directory to index")
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches();

    // Load configuration
    let config_path = PathBuf::from(m.value_of("config").unwrap());
    let config = Config::from_file(config_path.as_path())
        .context(format!(
            "Failed to load config from {}",
            &config_path.to_string_lossy()
        ))
        .unwrap();
    println!("Loaded config: {:?}", config);

    // Open the database (creating it if it does not exist)
    fs::create_dir_all(&config.database_location)
        .expect("failed to create directory for the index");
    let database_path = PathBuf::from(&config.database_location);
    let mut index =
        TantivyIndex::create(&database_path.as_path()).expect("failed to create the index");

    // Begin an indexing pass prior to launching the web UI (TODO: update index concurrently)
    {
        let writer = &mut index
            .begin_add_documents()
            .expect("Failed to get a document writer");

        let mut paths: Vec<PathBuf> = Vec::new();
        for indexed_dir in config.indexed_directories {
            paths.push(PathBuf::from(indexed_dir.path));
        }

        let mut providers: Vec<Box<dyn MetadataProvider>> = Vec::new();
        providers.push(Box::new(BasicAttributesMetadataProvider::new()));

        let mut worker = IndexerWorker::create(&paths, providers);
        worker.index(writer.as_mut()).expect("failed to index");

        writer
            .commit()
            .expect("Failed to commit newly indexed documents. Uh oh.");
    }

    let stdin = io::stdin();
    print!("query: ");
    let _ = std::io::stdout().flush();

    for line in stdin.lock().lines() {
        let line = line.unwrap();

        let now = SystemTime::now();

        println!("searching...");
        let documents = index
            .search(&line, 10, 0)
            .expect("failed to execute the query.");

        for document in documents {
            println!("{:?}", document);
        }

        println!("took: {} millis", now.elapsed().unwrap().as_millis());
        print!("query: ");
        let _ = std::io::stdout().flush();
    }

    println!("Hello world.")
}
