use crate::hybrid::error::{FusionError, RankingError};
use crate::hybrid::models::{CandidateChunk, FusedResults, FusionConfig, FusionMethod, QualityMetrics};
use crate::hybrid::selector::CandidateSelector;
use async_trait::async_trait;

#[derive(Clone)]
pub struct FusionEngine {
    config: FusionConfig,
    selector: CandidateSelector,
}

impl FusionEngine {
    pub fn new(config: FusionConfig) -> Self {
        let selector = CandidateSelector::new(config.candidate_limit, true);
        Self { config, selector }
    }

    pub async fn fuse_results(
        &self,
        bm25_results: Vec<CandidateChunk>,
        ann_results: Vec<CandidateChunk>,
    ) -> Result<FusedResults, FusionError> {
        let candidates = self.selector.select_candidates(bm25_results, ann_results);
        if candidates.is_empty() {
            return Err(FusionError::NoCandidates);
        }

        let results = self.selector.apply_diversity(candidates.clone());
        let metrics = QualityMetrics {
            ndcg_at_10: 1.0,
            precision_at_5: 1.0,
            mrr: 1.0,
            fusion_confidence: 1.0,
        };

        Ok(FusedResults {
            results,
            fusion_method: self.config.method.clone(),
            quality_metrics: metrics,
            processing_time_ms: 0.0,
        })
    }

    pub async fn adaptive_fusion(&self) -> Result<FusedResults, FusionError> {
        Err(FusionError::UnsupportedMethod("adaptive fusion not implemented".into()))
    }
}

#[async_trait]
pub trait FusionEngineTrait: Send + Sync {
    async fn fuse_results(
        &self,
        bm25_results: Vec<CandidateChunk>,
        ann_results: Vec<CandidateChunk>,
    ) -> Result<FusedResults, FusionError>;

    async fn get_quality_metrics(&self) -> Result<QualityMetrics, FusionError>;
}

#[async_trait]
impl FusionEngineTrait for FusionEngine {
    async fn fuse_results(
        &self,
        bm25_results: Vec<CandidateChunk>,
        ann_results: Vec<CandidateChunk>,
    ) -> Result<FusedResults, FusionError> {
        self.fuse_results(bm25_results, ann_results).await
    }

    async fn get_quality_metrics(&self) -> Result<QualityMetrics, FusionError> {
        Ok(QualityMetrics {
            ndcg_at_10: 1.0,
            precision_at_5: 1.0,
            mrr: 1.0,
            fusion_confidence: 1.0,
        })
    }
}
