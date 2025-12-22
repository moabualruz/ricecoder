use tantivy::query::QueryParserError;
use thiserror::Error;

use crate::chunking::ChunkingError;

pub type LexicalResult<T> = Result<T, LexicalError>;

#[derive(Debug, Error)]
pub enum LexicalError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tantivy error: {0}")]
    Tantivy(#[from] tantivy::TantivyError),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Chunking error: {0}")]
    Chunking(#[from] ChunkingError),

    #[error("Query parsing error: {0}")]
    QueryParser(#[from] tantivy::query::QueryParserError),

    #[error("Chunk missing metadata: {0}")]
    MissingMetadata(String),

    #[error("Shard error: {0}")]
    Shard(String),
}
