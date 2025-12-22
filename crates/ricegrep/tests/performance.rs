use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use ricegrep::{
    chunking::{Chunk, ChunkMetadata, LanguageKind},
    lexical::{Bm25IndexBuilder, LexicalSearcher},
};
use tempfile::tempdir;

const TARGET_MEDIAN_LATENCY: Duration = Duration::from_millis(2);
const TARGET_THROUGHPUT_QPS: f64 = 400.0;
const MEASUREMENT_ROUNDS: usize = 64;

fn stub_chunk(id: u64, repository_id: Option<u32>, text: &str) -> Chunk {
    let tokens: Vec<String> = text
        .split_whitespace()
        .map(|token| token.to_string())
        .collect();
    let metadata = ChunkMetadata {
        chunk_id: id,
        repository_id,
        file_path: PathBuf::from(format!("chunk_{id}.rs")),
        language: LanguageKind::Rust,
        start_line: 1,
        end_line: 1,
        token_count: tokens.len() as u32,
        checksum: format!("chk{id}"),
    };
    Chunk {
        id,
        language: LanguageKind::Rust,
        file_path: metadata.file_path.clone(),
        start_line: 1,
        end_line: 1,
        text: text.to_string(),
        identifiers: tokens.clone(),
        identifier_tokens: tokens,
        metadata,
    }
}

fn build_searcher(num_chunks: usize) -> LexicalSearcher {
    let index_dir = tempdir().expect("temp index directory");
    let builder = Bm25IndexBuilder::create(index_dir.path()).expect("build index");
    let mut writer = builder.writer(50_000_000).expect("create writer");
    for chunk_id in 0..num_chunks as u64 {
        let text = format!(
            "fast lexical search chunk {chunk_id} alpha beta gamma",
            chunk_id = chunk_id
        );
        writer
            .add_chunk(&stub_chunk(chunk_id, Some((chunk_id % 4) as u32), &text))
            .expect("add chunk");
    }
    let handle = writer.commit().expect("commit index");
    LexicalSearcher::new(handle)
}

#[test]
fn lexical_query_latency_and_throughput_meet_targets() {
    let searcher = build_searcher(512);
    // warm-up run to prime the reader
    searcher.search("alpha", 5).expect("warm up query");

    let mut durations = Vec::with_capacity(MEASUREMENT_ROUNDS);
    let mut hits_seen = 0;
    for idx in 0..MEASUREMENT_ROUNDS {
        let query = if idx % 3 == 0 { "alpha" } else { "search" };
        let start = Instant::now();
        let hits = searcher.search(query, 5).expect("search query");
        durations.push(start.elapsed());
        hits_seen += hits.len();
    }

    assert!(hits_seen > 0, "queries should return results");

    durations.sort();
    let median = durations[durations.len() / 2];
    assert!(
        median <= TARGET_MEDIAN_LATENCY,
        "median latency ({median:?}) exceeded {TARGET_MEDIAN_LATENCY:?}, target is derived from F-1.2.8"
    );

    let total_duration: Duration = durations.iter().copied().sum();
    let throughput_qps = (durations.len() as f64) / total_duration.as_secs_f64();
    assert!(
        throughput_qps >= TARGET_THROUGHPUT_QPS,
        "throughput ({throughput_qps:.1} qps) fell below the {TARGET_THROUGHPUT_QPS} qps target derived from 2ms median latency"
    );
}
