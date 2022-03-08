use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{error::GuidebookError, index::Document, index::IndexWriter};

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
    pub fn index(&mut self, to: &mut dyn IndexWriter) -> Result<(), GuidebookError> {
        // TODO: is there a better way than cloning the paths here?
        for path in self.paths.clone() {
            self.index_directory(path.as_path(), to)?;
        }
        return Ok(());
    }

    fn index_directory(
        &mut self,
        dir: &Path,
        to: &mut dyn IndexWriter,
    ) -> Result<(), GuidebookError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;

            self.index_file(entry.path().as_path(), to)?;
        }

        return Ok(());
    }

    fn index_file(
        &mut self,
        file: &Path,
        to: &mut dyn IndexWriter,
    ) -> Result<Option<Document>, GuidebookError> {
        // TODO: test that this only invokes up to the first metadata provider that actually returns a thing.
        let document = self
            .metadata_providers
            .iter()
            .map(|provider| {
                return provider.provide_metadata(file);
            })
            .filter(|item| item.is_some())
            .next();

        if let Some(Some(document)) = document {
            to.add_document(&document.clone(), &Vec::new());
            println!(
                "indexed metadata for {} is {:?}",
                file.to_string_lossy(),
                &document
            );
            return Ok(Some(document));
        } else {
            println!("no metadata provided for {}", file.to_string_lossy());
        }
        return Ok(None);
    }
}

/**
 * Provides metadata for a given path
 */
struct IndexableDocument {}

pub trait MetadataProvider {
    fn provide_metadata(&self, path: &Path) -> Option<Document>;
}

pub mod metadata_providers {
    use crate::{
        index::{Document, DocumentMetadata},
        indexer_worker::MetadataProvider,
    };
    use std::path::Path;

    pub struct DefaultMetadataProvider {}

    impl DefaultMetadataProvider {
        pub fn create() -> DefaultMetadataProvider {
            return DefaultMetadataProvider {};
        }
    }

    impl MetadataProvider for DefaultMetadataProvider {
        fn provide_metadata(&self, path: &Path) -> Option<Document> {
            return Some(Document {
                metadata: DocumentMetadata::from_path(path).ok()?,
                title: String::from(path.to_str()?),
                preview_text: None,
                preview_img_path: None,
            });
        }
    }
}
