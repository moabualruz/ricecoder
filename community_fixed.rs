//! Provider community features and contribution system
//!
//! This module provides community-driven provider management including:
//! - Provider contribution and review system
//! - Community-vetted configurations
//! - Provider update synchronization
//! - Usage analytics and sharing

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use crate::error::ProviderError;
use crate::models::{ModelInfo, Pricing};
use crate::curation::{QualityScore, ReliabilityStatus};

/// Community contribution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContributionStatus {
    /// Contribution is pending review
    Pending,
    /// Contribution is under review
    UnderReview,
    /// Contribution has been approved
    Approved,
    /// Contribution has been rejected
    Rejected,
    /// Contribution is deprecated
    Deprecated,
}

/// Community provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityProviderConfig {
    /// Unique contribution ID
    pub id: String,
    /// Provider ID this config is for
    pub provider_id: String,
    /// Provider name
    pub name: String,
    /// Provider description
    pub description: String,
    /// Base URL for the provider
    pub base_url: Option<String>,
    /// Supported models
    pub models: Vec<ModelInfo>,
    /// Default configuration
    pub default_config: ProviderSettings,
    /// Contribution metadata
    pub metadata: ContributionMetadata,
    /// Review status
    pub status: ContributionStatus,
    /// Quality metrics from community testing
    pub quality_metrics: Option<CommunityQualityMetrics>,
}

/// Contribution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionMetadata {
    /// Contributor username/ID
    pub contributor: String,
    /// Contribution timestamp
    pub created_at: SystemTime,
    /// Last updated timestamp
    pub updated_at: SystemTime,
    /// Version of the contribution
    pub version: String,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Compatibility notes
    pub compatibility_notes: Option<String>,
}

/// Community quality metrics from testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityQualityMetrics {
    /// Number of community testers
    pub tester_count: usize,
    /// Average quality score from community
    pub avg_quality_score: f64,
    /// Average reliability score
    pub avg_reliability_score: f64,
    /// Average cost efficiency score
    pub avg_cost_efficiency: f64,
    /// Reported issues count
    pub reported_issues: usize,
    /// Successful test runs
    pub successful_tests: usize,
    /// Total test runs
    pub total_tests: usize,
    /// Last tested timestamp
    pub last_tested: SystemTime,
}

/// Provider usage analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAnalytics {
    /// Provider ID
    pub provider_id: String,
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Total tokens used
    pub total_tokens: u64,
    /// Total cost incurred
    pub total_cost: f64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Unique users
    pub unique_users: u64,
    /// Usage by time periods
    pub usage_by_period: HashMap<String, u64>,
    /// Popular models
    pub popular_models: HashMap<String, u64>,
    /// Error types and frequencies
    pub error_breakdown: HashMap<String, u64>,
    /// Last updated
    pub last_updated: SystemTime,
}

/// Provider settings for community configs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    /// Request timeout
    pub timeout: Option<Duration>,
    /// Number of retries
    pub retry_count: Option<usize>,
    /// Rate limiting settings
    pub rate_limit: Option<RateLimitSettings>,
    /// Custom headers
    pub headers: HashMap<String, String>,
}

/// Rate limiting settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitSettings {
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Requests per hour
    pub requests_per_hour: u32,
    /// Burst limit
    pub burst_limit: u32,
}

/// Community contribution review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionReview {
    /// Review ID
    pub id: String,
    /// Contribution ID being reviewed
    pub contribution_id: String,
    /// Reviewer username/ID
    pub reviewer: String,
    /// Review timestamp
    pub reviewed_at: SystemTime,
    /// Review decision
    pub decision: ContributionStatus,
    /// Review comments
    pub comments: String,
    /// Quality score assigned by reviewer
    pub quality_score: Option<f64>,
    /// Suggested improvements
    pub suggestions: Vec<String>,
}

/// Provider update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUpdate {
    /// Update ID
    pub id: String,
    /// Provider ID
    pub provider_id: String,
    /// Update type
    pub update_type: UpdateType,
    /// Update description
    pub description: String,
    /// Breaking changes flag
    pub breaking_changes: bool,
    /// Required actions for users
    pub required_actions: Vec<String>,
    /// Update timestamp
    pub updated_at: SystemTime,
    /// Version information
    pub version: String,
}

/// Type of provider update
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateType {
    /// New model added
    NewModel,
    /// Model removed
    ModelRemoved,
    /// Pricing changed
    PricingChange,
    /// API changes
    ApiChange,
    /// Security update
    SecurityUpdate,
    /// Feature addition
    FeatureAddition,
    /// Bug fix
    BugFix,
}

/// Community provider registry
pub struct CommunityProviderRegistry {
    /// Approved community configurations
    approved_configs: HashMap<String, CommunityProviderConfig>,
    /// Pending contributions
    pending_contributions: HashMap<String, CommunityProviderConfig>,
    /// Contribution reviews
    reviews: HashMap<String, Vec<ContributionReview>>,
    /// Provider analytics
    analytics: HashMap<String, ProviderAnalytics>,
    /// Provider updates
    updates: HashMap<String, Vec<ProviderUpdate>>,
    /// Trusted contributors
    trusted_contributors: HashSet<String>,
}

impl CommunityProviderRegistry {
    /// Create a new community registry
    pub fn new() -> Self {
        Self {
            approved_configs: HashMap::new(),
            pending_contributions: HashMap::new(),
            reviews: HashMap::new(),
            analytics: HashMap::new(),
            updates: HashMap::new(),
            trusted_contributors: HashSet::new(),
        }
    }

    /// Submit a provider contribution
    pub fn submit_contribution(&mut self, config: CommunityProviderConfig) -> Result<String, ProviderError> {
        let contribution_id = format!("contrib_{}_{}", config.provider_id, config.metadata.created_at.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());

        let mut config = config;
        config.id = contribution_id.clone();
        config.status = ContributionStatus::Pending;

        self.pending_contributions.insert(contribution_id.clone(), config);
        Ok(contribution_id)
    }

    /// Review a contribution
    pub fn review_contribution(&mut self, review: ContributionReview) -> Result<(), ProviderError> {
        if !self.pending_contributions.contains_key(&review.contribution_id) &&
           !self.approved_configs.contains_key(&review.contribution_id) {
            return Err(ProviderError::NotFound(format!("Contribution {} not found", review.contribution_id)));
        }

        self.reviews.entry(review.contribution_id.clone())
            .or_insert_with(Vec::new)
            .push(review.clone());

        // Auto-approve if from trusted contributor and meets criteria
        if self.trusted_contributors.contains(&review.reviewer) &&
           review.decision == ContributionStatus::Approved &&
           review.quality_score.unwrap_or(0.0) >= 0.8 {

            if let Some(config) = self.pending_contributions.remove(&review.contribution_id) {
                let mut approved_config = config;
                approved_config.status = ContributionStatus::Approved;
                approved_config.metadata.updated_at = SystemTime::now();
                self.approved_configs.insert(review.contribution_id, approved_config);
            }
        }

        Ok(())
    }

    /// Get approved community configuration for a provider
    pub fn get_approved_config(&self, provider_id: &str) -> Option<&CommunityProviderConfig> {
        self.approved_configs.values().find(|config| config.provider_id == provider_id)
    }

    /// Get all approved configurations
    pub fn get_all_approved_configs(&self) -> Vec<&CommunityProviderConfig> {
        self.approved_configs.values().collect()
    }

    /// Get pending contributions
    pub fn get_pending_contributions(&self) -> Vec<&CommunityProviderConfig> {
        self.pending_contributions.values().collect()
    }

    /// Get reviews for a contribution
    pub fn get_reviews(&self, contribution_id: &str) -> Vec<&ContributionReview> {
        self.reviews.get(contribution_id)
            .map(|reviews| reviews.iter().collect())
            .unwrap_or_default()
    }

    /// Record provider usage analytics
    pub fn record_usage(&mut self, provider_id: &str, usage: ProviderUsage) {
        let analytics = self.analytics.entry(provider_id.to_string())
            .or_insert_with(|| ProviderAnalytics {
                provider_id: provider_id.to_string(),
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                total_tokens: 0,
                total_cost: 0.0,
                avg_response_time_ms: 0.0,
                unique_users: 0,
                usage_by_period: HashMap::new(),
                popular_models: HashMap::new(),
                error_breakdown: HashMap::new(),
                last_updated: SystemTime::now(),
            });

        analytics.total_requests += 1;
        analytics.total_tokens += usage.tokens_used;
        analytics.total_cost += usage.cost;

        if usage.success {
            analytics.successful_requests += 1;
        } else {
            analytics.failed_requests += 1;
            *analytics.error_breakdown.entry(usage.error_type.unwrap_or_else(|| "unknown".to_string())).or_insert(0) += 1;
        }

        // Update average response time
        let total_time = analytics.avg_response_time_ms * (analytics.total_requests - 1) as f64 + usage.response_time_ms as f64;
        analytics.avg_response_time_ms = total_time / analytics.total_requests as f64;

        // Update model popularity
        *analytics.popular_models.entry(usage.model.clone()).or_insert(0) += 1;

        analytics.last_updated = SystemTime::now();
    }

    /// Get provider analytics
    pub fn get_analytics(&self, provider_id: &str) -> Option<&ProviderAnalytics> {
        self.analytics.get(provider_id)
    }

    /// Get all provider analytics
    pub fn get_all_analytics(&self) -> Vec<&ProviderAnalytics> {
        self.analytics.values().collect()
    }

    /// Add a provider update
    pub fn add_update(&mut self, update: ProviderUpdate) {
        self.updates.entry(update.provider_id.clone())
            .or_insert_with(Vec::new)
            .push(update);
    }

    /// Get updates for a provider
    pub fn get_updates(&self, provider_id: &str) -> Vec<&ProviderUpdate> {
        self.updates.get(provider_id)
            .map(|updates| updates.iter().collect())
            .unwrap_or_default()
    }

    /// Add trusted contributor
    pub fn add_trusted_contributor(&mut self, contributor: String) {
        self.trusted_contributors.insert(contributor);
    }

    /// Check if contributor is trusted
    pub fn is_trusted_contributor(&self, contributor: &str) -> bool {
        self.trusted_contributors.contains(contributor)
    }

    /// Get community quality metrics for a provider
    pub fn get_community_quality_metrics(&self, provider_id: &str) -> Option<CommunityQualityMetrics> {
        // Aggregate quality metrics from approved configs and analytics
        let config = self.get_approved_config(provider_id)?;
        let analytics = self.get_analytics(provider_id)?;

        if let Some(metrics) = &config.quality_metrics {
            Some(metrics.clone())
        } else {
            // Calculate basic metrics from analytics
            let success_rate = if analytics.total_requests > 0 {
                analytics.successful_requests as f64 / analytics.total_requests as f64
            } else {
                0.0
            };

            Some(CommunityQualityMetrics {
                tester_count: 1, // Basic estimate
                avg_quality_score: success_rate * 0.8, // Rough estimate
                avg_reliability_score: success_rate,
                avg_cost_efficiency: if analytics.total_cost > 0.0 {
                    1.0 / (1.0 + analytics.total_cost / analytics.total_requests as f64)
                } else {
                    0.9
                },
                reported_issues: analytics.failed_requests as usize,
                successful_tests: analytics.successful_requests as usize,
                total_tests: analytics.total_requests as usize,
                last_tested: analytics.last_updated,
            })
        }
    }

    /// Get popular providers based on usage
    pub fn get_popular_providers(&self, limit: usize) -> Vec<(String, u64)> {
        let mut providers: Vec<(String, u64)> = self.analytics.iter()
            .map(|(id, analytics)| (id.clone(), analytics.total_requests))
            .collect();

        providers.sort_by(|a, b| b.1.cmp(&a.1));
        providers.into_iter().take(limit).collect()
    }

    /// Get providers by quality score
    pub fn get_providers_by_community_quality(&self, limit: usize) -> Vec<(String, f64)> {
        let mut providers: Vec<(String, f64)> = self.approved_configs.values()
            .filter_map(|config| {
                config.quality_metrics.as_ref()
                    .map(|metrics| (config.provider_id.clone(), metrics.avg_quality_score))
            })
            .collect();

        providers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        providers.into_iter().take(limit).collect()
    }
}

/// Usage data for analytics
#[derive(Debug, Clone)]
pub struct ProviderUsage {
    /// Whether the request was successful
    pub success: bool,
    /// Tokens used in the request
    pub tokens_used: u64,
    /// Cost incurred
    pub cost: f64,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Model used
    pub model: String,
    /// Error type if failed
    pub error_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_contribution_submission() {
        let mut registry = CommunityProviderRegistry::new();

        let config = CommunityProviderConfig {
            id: "".to_string(),
            provider_id: "test_provider".to_string(),
            name: "Test Provider".to_string(),
            description: "A test provider".to_string(),
            base_url: Some("https://api.test.com".to_string()),
            models: vec![],
            default_config: ProviderSettings {
                timeout: Some(Duration::from_secs(30)),
                retry_count: Some(3),
                rate_limit: None,
                headers: HashMap::new(),
            },
            metadata: ContributionMetadata {
                contributor: "test_user".to_string(),
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                version: "1.0.0".to_string(),
                tags: vec!["test".to_string()],
                compatibility_notes: None,
            },
            status: ContributionStatus::Pending,
            quality_metrics: None,
        };

        let contribution_id = registry.submit_contribution(config).unwrap();
        assert!(contribution_id.starts_with("contrib_test_provider_"));

        let pending = registry.get_pending_contributions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].provider_id, "test_provider");
    }

    #[test]
    fn test_usage_analytics() {
        let mut registry = CommunityProviderRegistry::new();

        let usage = ProviderUsage {
            success: true,
            tokens_used: 100,
            cost: 0.01,
            response_time_ms: 500,
            model: "gpt-4".to_string(),
            error_type: None,
        };

        registry.record_usage("test_provider", usage);

        let analytics = registry.get_analytics("test_provider").unwrap();
        assert_eq!(analytics.total_requests, 1);
        assert_eq!(analytics.successful_requests, 1);
        assert_eq!(analytics.total_tokens, 100);
        assert_eq!(analytics.total_cost, 0.01);
        assert_eq!(analytics.avg_response_time_ms, 500.0);
    }

    #[test]
    fn test_popular_providers() {
        let mut registry = CommunityProviderRegistry::new();

        // Record usage for different providers
        registry.record_usage("provider_a", ProviderUsage {
            success: true, tokens_used: 100, cost: 0.01, response_time_ms: 500,
            model: "model1".to_string(), error_type: None,
        });

        registry.record_usage("provider_a", ProviderUsage {
            success: true, tokens_used: 200, cost: 0.02, response_time_ms: 600,
            model: "model1".to_string(), error_type: None,
        });

        registry.record_usage("provider_b", ProviderUsage {
            success: true, tokens_used: 50, cost: 0.005, response_time_ms: 300,
            model: "model2".to_string(), error_type: None,
        });

        let popular = registry.get_popular_providers(2);
        assert_eq!(popular.len(), 2);
        assert_eq!(popular[0], ("provider_a".to_string(), 2));
        assert_eq!(popular[1], ("provider_b".to_string(), 1));
    }
}</content>
