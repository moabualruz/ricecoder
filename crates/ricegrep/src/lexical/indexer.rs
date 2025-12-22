use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tantivy::{
    directory::Directory,
    doc,
    indexer::IndexWriter,
    schema::{Document, TantivyDocument},
    Index,
};

use crate::{
    chunking::Chunk,
    lexical::{errors::LexicalResult, schema::LexicalSchema},
};

const METADATA_FILE: &str = "bm25_meta.json";

/// Builder responsible for creating a new Tantivy index on disk.
pub struct Bm25IndexBuilder {
    schema: LexicalSchema,
    index: Index,
    directory_path: PathBuf,
}

impl Bm25IndexBuilder {
    pub fn create<P: AsRef<Path>>(directory: P) -> LexicalResult<Self> {
        let schema = LexicalSchema::build();
        let directory_path = directory.as_ref().to_path_buf();
        let index = Index::create_in_dir(&directory_path, schema.schema().clone())?;
        Ok(Self {
            schema,
            index,
            directory_path,
        })
    }

    pub fn writer(self, heap_bytes: usize) -> LexicalResult<Bm25IndexWriter> {
        Bm25IndexWriter::create(
            self.index,
            self.schema,
            heap_bytes,
            LexicalIndexMetadata::default(),
            self.directory_path,
        )
    }
}

pub struct Bm25IndexWriter {
    schema: LexicalSchema,
    index: Index,
    writer: IndexWriter,
    total_token_count: u64,
    document_count: u64,
    directory_path: PathBuf,
}

impl Bm25IndexWriter {
    fn create(
        index: Index,
        schema: LexicalSchema,
        heap_bytes: usize,
        metadata: LexicalIndexMetadata,
        directory_path: PathBuf,
    ) -> LexicalResult<Self> {
        let writer = index.writer(heap_bytes)?;
        Ok(Self {
            schema,
            index,
            writer,
            total_token_count: metadata.total_tokens,
            document_count: metadata.doc_count,
            directory_path,
        })
    }

    pub fn directory(&self) -> &Path {
        &self.directory_path
    }

    pub fn add_chunk(&mut self, chunk: &Chunk) -> LexicalResult<()> {
        let doc = doc!(
            self.schema.identifier_field => chunk.identifier_tokens.join(" "),
            self.schema.comment_field => chunk.text.clone(), // comments already included in chunk text; will refine later
            self.schema.code_field => chunk.text.clone(),
            self.schema.chunk_id_field => chunk.id as i64,
            self.schema.file_path_field => chunk.file_path.to_string_lossy().to_string(),
            self.schema.language_field => format!("{:?}", chunk.language),
            self.schema.repository_field => chunk.metadata.repository_id.map(|id| id as i64).unwrap_or(-1),
            self.schema.token_count_field => chunk.metadata.token_count as i64,
        );
        self.writer.add_document(doc);
        self.total_token_count += chunk.metadata.token_count as u64;
        self.document_count += 1;
        Ok(())
    }

    pub fn commit(mut self) -> LexicalResult<Bm25IndexHandle> {
        self.writer.commit()?;
        self.writer.garbage_collect_files().wait()?;
        self.persist_metadata()?;
        let reader = self.index.reader()?;
        let metadata = LexicalIndexMetadata::load_or_default(&self.index, &self.schema, &reader)?;
        Ok(Bm25IndexHandle {
            schema: self.schema,
            index: self.index,
            reader,
            metadata,
            directory_path: self.directory_path.clone(),
        })
    }

    fn persist_metadata(&self) -> LexicalResult<()> {
        let metadata = LexicalIndexMetadata {
            total_tokens: self.total_token_count,
            doc_count: self.document_count,
        };
        let data = serde_json::to_vec(&metadata)?;
        self.index
            .directory()
            .atomic_write(Path::new(METADATA_FILE), &data)?;
        Ok(())
    }
}

pub struct Bm25IndexHandle {
    pub(crate) schema: LexicalSchema,
    pub(crate) index: Index,
    pub(crate) reader: tantivy::IndexReader,
    pub(crate) metadata: LexicalIndexMetadata,
    directory_path: PathBuf,
}

impl Bm25IndexHandle {
    pub fn open<P: AsRef<Path>>(directory: P) -> LexicalResult<Self> {
        let schema = LexicalSchema::build();
        let directory_path = directory.as_ref().to_path_buf();
        let index = Index::open_in_dir(&directory_path)?;
        let reader = index.reader()?;
        let metadata = LexicalIndexMetadata::load_or_default(&index, &schema, &reader)?;
        Ok(Self {
            schema,
            index,
            reader,
            metadata,
            directory_path,
        })
    }

    pub fn reopen_writer(&self, heap_bytes: usize) -> LexicalResult<Bm25IndexWriter> {
        Bm25IndexWriter::create(
            self.index.clone(),
            self.schema.clone(),
            heap_bytes,
            self.metadata.clone(),
            self.directory_path.clone(),
        )
    }

    pub fn optimize(&self) -> LexicalResult<()> {
        let mut writer = self.index.writer::<TantivyDocument>(50_000_000)?;
        writer.commit()?;
        Ok(())
    }

    pub fn document_count(&self) -> u64 {
        self.metadata.doc_count
    }

    pub fn token_count(&self) -> u64 {
        self.metadata.total_tokens
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LexicalIndexMetadata {
    pub total_tokens: u64,
    pub doc_count: u64,
}

impl LexicalIndexMetadata {
    fn load_or_default(
        index: &Index,
        schema: &LexicalSchema,
        reader: &tantivy::IndexReader,
    ) -> LexicalResult<Self> {
        let directory = index.directory();
        match directory.open_read(Path::new(METADATA_FILE)) {
            Ok(file_slice) => {
                let bytes = file_slice.read_bytes()?;
                let metadata: LexicalIndexMetadata = serde_json::from_slice(bytes.as_slice())?;
                Ok(metadata)
            }
            Err(_) => Self::compute_from_index(schema, reader),
        }
    }

    fn compute_from_index(
        schema: &LexicalSchema,
        reader: &tantivy::IndexReader,
    ) -> LexicalResult<Self> {
        let searcher = reader.searcher();
        let mut total_tokens = 0u64;
        let token_field_name = schema.field_name(schema.token_count_field);
        for segment_reader in searcher.segment_readers() {
            let fast_fields = segment_reader.fast_fields();
            if let Ok(reader) = fast_fields.i64(token_field_name) {
                for doc in 0..segment_reader.max_doc() {
                    let value = reader.values_for_doc(doc).next().unwrap_or(0);
                    total_tokens += value as u64;
                }
            }
        }
        Ok(Self {
            total_tokens,
            doc_count: searcher.num_docs().max(1) as u64,
        })
    }
}
