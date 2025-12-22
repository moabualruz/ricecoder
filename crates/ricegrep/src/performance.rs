use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::lexical::{
    Bm25IndexBuilder, Bm25IndexHandle, LexicalError, LexicalHit, LexicalSearcher,
};

/// Retrieval modes exercised by the benchmarking harness.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkMode {
    Bm25,
    Ann,
    Hybrid,
    Fallback,
}

/// Describes a single benchmark query and its ground truth chunks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkQuery {
    pub id: String,
    pub query: String,
    pub expected_chunk_ids: Vec<u64>,
    pub k: usize,
}

impl BenchmarkQuery {
    pub fn new(
        id: impl Into<String>,
        query: impl Into<String>,
        expected: Vec<u64>,
        k: usize,
    ) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
            expected_chunk_ids: expected,
            k: k.max(1),
        }
    }
}

/// Results produced by running the benchmark harness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub mode: BenchmarkMode,
    pub recall_at_k: f64,
    pub precision_at_k: f64,
    pub mrr: f64,
    pub ndcg: f64,
    pub median_latency_ms: f64,
    pub throughput_qps: f64,
    pub duration_ms: f64,
    pub total_queries: usize,
}

/// Errors emitted by benchmarking operations.
#[derive(Debug, Error)]
pub enum BenchmarkError {
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("lexical failure: {0}")]
    Lexical(#[from] LexicalError),
}

/// A simple benchmarking harness that reads queries from a golden set and evaluates them over a BM25 index.
pub struct BenchmarkHarness {
    index_dir: PathBuf,
    queries: Vec<BenchmarkQuery>,
    duration_bucket: Duration,
}

impl BenchmarkHarness {
    pub fn new(index_dir: PathBuf, queries: Vec<BenchmarkQuery>) -> Self {
        Self {
            index_dir,
            queries,
            duration_bucket: Duration::from_secs(30),
        }
    }

    pub fn with_bucket(mut self, duration: Duration) -> Self {
        self.duration_bucket = duration;
        self
    }

    pub fn run(&self, mode: BenchmarkMode) -> Result<BenchmarkResult, BenchmarkError> {
        let handle = Bm25IndexHandle::open(&self.index_dir)?;
        let searcher = LexicalSearcher::new(handle);
        let mut recall_acc = 0.0;
        let mut precision_acc = 0.0;
        let mut mrr_acc = 0.0;
        let mut ndcg_acc = 0.0;
        let mut durations = Vec::with_capacity(self.queries.len());

        for query in &self.queries {
            let now = Instant::now();
            let hits = searcher.search(&query.query, query.k)?;
            let elapsed = now.elapsed();
            durations.push(elapsed);

            let found: HashSet<u64> = hits.iter().map(|hit| hit.chunk_id).collect();
            let relevant = query.expected_chunk_ids.len().max(1);
            let true_positives = query
                .expected_chunk_ids
                .iter()
                .filter(|id| found.contains(id))
                .count();
            recall_acc += true_positives as f64 / relevant as f64;
            precision_acc += if hits.is_empty() {
                1.0
            } else {
                true_positives as f64 / hits.len() as f64
            };

            let mut mrr = 0.0;
            for (rank, hit) in hits.iter().enumerate() {
                if query.expected_chunk_ids.contains(&hit.chunk_id) {
                    mrr = 1.0 / (rank as f64 + 1.0);
                    break;
                }
            }
            mrr_acc += mrr;
            ndcg_acc += dcg(&query, &hits) / idcg(query.expected_chunk_ids.len(), query.k);
        }

        let total = (self.queries.len()).max(1) as f64;
        let throughput_qps = durations.len() as f64
            / durations
                .iter()
                .copied()
                .map(|dur| dur.as_secs_f64())
                .sum::<f64>()
                .max(1e-9);
        let median_latency_ms = median_duration_ms(&durations);
        let duration_ms = durations.iter().map(|dur| dur.as_millis()).sum::<u128>() as f64;

        Ok(BenchmarkResult {
            mode,
            recall_at_k: recall_acc / total,
            precision_at_k: precision_acc / total,
            mrr: mrr_acc / total,
            ndcg: ndcg_acc / total,
            median_latency_ms,
            throughput_qps,
            duration_ms,
            total_queries: self.queries.len(),
        })
    }

    pub fn default_queries() -> Vec<BenchmarkQuery> {
        vec![
            BenchmarkQuery::new("alpha", "fast search", vec![0], 5),
            BenchmarkQuery::new("beta", "search chunk 1", vec![1], 5),
        ]
    }
}

fn idcg(relevant: usize, k: usize) -> f64 {
    let k = k.max(1);
    let mut total = 0.0;
    for rank in 1..=k.min(relevant) {
        total += 1.0 / ((rank + 1) as f64).log2();
    }
    total.max(1e-9)
}

fn dcg(query: &BenchmarkQuery, hits: &[LexicalHit]) -> f64 {
    hits.iter()
        .enumerate()
        .filter(|(_, hit)| query.expected_chunk_ids.contains(&hit.chunk_id))
        .map(|(rank, _)| 1.0 / ((rank + 2) as f64).log2())
        .sum::<f64>()
}

fn median_duration_ms(durations: &[Duration]) -> f64 {
    if durations.is_empty() {
        return 0.0;
    }
    let mut sorted = durations.to_owned();
    sorted.sort();
    let median = sorted[sorted.len() / 2];
    median.as_secs_f64() * 1_000.0
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::chunking::{Chunk, ChunkMetadata, LanguageKind};
    use crate::lexical::Bm25IndexWriter;

    fn stub_chunk(id: u64, text: &str) -> Chunk {
        let tokens: Vec<String> = text.split_whitespace().map(|t| t.to_string()).collect();
        let metadata = ChunkMetadata {
            chunk_id: id,
            repository_id: Some(1),
            file_path: PathBuf::from(format!("chunk_{id}.rs")),
            language: LanguageKind::Rust,
            start_line: 1,
            end_line: 1,
            token_count: tokens.len() as u32,
            checksum: format!("chk-{id}"),
        };
        Chunk {
            id,
            language: LanguageKind::Rust,
            file_path: metadata.file_path.clone(),
            start_line: 1,
            end_line: 1,
            text: text.into(),
            identifiers: tokens.clone(),
            identifier_tokens: tokens,
            metadata,
        }
    }

    #[test]
    fn benchmark_run_returns_valid_metrics() {
        let index_dir = tempdir().unwrap();
        let builder = Bm25IndexBuilder::create(index_dir.path()).expect("build index");
        let mut writer = builder.writer(50_000_000).expect("create writer");
        for idx in 0..10 {
            writer
                .add_chunk(&stub_chunk(idx, &format!("fast search test chunk {idx}")))
                .expect("add chunk");
        }
        let handle = writer.commit().expect("commit index");
        let queries = BenchmarkHarness::default_queries();
        let harness = BenchmarkHarness::new(index_dir.path().to_path_buf(), queries);
        let result = harness.run(BenchmarkMode::Bm25).expect("run benchmark");
        assert!(result.recall_at_k >= 0.0 && result.recall_at_k <= 1.0);
        assert!(result.precision_at_k >= 0.0 && result.precision_at_k <= 1.0);
        assert!(result.mrr >= 0.0 && result.mrr <= 1.0);
        assert!(result.ndcg >= 0.0 && result.ndcg <= 1.0);
        assert!(result.median_latency_ms >= 0.0);
    }
}
