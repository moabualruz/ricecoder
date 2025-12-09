//! GitHub Manager - Central coordinator for GitHub operations

use crate::errors::{GitHubError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// GitHub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub API token
    pub token: String,
    /// Repository owner
    pub owner: String,
    /// Repository name
    pub repo: String,
    /// Base branch (default: main)
    pub base_branch: String,
    /// API timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Max retries for transient errors
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Retry backoff in milliseconds
    #[serde(default = "default_retry_backoff")]
    pub retry_backoff_ms: u64,
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_backoff() -> u64 {
    100
}

impl GitHubConfig {
    /// Create a new GitHub configuration
    pub fn new(token: impl Into<String>, owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            owner: owner.into(),
            repo: repo.into(),
            base_branch: "main".to_string(),
            timeout_secs: default_timeout(),
            max_retries: default_max_retries(),
            retry_backoff_ms: default_retry_backoff(),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.token.is_empty() {
            return Err(GitHubError::config_error("GitHub token is required"));
        }
        if self.owner.is_empty() {
            return Err(GitHubError::config_error("Repository owner is required"));
        }
        if self.repo.is_empty() {
            return Err(GitHubError::config_error("Repository name is required"));
        }
        if self.timeout_secs == 0 {
            return Err(GitHubError::config_error("Timeout must be greater than 0"));
        }
        Ok(())
    }
}

/// Rate limit information
#[derive(Debug, Clone)]
pub struct RateLimit {
    /// Remaining requests
    pub remaining: u32,
    /// Total limit
    pub limit: u32,
    /// Reset time (Unix timestamp)
    pub reset_at: u64,
}

impl RateLimit {
    /// Check if rate limit is exceeded
    pub fn is_exceeded(&self) -> bool {
        self.remaining == 0
    }

    /// Get time until reset in seconds
    pub fn time_until_reset(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.reset_at.saturating_sub(now)
    }
}

/// GitHub Manager - Central coordinator for all GitHub operations
pub struct GitHubManager {
    /// Configuration
    config: Arc<RwLock<GitHubConfig>>,
    /// Rate limit information
    rate_limit: Arc<RwLock<Option<RateLimit>>>,
    /// Octocrab client
    client: Arc<RwLock<Option<octocrab::Octocrab>>>,
}

impl GitHubManager {
    /// Create a new GitHub manager
    pub fn new(config: GitHubConfig) -> Result<Self> {
        config.validate()?;
        info!(
            owner = %config.owner,
            repo = %config.repo,
            "Creating GitHub manager"
        );
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            rate_limit: Arc::new(RwLock::new(None)),
            client: Arc::new(RwLock::new(None)),
        })
    }

    /// Initialize the GitHub client
    pub async fn initialize(&self) -> Result<()> {
        let config = self.config.read().await;
        debug!("Initializing GitHub client");

        // Create octocrab client
        let client = octocrab::OctocrabBuilder::new()
            .personal_token(config.token.clone())
            .build()
            .map_err(|e| GitHubError::auth_error(format!("Failed to create GitHub client: {}", e)))?;

        // Test authentication by getting the current user
        client
            .current()
            .user()
            .await
            .map_err(|e| GitHubError::auth_error(format!("Authentication failed: {}", e)))?;

        info!("GitHub client initialized successfully");

        let mut client_lock = self.client.write().await;
        *client_lock = Some(client);

        Ok(())
    }

    /// Get the GitHub client
    pub async fn get_client(&self) -> Result<Arc<octocrab::Octocrab>> {
        let client_lock = self.client.read().await;
        if let Some(client) = client_lock.as_ref() {
            Ok(Arc::new(client.clone()))
        } else {
            Err(GitHubError::auth_error(
                "GitHub client not initialized. Call initialize() first.",
            ))
        }
    }

    /// Get current configuration
    pub async fn get_config(&self) -> GitHubConfig {
        self.config.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: GitHubConfig) -> Result<()> {
        config.validate()?;
        let mut config_lock = self.config.write().await;
        *config_lock = config;
        info!("GitHub configuration updated");
        Ok(())
    }

    /// Get rate limit information
    pub async fn get_rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit.read().await.clone()
    }

    /// Update rate limit information
    pub async fn update_rate_limit(&self, rate_limit: RateLimit) {
        let mut limit_lock = self.rate_limit.write().await;
        *limit_lock = Some(rate_limit);
    }

    /// Check if rate limited
    pub async fn is_rate_limited(&self) -> bool {
        if let Some(limit) = self.rate_limit.read().await.as_ref() {
            limit.is_exceeded()
        } else {
            false
        }
    }

    /// Wait for rate limit reset
    pub async fn wait_for_rate_limit_reset(&self) -> Result<()> {
        if let Some(limit) = self.rate_limit.read().await.as_ref() {
            let wait_time = limit.time_until_reset();
            if wait_time > 0 {
                warn!(
                    wait_seconds = wait_time,
                    "Rate limited, waiting for reset"
                );
                tokio::time::sleep(Duration::from_secs(wait_time + 1)).await;
            }
        }
        Ok(())
    }

    /// Perform operation with retry logic
    pub async fn with_retry<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> futures::future::BoxFuture<'static, Result<T>>,
    {
        let config = self.config.read().await;
        let max_retries = config.max_retries;
        let retry_backoff = config.retry_backoff_ms;
        drop(config);

        let mut attempt = 0;
        loop {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        debug!(attempts = attempt, "Operation succeeded after retries");
                    }
                    return Ok(result);
                }
                Err(e) => {
                    attempt += 1;

                    // Check if error is retryable
                    let is_retryable = matches!(
                        e,
                        GitHubError::NetworkError(_)
                            | GitHubError::Timeout
                            | GitHubError::RateLimitExceeded
                    );

                    if !is_retryable || attempt >= max_retries {
                        error!(
                            attempt,
                            max_retries,
                            error = %e,
                            "Operation failed permanently"
                        );
                        return Err(e);
                    }

                    let wait_ms = retry_backoff * 2_u64.pow(attempt - 1);
                    warn!(
                        attempt,
                        wait_ms,
                        error = %e,
                        "Operation failed, retrying"
                    );
                    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                }
            }
        }
    }

    /// Get repository information
    pub async fn get_repository(&self) -> Result<String> {
        let config = self.config.read().await;
        Ok(format!("{}/{}", config.owner, config.repo))
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        debug!("Performing health check");

        let client = self.get_client().await?;
        let config = self.config.read().await;

        // Try to get repository info
        client
            .repos(&config.owner, &config.repo)
            .get()
            .await
            .map_err(|e| GitHubError::api_error(format!("Health check failed: {}", e)))?;

        info!("Health check passed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_config_creation() {
        let config = GitHubConfig::new("token123", "owner", "repo");
        assert_eq!(config.token, "token123");
        assert_eq!(config.owner, "owner");
        assert_eq!(config.repo, "repo");
        assert_eq!(config.base_branch, "main");
    }

    #[test]
    fn test_github_config_validation_success() {
        let config = GitHubConfig::new("token123", "owner", "repo");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_github_config_validation_empty_token() {
        let config = GitHubConfig {
            token: String::new(),
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            base_branch: "main".to_string(),
            timeout_secs: 30,
            max_retries: 3,
            retry_backoff_ms: 100,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_github_config_validation_empty_owner() {
        let config = GitHubConfig {
            token: "token123".to_string(),
            owner: String::new(),
            repo: "repo".to_string(),
            base_branch: "main".to_string(),
            timeout_secs: 30,
            max_retries: 3,
            retry_backoff_ms: 100,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_github_config_validation_empty_repo() {
        let config = GitHubConfig {
            token: "token123".to_string(),
            owner: "owner".to_string(),
            repo: String::new(),
            base_branch: "main".to_string(),
            timeout_secs: 30,
            max_retries: 3,
            retry_backoff_ms: 100,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_rate_limit_is_exceeded() {
        let limit = RateLimit {
            remaining: 0,
            limit: 60,
            reset_at: 0,
        };
        assert!(limit.is_exceeded());
    }

    #[test]
    fn test_rate_limit_not_exceeded() {
        let limit = RateLimit {
            remaining: 10,
            limit: 60,
            reset_at: 0,
        };
        assert!(!limit.is_exceeded());
    }

    #[tokio::test]
    async fn test_github_manager_creation() {
        let config = GitHubConfig::new("token123", "owner", "repo");
        let manager = GitHubManager::new(config);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_github_manager_invalid_config() {
        let config = GitHubConfig {
            token: String::new(),
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            base_branch: "main".to_string(),
            timeout_secs: 30,
            max_retries: 3,
            retry_backoff_ms: 100,
        };
        let manager = GitHubManager::new(config);
        assert!(manager.is_err());
    }

    #[tokio::test]
    async fn test_get_repository() {
        let config = GitHubConfig::new("token123", "owner", "repo");
        let manager = GitHubManager::new(config).unwrap();
        let repo = manager.get_repository().await.unwrap();
        assert_eq!(repo, "owner/repo");
    }

    #[tokio::test]
    async fn test_update_config() {
        let config = GitHubConfig::new("token123", "owner", "repo");
        let manager = GitHubManager::new(config).unwrap();

        let new_config = GitHubConfig::new("token456", "owner2", "repo2");
        assert!(manager.update_config(new_config).await.is_ok());

        let repo = manager.get_repository().await.unwrap();
        assert_eq!(repo, "owner2/repo2");
    }

    #[tokio::test]
    async fn test_rate_limit_tracking() {
        let config = GitHubConfig::new("token123", "owner", "repo");
        let manager = GitHubManager::new(config).unwrap();

        assert!(manager.get_rate_limit().await.is_none());

        let limit = RateLimit {
            remaining: 10,
            limit: 60,
            reset_at: 0,
        };
        manager.update_rate_limit(limit).await;

        assert!(manager.get_rate_limit().await.is_some());
        assert!(!manager.is_rate_limited().await);
    }
}
