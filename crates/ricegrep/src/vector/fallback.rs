use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

use crate::{chunking::Chunk, lexical::LexicalHit};

#[derive(Default, Clone)]
pub struct PmiGraph {
    cooccurrences: DashMap<(String, String), u64>,
    marginals: DashMap<String, u64>,
}

impl PmiGraph {
    pub fn update(&self, tokens: &[String]) {
        for (i, t1) in tokens.iter().enumerate() {
            *self.marginals.entry(t1.clone()).or_insert(0) += 1;
            for t2 in tokens.iter().skip(i + 1) {
                *self
                    .cooccurrences
                    .entry((t1.clone(), t2.clone()))
                    .or_insert(0) += 1;
            }
        }
    }

    pub fn expand(&self, term: &str, threshold: f32, limit: usize) -> Vec<(String, f32)> {
        let term_count = self.marginals.get(term).map(|v| *v).unwrap_or(0) as f32;
        if term_count == 0.0 {
            return Vec::new();
        }
        let mut expansions = Vec::new();
        for entry in self.cooccurrences.iter() {
            let ((ref a, ref b), &count) = entry.pair();
            if a != term {
                continue;
            }
            let b_count = self.marginals.get(b).map(|v| *v).unwrap_or(0) as f32;
            if b_count == 0.0 {
                continue;
            }
            let pmi = ((count as f32) / (term_count * b_count)).ln();
            if pmi > threshold {
                expansions.push((b.clone(), pmi));
            }
        }
        expansions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        expansions.truncate(limit);
        expansions
    }

    pub fn snapshot(&self) -> PmiSnapshot {
        let edges = self
            .cooccurrences
            .iter()
            .map(|entry| PmiEdge {
                term: entry.key().0.clone(),
                neighbor: entry.key().1.clone(),
                count: *entry.value(),
            })
            .collect();
        let marginals = self
            .marginals
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        PmiSnapshot { edges, marginals }
    }

    pub fn from_snapshot(snapshot: PmiSnapshot) -> Self {
        let graph = PmiGraph::default();
        for (term, count) in snapshot.marginals {
            graph.marginals.insert(term, count);
        }
        for edge in snapshot.edges {
            graph
                .cooccurrences
                .insert((edge.term, edge.neighbor), edge.count);
        }
        graph
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackWeights {
    pub bm25: f32,
    pub identifier: f32,
    pub pmi: f32,
    pub ngram: f32,
    pub pmi_threshold: f32,
    pub expansion_limit: usize,
}

impl Default for FallbackWeights {
    fn default() -> Self {
        Self {
            bm25: 1.0,
            identifier: 0.5,
            pmi: 0.35,
            ngram: 0.3,
            pmi_threshold: 2.0,
            expansion_limit: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGramVector {
    pub trigrams: HashMap<String, f32>,
    pub quadgrams: HashMap<String, f32>,
}

impl NGramVector {
    pub fn from_text(text: &str) -> Self {
        let mut trigrams = HashMap::new();
        let mut quadgrams = HashMap::new();
        for word in text.graphemes(true) {
            // consume graphemes to ensure proper Unicode iteration
            let _ = word;
        }
        let chars: Vec<char> = text.chars().collect();
        for window in chars.windows(3) {
            let key: String = window.iter().collect();
            *trigrams.entry(key).or_insert(0.0) += 1.0;
        }
        for window in chars.windows(4) {
            let key: String = window.iter().collect();
            *quadgrams.entry(key).or_insert(0.0) += 1.0;
        }
        Self {
            trigrams: normalize(trigrams),
            quadgrams: normalize(quadgrams),
        }
    }

    pub fn cosine_similarity(&self, other: &Self) -> f32 {
        dot(&self.trigrams, &other.trigrams) + dot(&self.quadgrams, &other.quadgrams)
    }
}

fn normalize(mut map: HashMap<String, f32>) -> HashMap<String, f32> {
    let norm = map.values().map(|v| v * v).sum::<f32>().sqrt();
    if norm == 0.0 {
        return map;
    }
    for value in map.values_mut() {
        *value /= norm;
    }
    map
}

fn dot(a: &HashMap<String, f32>, b: &HashMap<String, f32>) -> f32 {
    a.iter()
        .filter_map(|(k, v)| b.get(k).map(|other| v * other))
        .sum()
}

#[derive(Clone, Default)]
pub struct IdentifierProfile {
    tokens: Arc<Vec<String>>,
}

impl IdentifierProfile {
    pub fn from_chunk(chunk: &Chunk) -> Self {
        Self {
            tokens: Arc::new(chunk.identifier_tokens.clone()),
        }
    }

    pub fn from_tokens(tokens: Vec<String>) -> Self {
        Self {
            tokens: Arc::new(tokens),
        }
    }

    pub fn score_overlap(&self, query_terms: &[String]) -> f32 {
        if query_terms.is_empty() || self.tokens.is_empty() {
            return 0.0;
        }
        let query: HashSet<&String> = query_terms.iter().collect();
        let overlap = self
            .tokens
            .iter()
            .filter(|token| query.contains(token))
            .count() as f32;
        overlap / query.len() as f32
    }
}

#[derive(Clone)]
pub struct IdentifierScorer {
    weights: FallbackWeights,
}

impl IdentifierScorer {
    pub fn new(weights: FallbackWeights) -> Self {
        Self { weights }
    }

    pub fn score(&self, profile: &IdentifierProfile, query_terms: &[String]) -> f32 {
        profile.score_overlap(query_terms) * self.weights.identifier
    }
}

#[derive(Clone, Default)]
pub struct FallbackArtifacts {
    pmi: Arc<PmiGraph>,
    ngrams: DashMap<u64, NGramVector>,
    identifiers: DashMap<u64, IdentifierProfile>,
}

impl FallbackArtifacts {
    pub fn new(pmi: Arc<PmiGraph>) -> Self {
        Self {
            pmi,
            ngrams: DashMap::new(),
            identifiers: DashMap::new(),
        }
    }

    pub fn record_chunk(&self, chunk: &Chunk, ngram: &NGramVector) {
        self.pmi.update(&chunk.identifier_tokens);
        self.ngrams.insert(chunk.id, ngram.clone());
        self.identifiers
            .insert(chunk.id, IdentifierProfile::from_chunk(chunk));
    }

    pub fn ngram(&self, chunk_id: u64) -> Option<NGramVector> {
        self.ngrams.get(&chunk_id).map(|entry| entry.clone())
    }

    pub fn identifier(&self, chunk_id: u64) -> Option<IdentifierProfile> {
        self.identifiers
            .get(&chunk_id)
            .map(|profile| profile.clone())
    }

    pub fn pmi(&self) -> Arc<PmiGraph> {
        Arc::clone(&self.pmi)
    }

    pub fn persist<P: AsRef<Path>>(&self, dir: P) -> Result<(), FallbackPersistenceError> {
        let directory = dir.as_ref();
        std::fs::create_dir_all(directory)?;
        let pmi_path = directory.join("pmi_graph.json");
        let ngrams_path = directory.join("ngrams.json");
        let identifiers_path = directory.join("identifiers.json");

        let pmi_snapshot = self.pmi.snapshot();
        serde_json::to_writer_pretty(File::create(pmi_path)?, &pmi_snapshot)?;

        let ngram_records: Vec<NGramRecord> = self
            .ngrams
            .iter()
            .map(|entry| NGramRecord {
                chunk_id: *entry.key(),
                vector: entry.value().clone(),
            })
            .collect();
        serde_json::to_writer_pretty(File::create(ngrams_path)?, &ngram_records)?;

        let identifier_records: Vec<IdentifierRecord> = self
            .identifiers
            .iter()
            .map(|entry| IdentifierRecord {
                chunk_id: *entry.key(),
                tokens: entry.value().token_list(),
            })
            .collect();
        serde_json::to_writer_pretty(File::create(identifiers_path)?, &identifier_records)?;

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(dir: P) -> Result<Self, FallbackPersistenceError> {
        let directory = dir.as_ref();
        let pmi_path = directory.join("pmi_graph.json");
        let ngrams_path = directory.join("ngrams.json");
        let identifiers_path = directory.join("identifiers.json");

        let pmi_snapshot: PmiSnapshot = if pmi_path.exists() {
            serde_json::from_reader(File::open(pmi_path)?)?
        } else {
            PmiSnapshot::default()
        };
        let graph = Arc::new(PmiGraph::from_snapshot(pmi_snapshot));

        let artifacts = FallbackArtifacts::new(graph);

        if ngrams_path.exists() {
            let ngram_records: Vec<NGramRecord> =
                serde_json::from_reader(File::open(ngrams_path)?)?;
            for record in ngram_records {
                artifacts.ngrams.insert(record.chunk_id, record.vector);
            }
        }

        if identifiers_path.exists() {
            let identifier_records: Vec<IdentifierRecord> =
                serde_json::from_reader(File::open(identifiers_path)?)?;
            for record in identifier_records {
                artifacts.identifiers.insert(
                    record.chunk_id,
                    IdentifierProfile::from_tokens(record.tokens),
                );
            }
        }

        Ok(artifacts)
    }
}

impl IdentifierProfile {
    fn token_list(&self) -> Vec<String> {
        self.tokens.iter().cloned().collect()
    }
}

#[derive(Debug, Error)]
pub enum FallbackPersistenceError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize, Default)]
struct PmiSnapshot {
    edges: Vec<PmiEdge>,
    marginals: HashMap<String, u64>,
}

#[derive(Serialize, Deserialize)]
struct PmiEdge {
    term: String,
    neighbor: String,
    count: u64,
}

#[derive(Serialize, Deserialize)]
struct NGramRecord {
    chunk_id: u64,
    vector: NGramVector,
}

#[derive(Serialize, Deserialize)]
struct IdentifierRecord {
    chunk_id: u64,
    tokens: Vec<String>,
}

pub struct FallbackEngine {
    artifacts: Arc<FallbackArtifacts>,
    weights: FallbackWeights,
    scorer: IdentifierScorer,
}

impl FallbackEngine {
    pub fn new(artifacts: Arc<FallbackArtifacts>, weights: FallbackWeights) -> Self {
        let scorer = IdentifierScorer::new(weights.clone());
        Self {
            artifacts,
            weights,
            scorer,
        }
    }

    pub fn rerank(&self, query: &str, hits: Vec<LexicalHit>, limit: usize) -> FallbackResult {
        let query_terms: Vec<String> = query.split_whitespace().map(|t| t.to_lowercase()).collect();
        let overall_start = Instant::now();
        let pmi_start = Instant::now();
        let expansions = self.expand_terms(&query_terms);
        let pmi_latency_ms = pmi_start.elapsed().as_micros() as f32 / 1000.0;
        let query_ngrams = NGramVector::from_text(query);
        let mut rescored = Vec::with_capacity(hits.len());
        let ngram_start = Instant::now();
        for hit in hits.into_iter() {
            let identifier_profile = self.artifacts.identifier(hit.chunk_id).unwrap_or_default();
            let identifier_score = self.scorer.score(&identifier_profile, &query_terms);
            let pmi_score = self.score_pmi(&identifier_profile, &expansions);
            let ngram_score = self
                .artifacts
                .ngram(hit.chunk_id)
                .map(|chunk_vec| chunk_vec.cosine_similarity(&query_ngrams))
                .unwrap_or(0.0);
            let total = self.weights.bm25 * hit.score
                + identifier_score
                + self.weights.pmi * pmi_score
                + self.weights.ngram * ngram_score;
            rescored.push(FallbackHit {
                chunk_id: hit.chunk_id,
                file_path: hit.file_path,
                language: hit.language,
                repository_id: hit.repository_id,
                bm25_score: hit.score,
                identifier_score,
                pmi_score,
                ngram_score,
                final_score: total,
            });
        }

        let ngram_latency_ms = ngram_start.elapsed().as_micros() as f32 / 1000.0;
        rescored.sort_by(|a, b| {
            b.final_score
                .partial_cmp(&a.final_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        rescored.truncate(limit);
        let total_latency_ms = overall_start.elapsed().as_micros() as f32 / 1000.0;
        FallbackResult {
            hits: rescored,
            telemetry: FallbackTelemetry {
                pmi_latency_ms,
                ngram_latency_ms,
                total_latency_ms,
            },
        }
    }

    fn expand_terms(&self, query_terms: &[String]) -> Vec<(String, f32)> {
        let mut expanded = Vec::new();
        for term in query_terms {
            let mut items = self.artifacts.pmi().expand(
                term,
                self.weights.pmi_threshold,
                self.weights.expansion_limit,
            );
            expanded.append(&mut items);
        }
        expanded
    }

    fn score_pmi(&self, profile: &IdentifierProfile, expansions: &[(String, f32)]) -> f32 {
        if expansions.is_empty() {
            return 0.0;
        }
        let tokens: HashSet<&String> = profile.tokens.iter().collect();
        expansions
            .iter()
            .filter(|(term, _)| tokens.contains(term))
            .map(|(_, weight)| *weight)
            .sum::<f32>()
            / expansions.len() as f32
    }
}

pub struct FallbackResult {
    pub hits: Vec<FallbackHit>,
    pub telemetry: FallbackTelemetry,
}

#[derive(Debug, Clone)]
pub struct FallbackHit {
    pub chunk_id: u64,
    pub file_path: String,
    pub language: String,
    pub repository_id: Option<u32>,
    pub bm25_score: f32,
    pub identifier_score: f32,
    pub pmi_score: f32,
    pub ngram_score: f32,
    pub final_score: f32,
}

#[derive(Debug, Clone, Default)]
pub struct FallbackTelemetry {
    pub pmi_latency_ms: f32,
    pub ngram_latency_ms: f32,
    pub total_latency_ms: f32,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::chunking::LanguageKind;
    use tempfile::tempdir;

    fn stub_chunk(id: u64, identifiers: &[&str]) -> Chunk {
        Chunk {
            id,
            language: LanguageKind::Rust,
            file_path: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 1,
            text: "fn test() {}".to_string(),
            identifiers: identifiers.iter().map(|s| s.to_string()).collect(),
            identifier_tokens: identifiers.iter().map(|s| s.to_string()).collect(),
            metadata: crate::chunking::ChunkMetadata {
                chunk_id: id,
                repository_id: None,
                file_path: PathBuf::from("test.rs"),
                language: LanguageKind::Rust,
                start_line: 1,
                end_line: 1,
                token_count: 4,
                checksum: "abc".to_string(),
            },
        }
    }

    #[test]
    fn ngram_similarity_behaves() {
        let a = NGramVector::from_text("normalize_json");
        let b = NGramVector::from_text("normalize_json");
        let c = NGramVector::from_text("parse_http");
        assert!(a.cosine_similarity(&b) > 0.9);
        assert!(a.cosine_similarity(&c) < 0.5);
    }

    #[test]
    fn pmi_expansion_limits_terms() {
        let graph = Arc::new(PmiGraph::default());
        let artifacts = FallbackArtifacts::new(graph.clone());
        let chunk = stub_chunk(1, &["normalize", "json", "file"]);
        artifacts.record_chunk(&chunk, &NGramVector::from_text(&chunk.text));
        let weights = FallbackWeights::default();
        let engine = FallbackEngine::new(Arc::new(artifacts), weights);
        let hits = vec![LexicalHit {
            chunk_id: 1,
            file_path: "test.rs".into(),
            language: "rust".into(),
            repository_id: None,
            score: 1.0,
        }];
        let result = engine.rerank("normalize json", hits, 10);
        assert_eq!(result.hits.len(), 1);
        assert!(result.hits[0].identifier_score > 0.0);
    }

    #[test]
    fn fallback_telemetry_reports_latencies() {
        let graph = Arc::new(PmiGraph::default());
        let artifacts = FallbackArtifacts::new(graph.clone());
        let chunk = stub_chunk(2, &["async", "runtime"]);
        artifacts.record_chunk(&chunk, &NGramVector::from_text(&chunk.text));
        let weights = FallbackWeights::default();
        let engine = FallbackEngine::new(Arc::new(artifacts), weights);
        let hits = vec![LexicalHit {
            chunk_id: 2,
            file_path: "async.rs".into(),
            language: "rust".into(),
            repository_id: None,
            score: 1.0,
        }];
        let result = engine.rerank("async runtime", hits, 5);
        assert!(result.telemetry.total_latency_ms >= result.telemetry.pmi_latency_ms);
        assert!(result.telemetry.total_latency_ms >= result.telemetry.ngram_latency_ms);
        assert!(result.telemetry.pmi_latency_ms >= 0.0);
        assert!(result.telemetry.ngram_latency_ms >= 0.0);
    }

    #[test]
    fn persistence_roundtrip_restores_data() -> Result<(), FallbackPersistenceError> {
        let graph = Arc::new(PmiGraph::default());
        let artifacts = FallbackArtifacts::new(graph.clone());
        let chunk = stub_chunk(5, &["persist"]);
        artifacts.record_chunk(&chunk, &NGramVector::from_text(&chunk.text));

        let dir = tempdir()?;
        artifacts.persist(dir.path())?;
        let loaded = FallbackArtifacts::load(dir.path())?;

        let restored_ngram = loaded
            .ngram(chunk.id)
            .expect("ngram restored")
            .cosine_similarity(&NGramVector::from_text(&chunk.text));
        assert!(restored_ngram > 0.9);

        let identifier = loaded.identifier(chunk.id).expect("identifier restored");
        assert!(identifier.score_overlap(&["persist".into()]) > 0.0);
        Ok(())
    }
}
