use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridWeights {
    pub lexical_weight: f32,
    pub semantic_weight: f32,
    pub rrf_k: usize,
    pub adaptive_weighting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FusionMethod {
    ReciprocalRankFusion { k: usize },
    WeightedInterpolation { weights: HybridWeights },
    LearnedFusion { model_path: String },
    AdaptiveFusion,
}

#[derive(Debug, Clone)]
pub struct FusionConfig {
    pub method: FusionMethod,
    pub candidate_limit: usize,
    pub quality_threshold: f32,
    pub enable_diversity: bool,
}

#[derive(Debug, Clone)]
pub struct CandidateChunk {
    pub chunk_id: u64,
    pub score: f32,
    pub source: RetrievalSource,
    pub normalized_score: f32,
}

#[derive(Debug, Clone)]
pub enum RetrievalSource {
    BM25,
    ANN,
    Hybrid,
}

#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub ndcg_at_10: f32,
    pub precision_at_5: f32,
    pub mrr: f32,
    pub fusion_confidence: f32,
}

#[derive(Debug, Clone)]
pub struct FusedResults {
    pub results: Vec<CandidateChunk>,
    pub fusion_method: FusionMethod,
    pub quality_metrics: QualityMetrics,
    pub processing_time_ms: f64,
}
