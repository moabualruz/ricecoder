use std::{
    collections::{HashMap, HashSet},
    future::Future,
    sync::Arc,
    time::Instant,
};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use parking_lot::Mutex;
use qdrant_client::{
    qdrant::{
        self, value::Kind, Condition, CreateCollection, Distance, Filter, HnswConfigDiff,
        MaxOptimizationThreadsBuilder, OptimizersConfigDiffBuilder, PointStruct,
        PointsOperationResponse, QuantizationConfig, QuantizationType, ScalarQuantization,
        SearchParams, SearchPoints, UpdateCollectionBuilder, UpsertPoints, Value, VectorParams,
    },
    Qdrant,
};
use serde::Deserialize;
use tracing::warn;

use crate::vector::observability::{
    VectorError, VectorHealth, VectorHealthStatus, VectorTelemetry,
};

#[derive(Clone, Deserialize)]
pub struct QdrantConfig {
    pub uri: String,
    pub collection_name: String,
    pub dimension: u64,
    #[serde(default = "default_m")]
    pub m: u64,
    #[serde(default = "default_ef_construct")]
    pub ef_construct: u64,
    #[serde(default = "default_ef_search")]
    pub ef_search: u64,
    #[serde(default = "default_enable_sharding")]
    pub enable_sharding: bool,
}

fn default_m() -> u64 {
    32
}

fn default_ef_construct() -> u64 {
    256
}

fn default_ef_search() -> u64 {
    256
}

fn default_enable_sharding() -> bool {
    true
}

#[derive(Clone, Debug)]
pub struct SearchFilters {
    pub language: Option<String>,
    pub repository_id: Option<u32>,
    pub file_path_pattern: Option<String>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            language: None,
            repository_id: None,
            file_path_pattern: None,
        }
    }
}

pub struct VectorIndexer {
    client: Qdrant,
    config: QdrantConfig,
    shard_cache: Mutex<HashSet<String>>,
    telemetry: Arc<VectorTelemetry>,
    health: Arc<VectorHealth>,
}

impl VectorIndexer {
    pub async fn new(config: QdrantConfig) -> Result<Self> {
        Self::with_observability(
            config,
            Arc::new(VectorTelemetry::default()),
            Arc::new(VectorHealth::default()),
        )
        .await
    }

    pub async fn with_observability(
        config: QdrantConfig,
        telemetry: Arc<VectorTelemetry>,
        health: Arc<VectorHealth>,
    ) -> Result<Self> {
        let client = Qdrant::from_url(&config.uri).build()?;
        Ok(Self {
            client,
            config,
            shard_cache: Mutex::new(HashSet::new()),
            telemetry,
            health,
        })
    }

    pub async fn ensure_collection(&self) -> Result<()> {
        let exists = self
            .call_with_retry("ensure_collection", || async {
                let name = self.config.collection_name.clone();
                self.client
                    .collection_exists(&name)
                    .await
                    .map_err(Into::into)
            })
            .await?;
        if !exists {
            self.create_collection(&self.config.collection_name).await?;
        }
        Ok(())
    }

    pub async fn upsert_embeddings(
        &self,
        points: Vec<PointStruct>,
    ) -> Result<PointsOperationResponse> {
        if points.is_empty() {
            return Ok(PointsOperationResponse {
                result: None,
                time: 0.0,
                usage: None,
            });
        }
        let shared_points = Arc::new(points);
        self.call_with_retry("upsert", || {
            let points = shared_points.clone();
            async move {
                let response = self
                    .client
                    .upsert_points(UpsertPoints {
                        collection_name: self.config.collection_name.clone(),
                        wait: Some(true),
                        points: points.as_ref().clone(),
                        ..Default::default()
                    })
                    .await?;
                if self.config.enable_sharding {
                    self.upsert_to_shards(points.as_ref()).await?;
                }
                Ok(response)
            }
        })
        .await
    }

    pub async fn search_points(
        &self,
        vector: Vec<f32>,
        limit: usize,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<VectorHit>> {
        let shared_vector = Arc::new(vector);
        let filters = filters.clone();
        self.call_with_retry("search", || {
            let vector = shared_vector.clone();
            let filters = filters.clone();
            async move { self.search_points_inner(vector, limit, filters).await }
        })
        .await
    }

    async fn search_points_inner(
        &self,
        vector: Arc<Vec<f32>>,
        limit: usize,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<VectorHit>> {
        let response = self
            .client
            .search_points(SearchPoints {
                collection_name: self.config.collection_name.clone(),
                vector: vector.as_ref().clone(),
                limit: limit as u64,
                params: Some(SearchParams {
                    hnsw_ef: Some(self.config.ef_search),
                    ..Default::default()
                }),
                filter: Self::build_filter(filters),
                with_payload: Some(qdrant::WithPayloadSelector {
                    selector_options: Some(qdrant::with_payload_selector::SelectorOptions::Enable(
                        true,
                    )),
                }),
                ..Default::default()
            })
            .await?;

        let mut hits = Vec::with_capacity(response.result.len());
        for point in response.result {
            let chunk_id = point
                .id
                .as_ref()
                .and_then(|point_id| {
                    point_id
                        .point_id_options
                        .as_ref()
                        .and_then(|inner| match inner {
                            qdrant::point_id::PointIdOptions::Num(num) => Some(*num as u64),
                            _ => None,
                        })
                })
                .unwrap_or_default();
            let payload = &point.payload;
            let file_path = payload_to_string(payload.get("file_path")).unwrap_or_default();
            let language = payload_to_string(payload.get("language")).unwrap_or_default();
            let repository_id = payload_to_i64(payload.get("repository_id")).map(|id| id as u32);
            hits.push(VectorHit {
                chunk_id,
                score: point.score,
                file_path,
                language,
                repository_id,
            });
        }
        Ok(hits)
    }

    async fn call_with_retry<T, F, Fut>(&self, stage: &'static str, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        loop {
            let start = Instant::now();
            match operation().await {
                Ok(value) => {
                    self.telemetry.record_qdrant(start.elapsed());
                    self.health.record_success();
                    return Ok(value);
                }
                Err(err) => {
                    let message = err.to_string();
                    let vector_error = VectorError::qdrant(stage, message.clone());
                    self.telemetry.record_error(&vector_error);
                    self.health.record_failure(message.clone());
                    attempts += 1;
                    if attempts > 1 {
                        return Err(err);
                    }
                    warn!(stage = stage, message = "retrying after error", reason = %message);
                }
            }
        }
    }

    pub fn dimension(&self) -> usize {
        self.config.dimension as usize
    }

    async fn create_collection(&self, name: &str) -> Result<()> {
        let start = Instant::now();
        let quantization = QuantizationConfig {
            quantization: Some(qdrant::quantization_config::Quantization::Scalar(
                ScalarQuantization {
                    r#type: QuantizationType::Int8 as i32,
                    quantile: Some(0.95),
                    always_ram: Some(false),
                },
            )),
        };
        let optimizers = OptimizersConfigDiffBuilder::default()
            .memmap_threshold(1024)
            .indexing_threshold(20_000)
            .max_optimization_threads(MaxOptimizationThreadsBuilder::auto())
            .build();
        let result = self
            .client
            .create_collection(CreateCollection {
                collection_name: name.to_string(),
                vectors_config: Some(qdrant::VectorsConfig {
                    config: Some(qdrant::vectors_config::Config::Params(VectorParams {
                        size: self.config.dimension,
                        distance: Distance::Cosine as i32,
                        ..Default::default()
                    })),
                }),
                hnsw_config: Some(HnswConfigDiff {
                    m: Some(self.config.m),
                    ef_construct: Some(self.config.ef_construct),
                    ..Default::default()
                }),
                optimizers_config: Some(optimizers.clone()),
                quantization_config: Some(quantization),
                ..Default::default()
            })
            .await;
        self.telemetry.record_qdrant(start.elapsed());
        match result {
            Ok(_) => {
                self.health.record_success();
                Ok(())
            }
            Err(err) => {
                let message = err.to_string();
                let vector_error = VectorError::qdrant("create_collection", message.clone());
                self.telemetry.record_error(&vector_error);
                self.health.record_failure(message);
                Err(err.into())
            }
        }
    }

    pub async fn optimize_collection(&self) -> Result<()> {
        let optimizers = OptimizersConfigDiffBuilder::default()
            .memmap_threshold(2048)
            .indexing_threshold(15_000)
            .max_optimization_threads(MaxOptimizationThreadsBuilder::auto())
            .build();
        let builder = UpdateCollectionBuilder::new(&self.config.collection_name)
            .optimizers_config(optimizers);
        self.client.update_collection(builder).await?;
        Ok(())
    }

    async fn upsert_to_shards(&self, points: &[PointStruct]) -> Result<()> {
        for point in points.iter() {
            if let Some(name) = self.shard_name_for_payload(&point.payload) {
                let mut shards = self.shard_cache.lock();
                if shards.insert(name.clone()) {
                    self.create_collection(&name).await?;
                }
                drop(shards);
                let shard_name = name.clone();
                let payload_point = point.clone();
                self.call_with_retry("shard_upsert", || {
                    let shard_name = shard_name.clone();
                    let payload_point = payload_point.clone();
                    async move {
                        self.client
                            .upsert_points(UpsertPoints {
                                collection_name: shard_name,
                                wait: Some(true),
                                points: vec![payload_point],
                                ..Default::default()
                            })
                            .await
                            .map(|_| ())
                            .map_err(Into::into)
                    }
                })
                .await?;
            }
        }
        Ok(())
    }

    fn shard_name_for_payload(&self, payload: &HashMap<String, Value>) -> Option<String> {
        let language = payload_to_string(payload.get("language"))
            .unwrap_or_default()
            .to_lowercase();
        if language.is_empty() {
            return None;
        }
        let repo_id = payload_to_i64(payload.get("repository_id")).map(|id| id as u32);
        let language_slug = language
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>();
        let repo_slug = repo_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "global".to_string());
        Some(format!(
            "{}_shard_{}_{}",
            self.config.collection_name, repo_slug, language_slug
        ))
    }

    pub async fn health_check(&self) -> Result<VectorHealthStatus> {
        let name = self.config.collection_name.clone();
        self.call_with_retry("health_check", || {
            let name = name.clone();
            async move {
                let exists = self.client.collection_exists(&name).await?;
                if exists {
                    Ok(())
                } else {
                    Err(anyhow!("collection {} missing", name))
                }
            }
        })
        .await
        .map(|_| self.health.status())
    }

    fn build_filter(filters: Option<SearchFilters>) -> Option<Filter> {
        let mut conditions = Vec::new();
        if let Some(filters) = filters {
            if let Some(repo) = filters.repository_id {
                conditions.push(Condition::matches("repository_id", repo as i64));
            }
            if let Some(lang) = filters.language {
                conditions.push(Condition::matches("language", lang));
            }
            if let Some(pattern) = filters.file_path_pattern {
                conditions.push(Condition::matches_text("file_path", pattern));
            }
        }
        if conditions.is_empty() {
            None
        } else {
            Some(Filter {
                must: conditions,
                ..Default::default()
            })
        }
    }
}

fn payload_to_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|value| value.kind.as_ref())
        .and_then(|kind| match kind {
            Kind::StringValue(text) => Some(text.clone()),
            _ => None,
        })
}

fn payload_to_i64(value: Option<&Value>) -> Option<i64> {
    value
        .and_then(|value| value.kind.as_ref())
        .and_then(|kind| match kind {
            Kind::IntegerValue(val) => Some(*val),
            _ => None,
        })
}

#[derive(Debug, Clone)]
pub struct VectorHit {
    pub chunk_id: u64,
    pub score: f32,
    pub file_path: String,
    pub language: String,
    pub repository_id: Option<u32>,
}

#[async_trait]
pub trait VectorSearchBackend: Send + Sync {
    fn dimension(&self) -> usize;
    async fn search_vectors(
        &self,
        vector: Vec<f32>,
        limit: usize,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<VectorHit>>;
}

#[async_trait]
impl VectorSearchBackend for VectorIndexer {
    fn dimension(&self) -> usize {
        self.dimension()
    }

    async fn search_vectors(
        &self,
        vector: Vec<f32>,
        limit: usize,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<VectorHit>> {
        self.search_points(vector, limit, filters).await
    }
}
