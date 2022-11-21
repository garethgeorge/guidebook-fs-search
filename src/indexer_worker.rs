use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::index::{Document, DocumentMetadata, IndexWriter};
use anyhow::{Context, Error, Result};
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
        // TODO: is there a better way than cloning the paths here?
        for path in self.paths.clone() {
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

        for provider in &self.metadata_providers {
            if let Some(metadata) = provider.provide_metadata(file) {
                if to.should_add_document(&metadata) {
                    document = Some(provider.document_for_metadata(&metadata)?);
                } else {
                    println!("skipped adding document, already indexed.");
                }
                break;
            }
        }

        if let Some(document) = document {
            to.add_document(&document.document.clone(), &Vec::new())
                .context("failed to add document to index writer transaction")?;
            println!(
                "indexed metadata for {} is {:?}",
                file.to_string_lossy(),
                &document.document
            );
            return Ok(Some(document.document));
        } else {
            println!("no metadata provided for {}", file.to_string_lossy());
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

// TODO(garethgeorge): replace &Path with a file trait that abstracts away the storage.
// TODO(garethgeorge): this interface is awkward, provider should not be tied into the implementation details of determining whether a file has been indexed.
pub trait MetadataProvider {
    fn provide_metadata(&self, path: &Path) -> Option<DocumentMetadata>;
    fn document_for_metadata(&self, metadata: &DocumentMetadata) -> Result<DocumentAndKeywords>;
}

pub mod metadata_providers {
    use crate::index::{Document, DocumentMetadata};
    use anyhow::Result;
    use std::{fs, path::Path};

    use super::{DocumentAndKeywords, MetadataProvider};

    pub struct DefaultMetadataProvider {}

    impl DefaultMetadataProvider {
        pub fn create() -> DefaultMetadataProvider {
            return DefaultMetadataProvider {};
        }
    }

    impl MetadataProvider for DefaultMetadataProvider {
        fn provide_metadata(&self, path: &Path) -> Option<DocumentMetadata> {
            return DocumentMetadata::from_path(path).ok();
        }

        fn document_for_metadata(
            &self,
            metadata: &DocumentMetadata,
        ) -> Result<DocumentAndKeywords> {
            let mut keywords: Vec<String> = Vec::new();

            if metadata.size < 1_000_000 {
                let contents = fs::read_to_string(&metadata.path).unwrap_or_default();

                if contents.is_ascii() {
                    keywords.push(contents);
                    println!("added extra keywords!");
                }
            }

            return Ok(DocumentAndKeywords {
                document: Document {
                    metadata: metadata.clone(),
                    title: String::from(metadata.path.to_string_lossy()),
                    preview_text: None,
                    preview_img_path: None,
                },
                keywords: keywords,
            });
        }
    }
}
