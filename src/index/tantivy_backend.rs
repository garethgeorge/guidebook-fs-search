use lmdb::LmdbResultExt;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{ReloadPolicy, TantivyError};

use crate::index::*;
use anyhow::{Context, Result};
use lmdb_zero as lmdb;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/**
 * Internal representation of the schema and fields that have been added to the index.
 */
struct TantivyIndexLayout {
    schema: tantivy::schema::Schema,
    field_title: tantivy::schema::Field,
    field_keyword: tantivy::schema::Field,
    field_path: tantivy::schema::Field,
}

/**
 * Tantivy based backend for search indexing of documents.
 */
pub struct TantivyIndex {
    path: PathBuf,
    path_index: PathBuf,

    layout: TantivyIndexLayout,
    index: tantivy::Index,
    lmdb_env: Arc<lmdb::Environment>,
    db_indexed_files: lmdb::Database<'static>,
}

impl TantivyIndex {
    pub fn create(dir: &Path) -> Result<TantivyIndex> {
        // setup directory structure
        let path_index = dir.join("index");
        if !path_index.exists() {
            fs::create_dir(&path_index)?
        }

        let path_kvstore = dir.join("kvstore");
        if !path_kvstore.exists() {
            fs::create_dir(&path_kvstore)?
        }

        // configure tantivy
        let mut schema_builder = tantivy::schema::Schema::builder();
        let field_title = schema_builder.add_text_field("title", tantivy::schema::TEXT);
        let field_keyword = schema_builder.add_text_field("keywords", tantivy::schema::TEXT);
        let field_path = schema_builder.add_facet_field(
            "path",
            tantivy::schema::FacetOptions::default()
                .set_stored()
                .set_indexed(),
        );
        let schema = schema_builder.build();

        let index = tantivy::Index::create_in_dir(&path_index, schema.clone())
            .or_else(|error| match error {
                TantivyError::IndexAlreadyExists => Ok(tantivy::Index::open_in_dir(&path_index)?),
                _ => Err(error),
            })
            .context("failed to open the tantivy index")?;

        // configure lmdb as a keyvalue store. We're joining the two databases here.
        let lmdb_env = Arc::new(unsafe {
            let GB = 1024 * 1024 * 1024;
            let mut builder = lmdb::EnvBuilder::new().unwrap();
            builder.set_maxdbs(2)?;
            builder.set_mapsize(128 * GB);
            builder
                .open(
                    &path_kvstore.to_string_lossy(),
                    lmdb::open::Flags::empty(),
                    0o600,
                )
                .context("failed to create the keyvalue store environment.")?
        });

        return Ok(TantivyIndex {
            path: PathBuf::from(dir),
            path_index: PathBuf::from(&path_index),

            layout: TantivyIndexLayout {
                field_title: field_title,
                field_keyword: field_keyword,
                field_path: field_path,
                schema: schema.clone(),
            },

            index: index,
            lmdb_env: lmdb_env.clone(),
            db_indexed_files: lmdb::Database::open(
                lmdb_env.clone(),
                Some("indexed_files"),
                &lmdb::DatabaseOptions::create_map::<str>(),
            )
            .context("failed to create keyvalue store tracking indexed files")?,
        });
    }
}

impl WritableIndex for TantivyIndex {
    fn begin_add_documents<'a>(&'a mut self) -> Result<Box<dyn IndexWriter + 'a>> {
        return Ok(Box::new(TantivyIndexWriter::create(self)?));
    }
}

impl SearchableIndex for TantivyIndex {
    fn search(
        &mut self,
        query: &str,
        result_limit: usize,
        result_offset: usize,
    ) -> Result<Vec<Document>> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()
            .unwrap();
        let searcher = reader.searcher();
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.layout.field_title, self.layout.field_keyword],
        );
        let query = query_parser.parse_query(query).unwrap();
        let top_docs = searcher
            .search(
                &query,
                &TopDocs::with_limit(result_limit).and_offset(result_offset),
            )
            .unwrap();

        let mut results: Vec<Document> = Vec::new();

        let indexed_files_read_txn = lmdb::ReadTransaction::new(self.lmdb_env.clone())?;
        let indexed_files_reader = indexed_files_read_txn.access();

        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address).unwrap();
            let path_field = retrieved_doc.get_first(self.layout.field_path).unwrap();
            let path = path_field.path().unwrap();

            let document_metadata_json: &str = indexed_files_reader
                .get(&&self.db_indexed_files, path.as_bytes())
                .expect("failed to find metadata for document. Index is in corrupt state.");

            let document: Document =
                serde_json::from_str(document_metadata_json).expect("failed to parse document");
            results.push(document);
        }

        return Ok(results);
    }
}

/**
 * Write handle on the tantivy index, allows for adding batches of documments and atomically committing them.
 */
struct TantivyIndexWriter<'a> {
    index: &'a TantivyIndex,
    tantivy_writer: tantivy::IndexWriter,
    indexed_files_txn: Option<Arc<lmdb::WriteTransaction<'static>>>,
}

impl TantivyIndexWriter<'_> {
    fn create(index: &mut TantivyIndex) -> Result<TantivyIndexWriter> {
        return Ok(TantivyIndexWriter {
            index: index,
            tantivy_writer: index.index.writer(50_000_000 /* 50 MB heap size */)?,
            indexed_files_txn: Some(Arc::new(lmdb::WriteTransaction::new(
                index.lmdb_env.clone(),
            )?)),
        });
    }
}

impl IndexWriter for TantivyIndexWriter<'_> {
    fn should_add_document(&mut self, path: &Path) -> bool {
        let reader = self
            .indexed_files_txn
            .as_ref()
            .expect("IndexWriter used after commit")
            .access();

        let doc: Option<&str> = reader
            .get(
                &(&self.index.db_indexed_files),
                path.to_string_lossy().as_bytes(),
            )
            .to_opt()
            .unwrap();
        return !doc.is_some();
    }

    fn add_document(&mut self, doc: &Document, keywords: &Vec<String>) -> Result<()> {
        // insert the full document in leveldb for later retrieval
        {
            let mut access = self
                .indexed_files_txn
                .as_ref()
                .expect("IndexWriter used after commit")
                .access();
            access
                .put(
                    &(&self.index.db_indexed_files),
                    doc.metadata.path.to_string_lossy().as_bytes(),
                    serde_json::to_string(&doc)?.as_bytes(),
                    lmdb::put::Flags::empty(),
                )
                .unwrap();
        }

        // create the tantivy document to insert
        let path = doc.metadata.path.to_string_lossy();
        let mut tantivy_doc = tantivy::doc! {
            self.index.layout.field_title => doc.title.clone()
        };
        tantivy_doc.add_facet(
            self.index.layout.field_path,
            tantivy::schema::Facet::from(&path),
        );

        for keyword in keywords {
            tantivy_doc.add_text(self.index.layout.field_keyword, keyword);
        }
        self.tantivy_writer.add_document(tantivy_doc);

        return Ok(());
    }

    fn commit(&mut self) -> Result<()> {
        self.tantivy_writer.commit()?;

        // we clear out the IndexWriter's handle on the WriteTransaction and then unwrap and commit the only remaining handle.
        let indexed_files_txn = self.indexed_files_txn.as_ref().unwrap().clone();
        self.indexed_files_txn = None;

        Arc::try_unwrap(indexed_files_txn).unwrap().commit()?;
        return Ok(());
    }
}
