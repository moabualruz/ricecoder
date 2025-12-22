use std::{cmp::Ordering, collections::HashMap, path::PathBuf};

use futures::StreamExt;
use tokio::pin;
use tracing::info;

use crate::{
    chunking::{Chunk, ChunkProducer, LanguageKind, RepositorySource},
    lexical::{
        Bm25IndexBuilder, Bm25IndexHandle, Bm25IndexWriter, LexicalConfig, LexicalError,
        LexicalHit, LexicalResult, LexicalSearcher, SearchFilters,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShardKey {
    pub repository_id: Option<u32>,
    pub language: LanguageKind,
}

impl ShardKey {
    pub fn from_chunk(chunk: &Chunk) -> Self {
        Self {
            repository_id: chunk.metadata.repository_id,
            language: chunk.language,
        }
    }

    pub fn matches_filters(&self, filters: &SearchFilters) -> bool {
        if let Some(repo) = filters.repository_id {
            if self.repository_id != Some(repo) {
                return false;
            }
        }
        if let Some(lang) = &filters.language {
            if !format!("{:?}", self.language)
                .to_lowercase()
                .contains(&lang.to_lowercase())
            {
                return false;
            }
        }
        true
    }

    fn path_segment(&self) -> String {
        let repo = self
            .repository_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "none".into());
        format!("repo-{repo}_lang-{}", self.language)
    }
}

pub struct LexicalShardManager {
    base_dir: PathBuf,
    heap_bytes: usize,
    writers: HashMap<ShardKey, Bm25IndexWriter>,
}

impl LexicalShardManager {
    pub fn new<P: Into<PathBuf>>(base_dir: P, heap_bytes: usize) -> Self {
        Self {
            base_dir: base_dir.into(),
            heap_bytes,
            writers: HashMap::new(),
        }
    }

    pub async fn ingest_repository(
        &mut self,
        source: RepositorySource,
        chunk_producer: &ChunkProducer,
    ) -> LexicalResult<LexicalShardSet> {
        let mut handles = HashMap::new();
        let stream = chunk_producer.chunk_stream(source)?;
        pin!(stream);
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            let key = ShardKey::from_chunk(&chunk);
            let writer = self.writer_for(&key)?;
            writer.add_chunk(&chunk)?;
        }

        for (key, writer) in self.writers.drain() {
            let handle = writer.commit()?;
            handles.insert(key, handle);
        }

        info!(shards = handles.len(), "Completed lexical shard ingestion");
        Ok(LexicalShardSet { handles })
    }

    fn writer_for(&mut self, key: &ShardKey) -> LexicalResult<&mut Bm25IndexWriter> {
        if !self.writers.contains_key(key) {
            let path = self.base_dir.join(key.path_segment());
            std::fs::create_dir_all(&path)?;
            let builder = Bm25IndexBuilder::create(&path)?;
            let writer = builder.writer(self.heap_bytes)?;
            self.writers.insert(key.clone(), writer);
        }
        self.writers
            .get_mut(key)
            .ok_or_else(|| LexicalError::Shard("failed to access shard writer".into()))
    }
}

pub struct LexicalShardSet {
    pub handles: HashMap<ShardKey, Bm25IndexHandle>,
}

impl LexicalShardSet {
    pub fn into_searcher(self, config: LexicalConfig) -> ShardedLexicalSearcher {
        let searchers = self
            .handles
            .into_iter()
            .map(|(key, handle)| (key, LexicalSearcher::with_config(handle, config.clone())))
            .collect();
        ShardedLexicalSearcher { searchers }
    }
}

pub struct ShardedLexicalSearcher {
    searchers: Vec<(ShardKey, LexicalSearcher)>,
}

impl ShardedLexicalSearcher {
    pub fn search(
        &self,
        query: &str,
        filters: &SearchFilters,
        limit: usize,
    ) -> LexicalResult<Vec<LexicalHit>> {
        let mut results = Vec::new();
        for (key, searcher) in &self.searchers {
            if !key.matches_filters(filters) {
                continue;
            }
            let shard_results = searcher.search_with_filters(query, filters, limit)?;
            results.extend(shard_results);
        }
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.chunk_id.cmp(&b.chunk_id))
        });
        results.truncate(limit);
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use tokio::fs;

    use super::*;
    use crate::chunking::ChunkProducer;

    #[tokio::test]
    async fn shard_manager_creates_handles() {
        let repo_dir = tempdir().unwrap();
        fs::write(repo_dir.path().join("lib.rs"), "pub fn a() {}\n")
            .await
            .unwrap();

        let chunk_producer = ChunkProducer::builder().build();
        let mut manager = LexicalShardManager::new(tempdir().unwrap().into_path(), 50_000_000);
        let source = RepositorySource::with_repository_id(repo_dir.path(), 7);
        let shards = manager
            .ingest_repository(source, &chunk_producer)
            .await
            .unwrap();
        assert_eq!(shards.handles.len(), 1);
    }

    #[tokio::test]
    async fn sharded_search_filters_languages() {
        let repo_dir = tempdir().unwrap();
        fs::write(repo_dir.path().join("lib.rs"), "pub fn rust_alpha() {}\n")
            .await
            .unwrap();
        fs::write(
            repo_dir.path().join("script.py"),
            "def python_alpha():\n    return 0\n",
        )
        .await
        .unwrap();

        let chunk_producer = ChunkProducer::builder().build();
        let mut manager = LexicalShardManager::new(tempdir().unwrap().into_path(), 50_000_000);
        let source = RepositorySource::with_repository_id(repo_dir.path(), 3);
        let shards = manager
            .ingest_repository(source, &chunk_producer)
            .await
            .unwrap();
        let sharded_searcher = shards.into_searcher(LexicalConfig::default());
        let mut filters = SearchFilters::default();
        filters.language = Some("Python".into());
        let hits = sharded_searcher
            .search("python_alpha", &filters, 10)
            .unwrap();
        assert!(hits.iter().all(|hit| hit.language.contains("Python")));
    }
}
