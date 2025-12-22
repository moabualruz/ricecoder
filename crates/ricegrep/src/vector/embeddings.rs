use std::{collections::HashMap, num::NonZeroUsize, path::PathBuf, sync::Arc};

use anyhow::{anyhow, bail, Context, Result};
use lru::LruCache;
use ndarray::Array2;
use once_cell::sync::Lazy;
use onnxruntime::{
    environment::Environment, session::Session, tensor::OrtOwnedTensor, GraphOptimizationLevel,
    LoggingLevel,
};
use parking_lot::{Mutex, RwLock};
use serde::Deserialize;
use tokenizers::{EncodeInput, Tokenizer};

use crate::{chunking::Chunk, vector::observability::VectorTelemetry};

static ENVIRONMENT: Lazy<Environment> = Lazy::new(|| {
    Environment::builder()
        .with_name("ricegrep-embeddings")
        .with_log_level(LoggingLevel::Warning)
        .build()
        .expect("failed to create ONNX environment")
});

const MIN_DIMENSION: usize = 256;
const MAX_DIMENSION: usize = 2048;

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash)]
pub enum EmbeddingModelKind {
    VoyageCode3,
    CodeSageLargeV2,
    JinaCodeV2,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingModelConfig {
    pub model_path: PathBuf,
    pub tokenizer_path: PathBuf,
    pub dimension: usize,
    #[serde(default = "default_max_length")]
    pub max_length: usize,
    #[serde(default = "default_input_tensor")]
    pub input_tensor: String,
    #[serde(default = "default_attention_tensor")]
    pub attention_tensor: String,
    #[serde(default = "default_output_tensor")]
    pub output_tensor: String,
    #[serde(default = "default_normalize")]
    pub normalize: bool,
}

fn default_max_length() -> usize {
    512
}

fn default_input_tensor() -> String {
    "input_ids".to_string()
}

fn default_attention_tensor() -> String {
    "attention_mask".to_string()
}

fn default_output_tensor() -> String {
    "last_hidden_state".to_string()
}

fn default_normalize() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingConfig {
    pub batch_size: usize,
    pub cache_size: usize,
    pub models: HashMap<EmbeddingModelKind, EmbeddingModelConfig>,
    pub active_model: EmbeddingModelKind,
}

pub struct EmbeddingGenerator {
    tokenizer: Tokenizer,
    session: Mutex<Session<'static>>,
    config: EmbeddingModelConfig,
    cache: Mutex<LruCache<u64, Vec<f32>>>,
    runtime_dimension: Mutex<Option<usize>>,
    telemetry: Option<Arc<VectorTelemetry>>,
}

// The ONNX Runtime session is not `Send`/`Sync` by default, but we guard calls with a mutex and
// never expose the raw pointer outside this type, so reasoning about thread safety allows us to mark
// the generator `Send + Sync` manually to satisfy the broader pipeline requirements.
unsafe impl Send for EmbeddingGenerator {}
unsafe impl Sync for EmbeddingGenerator {}

impl EmbeddingGenerator {
    pub fn load(
        config: &EmbeddingModelConfig,
        cache_size: usize,
        telemetry: Option<Arc<VectorTelemetry>>,
    ) -> Result<Self> {
        let config = config.clone();
        let tokenizer = Tokenizer::from_file(config.tokenizer_path.clone())
            .map_err(|e| anyhow!("loading embedding tokenizer: {e}"))?;
        let session = ENVIRONMENT
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::All)?
            .with_model_from_file(config.model_path.clone())?;
        Ok(Self {
            tokenizer,
            session: Mutex::new(session),
            config,
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(cache_size).unwrap_or(NonZeroUsize::new(1024).unwrap()),
            )),
            runtime_dimension: Mutex::new(None),
            telemetry,
        })
    }

    pub fn embed_chunk(&self, chunk: &Chunk) -> Result<Vec<f32>> {
        {
            let mut cache = self.cache.lock();
            if let Some(embedding) = cache.get(&chunk.id).cloned() {
                self.record_cache_event(true);
                return Ok(embedding);
            }
        }
        let embedding = self.embed_text(&chunk.text)?;
        {
            let mut cache = self.cache.lock();
            cache.put(chunk.id, embedding.clone());
        }
        self.record_cache_event(false);
        Ok(embedding)
    }

    pub fn embed_batch(&self, chunks: &[Chunk]) -> Result<Vec<(u64, Vec<f32>)>> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }
        let mut results = Vec::with_capacity(chunks.len());
        let mut pending_indices = Vec::new();
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

        if !pending_indices.is_empty() {
            let mut texts = Vec::with_capacity(pending_indices.len());
            for idx in &pending_indices {
                texts.push(chunks[*idx].text.clone());
            }
            let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
            let embeddings = self.embed_texts(&refs)?;
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

    fn embed_texts(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let (input_ids, attention) = self.encode_batch(texts)?;
        let mut session_guard = self.session.lock();
        let outputs: Vec<OrtOwnedTensor<f32, _>> =
            session_guard.run(vec![input_ids.into_dyn(), attention.into_dyn()])?;
        let tensor = outputs
            .get(0)
            .context("embedding model returned no tensor")?;
        let view = tensor.view();
        let shape = view.shape().to_vec();
        if shape.len() != 2 {
            bail!("expected 2D embedding output, got {:?}", shape);
        }
        let batch = shape[0] as usize;
        let dim = shape[1] as usize;
        self.update_dimension(dim)?;
        let mut result = Vec::with_capacity(batch);
        for row in 0..batch {
            let mut values = Vec::with_capacity(dim);
            for col in 0..dim {
                values.push(view[[row, col]]);
            }
            if self.config.normalize {
                normalize(&mut values);
            }
            result.push(values);
        }
        Ok(result)
    }

    fn update_dimension(&self, dim: usize) -> Result<()> {
        if dim < MIN_DIMENSION || dim > MAX_DIMENSION {
            bail!(
                "embedding dimension {} outside supported range {}-{}",
                dim,
                MIN_DIMENSION,
                MAX_DIMENSION
            );
        }
        let mut guard = self.runtime_dimension.lock();
        if let Some(existing) = *guard {
            if existing != dim {
                bail!(
                    "model dimension changed from {} to {} during inference",
                    existing,
                    dim
                );
            }
        } else {
            *guard = Some(dim);
        }
        if self.config.dimension != 0 && self.config.dimension != dim {
            bail!(
                "configured dimension {} does not match actual output dimension {}",
                self.config.dimension,
                dim
            );
        }
        Ok(())
    }

    pub fn dimension(&self) -> usize {
        match *self.runtime_dimension.lock() {
            Some(dim) => dim,
            None => self.config.dimension,
        }
    }

    fn encode_batch(&self, texts: &[&str]) -> Result<(Array2<i64>, Array2<i64>)> {
        let batch = texts.len();
        let max_len = self.config.max_length;
        let mut ids = Array2::<i64>::zeros((batch, max_len));
        let mut mask = Array2::<i64>::zeros((batch, max_len));
        let inputs: Vec<EncodeInput> = texts.iter().map(|text| (*text).into()).collect();
        let encodings = self
            .tokenizer
            .encode_batch(inputs, true)
            .map_err(|e| anyhow!("tokenizer encode batch failed: {e}"))?;
        for (row, encoding) in encodings.into_iter().enumerate() {
            let tokens = encoding.get_ids();
            let attn = encoding.get_attention_mask();
            for (col, token) in tokens.iter().take(max_len).enumerate() {
                ids[[row, col]] = *token as i64;
            }
            for (col, flag) in attn.iter().take(max_len).enumerate() {
                mask[[row, col]] = *flag as i64;
            }
        }
        Ok((ids, mask))
    }

    pub(crate) fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_texts(&[text])?;
        embeddings
            .into_iter()
            .next()
            .context("embedding inference returned no values")
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

pub struct ModelManager {
    models: HashMap<EmbeddingModelKind, Arc<EmbeddingGenerator>>,
    active_model: RwLock<EmbeddingModelKind>,
    batch_size: usize,
}

impl ModelManager {
    pub fn new(config: EmbeddingConfig, telemetry: Option<Arc<VectorTelemetry>>) -> Result<Self> {
        let mut models = HashMap::new();
        for (kind, model_cfg) in config.models.iter() {
            models.insert(
                kind.clone(),
                Arc::new(EmbeddingGenerator::load(
                    model_cfg,
                    config.cache_size,
                    telemetry.clone(),
                )?),
            );
        }
        Ok(Self {
            models,
            active_model: RwLock::new(config.active_model),
            batch_size: config.batch_size,
        })
    }

    pub fn switch_model(&self, kind: EmbeddingModelKind) -> Result<()> {
        if !self.models.contains_key(&kind) {
            anyhow::bail!("Model {kind:?} not loaded");
        }
        *self.active_model.write() = kind;
        Ok(())
    }

    pub fn active(&self) -> Arc<EmbeddingGenerator> {
        let kind = self.active_model.read().clone();
        self.models
            .get(&kind)
            .expect("active model missing")
            .clone()
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    pub fn active_dimension(&self) -> usize {
        self.active().dimension()
    }
}

fn normalize(values: &mut [f32]) {
    let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in values {
            *v /= norm;
        }
    }
}
