use std::{
    cmp::Ordering,
    collections::HashMap,
    time::{Duration, Instant},
};

use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{
        document::{CompactDocValue, TantivyDocument},
        Value,
    },
    Term,
};
use tracing::{info, warn};

use crate::lexical::{errors::LexicalResult, indexer::Bm25IndexHandle};

pub struct LexicalSearcher {
    handle: Bm25IndexHandle,
    config: LexicalConfig,
}

impl LexicalSearcher {
    pub fn new(handle: Bm25IndexHandle) -> Self {
        Self {
            handle,
            config: LexicalConfig::default(),
        }
    }

    pub fn with_config(handle: Bm25IndexHandle, config: LexicalConfig) -> Self {
        Self { handle, config }
    }

    pub fn search(&self, query: &str, limit: usize) -> LexicalResult<Vec<LexicalHit>> {
        self.search_with_filters(query, &SearchFilters::default(), limit)
    }

    pub fn search_with_filters(
        &self,
        query: &str,
        filters: &SearchFilters,
        limit: usize,
    ) -> LexicalResult<Vec<LexicalHit>> {
        let start = Instant::now();
        let searcher = self.handle.reader.searcher();
        let mut parser = QueryParser::for_index(
            &self.handle.index,
            vec![
                self.handle.schema.identifier_field,
                self.handle.schema.comment_field,
                self.handle.schema.code_field,
            ],
        );
        parser.set_field_boost(self.handle.schema.identifier_field, 3.0);
        parser.set_field_boost(self.handle.schema.comment_field, 2.0);
        parser.set_field_boost(self.handle.schema.code_field, 1.0);
        let truncated_query = self.truncate_terms(query);
        let query_terms = self.tokenize(&truncated_query);
        let query = parser.parse_query(&truncated_query)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        let mut docs = Vec::with_capacity(top_docs.len());
        for (score, doc_address) in top_docs {
            let retrieved: TantivyDocument = searcher.doc(doc_address)?;
            let chunk_id = retrieved
                .get_first(self.handle.schema.chunk_id_field)
                .and_then(|value| value.as_i64())
                .unwrap_or_default() as u64;
            let file_path = retrieved
                .get_first(self.handle.schema.file_path_field)
                .and_then(|value| value.as_str())
                .map(str::to_string)
                .unwrap_or_default();
            let language = retrieved
                .get_first(self.handle.schema.language_field)
                .and_then(|value| value.as_str())
                .map(str::to_string)
                .unwrap_or_default();
            let repository_id = retrieved
                .get_first(self.handle.schema.repository_field)
                .and_then(|value| value.as_i64())
                .map(|raw| if raw < 0 { None } else { Some(raw as u32) })
                .flatten();
            if !filters.matches(repository_id, &language, &file_path) {
                continue;
            }
            let token_count = retrieved
                .get_first(self.handle.schema.token_count_field)
                .and_then(|value| value.as_i64())
                .unwrap_or_default() as f32;
            docs.push(ScoredDoc {
                hit: LexicalHit {
                    chunk_id,
                    file_path,
                    language,
                    repository_id,
                    score,
                },
                document: retrieved,
                token_count,
            });
        }

        let rescored = if query_terms.is_empty() {
            docs.into_iter().map(|doc| doc.hit).collect()
        } else {
            self.rescore_hits(&query_terms, docs)?
        };

        let elapsed = start.elapsed();
        if elapsed > Duration::from_millis(10) {
            warn!(
                elapsed_ms = elapsed.as_millis(),
                "Lexical search exceeded latency target"
            );
        } else {
            info!(elapsed_ms = elapsed.as_micros(), "Lexical search completed");
        }

        Ok(rescored)
    }

    fn rescore_hits(
        &self,
        query_terms: &[String],
        docs: Vec<ScoredDoc>,
    ) -> LexicalResult<Vec<LexicalHit>> {
        let searcher = self.handle.reader.searcher();
        let avg_doc_len = self.average_doc_length(&searcher)?;
        let mut idf_cache = HashMap::new();
        for term in query_terms {
            let idf = self.term_idf(term, &searcher)?;
            idf_cache.insert(term.clone(), idf);
        }

        let mut rescored = Vec::with_capacity(docs.len());
        for doc in docs {
            let tokens = self.extract_tokens(&doc.document);
            let doc_len = if doc.token_count > 0.0 {
                doc.token_count
            } else {
                tokens.len() as f32
            };
            let mut score = 0f32;
            for term in query_terms {
                let tf = tokens.iter().filter(|t| *t == term).count() as f32;
                if tf == 0.0 {
                    continue;
                }
                let idf = idf_cache.get(term).copied().unwrap_or(1.0);
                let numerator = tf * (self.config.bm25_k1 + 1.0);
                let denominator = tf
                    + self.config.bm25_k1
                        * (1.0 - self.config.bm25_b + self.config.bm25_b * (doc_len / avg_doc_len));
                score += idf * (numerator / denominator);
            }
            let mut hit = doc.hit;
            hit.score = score;
            rescored.push(hit);
        }

        rescored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.chunk_id.cmp(&b.chunk_id))
        });

        Ok(rescored)
    }

    fn term_idf(&self, term: &str, searcher: &tantivy::Searcher) -> LexicalResult<f32> {
        let mut doc_freq = 0u64;
        for field in [
            self.handle.schema.identifier_field,
            self.handle.schema.comment_field,
            self.handle.schema.code_field,
        ] {
            let tantivy_term = Term::from_field_text(field, term);
            doc_freq += searcher.doc_freq(&tantivy_term)?;
        }
        let doc_count = searcher.num_docs().max(1) as f32;
        let freq = doc_freq.max(1) as f32;
        Ok(((doc_count - freq + 0.5) / (freq + 0.5)).ln() + 1.0)
    }

    fn average_doc_length(&self, searcher: &tantivy::Searcher) -> LexicalResult<f32> {
        if self.handle.metadata.total_tokens > 0 && self.handle.metadata.doc_count > 0 {
            return Ok(
                self.handle.metadata.total_tokens as f32 / self.handle.metadata.doc_count as f32
            );
        }
        let mut total = 0f32;
        let mut count = 0f32;
        for segment_reader in searcher.segment_readers() {
            let fast_fields = segment_reader.fast_fields();
            let token_field_name = self
                .handle
                .schema
                .field_name(self.handle.schema.token_count_field);
            if let Ok(reader) = fast_fields.i64(token_field_name) {
                let max_doc = segment_reader.max_doc();
                for doc in 0..max_doc {
                    let value = reader.values_for_doc(doc).next().unwrap_or(0);
                    total += value as f32;
                    count += 1.0;
                }
            }
        }
        if count == 0.0 {
            return Ok(1.0);
        }
        Ok((total / count).max(1.0))
    }

    fn extract_tokens(&self, document: &TantivyDocument) -> Vec<String> {
        let mut tokens = Vec::new();
        for field in [
            self.handle.schema.identifier_field,
            self.handle.schema.comment_field,
            self.handle.schema.code_field,
        ] {
            for value in document.get_all(field) {
                if let Some(text) = value.as_str() {
                    tokens.extend(self.tokenize(text));
                }
            }
        }
        tokens
    }

    fn tokenize<T: AsRef<str>>(&self, text: T) -> Vec<String> {
        text.as_ref()
            .split_whitespace()
            .map(|t| t.to_lowercase())
            .collect()
    }

    fn truncate_terms(&self, query: &str) -> String {
        let tokens: Vec<&str> = query.split_whitespace().collect();
        if tokens.len() <= self.config.max_query_terms {
            query.to_string()
        } else {
            tokens[..self.config.max_query_terms].join(" ")
        }
    }
}

#[derive(Debug, Clone)]
pub struct LexicalHit {
    pub chunk_id: u64,
    pub file_path: String,
    pub language: String,
    pub repository_id: Option<u32>,
    pub score: f32,
}

struct ScoredDoc {
    hit: LexicalHit,
    document: TantivyDocument,
    token_count: f32,
}

#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub language: Option<String>,
    pub repository_id: Option<u32>,
    pub file_path_prefix: Option<String>,
}

impl SearchFilters {
    pub fn matches(&self, repository_id: Option<u32>, language: &str, file_path: &str) -> bool {
        if let Some(expected) = self.repository_id {
            if repository_id != Some(expected) {
                return false;
            }
        }
        if let Some(lang) = &self.language {
            if !language.eq_ignore_ascii_case(lang) {
                return false;
            }
        }
        if let Some(prefix) = &self.file_path_prefix {
            if !file_path.starts_with(prefix) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct LexicalConfig {
    pub bm25_k1: f32,
    pub bm25_b: f32,
    pub max_query_terms: usize,
}

impl Default for LexicalConfig {
    fn default() -> Self {
        Self {
            bm25_k1: 1.2,
            bm25_b: 0.75,
            max_query_terms: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::*;
    use crate::{
        chunking::{Chunk, ChunkMetadata, LanguageKind},
        lexical::Bm25IndexBuilder,
    };

    fn stub_chunk(id: u64, repo: Option<u32>, text: &str, language: LanguageKind) -> Chunk {
        let tokens: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        let metadata = ChunkMetadata {
            chunk_id: id,
            repository_id: repo,
            file_path: PathBuf::from(format!("file_{id}.rs")),
            language,
            start_line: 1,
            end_line: 1,
            token_count: tokens.len() as u32,
            checksum: format!("chk{id}"),
        };
        Chunk {
            id,
            language,
            file_path: metadata.file_path.clone(),
            start_line: 1,
            end_line: 1,
            text: text.to_string(),
            identifiers: tokens.clone(),
            identifier_tokens: tokens,
            metadata,
        }
    }

    #[test]
    fn filters_by_repository_id() {
        let dir = tempdir().unwrap();
        let builder = Bm25IndexBuilder::create(dir.path()).unwrap();
        let mut writer = builder.writer(50_000_000).unwrap();
        writer
            .add_chunk(&stub_chunk(
                1,
                Some(7),
                "lexical alpha beta",
                LanguageKind::Rust,
            ))
            .unwrap();
        writer
            .add_chunk(&stub_chunk(
                2,
                Some(9),
                "lexical gamma delta",
                LanguageKind::Rust,
            ))
            .unwrap();
        let handle = writer.commit().unwrap();
        let searcher = LexicalSearcher::new(handle);
        let mut filters = SearchFilters::default();
        filters.repository_id = Some(7);
        let hits = searcher
            .search_with_filters("lexical", &filters, 10)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].repository_id, Some(7));
    }

    #[test]
    fn filters_by_file_path_prefix() {
        let dir = tempdir().unwrap();
        let builder = Bm25IndexBuilder::create(dir.path()).unwrap();
        let mut writer = builder.writer(50_000_000).unwrap();
        writer
            .add_chunk(&stub_chunk(
                1,
                Some(1),
                "alpha omega search",
                LanguageKind::Rust,
            ))
            .unwrap();
        writer
            .add_chunk(&stub_chunk(
                2,
                Some(1),
                "beta omega search",
                LanguageKind::Rust,
            ))
            .unwrap();
        let handle = writer.commit().unwrap();
        let searcher = LexicalSearcher::new(handle);
        let mut filters = SearchFilters::default();
        filters.file_path_prefix = Some("file_1".into());
        let hits = searcher.search_with_filters("omega", &filters, 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].file_path.contains("file_1"));
    }

    #[test]
    fn lexical_search_latency_is_small() {
        let dir = tempdir().unwrap();
        let builder = Bm25IndexBuilder::create(dir.path()).unwrap();
        let mut writer = builder.writer(50_000_000).unwrap();
        for i in 0..25 {
            writer
                .add_chunk(&stub_chunk(
                    i,
                    Some(1),
                    "fast search validation task",
                    LanguageKind::Rust,
                ))
                .unwrap();
        }
        let handle = writer.commit().unwrap();
        let searcher = LexicalSearcher::new(handle);
        let start = Instant::now();
        let hits = searcher.search("validation", 5).unwrap();
        assert!(!hits.is_empty());
        assert!(
            start.elapsed() < Duration::from_millis(50),
            "lexical search exceeded latency budget"
        );
    }
}
