//! Community provider database synchronization
//!
//! This module handles synchronization of provider configurations
//! from community-maintained databases and repositories.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::{
    community::{CommunityProviderConfig, CommunityProviderRegistry},
    error::ProviderError,
};

/// Community database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityDatabaseConfig {
    /// Database endpoints
    pub endpoints: Vec<String>,
    /// Sync interval
    pub sync_interval: Duration,
    /// Trusted sources
    pub trusted_sources: Vec<String>,
    /// Auto-approve from trusted sources
    pub auto_approve_trusted: bool,
    /// Validation rules
    pub validation_rules: ValidationRules,
}

/// Validation rules for community configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Require pricing information
    pub require_pricing: bool,
    /// Require capability declarations
    pub require_capabilities: bool,
    /// Maximum models per provider
    pub max_models_per_provider: usize,
    /// Minimum quality score threshold
    pub min_quality_score: f64,
}

/// Sync status for a database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// Database endpoint
    pub endpoint: String,
    /// Last sync time
    pub last_sync: Option<SystemTime>,
    /// Sync success
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Configurations synced
    pub configs_synced: usize,
}

/// Community database synchronizer
pub struct CommunityDatabaseSync {
    config: CommunityDatabaseConfig,
    registry: Arc<RwLock<CommunityProviderRegistry>>,
    sync_status: Arc<RwLock<HashMap<String, SyncStatus>>>,
    client: reqwest::Client,
}

impl CommunityDatabaseSync {
    /// Create a new community database sync
    pub fn new(
        config: CommunityDatabaseConfig,
        registry: Arc<RwLock<CommunityProviderRegistry>>,
    ) -> Self {
        Self {
            config,
            registry,
            sync_status: Arc::new(RwLock::new(HashMap::new())),
            client: reqwest::Client::new(),
        }
    }

    /// Start automatic synchronization
    pub async fn start_sync(&self) {
        let config = self.config.clone();
        let registry = self.registry.clone();
        let sync_status = self.sync_status.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            loop {
                for endpoint in &config.endpoints {
                    match Self::sync_from_endpoint(&client, endpoint, &config, &registry).await {
                        Ok(count) => {
                            let mut status = sync_status.write().await;
                            status.insert(
                                endpoint.clone(),
                                SyncStatus {
                                    endpoint: endpoint.clone(),
                                    last_sync: Some(SystemTime::now()),
                                    success: true,
                                    error_message: None,
                                    configs_synced: count,
                                },
                            );
                            info!(
                                "Successfully synced {} configurations from {}",
                                count, endpoint
                            );
                        }
                        Err(e) => {
                            let mut status = sync_status.write().await;
                            status.insert(
                                endpoint.clone(),
                                SyncStatus {
                                    endpoint: endpoint.clone(),
                                    last_sync: Some(SystemTime::now()),
                                    success: false,
                                    error_message: Some(e.to_string()),
                                    configs_synced: 0,
                                },
                            );
                            error!("Failed to sync from {}: {}", endpoint, e);
                        }
                    }
                }

                tokio::time::sleep(config.sync_interval).await;
            }
        });
    }

    /// Manually trigger sync from all endpoints
    pub async fn sync_now(&self) -> Result<usize, ProviderError> {
        let mut total_synced = 0;

        for endpoint in &self.config.endpoints {
            let count =
                Self::sync_from_endpoint(&self.client, endpoint, &self.config, &self.registry)
                    .await?;
            total_synced += count;
        }

        Ok(total_synced)
    }

    /// Sync from a specific endpoint
    async fn sync_from_endpoint(
        client: &reqwest::Client,
        endpoint: &str,
        config: &CommunityDatabaseConfig,
        registry: &Arc<RwLock<CommunityProviderRegistry>>,
    ) -> Result<usize, ProviderError> {
        debug!("Syncing from community database: {}", endpoint);

        let response = client
            .get(endpoint)
            .header("User-Agent", "RiceCoder-CommunitySync/1.0")
            .send()
            .await
            .map_err(|e| {
                ProviderError::NetworkError(format!("Failed to fetch from {}: {}", endpoint, e))
            })?;

        if !response.status().is_success() {
            return Err(ProviderError::ProviderError(format!(
                "HTTP error from {}: {}",
                endpoint,
                response.status()
            )));
        }

        let community_data: CommunityDatabaseResponse = response.json().await.map_err(|e| {
            ProviderError::ParseError(format!("Failed to parse response from {}: {}", endpoint, e))
        })?;

        let mut synced_count = 0;
        let mut registry_guard = registry.write().await;

        for config_data in community_data.configurations {
            // Validate the configuration
            if Self::validate_configuration(&config_data, config)? {
                // Check if it's from a trusted source
                let is_trusted = config
                    .trusted_sources
                    .contains(&config_data.metadata.contributor);

                // Submit the contribution
                match registry_guard.submit_contribution(config_data.clone()) {
                    Ok(contribution_id) => {
                        synced_count += 1;

                        // Auto-approve if from trusted source and enabled
                        if is_trusted && config.auto_approve_trusted {
                            // Create an auto-approval review
                            let review = crate::community::ContributionReview {
                                id: format!("auto_{}", contribution_id),
                                contribution_id: contribution_id.clone(),
                                reviewer: "community-sync".to_string(),
                                reviewed_at: SystemTime::now(),
                                decision: crate::community::ContributionStatus::Approved,
                                comments: "Auto-approved from trusted community source".to_string(),
                                quality_score: config_data
                                    .quality_metrics
                                    .as_ref()
                                    .map(|m| m.avg_quality_score),
                                suggestions: vec![],
                            };

                            if let Err(e) = registry_guard.review_contribution(review) {
                                warn!(
                                    "Failed to auto-approve contribution {}: {}",
                                    contribution_id, e
                                );
                            }
                        }

                        debug!("Synced configuration: {}", contribution_id);
                    }
                    Err(e) => {
                        warn!("Failed to submit contribution from {}: {}", endpoint, e);
                    }
                }
            } else {
                warn!(
                    "Configuration validation failed for provider: {}",
                    config_data.provider_id
                );
            }
        }

        Ok(synced_count)
    }

    /// Validate a community configuration
    fn validate_configuration(
        config: &CommunityProviderConfig,
        sync_config: &CommunityDatabaseConfig,
    ) -> Result<bool, ProviderError> {
        let rules = &sync_config.validation_rules;

        // Check required fields
        if config.provider_id.is_empty() || config.name.is_empty() {
            return Ok(false);
        }

        // Check pricing requirement
        if rules.require_pricing {
            let has_pricing = config.models.iter().any(|m| m.pricing.is_some());
            if !has_pricing {
                return Ok(false);
            }
        }

        // Check capabilities requirement
        if rules.require_capabilities {
            let has_capabilities = config.models.iter().all(|m| !m.capabilities.is_empty());
            if !has_capabilities {
                return Ok(false);
            }
        }

        // Check model count limit
        if config.models.len() > rules.max_models_per_provider {
            return Ok(false);
        }

        // Check quality score threshold
        if let Some(metrics) = &config.quality_metrics {
            if metrics.avg_quality_score < rules.min_quality_score {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get sync status for all endpoints
    pub async fn get_sync_status(&self) -> HashMap<String, SyncStatus> {
        self.sync_status.read().await.clone()
    }

    /// Get sync status for a specific endpoint
    pub async fn get_endpoint_status(&self, endpoint: &str) -> Option<SyncStatus> {
        self.sync_status.read().await.get(endpoint).cloned()
    }

    /// Export approved configurations for backup/sharing
    pub async fn export_approved_configs(&self) -> Vec<CommunityProviderConfig> {
        let registry = self.registry.read().await;
        registry
            .get_all_approved_configs()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Import configurations from backup
    pub async fn import_configs(
        &self,
        configs: Vec<CommunityProviderConfig>,
    ) -> Result<usize, ProviderError> {
        let mut registry = self.registry.write().await;
        let mut imported = 0;

        for config in configs {
            if registry.submit_contribution(config).is_ok() {
                imported += 1;
            }
        }

        Ok(imported)
    }
}

/// Community database response structure
#[derive(Debug, Deserialize)]
struct CommunityDatabaseResponse {
    /// Provider configurations
    configurations: Vec<CommunityProviderConfig>,
    /// Database metadata
    metadata: Option<CommunityDatabaseMetadata>,
}

/// Community database metadata
#[derive(Debug, Deserialize)]
struct CommunityDatabaseMetadata {
    /// Database version
    version: String,
    /// Last updated
    last_updated: SystemTime,
    /// Total configurations
    total_configs: usize,
}

/// Provider contribution validator
pub struct ContributionValidator {
    rules: ValidationRules,
}

impl ContributionValidator {
    /// Create a new validator
    pub fn new(rules: ValidationRules) -> Self {
        Self { rules }
    }

    /// Validate a contribution
    pub fn validate(&self, config: &CommunityProviderConfig) -> Result<(), ProviderError> {
        // Basic validation
        if config.provider_id.is_empty() {
            return Err(ProviderError::ConfigError(
                "Provider ID is required".to_string(),
            ));
        }

        if config.name.is_empty() {
            return Err(ProviderError::ConfigError(
                "Provider name is required".to_string(),
            ));
        }

        if config.models.is_empty() {
            return Err(ProviderError::ConfigError(
                "At least one model is required".to_string(),
            ));
        }

        // Advanced validation based on rules
        if self.rules.require_pricing {
            let has_pricing = config.models.iter().any(|m| m.pricing.is_some());
            if !has_pricing {
                return Err(ProviderError::ConfigError(
                    "Pricing information is required".to_string(),
                ));
            }
        }

        if self.rules.require_capabilities {
            for model in &config.models {
                if model.capabilities.is_empty() {
                    return Err(ProviderError::ConfigError(format!(
                        "Capabilities are required for model {}",
                        model.id
                    )));
                }
            }
        }

        if config.models.len() > self.rules.max_models_per_provider {
            return Err(ProviderError::ConfigError(format!(
                "Too many models: {} (max {})",
                config.models.len(),
                self.rules.max_models_per_provider
            )));
        }

        if let Some(metrics) = &config.quality_metrics {
            if metrics.avg_quality_score < self.rules.min_quality_score {
                return Err(ProviderError::ConfigError(format!(
                    "Quality score too low: {} (min {})",
                    metrics.avg_quality_score, self.rules.min_quality_score
                )));
            }
        }

        Ok(())
    }
}

impl Default for CommunityDatabaseConfig {
    fn default() -> Self {
        Self {
            endpoints: vec![
                "https://raw.githubusercontent.com/ricecoder-community/providers/main/configs.json"
                    .to_string(),
                "https://api.ricecoder.community/providers".to_string(),
            ],
            sync_interval: Duration::from_secs(3600), // 1 hour
            trusted_sources: vec![
                "ricecoder-team".to_string(),
                "verified-contributor".to_string(),
            ],
            auto_approve_trusted: true,
            validation_rules: ValidationRules::default(),
        }
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            require_pricing: true,
            require_capabilities: true,
            max_models_per_provider: 20,
            min_quality_score: 0.6,
        }
    }
}
