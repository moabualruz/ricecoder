//! Batch processor for embedding requests with concurrency controls.

use std::{sync::Arc, time::Instant};

use anyhow::{anyhow, Result};
use futures::stream::{self, StreamExt};

use crate::{
    chunking::Chunk,
    vector::{
        embeddings::ModelManager,
        observability::{VectorError, VectorTelemetry},
    },
};

pub struct BatchProcessor {
    manager: Arc<ModelManager>,
    concurrency: usize,
    telemetry: Option<Arc<VectorTelemetry>>,
}

impl BatchProcessor {
    pub fn new(manager: Arc<ModelManager>, concurrency: usize) -> Self {
        let concurrency = concurrency.max(1);
        Self {
            manager,
            concurrency,
            telemetry: None,
        }
    }

    pub fn with_telemetry(
        manager: Arc<ModelManager>,
        concurrency: usize,
        telemetry: Arc<VectorTelemetry>,
    ) -> Self {
        let concurrency = concurrency.max(1);
        Self {
            manager,
            concurrency,
            telemetry: Some(telemetry),
        }
    }

    pub async fn embed_chunks(
        &self,
        chunks: &[Chunk],
    ) -> Result<std::collections::HashMap<u64, Vec<f32>>> {
        if chunks.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let batch_size = self.manager.batch_size();
        let batches: Vec<Vec<Chunk>> = chunks
            .chunks(batch_size)
            .map(|slice| slice.to_vec())
            .collect();
        let mut results = std::collections::HashMap::new();
        let stream = stream::iter(batches.into_iter().map(|batch| {
            let manager = self.manager.clone();
            tokio::task::spawn_blocking(move || manager.active().embed_batch(&batch))
        }))
        .buffer_unordered(self.concurrency);
        tokio::pin!(stream);
        while let Some(handle) = stream.next().await {
            let start = Instant::now();
            let pair = match handle {
                Ok(Ok(pair)) => pair,
                Ok(Err(err)) => {
                    if let Some(telemetry) = &self.telemetry {
                        telemetry.record_error(&VectorError::embedding(err.to_string()));
                    }
                    return Err(err);
                }
                Err(join_err) => {
                    let err = anyhow!("batch worker cancelled: {join_err}");
                    if let Some(telemetry) = &self.telemetry {
                        telemetry.record_error(&VectorError::embedding(err.to_string()));
                    }
                    return Err(err);
                }
            };
            if let Some(telemetry) = &self.telemetry {
                telemetry.record_embedding(start.elapsed(), pair.len());
            }
            for (id, embedding) in pair {
                results.insert(id, embedding);
            }
        }
        Ok(results)
    }
}
