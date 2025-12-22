use std::{
    collections::HashSet,
    io,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::Serialize;

use futures::StreamExt;
use tokio::pin;
use tracing::{info, warn};
use walkdir::WalkDir;

use crate::{
    chunking::{Chunk, ChunkProducer, RepositorySource},
    lexical::{errors::LexicalResult, Bm25IndexWriter},
    metadata::MetadataWriter,
    vector::observability::{VectorIndexStats, VectorTelemetry},
};

const DEFAULT_BATCH_SIZE: usize = 256;
const DEFAULT_PROGRESS_INTERVAL: usize = 1_000;

/// Statistics returned after an ingestion run.
#[derive(Debug, Clone, Serialize)]
pub struct LexicalIngestStats {
    pub chunks_indexed: usize,
    pub files_indexed: usize,
    pub errors: usize,
    pub duration: Duration,
}

impl LexicalIngestStats {
    fn new() -> Self {
        Self {
            chunks_indexed: 0,
            files_indexed: 0,
            errors: 0,
            duration: Duration::default(),
        }
    }
}

/// Streaming ingestion pipeline for BM25 indexing.
pub struct LexicalIngestPipeline {
    chunk_producer: ChunkProducer,
    batch_size: usize,
    progress_interval: usize,
    telemetry: Option<Arc<VectorTelemetry>>,
}

impl LexicalIngestPipeline {
    pub fn new(chunk_producer: ChunkProducer) -> Self {
        Self {
            chunk_producer,
            batch_size: DEFAULT_BATCH_SIZE,
            progress_interval: DEFAULT_PROGRESS_INTERVAL,
            telemetry: None,
        }
    }

    /// Override the batch size used before flushing documents to Tantivy.
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size.max(1);
        self
    }

    /// Override how often (in number of chunks) progress is logged.
    pub fn with_progress_interval(mut self, interval: usize) -> Self {
        self.progress_interval = interval.max(1);
        self
    }

    pub fn with_telemetry(mut self, telemetry: Arc<VectorTelemetry>) -> Self {
        self.telemetry = Some(telemetry);
        self
    }

    /// Streams chunks from the repository into the BM25 index writer.
    pub async fn ingest_repository(
        &self,
        source: RepositorySource,
        writer: &mut Bm25IndexWriter,
    ) -> LexicalResult<LexicalIngestStats> {
        self.ingest_repository_inner(source, writer, None).await
    }

    pub async fn ingest_repository_with_metadata(
        &self,
        source: RepositorySource,
        writer: &mut Bm25IndexWriter,
        metadata_writer: &mut MetadataWriter,
    ) -> LexicalResult<LexicalIngestStats> {
        self.ingest_repository_inner(source, writer, Some(metadata_writer))
            .await
    }

    async fn ingest_repository_inner(
        &self,
        source: RepositorySource,
        writer: &mut Bm25IndexWriter,
        mut metadata_writer: Option<&mut MetadataWriter>,
    ) -> LexicalResult<LexicalIngestStats> {
        let mut stats = LexicalIngestStats::new();
        let start = Instant::now();
        let stream = self.chunk_producer.chunk_stream(source)?;
        pin!(stream);

        let mut batch = Vec::with_capacity(self.batch_size);
        let mut files_seen = HashSet::new();
        let mut last_file = None::<String>;

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    let current_path = chunk.file_path.to_string_lossy().to_string();
                    if files_seen.insert(chunk.file_path.clone()) {
                        stats.files_indexed += 1;
                    }
                    if let Some(writer) = metadata_writer.as_deref_mut() {
                        writer.add_chunk(&chunk);
                    }
                    batch.push(chunk);
                    if batch.len() >= self.batch_size {
                        self.flush_batch(writer, &batch)?;
                        batch.clear();
                    }
                    stats.chunks_indexed += 1;
                    last_file = Some(current_path);
                    if stats.chunks_indexed % self.progress_interval == 0 {
                        info!(
                            chunks = stats.chunks_indexed,
                            files = stats.files_indexed,
                            path = last_file.as_deref().unwrap_or("unknown"),
                            "Indexed lexical chunks"
                        );
                    }
                }
                Err(err) => {
                    stats.errors += 1;
                    warn!(%err, "Skipping chunk due to chunking error");
                }
            }
        }

        if !batch.is_empty() {
            self.flush_batch(writer, &batch)?;
        }

        stats.duration = start.elapsed();
        if let Some(telemetry) = &self.telemetry {
            if let Err(err) = emit_index_stats(telemetry, writer, stats.duration) {
                warn!(error = %err, "failed to capture lexical index metrics");
            }
        }
        info!(
            chunks = stats.chunks_indexed,
            files = stats.files_indexed,
            errors = stats.errors,
            elapsed_ms = stats.duration.as_millis(),
            "Completed lexical ingestion"
        );

        Ok(stats)
    }

    fn flush_batch(&self, writer: &mut Bm25IndexWriter, batch: &[Chunk]) -> LexicalResult<()> {
        for chunk in batch {
            writer.add_chunk(chunk)?;
        }
        Ok(())
    }
}

fn emit_index_stats(
    telemetry: &VectorTelemetry,
    writer: &Bm25IndexWriter,
    duration: Duration,
) -> io::Result<()> {
    let size_bytes = directory_size(writer.directory())?;
    telemetry.record_index_stats(VectorIndexStats {
        index_size_bytes: size_bytes,
        index_build_duration_seconds: duration.as_secs_f64(),
    });
    Ok(())
}

fn directory_size(path: &Path) -> io::Result<u64> {
    let mut size: u64 = 0;
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            size = size.saturating_add(entry.metadata()?.len());
        }
    }
    Ok(size)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use tokio::fs;

    use super::*;
    use crate::vector::observability::VectorTelemetry;
    use crate::{chunking::ChunkProducer, lexical::Bm25IndexBuilder};
    use std::sync::Arc;

    #[tokio::test]
    async fn streams_and_indexes_repository() {
        let repo_dir = tempdir().unwrap();
        fs::create_dir_all(repo_dir.path().join("src"))
            .await
            .unwrap();
        fs::write(
            repo_dir.path().join("src/lib.rs"),
            "pub fn lexical_test() -> usize { 42 }\n",
        )
        .await
        .unwrap();

        let chunk_producer = ChunkProducer::builder().build();
        let telemetry = Arc::new(VectorTelemetry::default());
        let pipeline = LexicalIngestPipeline::new(chunk_producer)
            .with_batch_size(2)
            .with_progress_interval(1)
            .with_telemetry(telemetry.clone());

        let index_dir = tempdir().unwrap();
        let builder = Bm25IndexBuilder::create(index_dir.path()).unwrap();
        let mut writer = builder.writer(50_000_000).unwrap();

        let stats = pipeline
            .ingest_repository(RepositorySource::new(repo_dir.path()), &mut writer)
            .await
            .unwrap();

        assert!(stats.chunks_indexed > 0);
        assert_eq!(stats.errors, 0);

        writer.commit().unwrap();

        let snapshot = telemetry.snapshot();
        assert!(snapshot.index_size_bytes.unwrap_or(0) > 0);
        assert!(snapshot.index_build_duration_seconds.unwrap_or(0.0) > 0.0);
    }
}
