use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::index::{Document, IndexWriter};
use anyhow::{Context, Result};
use jwalk;

pub struct IndexerWorker {
    paths: Vec<PathBuf>,
    metadata_providers: Vec<Box<dyn MetadataProvider>>,
}

impl IndexerWorker {
    pub fn create(
        paths: &Vec<PathBuf>,
        metadata_providers: Vec<Box<dyn MetadataProvider>>,
    ) -> IndexerWorker {
        return IndexerWorker {
            paths: paths.clone(),
            metadata_providers: metadata_providers,
        };
    }

    /**
     * Runs an indexing pass writing to the IndexWriter
     */
    pub fn index(&mut self, to: &mut dyn IndexWriter) -> Result<()> {
        for path in &self.paths.clone() {
            for entry in jwalk::WalkDir::new(path) {
                let entry = entry?;

                println!("indexing {:?}", entry.path());

                if entry.file_type().is_dir() {
                    continue;
                }

                if entry.file_type().is_file() {
                    self.index_file(&entry.path().as_path(), to)
                        .context(format!("failed to index {:?}", &entry.path()))?;
                }
            }
        }
        return Ok(());
    }

    fn index_directory(&mut self, dir: &Path, to: &mut dyn IndexWriter) -> Result<()> {
        println!("indexing directory {:?}", dir);
        let dir_iterator = fs::read_dir(dir)
            .expect(format!("failed to read directory {}", dir.display()).as_str());

        for entry in dir_iterator {
            let entry = entry?;
            if entry.file_name().to_string_lossy().starts_with(".DS_Store") {
                continue;
            }

            if let Ok(filetype) = entry.file_type() {
                if filetype.is_file() {
                    self.index_directory(&entry.path(), to).context(format!(
                        "failed to index file {}",
                        &entry.path().to_string_lossy(),
                    ))?;
                } else if filetype.is_dir() {
                    self.index_directory(&entry.path(), to).context(format!(
                        "failed to index directory {}",
                        &entry.path().to_string_lossy(),
                    ))?;
                }
            }
        }

        return Ok(());
    }

    fn index_file(&mut self, file: &Path, to: &mut dyn IndexWriter) -> Result<Option<Document>> {
        // TODO: test that this only invokes up to the first metadata provider that actually returns a thing.
        let mut document: Option<DocumentAndKeywords> = None;

        println!("file {}", file.to_string_lossy());

        if !to.should_add_document(file) {
            println!("skipping indexing file, already indexed.");
            return Ok(None);
        }

        for provider in self.metadata_providers.iter_mut() {
            let result = provider.index_document(file)?;
            if result.is_some() {
                document = result;
                break;
            }
        }

        if let Some(document) = document {
            to.add_document(&document.document.clone(), &document.keywords)
                .context("failed to add document to index writer transaction")?;
            println!(
                "indexed metadata for {:?} is {:?}",
                file, &document.document
            );
            return Ok(Some(document.document));
        } else {
            println!("no indexer provided metadata for {:?}", file);
        }
        return Ok(None);
    }
}

/**
 * Provides metadata for a given path
 */
pub struct DocumentAndKeywords {
    document: Document,
    keywords: Vec<String>,
}

pub trait MetadataProvider {
    // TODO(garethgeorge): provide a caching layer that provides basic file information to avoid each indexer querying the filesystem.
    // TODO(garethgeorge): replace &Path with a file trait that abstracts away the storage.
    fn index_document(&self, path: &Path) -> Result<Option<DocumentAndKeywords>>;
}

pub mod metadata_providers {
    use crate::index::{Document, DocumentMetadata};
    use anyhow::Result;
    use std::{
        fs::{self, Metadata},
        path::Path,
    };

    use super::{DocumentAndKeywords, MetadataProvider};

    pub struct BasicAttributesMetadataProvider {}

    impl BasicAttributesMetadataProvider {
        pub fn new() -> BasicAttributesMetadataProvider {
            return BasicAttributesMetadataProvider {};
        }
    }

    impl MetadataProvider for BasicAttributesMetadataProvider {
        fn index_document(&self, path: &Path) -> Result<Option<DocumentAndKeywords>> {
            let metadata = DocumentMetadata::from_path(path)?;

            let mut keywords: Vec<String> = Vec::new();

            // only attempt fulltext indexing of documents less than 100KB.
            if metadata.size < 100_000 {
                let contents = fs::read_to_string(&metadata.path).unwrap_or_default();

                if contents.is_ascii() {
                    keywords.push(contents);
                    println!("added extra keywords!");
                }
            }

            return Ok(Some(DocumentAndKeywords {
                document: Document {
                    metadata: metadata.clone(),
                    title: String::from(metadata.path.to_string_lossy()),
                    preview_text: None,
                    preview_img_path: None,
                },
                keywords: keywords,
            }));
        }
    }
}
