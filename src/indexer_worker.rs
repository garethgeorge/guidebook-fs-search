use crate::index::{IndexWriter};


pub struct IndexerWorker {
    paths: Vec<PathBuf>,
    metadata_providers: Vec<Box<dyn MetadataProvider>>,
}

impl IndexerWorker {
    pub fn create(paths: &Vec<PathBuf>, metadata_providers: Vec<Box<dyn MetadataProvider>>) {
        return IndexerWorker {
            paths: paths.clone(),
            metadata_providers: metadata_providers,
        }
    }

    pub fn index(&mut self, to: IndexWriter) {
        for path in paths {
            self.
        }
    }
}

/**
 * Provides metadata for a given path
 */
pub trait MetadataProvider {
    fn provide_metadata(path: PathBuf): Option<index::Document>;
}