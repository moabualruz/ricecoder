use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use qdrant_client::{
    qdrant::{self, value::Kind},
    Payload,
};

use crate::{
    chunking::{Chunk, ChunkProducer, RepositorySource},
    vector::{
        batch::BatchProcessor,
        embeddings::{EmbeddingModelKind, ModelManager},
        fallback::{FallbackArtifacts, NGramVector},
        qdrant::VectorIndexer,
    },
};

pub struct VectorPipeline {
    chunk_producer: ChunkProducer,
    embeddings: Arc<ModelManager>,
    vector_indexer: Arc<VectorIndexer>,
    fallback_artifacts: Arc<FallbackArtifacts>,
    batch_processor: Arc<BatchProcessor>,
}

impl VectorPipeline {
    pub fn new(
        chunk_producer: ChunkProducer,
        embeddings: Arc<ModelManager>,
        vector_indexer: Arc<VectorIndexer>,
        fallback_artifacts: Arc<FallbackArtifacts>,
        batch_processor: Arc<BatchProcessor>,
    ) -> Self {
        Self {
            chunk_producer,
            embeddings,
            vector_indexer,
            fallback_artifacts,
            batch_processor,
        }
    }

    pub async fn ingest_repository(&self, source: RepositorySource) -> Result<()> {
        self.vector_indexer.ensure_collection().await?;
        let stream = self.chunk_producer.chunk_stream(source)?;
        tokio::pin!(stream);
        let mut buffer = Vec::with_capacity(self.embeddings.batch_size());
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            buffer.push(chunk);
            if buffer.len() >= self.embeddings.batch_size() {
                self.flush_batch(&mut buffer).await?;
            }
        }
        if !buffer.is_empty() {
            self.flush_batch(&mut buffer).await?;
        }
        Ok(())
    }

    pub fn switch_embedding_model(&self, kind: EmbeddingModelKind) -> Result<()> {
        self.embeddings.switch_model(kind)?;
        let dim = self.embeddings.active_dimension();
        let index_dim = self.vector_indexer.dimension();
        if dim != index_dim {
            return Err(anyhow!(
                "embedding dimension {} does not match vector index dimension {}",
                dim,
                index_dim
            ));
        }
        Ok(())
    }

    async fn flush_batch(&self, batch: &mut Vec<Chunk>) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }
        let mut chunks = Vec::with_capacity(batch.len());
        std::mem::swap(batch, &mut chunks);
        let embedding_map = self.batch_processor.embed_chunks(&chunks).await?;
        let mut points = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let ngram_vector = NGramVector::from_text(&chunk.text);
            self.fallback_artifacts.record_chunk(&chunk, &ngram_vector);
            let embedding = embedding_map
                .get(&chunk.id)
                .cloned()
                .context("missing embedding for chunk")?;
            if embedding.len() != self.vector_indexer.dimension() {
                return Err(anyhow!(
                    "embedding dimension {} != vector index dimension {}",
                    embedding.len(),
                    self.vector_indexer.dimension()
                ));
            }
            let mut payload_map = HashMap::new();
            payload_map.insert(
                "chunk_id".to_string(),
                qdrant::Value {
                    kind: Some(Kind::IntegerValue(chunk.id as i64)),
                },
            );
            payload_map.insert(
                "file_path".to_string(),
                qdrant::Value {
                    kind: Some(Kind::StringValue(
                        chunk.file_path.to_string_lossy().into_owned(),
                    )),
                },
            );
            payload_map.insert(
                "language".to_string(),
                qdrant::Value {
                    kind: Some(Kind::StringValue(format!("{:?}", chunk.language))),
                },
            );
            payload_map.insert(
                "start_line".to_string(),
                qdrant::Value {
                    kind: Some(Kind::IntegerValue(chunk.start_line as i64)),
                },
            );
            payload_map.insert(
                "end_line".to_string(),
                qdrant::Value {
                    kind: Some(Kind::IntegerValue(chunk.end_line as i64)),
                },
            );
            if let Some(repo_id) = chunk.metadata.repository_id {
                payload_map.insert(
                    "repository_id".to_string(),
                    qdrant::Value {
                        kind: Some(Kind::IntegerValue(repo_id as i64)),
                    },
                );
            }
            let payload: Payload = payload_map.into();
            points.push(qdrant::PointStruct::new(
                qdrant::PointId::from(chunk.id as u64),
                embedding,
                payload,
            ));
        }

        self.vector_indexer.upsert_embeddings(points).await?;
        Ok(())
    }
}
