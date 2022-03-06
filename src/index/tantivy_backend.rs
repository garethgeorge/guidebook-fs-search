use crate::error::GuidebookError;
use crate::index::*;
use std::fs;
use std::path::{Path, PathBuf};

struct TantivyIndexLayout {
    schema: tantivy::schema::Schema,
    field_title: tantivy::schema::Field,
    field_keyword: tantivy::schema::Field,
    field_path: tantivy::schema::Field,
}

pub struct TantivyIndex {
    path: PathBuf,
    path_index: PathBuf,

    layout: TantivyIndexLayout,
    index: tantivy::Index,
}

impl TantivyIndex {
    pub fn create(dir: &Path) -> tantivy::Result<TantivyIndex> {
        let path_index = dir.join("index");
        if !path_index.exists() {
            fs::create_dir(&path_index)?;
        }

        let mut schema_builder = tantivy::schema::Schema::builder();
        let field_title = schema_builder.add_text_field("title", tantivy::schema::TEXT);
        let field_keyword = schema_builder.add_text_field("keywords", tantivy::schema::TEXT);
        let field_path = schema_builder.add_facet_field(
            "path",
            tantivy::schema::FacetOptions::default().set_stored(),
        );
        let schema = schema_builder.build();

        return Ok(TantivyIndex {
            path: PathBuf::from(dir),
            path_index: PathBuf::from(&path_index),

            layout: TantivyIndexLayout {
                field_title: field_title,
                field_keyword: field_keyword,
                field_path: field_path,
                schema: schema.clone(),
            },

            index: tantivy::Index::open_or_create(
                tantivy::directory::MmapDirectory::open(&path_index)?,
                schema,
            )?,
        });
    }

    pub fn index_directory(&mut self, dir: &Path) -> Result<(), GuidebookError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            let metadata = fs::metadata(&path)?;
            let last_modified = metadata.modified()?.elapsed()?.as_secs();

            println!("file {:?} last modified {}", path, last_modified);
        }

        return Ok(());
    }

    pub fn get_document_writer(&mut self) -> Result<Box<dyn IndexWriter + '_>, GuidebookError> {
        return Ok(Box::new(TantivyIndexWriter::create(self)?));
    }
}

struct TantivyIndexWriter<'a> {
    writer: tantivy::IndexWriter,
    layout: &'a TantivyIndexLayout,
}

impl TantivyIndexWriter<'_> {
    fn create(index: &mut TantivyIndex) -> Result<TantivyIndexWriter, GuidebookError> {
        let writer = index.index.writer(50_000_000 /* 50 MB heap size */)?;
        return Ok(TantivyIndexWriter {
            writer: writer,
            layout: &index.layout,
        });
    }
}

impl IndexWriter for TantivyIndexWriter<'_> {
    fn should_add_document(&mut self, metadata: &DocumentMetadata) -> bool {
        return true; // TODO: add already indexed detection. For now: wipeout index between runs.
    }

    fn add_document(&mut self, doc: &Document) -> () {
        let mut tantivyDoc = tantivy::doc! {};

        if let Some(path) = doc.metadata.path.to_str() {
            tantivyDoc.add_facet(self.layout.field_path, tantivy::schema::Facet::from(path));
        }
        for keyword in &doc.keywords {
            tantivyDoc.add_text(self.layout.field_keyword, keyword);
        }
        self.writer.add_document(tantivyDoc);

        return ();
    }

    fn commit(&mut self) -> Result<(), GuidebookError> {
        self.writer.commit();
        return Ok(());
    }
}
