use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::chunking::{ChunkProducerBuilder, ChunkingConfig, RepositorySource};
use crate::lexical::{
    Bm25IndexBuilder, Bm25IndexHandle, LexicalError, LexicalIngestPipeline, LexicalIngestStats,
};
use crate::metadata::{MetadataError, MetadataWriter};
use crate::vector::observability::VectorTelemetry;

/// Administrative actions exposed via the API surface.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdminAction {
    Reindex,
    Optimize,
    ClearCache,
    UpdateConfig,
}

impl std::fmt::Display for AdminAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            AdminAction::Reindex => "reindex",
            AdminAction::Optimize => "optimize",
            AdminAction::ClearCache => "clear_cache",
            AdminAction::UpdateConfig => "update_config",
        };
        write!(f, "{}", label)
    }
}

/// Request body for admin commands.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdminCommandRequest {
    pub action: AdminAction,
    pub repository_path: Option<PathBuf>,
    pub config_key: Option<String>,
    pub config_value: Option<String>,
}

/// Response body returned after executing an admin action.
#[derive(Debug, Clone, Serialize)]
pub struct AdminCommandResponse {
    pub action: AdminAction,
    pub summary: String,
    pub stats: Option<LexicalIngestStats>,
}

/// Errors emitted by admin operations.
#[derive(Debug, Error)]
pub enum AdminError {
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("lexical failure: {0}")]
    Lexical(#[from] LexicalError),
    #[error("metadata failure: {0}")]
    Metadata(#[from] MetadataError),
    #[error("invalid parameters: {0}")]
    Validation(String),
}

/// Toolset that drives admin commands such as reindexing and cache management.
pub struct AdminToolset {
    index_dir: PathBuf,
    chunking_config: ChunkingConfig,
    telemetry: Option<Arc<VectorTelemetry>>,
    heap_bytes: usize,
    batch_size: usize,
    progress_interval: usize,
    config_overrides: Mutex<HashMap<String, String>>,
    /// If true, ignore .gitignore, .ignore, and other ignore files during scanning.
    no_ignore: bool,
}

impl AdminToolset {
    pub fn new(index_dir: PathBuf, telemetry: Option<Arc<VectorTelemetry>>) -> Self {
        Self {
            index_dir,
            chunking_config: ChunkingConfig::default(),
            telemetry,
            heap_bytes: 50_000_000,
            batch_size: 256,
            progress_interval: 1_000,
            config_overrides: Mutex::new(HashMap::new()),
            no_ignore: false,
        }
    }

    pub fn with_heap_bytes(mut self, heap_bytes: usize) -> Self {
        self.heap_bytes = heap_bytes;
        self
    }

    pub fn with_progress_interval(mut self, interval: usize) -> Self {
        self.progress_interval = interval.max(1);
        self
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size.max(1);
        self
    }

    /// If true, ignore .gitignore, .ignore, and other ignore files during scanning.
    pub fn with_no_ignore(mut self, no_ignore: bool) -> Self {
        self.no_ignore = no_ignore;
        self
    }

    pub async fn reindex_repository(
        &self,
        repository: &Path,
    ) -> Result<LexicalIngestStats, AdminError> {
        self.prepare_index_directory()?;
        let chunk_producer = ChunkProducerBuilder::default()
            .config(self.chunking_config.clone())
            .no_ignore(self.no_ignore)
            .build();
        let mut pipeline = LexicalIngestPipeline::new(chunk_producer)
            .with_batch_size(self.batch_size)
            .with_progress_interval(self.progress_interval);
        if let Some(telemetry) = &self.telemetry {
            pipeline = pipeline.with_telemetry(telemetry.clone());
        }
        let builder = Bm25IndexBuilder::create(&self.index_dir)?;
        let mut writer = builder.writer(self.heap_bytes)?;
        let stats = pipeline
            .ingest_repository(RepositorySource::new(repository), &mut writer)
            .await?;
        writer.commit()?;
        Ok(stats)
    }

    pub async fn reindex_repository_with_metadata(
        &self,
        repository: &Path,
        metadata_path: &Path,
    ) -> Result<LexicalIngestStats, AdminError> {
        self.prepare_index_directory()?;
        let chunk_producer = ChunkProducerBuilder::default()
            .config(self.chunking_config.clone())
            .no_ignore(self.no_ignore)
            .build();
        let mut pipeline = LexicalIngestPipeline::new(chunk_producer)
            .with_batch_size(self.batch_size)
            .with_progress_interval(self.progress_interval);
        if let Some(telemetry) = &self.telemetry {
            pipeline = pipeline.with_telemetry(telemetry.clone());
        }
        let builder = Bm25IndexBuilder::create(&self.index_dir)?;
        let mut writer = builder.writer(self.heap_bytes)?;
        let mut metadata_writer = MetadataWriter::new();
        let stats = pipeline
            .ingest_repository_with_metadata(
                RepositorySource::new(repository),
                &mut writer,
                &mut metadata_writer,
            )
            .await?;
        writer.commit()?;
        metadata_writer.finalize(metadata_path)?;
        Ok(stats)
    }

    pub fn optimize_index(&self) -> Result<(), AdminError> {
        let handle = Bm25IndexHandle::open(&self.index_dir)?;
        handle.optimize()?;
        Ok(())
    }

    pub fn clear_cache(&self) -> Result<(), AdminError> {
        let cache_dir = self.index_dir.join("cache");
        if cache_dir.exists() {
            fs::remove_dir_all(cache_dir)?;
        }
        Ok(())
    }

    pub fn update_config(&self, key: String, value: String) {
        let mut overrides = self.config_overrides.lock().unwrap();
        overrides.insert(key, value);
    }

    fn prepare_index_directory(&self) -> Result<(), std::io::Error> {
        if self.index_dir.exists() {
            fs::remove_dir_all(&self.index_dir)?;
        }
        fs::create_dir_all(&self.index_dir)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::AdminToolset;

    #[tokio::test]
    async fn reindex_repository_returns_stats() {
        let repo_dir = tempdir().expect("repo dir");
        let src = repo_dir.path().join("src");
        tokio::fs::create_dir_all(&src)
            .await
            .expect("create src dir");
        tokio::fs::write(src.join("lib.rs"), "pub fn admin_test() {}\n")
            .await
            .expect("write source");

        let index_dir = tempdir().expect("index dir");
        let toolset = AdminToolset::new(index_dir.path().to_path_buf(), None);
        let stats = toolset
            .reindex_repository(repo_dir.path())
            .await
            .expect("reindex should succeed");
        assert!(stats.chunks_indexed > 0, "expected some chunks indexed");
    }
}
