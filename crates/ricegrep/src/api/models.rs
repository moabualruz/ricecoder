use serde::{Deserialize, Serialize};

use crate::{
    chunking::ChunkMetadata,
    performance::{BenchmarkMode, BenchmarkQuery, BenchmarkResult},
    vector::{alerting::AlertSummary, metrics_storage::MetricsHistoryEntry},
};
use std::collections::HashMap;

/// Filters that may be supplied alongside a search request.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    pub repository_id: Option<u32>,
    pub language: Option<String>,
    pub file_path_pattern: Option<String>,
}

/// Search request payload delivered to the API handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub filters: Option<SearchFilters>,
    pub ranking: Option<RankingConfig>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingConfig {
    pub lexical_weight: f32,
    pub semantic_weight: f32,
    pub rrf_k: usize,
}

/// Response returned by the API search endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_found: usize,
    pub query_time_ms: f64,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk_id: u64,
    pub score: f32,
    pub content: String,
    pub metadata: ChunkMetadata,
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub filters: Option<SearchFilters>,
    pub ranking: Option<RankingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKConfig {
    pub language: String,
    pub version: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EnrichedQuery {
    pub original: String,
    pub tokens: Vec<String>,
    pub intent_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub message: String,
    pub alerts: Vec<AlertSummary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlertActionRequest {
    pub actor: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BenchmarkRequest {
    pub mode: Option<BenchmarkMode>,
    pub queries: Option<Vec<BenchmarkQuery>>,
    pub run_suite: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkResponse {
    pub mode: BenchmarkMode,
    pub summary: String,
    pub result: BenchmarkResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkSuiteResponse {
    pub summary: String,
    pub results: HashMap<BenchmarkMode, BenchmarkResult>,
}
