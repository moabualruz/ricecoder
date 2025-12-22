use std::path::PathBuf;

use anyhow::Context;
use async_stream::try_stream;
use futures::Stream;
use sha2::{Digest, Sha256};
use tokio::fs;
use tracing::{debug, warn};

use crate::chunking::{
    errors::{ChunkingError, ChunkingResult},
    language::{LanguageDetector, LanguageKind},
    parser::ParserPool,
    repository::{RepositoryScanner, RepositorySource},
    tokenizer::{self, IdentifierSplitResult},
    ChunkMetadata,
};

/// Default maximum number of tokens per chunk (per spec).
const DEFAULT_MAX_TOKENS: usize = 512;
const DEFAULT_CHUNK_OVERLAP: usize = 50;
const DEFAULT_MAX_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024; // 10MB safety guard

/// High-level configuration for the chunking pipeline.
#[derive(Debug, Clone)]
pub struct ChunkingConfig {
    pub supported_languages: Vec<LanguageKind>,
    pub max_chunk_tokens: usize,
    pub chunk_overlap_tokens: usize,
    pub max_file_size_bytes: u64,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            supported_languages: LanguageKind::default_supported(),
            max_chunk_tokens: DEFAULT_MAX_TOKENS,
            chunk_overlap_tokens: DEFAULT_CHUNK_OVERLAP,
            max_file_size_bytes: DEFAULT_MAX_FILE_SIZE_BYTES,
        }
    }
}

/// Builder for assembling a chunk producer with all dependencies.
pub struct ChunkProducerBuilder {
    config: ChunkingConfig,
    scanner: RepositoryScanner,
    detector: LanguageDetector,
}

impl Default for ChunkProducerBuilder {
    fn default() -> Self {
        Self {
            config: ChunkingConfig::default(),
            scanner: RepositoryScanner::default(),
            detector: LanguageDetector::default(),
        }
    }
}

impl ChunkProducerBuilder {
    pub fn config(mut self, config: ChunkingConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> ChunkProducer {
        ChunkProducer {
            config: self.config.clone(),
            scanner: self.scanner,
            detector: self.detector,
            parser_pool: ParserPool::new(&self.config.supported_languages),
        }
    }
}

/// Outputs chunks produced from repositories.
pub struct ChunkProducer {
    config: ChunkingConfig,
    scanner: RepositoryScanner,
    detector: LanguageDetector,
    parser_pool: ParserPool,
}

impl ChunkProducer {
    /// Creates a new builder.
    pub fn builder() -> ChunkProducerBuilder {
        ChunkProducerBuilder::default()
    }

    /// Produces a stream of chunks for the provided repository source.
    pub fn chunk_stream(
        &self,
        source: RepositorySource,
    ) -> ChunkingResult<impl Stream<Item = ChunkingResult<Chunk>> + '_> {
        let entries = self.scanner.scan(&source)?;
        let config = self.config.clone();
        let detector = self.detector.clone();
        let parser_pool = self.parser_pool.clone();

        Ok(try_stream! {
            let mut chunk_id: u64 = 0;
            let repository_id = source.repository_id;
            for entry in entries {
                if entry.size > config.max_file_size_bytes {
                    warn!(path = ?entry.path, size = entry.size, "Skipping oversized file");
                    continue;
                }

                let content = fs::read_to_string(&entry.path).await
                    .with_context(|| format!("reading {}", entry.path.display()))
                    .map_err(ChunkingError::from)?;

                let language = match detector.detect(&entry.path, &content) {
                    Some(lang) => lang,
                    None => {
                        debug!(path = ?entry.path, "Unsupported language, using fallback chunking");
                        for chunk in self.line_based_chunks(&mut chunk_id, &entry.path, &content, LanguageKind::PlainText, repository_id)? {
                            yield chunk;
                        }
                        continue;
                    }
                };

                match parser_pool.parse(language, &content) {
                    Ok(tree) => {
                        let semantic_units = parser_pool.collect_semantic_units(language, &tree, &content)?;
                        if semantic_units.is_empty() {
                            for chunk in self.line_based_chunks(&mut chunk_id, &entry.path, &content, language, repository_id)? {
                                yield chunk;
                            }
                            continue;
                        }

                        for unit in semantic_units {
                            let text = unit.extract_text(&content);
                            let chunk = self.build_chunk(
                                chunk_id,
                                &entry.path,
                                language,
                                unit.start_line,
                                unit.end_line,
                                text,
                                repository_id,
                            )?;
                            chunk_id += 1;
                            yield chunk;
                        }
                    }
                    Err(err) => {
                        warn!(path = ?entry.path, %err, "Parser failed, falling back to line chunking");
                        for chunk in self.line_based_chunks(&mut chunk_id, &entry.path, &content, language, repository_id)? {
                            yield chunk;
                        }
                    }
                }
            }
        })
    }

    fn line_based_chunks(
        &self,
        chunk_id: &mut u64,
        path: &PathBuf,
        content: &str,
        language: LanguageKind,
        repository_id: Option<u32>,
    ) -> ChunkingResult<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let mut current_lines = Vec::new();
        let mut start_line = 0u32;

        for (line_idx, line) in content.lines().enumerate() {
            if current_lines.is_empty() {
                start_line = line_idx as u32 + 1;
            }
            current_lines.push(line);
            let token_count = tokenizer::count_tokens(current_lines.join("\n").as_str());
            if token_count
                >= self
                    .config
                    .max_chunk_tokens
                    .saturating_sub(self.config.chunk_overlap_tokens)
            {
                let text = current_lines.join("\n");
                let chunk = self.build_chunk(
                    *chunk_id,
                    path,
                    language,
                    start_line,
                    line_idx as u32 + 1,
                    text,
                    repository_id,
                )?;
                *chunk_id += 1;
                chunks.push(chunk);
                if self.config.chunk_overlap_tokens > 0 {
                    current_lines = current_lines
                        .into_iter()
                        .rev()
                        .take(self.config.chunk_overlap_tokens)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();
                } else {
                    current_lines.clear();
                }
            }
        }

        if !current_lines.is_empty() {
            let end_line = (start_line as usize + current_lines.len()) as u32 - 1;
            let text = current_lines.join("\n");
            let chunk = self.build_chunk(
                *chunk_id,
                path,
                language,
                start_line,
                end_line,
                text,
                repository_id,
            )?;
            *chunk_id += 1;
            chunks.push(chunk);
        }

        Ok(chunks)
    }

    fn build_chunk(
        &self,
        chunk_id: u64,
        path: &PathBuf,
        language: LanguageKind,
        start_line: u32,
        end_line: u32,
        text: String,
        repository_id: Option<u32>,
    ) -> ChunkingResult<Chunk> {
        let IdentifierSplitResult {
            identifiers,
            identifier_tokens,
        } = tokenizer::extract_identifiers(&text);

        let checksum = {
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        let token_count = tokenizer::count_tokens(&text) as u32;

        let metadata = ChunkMetadata {
            chunk_id,
            repository_id,
            file_path: path.clone(),
            language,
            start_line,
            end_line,
            token_count,
            checksum,
        };

        Ok(Chunk {
            id: chunk_id,
            language,
            file_path: path.clone(),
            start_line,
            end_line,
            text,
            identifiers,
            identifier_tokens,
            metadata,
        })
    }
}

/// Output data returned by the chunking pipeline.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: u64,
    pub language: LanguageKind,
    pub file_path: PathBuf,
    pub start_line: u32,
    pub end_line: u32,
    pub text: String,
    pub identifiers: Vec<String>,
    pub identifier_tokens: Vec<String>,
    pub metadata: ChunkMetadata,
}
