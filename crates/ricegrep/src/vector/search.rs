//! Hybrid query engine that combines vector + fallback scoring for lexical hits.

use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::{
    lexical::LexicalHit,
    vector::{
        embeddings::ModelManager,
        fallback::{FallbackArtifacts, FallbackEngine, FallbackResult},
        qdrant::{SearchFilters, VectorHit, VectorSearchBackend},
    },
};

pub trait EmbeddingProvider: Send + Sync {
    fn dimension(&self) -> usize;
    fn embed_text(&self, text: &str) -> Result<Vec<f32>>;
}

pub struct HybridQueryEngine {
    embeddings: Arc<dyn EmbeddingProvider>,
    vector_backend: Arc<dyn VectorSearchBackend>,
    fallback_engine: Arc<FallbackEngine>,
    fallback_artifacts: Arc<FallbackArtifacts>,
}

impl HybridQueryEngine {
    pub fn new(
        embeddings: Arc<dyn EmbeddingProvider>,
        vector_backend: Arc<dyn VectorSearchBackend>,
        fallback_engine: Arc<FallbackEngine>,
        fallback_artifacts: Arc<FallbackArtifacts>,
    ) -> Self {
        Self {
            embeddings,
            vector_backend,
            fallback_engine,
            fallback_artifacts,
        }
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        filters: Option<SearchFilters>,
    ) -> Result<FallbackResult> {
        let target_dimension = self.embeddings.dimension();
        if target_dimension == 0 {
            return Err(anyhow!("embedding provider dimension is zero"));
        }
        if self.vector_backend.dimension() != target_dimension {
            return Err(anyhow!("vector backend dimension mismatch"));
        }
        let embedding = self.embeddings.embed_text(query)?;
        let vector_hits = self
            .vector_backend
            .search_vectors(embedding, limit, filters)
            .await?;
        let lexical_hits = vector_hits
            .into_iter()
            .map(|hit| LexicalHit {
                chunk_id: hit.chunk_id,
                file_path: hit.file_path,
                language: hit.language,
                repository_id: hit.repository_id,
                score: hit.score,
            })
            .collect::<Vec<_>>();
        Ok(self.fallback_engine.rerank(query, lexical_hits, limit))
    }
}

impl EmbeddingProvider for ModelManager {
    fn dimension(&self) -> usize {
        self.active_dimension()
    }

    fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        self.active().embed_text(text)
    }
}

#[cfg(test)]
mod tests {
    use tokio::test;

    use super::*;
    use crate::vector::fallback::{FallbackArtifacts, FallbackWeights, PmiGraph};

    struct DummyEmbedding {
        dimension: usize,
    }

    impl EmbeddingProvider for DummyEmbedding {
        fn dimension(&self) -> usize {
            self.dimension
        }

        fn embed_text(&self, _text: &str) -> Result<Vec<f32>> {
            Ok(vec![0.1; self.dimension])
        }
    }

    struct DummyVectorBackend {
        dimension: usize,
    }

    #[async_trait]
    impl VectorSearchBackend for DummyVectorBackend {
        fn dimension(&self) -> usize {
            self.dimension
        }

        async fn search_vectors(
            &self,
            _vector: Vec<f32>,
            limit: usize,
            _filters: Option<SearchFilters>,
        ) -> Result<Vec<VectorHit>> {
            Ok((0..limit)
                .map(|i| VectorHit {
                    chunk_id: i as u64,
                    score: 1.0 / (i as f32 + 1.0),
                    file_path: format!("file_{i}.rs"),
                    language: "Rust".into(),
                    repository_id: None,
                })
                .collect())
        }
    }

    #[test]
    async fn hybrid_search_returns_hits() -> Result<()> {
        let embeddings = Arc::new(DummyEmbedding { dimension: 8 });
        let vector_backend = Arc::new(DummyVectorBackend { dimension: 8 });
        let artifacts = Arc::new(FallbackArtifacts::new(Arc::new(PmiGraph::default())));
        let fallback_engine = Arc::new(FallbackEngine::new(
            artifacts.clone(),
            FallbackWeights::default(),
        ));
        let engine = HybridQueryEngine::new(embeddings, vector_backend, fallback_engine, artifacts);
        let result = engine.search("test", 3, None).await?;
        assert_eq!(result.hits.len(), 3);
        assert!(result.telemetry.total_latency_ms >= result.telemetry.pmi_latency_ms);
        assert!(result.telemetry.total_latency_ms >= result.telemetry.ngram_latency_ms);
        Ok(())
    }
}
