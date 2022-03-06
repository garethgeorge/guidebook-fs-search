pub struct IndexerWorker {

}

impl IndexerWorker {
    fn create(paths: Vec<PathBuf>, metadata_providers: Vec<Box<dyn MetadataProvider>>) {
        
    }
}

/**
 * Provides metadata for a given path
 */
pub trait MetadataProvider {
    fn provide_metadata(path: PathBuf): Option<index::Document>;
}