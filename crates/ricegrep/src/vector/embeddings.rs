//! Embedding generation using fastembed for local, offline-first embeddings.
//!
//! This module provides embedding generation capabilities using fastembed,
//! which bundles ONNX models and handles all tokenization internally.

use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use anyhow::{anyhow, Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use lru::LruCache;
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};
use serde::Deserialize;

use crate::{chunking::Chunk, vector::observability::VectorTelemetry};

/// Supported embedding model types.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash)]
pub enum EmbeddingModelKind {
    /// all-MiniLM-L6-v2: Fast, general-purpose (384 dimensions)
    AllMiniLML6V2,
    /// BGE-small-en-v1.5: Good quality/speed balance (384 dimensions)
    BgeSmallEnV15,
    /// BGE-base-en-v1.5: Higher quality (768 dimensions)
    BgeBaseEnV15,
    /// Multilingual-E5-small: Multilingual support (384 dimensions)
    MultilingualE5Small,
}

impl EmbeddingModelKind {
    /// Convert to fastembed's EmbeddingModel enum.
    fn to_fastembed_model(&self) -> EmbeddingModel {
        match self {
            EmbeddingModelKind::AllMiniLML6V2 => EmbeddingModel::AllMiniLML6V2,
            EmbeddingModelKind::BgeSmallEnV15 => EmbeddingModel::BGESmallENV15,
            EmbeddingModelKind::BgeBaseEnV15 => EmbeddingModel::BGEBaseENV15,
            EmbeddingModelKind::MultilingualE5Small => EmbeddingModel::MultilingualE5Small,
        }
    }

    /// Get the embedding dimension for this model.
    fn dimension(&self) -> usize {
        match self {
            EmbeddingModelKind::AllMiniLML6V2 => 384,
            EmbeddingModelKind::BgeSmallEnV15 => 384,
            EmbeddingModelKind::BgeBaseEnV15 => 768,
            EmbeddingModelKind::MultilingualE5Small => 384,
        }
    }
}

impl Default for EmbeddingModelKind {
    fn default() -> Self {
        EmbeddingModelKind::AllMiniLML6V2
    }
}

/// Configuration for the embedding system.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingConfig {
    /// Batch size for embedding multiple texts.
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// LRU cache size for embeddings.
    #[serde(default = "default_cache_size")]
    pub cache_size: usize,
    /// Which model to use.
    #[serde(default)]
    pub active_model: EmbeddingModelKind,
    /// Whether to show download progress when fetching models.
    #[serde(default = "default_show_progress")]
    pub show_download_progress: bool,
}

fn default_batch_size() -> usize {
    32
}

fn default_cache_size() -> usize {
    1024
}

fn default_show_progress() -> bool {
    true
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            batch_size: default_batch_size(),
            cache_size: default_cache_size(),
            active_model: EmbeddingModelKind::default(),
            show_download_progress: default_show_progress(),
        }
    }
}

/// Generates embeddings using fastembed.
pub struct EmbeddingGenerator {
    model: TextEmbedding,
    model_kind: EmbeddingModelKind,
    cache: Mutex<LruCache<u64, Vec<f32>>>,
    telemetry: Option<Arc<VectorTelemetry>>,
}

// TextEmbedding from fastembed is Send + Sync safe
unsafe impl Send for EmbeddingGenerator {}
unsafe impl Sync for EmbeddingGenerator {}

impl EmbeddingGenerator {
    /// Load an embedding model.
    pub fn load(
        model_kind: EmbeddingModelKind,
        cache_size: usize,
        show_progress: bool,
        telemetry: Option<Arc<VectorTelemetry>>,
    ) -> Result<Self> {
        let model = TextEmbedding::try_new(
            InitOptions::new(model_kind.to_fastembed_model())
                .with_show_download_progress(show_progress),
        )
        .map_err(|e| anyhow!("failed to load embedding model: {e}"))?;

        Ok(Self {
            model,
            model_kind,
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(cache_size).unwrap_or(NonZeroUsize::new(1024).unwrap()),
            )),
            telemetry,
        })
    }

    /// Embed a single chunk, using cache if available.
    pub fn embed_chunk(&self, chunk: &Chunk) -> Result<Vec<f32>> {
        // Check cache first
        {
            let mut cache = self.cache.lock();
            if let Some(embedding) = cache.get(&chunk.id).cloned() {
                self.record_cache_event(true);
                return Ok(embedding);
            }
        }

        // Generate embedding
        let embedding = self.embed_text(&chunk.text)?;

        // Cache result
        {
            let mut cache = self.cache.lock();
            cache.put(chunk.id, embedding.clone());
        }
        self.record_cache_event(false);

        Ok(embedding)
    }

    /// Embed multiple chunks in batch, using cache where available.
    pub fn embed_batch(&self, chunks: &[Chunk]) -> Result<Vec<(u64, Vec<f32>)>> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(chunks.len());
        let mut pending_indices = Vec::new();

        // Check cache for existing embeddings
        {
            let mut cache = self.cache.lock();
            for (idx, chunk) in chunks.iter().enumerate() {
                if let Some(hit) = cache.get(&chunk.id).cloned() {
                    results.push((chunk.id, hit));
                    self.record_cache_event(true);
                } else {
                    results.push((chunk.id, Vec::new()));
                    pending_indices.push(idx);
                }
            }
        }

        // Generate embeddings for cache misses
        if !pending_indices.is_empty() {
            let texts: Vec<&str> = pending_indices
                .iter()
                .map(|&idx| chunks[idx].text.as_str())
                .collect();

            let embeddings = self.embed_texts(&texts)?;

            let mut cache = self.cache.lock();
            for (slot_idx, embedding) in pending_indices.into_iter().zip(embeddings.into_iter()) {
                let chunk = &chunks[slot_idx];
                cache.put(chunk.id, embedding.clone());
                results[slot_idx] = (chunk.id, embedding);
                self.record_cache_event(false);
            }
        }

        Ok(results)
    }

    /// Embed multiple texts in batch.
    fn embed_texts(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let documents: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        
        self.model
            .embed(documents, None)
            .map_err(|e| anyhow!("embedding generation failed: {e}"))
    }

    /// Embed a single text.
    pub(crate) fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_texts(&[text])?;
        embeddings
            .into_iter()
            .next()
            .context("embedding inference returned no values")
    }

    /// Get the embedding dimension for the current model.
    pub fn dimension(&self) -> usize {
        self.model_kind.dimension()
    }

    fn record_cache_event(&self, hit: bool) {
        if let Some(telemetry) = &self.telemetry {
            if hit {
                telemetry.record_cache_hit();
            } else {
                telemetry.record_cache_miss();
            }
        }
    }
}

/// Manages multiple embedding models with hot-switching support.
pub struct ModelManager {
    generators: RwLock<HashMap<EmbeddingModelKind, Arc<EmbeddingGenerator>>>,
    active_model: RwLock<EmbeddingModelKind>,
    config: EmbeddingConfig,
    telemetry: Option<Arc<VectorTelemetry>>,
}

impl ModelManager {
    /// Create a new model manager with the given configuration.
    pub fn new(config: EmbeddingConfig, telemetry: Option<Arc<VectorTelemetry>>) -> Result<Self> {
        let manager = Self {
            generators: RwLock::new(HashMap::new()),
            active_model: RwLock::new(config.active_model.clone()),
            config,
            telemetry,
        };

        // Load the active model immediately
        manager.ensure_model_loaded(&manager.config.active_model.clone())?;

        Ok(manager)
    }

    /// Ensure a model is loaded, loading it if necessary.
    fn ensure_model_loaded(&self, kind: &EmbeddingModelKind) -> Result<()> {
        let needs_load = {
            let generators = self.generators.read();
            !generators.contains_key(kind)
        };

        if needs_load {
            let generator = Arc::new(EmbeddingGenerator::load(
                kind.clone(),
                self.config.cache_size,
                self.config.show_download_progress,
                self.telemetry.clone(),
            )?);

            let mut generators = self.generators.write();
            generators.insert(kind.clone(), generator);
        }

        Ok(())
    }

    /// Switch to a different model (lazy-loads if not already loaded).
    pub fn switch_model(&self, kind: EmbeddingModelKind) -> Result<()> {
        self.ensure_model_loaded(&kind)?;
        *self.active_model.write() = kind;
        Ok(())
    }

    /// Get the active embedding generator.
    pub fn active(&self) -> Arc<EmbeddingGenerator> {
        let kind = self.active_model.read().clone();
        let generators = self.generators.read();
        generators
            .get(&kind)
            .expect("active model should be loaded")
            .clone()
    }

    /// Get the configured batch size.
    pub fn batch_size(&self) -> usize {
        self.config.batch_size
    }

    /// Get the embedding dimension of the active model.
    pub fn active_dimension(&self) -> usize {
        self.active().dimension()
    }

    /// List all available model kinds.
    pub fn available_models() -> Vec<EmbeddingModelKind> {
        vec![
            EmbeddingModelKind::AllMiniLML6V2,
            EmbeddingModelKind::BgeSmallEnV15,
            EmbeddingModelKind::BgeBaseEnV15,
            EmbeddingModelKind::MultilingualE5Small,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_kind_dimensions() {
        assert_eq!(EmbeddingModelKind::AllMiniLML6V2.dimension(), 384);
        assert_eq!(EmbeddingModelKind::BgeSmallEnV15.dimension(), 384);
        assert_eq!(EmbeddingModelKind::BgeBaseEnV15.dimension(), 768);
        assert_eq!(EmbeddingModelKind::MultilingualE5Small.dimension(), 384);
    }

    #[test]
    fn test_default_config() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.cache_size, 1024);
        assert!(config.show_download_progress);
    }
}
