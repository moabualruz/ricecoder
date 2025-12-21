//! Provider curation and optimization system
//!
//! This module provides intelligent provider selection, quality scoring,
//! reliability monitoring, and automatic optimization based on performance data.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::error::ProviderError;
use crate::models::{ModelInfo, TokenUsage};
use crate::performance_monitor::{
    PerformanceThresholds, ProviderMetrics, ProviderPerformanceMonitor,
};

/// Quality score for a provider (0.0 to 1.0)
#[derive(Debug, Clone)]
pub struct QualityScore {
    /// Overall quality score
    pub overall: f64,
    /// Speed score (0.0 to 1.0)
    pub speed: f64,
    /// Reliability score (0.0 to 1.0)
    pub reliability: f64,
    /// Cost efficiency score (0.0 to 1.0)
    pub cost_efficiency: f64,
    /// Feature completeness score (0.0 to 1.0)
    pub features: f64,
    /// Last updated timestamp
    pub last_updated: SystemTime,
}

/// Reliability status of a provider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReliabilityStatus {
    /// Provider is highly reliable
    Excellent,
    /// Provider is generally reliable
    Good,
    /// Provider has some issues but is usable
    Fair,
    /// Provider has significant issues
    Poor,
    /// Provider is currently failing
    Critical,
}

/// Provider curation configuration
#[derive(Debug, Clone)]
pub struct CurationConfig {
    /// Minimum quality score to consider a provider (0.0 to 1.0)
    pub min_quality_score: f64,
    /// Minimum reliability score to consider a provider (0.0 to 1.0)
    pub min_reliability_score: f64,
    /// Maximum consecutive failures before marking as critical
    pub max_consecutive_failures: usize,
    /// Time window for reliability calculation
    pub reliability_window: Duration,
    /// Enable automatic provider switching
    pub auto_switch_enabled: bool,
    /// Cost optimization weight (0.0 to 1.0)
    pub cost_weight: f64,
    /// Speed optimization weight (0.0 to 1.0)
    pub speed_weight: f64,
    /// Reliability optimization weight (0.0 to 1.0)
    pub reliability_weight: f64,
}

impl Default for CurationConfig {
    fn default() -> Self {
        Self {
            min_quality_score: 0.6,
            min_reliability_score: 0.8,
            max_consecutive_failures: 5,
            reliability_window: Duration::from_secs(3600), // 1 hour
            auto_switch_enabled: true,
            cost_weight: 0.3,
            speed_weight: 0.4,
            reliability_weight: 0.3,
        }
    }
}

/// Provider reliability tracker
#[derive(Debug, Clone)]
pub struct ReliabilityTracker {
    /// Provider ID
    provider_id: String,
    /// Consecutive failure count
    consecutive_failures: usize,
    /// Total failures in reliability window
    total_failures: usize,
    /// Total requests in reliability window
    total_requests: usize,
    /// Last failure timestamp
    last_failure: Option<SystemTime>,
    /// Reliability status
    status: ReliabilityStatus,
}

impl ReliabilityTracker {
    /// Create a new reliability tracker
    pub fn new(provider_id: String) -> Self {
        Self {
            provider_id,
            consecutive_failures: 0,
            total_failures: 0,
            total_requests: 0,
            last_failure: None,
            status: ReliabilityStatus::Good,
        }
    }

    /// Record a successful request
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.total_requests += 1;
        self.update_status();
    }

    /// Record a failed request
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.total_failures += 1;
        self.total_requests += 1;
        self.last_failure = Some(SystemTime::now());
        self.update_status();
    }

    /// Get current reliability score (0.0 to 1.0)
    pub fn reliability_score(&self) -> f64 {
        if self.total_requests == 0 {
            1.0 // No data means assume reliable
        } else {
            let failure_rate = self.total_failures as f64 / self.total_requests as f64;
            1.0 - failure_rate.min(1.0)
        }
    }

    /// Get consecutive failure penalty
    pub fn consecutive_failure_penalty(&self) -> f64 {
        if self.consecutive_failures == 0 {
            0.0
        } else {
            // Exponential penalty for consecutive failures
            1.0 - (1.0 / (self.consecutive_failures as f64 + 1.0))
        }
    }

    /// Update reliability status based on current metrics
    fn update_status(&mut self) {
        let score = self.reliability_score();
        let penalty = self.consecutive_failure_penalty();

        self.status = if self.consecutive_failures >= 10 {
            ReliabilityStatus::Critical
        } else if score < 0.5 || penalty > 0.8 {
            ReliabilityStatus::Poor
        } else if score < 0.7 || penalty > 0.5 {
            ReliabilityStatus::Fair
        } else if score < 0.9 || penalty > 0.2 {
            ReliabilityStatus::Good
        } else {
            ReliabilityStatus::Excellent
        };
    }

    /// Check if provider should be avoided
    pub fn should_avoid(&self, config: &CurationConfig) -> bool {
        self.consecutive_failures >= config.max_consecutive_failures
            || self.reliability_score() < config.min_reliability_score
    }
}

/// Provider curator for intelligent provider selection and optimization
pub struct ProviderCurator {
    config: CurationConfig,
    quality_scores: HashMap<String, QualityScore>,
    reliability_trackers: HashMap<String, ReliabilityTracker>,
    performance_monitor: Arc<ProviderPerformanceMonitor>,
}

impl ProviderCurator {
    /// Create a new provider curator
    pub fn new(
        config: CurationConfig,
        performance_monitor: Arc<ProviderPerformanceMonitor>,
    ) -> Self {
        Self {
            config,
            quality_scores: HashMap::new(),
            reliability_trackers: HashMap::new(),
            performance_monitor,
        }
    }

    /// Create a curator with default configuration
    pub fn default(performance_monitor: Arc<ProviderPerformanceMonitor>) -> Self {
        Self::new(CurationConfig::default(), performance_monitor)
    }

    /// Calculate quality score for a provider
    pub fn calculate_quality_score(&self, provider_id: &str, models: &[ModelInfo]) -> QualityScore {
        let metrics = self.performance_monitor.get_metrics(provider_id);

        let speed_score = self.calculate_speed_score(&metrics);
        let reliability_score = self.calculate_reliability_score(provider_id);
        let cost_score = self.calculate_cost_score(&metrics);
        let feature_score = self.calculate_feature_score(models);

        let overall = (speed_score * self.config.speed_weight)
            + (reliability_score * self.config.reliability_weight)
            + (cost_score * self.config.cost_weight)
            + (feature_score * 0.1); // Features get 10% weight

        QualityScore {
            overall: overall.min(1.0),
            speed: speed_score,
            reliability: reliability_score,
            cost_efficiency: cost_score,
            features: feature_score,
            last_updated: SystemTime::now(),
        }
    }

    /// Calculate speed score based on response times
    fn calculate_speed_score(&self, metrics: &Option<ProviderMetrics>) -> f64 {
        if let Some(metrics) = metrics {
            if metrics.total_requests == 0 {
                0.5 // Neutral score for no data
            } else {
                // Score based on average response time
                // Lower response time = higher score
                let avg_time = metrics.avg_response_time_ms;
                if avg_time <= 500.0 {
                    1.0 // Excellent (< 500ms)
                } else if avg_time <= 2000.0 {
                    0.8 // Good (500ms - 2s)
                } else if avg_time <= 5000.0 {
                    0.6 // Fair (2s - 5s)
                } else if avg_time <= 10000.0 {
                    0.4 // Poor (5s - 10s)
                } else {
                    0.2 // Very poor (> 10s)
                }
            }
        } else {
            0.5
        }
    }

    /// Calculate reliability score
    fn calculate_reliability_score(&self, provider_id: &str) -> f64 {
        if let Some(tracker) = self.reliability_trackers.get(provider_id) {
            tracker.reliability_score()
        } else {
            0.8 // Default good score for new providers
        }
    }

    /// Calculate cost efficiency score
    fn calculate_cost_score(&self, metrics: &Option<ProviderMetrics>) -> f64 {
        if let Some(metrics) = metrics {
            if metrics.total_requests == 0 || metrics.total_cost == 0.0 {
                0.7 // Neutral-good score for no cost data
            } else {
                let avg_cost = metrics.total_cost / metrics.total_requests as f64;
                if avg_cost <= 0.001 {
                    1.0 // Excellent (< $0.001 per request)
                } else if avg_cost <= 0.01 {
                    0.8 // Good ($0.001 - $0.01)
                } else if avg_cost <= 0.05 {
                    0.6 // Fair ($0.01 - $0.05)
                } else if avg_cost <= 0.1 {
                    0.4 // Poor ($0.05 - $0.1)
                } else {
                    0.2 // Very expensive (>$0.1)
                }
            }
        } else {
            0.7
        }
    }

    /// Calculate feature completeness score
    fn calculate_feature_score(&self, models: &[ModelInfo]) -> f64 {
        if models.is_empty() {
            0.0
        } else {
            let mut score: f64 = 0.0;
            let model_count = models.len();

            // Score based on model variety and capabilities
            if model_count >= 5 {
                score += 0.3; // Good variety
            } else if model_count >= 3 {
                score += 0.2; // Decent variety
            } else if model_count >= 1 {
                score += 0.1; // Basic offering
            }

            // Check for advanced capabilities
            let has_vision = models
                .iter()
                .any(|m| m.capabilities.contains(&crate::models::Capability::Vision));
            let has_function_calling = models.iter().any(|m| {
                m.capabilities
                    .contains(&crate::models::Capability::FunctionCalling)
            });
            let has_streaming = models.iter().any(|m| {
                m.capabilities
                    .contains(&crate::models::Capability::Streaming)
            });

            if has_vision {
                score += 0.2;
            }
            if has_function_calling {
                score += 0.2;
            }
            if has_streaming {
                score += 0.2;
            }

            // Check for free models
            let has_free_models = models.iter().any(|m| m.is_free);
            if has_free_models {
                score += 0.1;
            }

            score.min(1.0f64)
        }
    }

    /// Update quality scores for all providers
    pub fn update_quality_scores(&mut self, provider_models: &HashMap<String, Vec<ModelInfo>>) {
        for (provider_id, models) in provider_models {
            let score = self.calculate_quality_score(provider_id, models);
            self.quality_scores.insert(provider_id.clone(), score);
        }
    }

    /// Get quality score for a provider
    pub fn get_quality_score(&self, provider_id: &str) -> Option<&QualityScore> {
        self.quality_scores.get(provider_id)
    }

    /// Get all quality scores
    pub fn get_all_quality_scores(&self) -> &HashMap<String, QualityScore> {
        &self.quality_scores
    }

    /// Record a successful request for reliability tracking
    pub fn record_success(&mut self, provider_id: &str) {
        let tracker = self
            .reliability_trackers
            .entry(provider_id.to_string())
            .or_insert_with(|| ReliabilityTracker::new(provider_id.to_string()));
        tracker.record_success();
    }

    /// Record a failed request for reliability tracking
    pub fn record_failure(&mut self, provider_id: &str) {
        let tracker = self
            .reliability_trackers
            .entry(provider_id.to_string())
            .or_insert_with(|| ReliabilityTracker::new(provider_id.to_string()));
        tracker.record_failure();
    }

    /// Get reliability tracker for a provider
    pub fn get_reliability_tracker(&self, provider_id: &str) -> Option<&ReliabilityTracker> {
        self.reliability_trackers.get(provider_id)
    }

    /// Select the best provider based on quality scores and constraints
    pub fn select_best_provider(
        &self,
        provider_ids: &[String],
        constraints: Option<&SelectionConstraints>,
    ) -> Option<String> {
        let default_constraints = SelectionConstraints::default();
        let constraints = constraints.unwrap_or(&default_constraints);

        provider_ids
            .iter()
            .filter(|id| self.is_provider_eligible(id, constraints))
            .max_by(|a, b| {
                let score_a = self
                    .quality_scores
                    .get(*a)
                    .map(|s| s.overall)
                    .unwrap_or(0.0);
                let score_b = self
                    .quality_scores
                    .get(*b)
                    .map(|s| s.overall)
                    .unwrap_or(0.0);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    /// Check if a provider is eligible based on constraints
    pub fn is_provider_eligible(
        &self,
        provider_id: &str,
        constraints: &SelectionConstraints,
    ) -> bool {
        // Check quality score
        if let Some(score) = self.quality_scores.get(provider_id) {
            if score.overall < constraints.min_quality_score {
                return false;
            }
        } else if constraints.require_quality_score {
            return false;
        }

        // Check reliability
        if let Some(tracker) = self.reliability_trackers.get(provider_id) {
            if tracker.should_avoid(&self.config) {
                return false;
            }
        }

        // Check performance thresholds
        if let Some(metrics) = self.performance_monitor.get_metrics(provider_id) {
            if !metrics.is_performing_well(&constraints.performance_thresholds) {
                return false;
            }
        } else if constraints.require_performance_data {
            return false;
        }

        true
    }

    /// Get providers sorted by quality score
    pub fn get_providers_by_quality(&self, provider_ids: &[String]) -> Vec<(String, f64)> {
        let mut providers: Vec<(String, f64)> = provider_ids
            .iter()
            .map(|id| {
                let score = self
                    .quality_scores
                    .get(id)
                    .map(|s| s.overall)
                    .unwrap_or(0.0);
                (id.clone(), score)
            })
            .collect();

        providers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        providers
    }

    /// Get curation configuration
    pub fn config(&self) -> &CurationConfig {
        &self.config
    }

    /// Update curation configuration
    pub fn update_config(&mut self, config: CurationConfig) {
        self.config = config;
    }
}

/// Constraints for provider selection
#[derive(Debug, Clone)]
pub struct SelectionConstraints {
    /// Minimum quality score required
    pub min_quality_score: f64,
    /// Whether quality score is required (vs optional)
    pub require_quality_score: bool,
    /// Whether performance data is required
    pub require_performance_data: bool,
    /// Performance thresholds to meet
    pub performance_thresholds: PerformanceThresholds,
    /// Maximum cost per request
    pub max_cost_per_request: Option<f64>,
    /// Required capabilities
    pub required_capabilities: Vec<crate::models::Capability>,
}

impl Default for SelectionConstraints {
    fn default() -> Self {
        Self {
            min_quality_score: 0.5,
            require_quality_score: false,
            require_performance_data: false,
            performance_thresholds: PerformanceThresholds::default(),
            max_cost_per_request: None,
            required_capabilities: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Capability;

    #[test]
    fn test_quality_score_calculation() {
        let monitor = Arc::new(ProviderPerformanceMonitor::default());
        let curator = ProviderCurator::default(monitor);

        let models = vec![ModelInfo {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            context_window: 8192,
            capabilities: vec![Capability::Chat, Capability::FunctionCalling],
            pricing: None,
            is_free: false,
        }];

        let score = curator.calculate_quality_score("openai", &models);
        assert!(score.overall >= 0.0 && score.overall <= 1.0);
        assert!(score.speed >= 0.0 && score.speed <= 1.0);
        assert!(score.reliability >= 0.0 && score.reliability <= 1.0);
        assert!(score.cost_efficiency >= 0.0 && score.cost_efficiency <= 1.0);
        assert!(score.features >= 0.0 && score.features <= 1.0);
    }

    #[test]
    fn test_reliability_tracking() {
        let monitor = Arc::new(ProviderPerformanceMonitor::default());
        let mut curator = ProviderCurator::default(monitor);

        // Record some successes and failures
        curator.record_success("test_provider");
        curator.record_success("test_provider");
        curator.record_failure("test_provider");
        curator.record_success("test_provider");

        let tracker = curator.get_reliability_tracker("test_provider").unwrap();
        assert_eq!(tracker.total_requests, 4);
        assert_eq!(tracker.total_failures, 1);
        assert_eq!(tracker.reliability_score(), 0.75);
    }

    #[test]
    fn test_provider_selection() {
        let monitor = Arc::new(ProviderPerformanceMonitor::default());
        let mut curator = ProviderCurator::default(monitor);

        // Set up quality scores
        let mut provider_models = HashMap::new();
        provider_models.insert(
            "provider_a".to_string(),
            vec![ModelInfo {
                id: "model1".to_string(),
                name: "Model 1".to_string(),
                provider: "provider_a".to_string(),
                context_window: 4096,
                capabilities: vec![Capability::Chat],
                pricing: None,
                is_free: false,
            }],
        );
        provider_models.insert(
            "provider_b".to_string(),
            vec![ModelInfo {
                id: "model2".to_string(),
                name: "Model 2".to_string(),
                provider: "provider_b".to_string(),
                context_window: 8192,
                capabilities: vec![Capability::Chat, Capability::FunctionCalling],
                pricing: None,
                is_free: false,
            }],
        );

        curator.update_quality_scores(&provider_models);

        let providers = vec!["provider_a".to_string(), "provider_b".to_string()];
        let best = curator.select_best_provider(&providers, None);

        // Should select one of the providers
        assert!(best.is_some());
        assert!(providers.contains(&best.unwrap()));
    }
}
