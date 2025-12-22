//! Chunking pipeline implementation.

use std::path::PathBuf;

pub use self::{
    chunker::{Chunk, ChunkProducer, ChunkProducerBuilder, ChunkingConfig},
    errors::{ChunkingError, ChunkingResult},
    language::{LanguageDetector, LanguageKind},
    repository::{FileEntry, RepositoryScanner, RepositoryScannerConfig, RepositorySource},
};

mod chunker;
mod errors;
mod language;
mod parser;
mod repository;
mod tokenizer;

/// Additional metadata captured for each chunk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkMetadata {
    pub chunk_id: u64,
    pub repository_id: Option<u32>,
    pub file_path: PathBuf,
    pub language: LanguageKind,
    pub start_line: u32,
    pub end_line: u32,
    pub token_count: u32,
    pub checksum: String,
}
