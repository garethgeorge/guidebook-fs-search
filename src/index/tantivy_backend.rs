use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{ReloadPolicy, TantivyError};

use crate::error::GuidebookError;
use crate::index::*;
use std::fs;
use std::path::{Path, PathBuf};

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
    kvstore: rocksdb::DB,
}

impl TantivyIndex {
    pub fn create(dir: &Path) -> Result<TantivyIndex, GuidebookError> {
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
            .expect("failed to open the search index.");

        let db = rocksdb::DB::open_default(path_kvstore).expect("failed to open keyvalue store.");

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
            kvstore: db,
        });
    }
}

impl WritableIndex for TantivyIndex {
    fn begin_add_documents<'a>(&'a mut self) -> Result<Box<dyn IndexWriter + 'a>, GuidebookError> {
        return Ok(Box::new(TantivyIndexWriter::create(self)?));
    }
}

impl SearchableIndex for TantivyIndex {
    fn search(
        &mut self,
        query: &str,
        result_limit: usize,
    ) -> Result<Vec<Document>, GuidebookError> {
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
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

        let mut results: Vec<Document> = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address).unwrap();
            let path_field = retrieved_doc.get_first(self.layout.field_path).unwrap();
            let path = path_field.path().unwrap();

            // documents can not exist in tantivy without a matching instance in rocksdb
            // TODO(garethgeorge): should we have error handling here?
            if let Some(document_json_bytes) = self
                .kvstore
                .get(path.as_bytes())
                .expect("failed to access kvstore")
            {
                let document: Document = serde_json::from_str(&String::from_utf8_lossy(&document_json_bytes)).expect("failed to parse document");
                results.push(document);
            } else {
                println!("warning: no document in self.kvstore for {}", path);
            }
        }

        return Ok(results);
    }
}

/**
 * Write handle on the tantivy index, allows for adding batches of documments and atomically committing them.
 */
struct TantivyIndexWriter<'a> {
    writer: tantivy::IndexWriter,
    index: &'a TantivyIndex,
}

impl TantivyIndexWriter<'_> {
    fn create(index: &mut TantivyIndex) -> Result<TantivyIndexWriter, GuidebookError> {
        let writer = index.index.writer(50_000_000 /* 50 MB heap size */)?;
        return Ok(TantivyIndexWriter {
            writer: writer,
            index: index,
        });
    }
}

impl IndexWriter for TantivyIndexWriter<'_> {
    fn should_add_document(&mut self, _metadata: &DocumentMetadata) -> bool {
        return true; // TODO: add already indexed detection. For now: wipeout index between runs.
    }

    fn add_document(
        &mut self,
        doc: &Document,
        keywords: &Vec<String>,
    ) -> Result<(), GuidebookError> {
        // insert the full document in leveldb for later retrieval
        self.index.kvstore.put(
            doc.metadata.path.to_string_lossy().as_bytes(),
            serde_json::to_string(&doc)?.as_bytes(),
        )?;

        // create the tantivy document to insert
        let mut tantivy_doc = tantivy::doc! {
            self.index.layout.field_title => doc.title.clone()
        };
        if let Some(path) = doc.metadata.path.to_str() {
            tantivy_doc.add_facet(
                self.index.layout.field_path,
                tantivy::schema::Facet::from(path),
            );
        }
        for keyword in keywords {
            tantivy_doc.add_text(self.index.layout.field_keyword, keyword);
        }
        self.writer.add_document(tantivy_doc);

        return Ok(());
    }

    fn commit(&mut self) -> Result<(), GuidebookError> {
        self.writer.commit()?;
        return Ok(());
    }
}
