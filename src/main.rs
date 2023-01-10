#![allow(dead_code)]

pub mod config;
pub mod index;
pub mod indexer_worker;
pub mod webserver;

use crate::config::Config;
use crate::index::tantivy_backend::*;
use crate::index::*;
use crate::indexer_worker::{
    metadata_providers::BasicAttributesMetadataProvider, IndexerWorker, MetadataProvider,
};
use anyhow::Context;
use clap::{App, Arg, SubCommand};
use std::borrow::BorrowMut;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use std::{fs, io};

fn main() {
    let default_config_path = format!(
        "{}/.config/guidebook.yml",
        std::env::var("HOME").unwrap().as_str()
    );

    let mut app = App::new("guidebook")
        .version("0.1.0")
        .author("Gareth George")
        .about("Guidebook fs search indexes your filesystem over time and makes it searchable!")
        .arg(
            clap::arg!([config] "Path to the config file to use.")
                .long("config")
                .default_value(&default_config_path.as_str()),
        )
        .arg(
            Arg::new("update_index")
                .long("update_index")
                .help("Updates the index")
                .takes_value(false),
        )
        .subcommand(SubCommand::with_name("startweb").about("starts the web ui"))
        .subcommand(SubCommand::with_name("cli").about("starts the CLI search interface"));
    let m = app.clone().get_matches();

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

    // Run an indexing pass
    if m.is_present("update_index") {
        do_indexing(&config, index.as_writable());
    }

    if let Some(_) = m.subcommand_matches("cli") {
        search_cli(index.as_searchable());
    } else if let Some(_) = m.subcommand_matches("startweb") {
        webserver::set_state(Arc::new(index));
        webserver::serve();
    } else {
        app.print_help().unwrap();
    }
}

fn search_cli(index: &dyn SearchableIndex) {
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
}

fn do_indexing(config: &Config, index: &mut dyn WritableIndex) {
    let writer = &mut index
        .begin_add_documents()
        .expect("Failed to get a document writer");

    let mut paths: Vec<PathBuf> = Vec::new();
    for indexed_dir in &config.indexed_directories {
        paths.push(PathBuf::from(&indexed_dir.path));
    }

    let mut providers: Vec<Box<dyn MetadataProvider>> = Vec::new();
    providers.push(Box::new(BasicAttributesMetadataProvider::new()));

    let mut worker = IndexerWorker::create(&paths, providers);
    worker.index(writer.as_mut()).expect("failed to index");

    writer
        .commit()
        .expect("Failed to commit newly indexed documents. Uh oh.");
}
