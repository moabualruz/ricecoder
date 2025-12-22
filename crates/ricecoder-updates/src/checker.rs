//! Automatic update checking with enterprise policy controls

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use semver::Version;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::{
    error::{Result, UpdateError},
    models::{ReleaseChannel, ReleaseInfo, UpdateCheckResult},
    policy::{PolicyResult, UpdatePolicy},
};

/// Update checker service
#[derive(Clone)]
pub struct UpdateChecker {
    client: Client,
    policy: Arc<RwLock<UpdatePolicy>>,
    update_server_url: String,
    current_version: Version,
    last_check: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl UpdateChecker {
    /// Create a new update checker
    pub fn new(policy: UpdatePolicy, update_server_url: String, current_version: Version) -> Self {
        Self {
            client: Client::new(),
            policy: Arc::new(RwLock::new(policy)),
            update_server_url,
            current_version,
            last_check: Arc::new(RwLock::new(None)),
        }
    }

    /// Check for updates
    pub async fn check_for_updates(&self) -> Result<UpdateCheckResult> {
        // Check if we should perform the update check based on policy
        if !self.should_check_updates().await {
            return Ok(UpdateCheckResult {
                update_available: false,
                latest_version: None,
                current_version: self.current_version.clone(),
                release_info: None,
                checked_at: Utc::now(),
                next_check: self.next_check_time().await,
            });
        }

        let checked_at = Utc::now();

        // Update last check time
        *self.last_check.write().await = Some(checked_at);

        // Fetch release information
        let release_info = match self.fetch_release_info().await {
            Ok(info) => info,
            Err(e) => {
                warn!("Failed to fetch release information: {}", e);
                return Ok(UpdateCheckResult {
                    update_available: false,
                    latest_version: None,
                    current_version: self.current_version.clone(),
                    release_info: None,
                    checked_at,
                    next_check: self.next_check_time().await,
                });
            }
        };

        // Check if update is available and allowed by policy
        let update_available = self.is_update_available(&release_info).await;
        let (latest_version, release_info_option) = if update_available {
            (Some(release_info.version.clone()), Some(release_info))
        } else {
            (None, None)
        };

        let result = UpdateCheckResult {
            update_available,
            latest_version,
            current_version: self.current_version.clone(),
            release_info: release_info_option,
            checked_at,
            next_check: self.next_check_time().await,
        };

        if update_available {
            info!(
                "Update available: {} -> {}",
                self.current_version,
                result.latest_version.as_ref().unwrap()
            );
        }

        Ok(result)
    }

    /// Check if updates should be checked based on policy and timing
    async fn should_check_updates(&self) -> bool {
        let policy = self.policy.read().await;

        // Check if auto updates are enabled
        if !policy.auto_updates_allowed() {
            return false;
        }

        // Check timing
        let last_check = *self.last_check.read().await;
        let now = Utc::now();

        if let Some(last) = last_check {
            let interval = Duration::hours(policy.check_interval_hours() as i64);
            if now - last < interval {
                return false;
            }
        }

        true
    }

    /// Get the next recommended check time
    async fn next_check_time(&self) -> DateTime<Utc> {
        let policy = self.policy.read().await;
        let interval = Duration::hours(policy.check_interval_hours() as i64);

        if let Some(last) = *self.last_check.read().await {
            last + interval
        } else {
            Utc::now() + interval
        }
    }

    /// Fetch release information from update server
    async fn fetch_release_info(&self) -> Result<ReleaseInfo> {
        let url = format!("{}/releases/latest", self.update_server_url);

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "RiceCoder-UpdateChecker")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(UpdateError::update_check(format!(
                "Server returned status: {}",
                response.status()
            )));
        }

        let release_info: ReleaseInfo = response.json().await?;
        Ok(release_info)
    }

    /// Check if an update is available and allowed by policy
    async fn is_update_available(&self, release_info: &ReleaseInfo) -> bool {
        // Check version
        if release_info.version <= self.current_version {
            return false;
        }

        // Check minimum version requirement
        if let Some(min_version) = &release_info.minimum_version {
            if self.current_version < *min_version {
                warn!(
                    "Update requires minimum version {}, current: {}",
                    min_version, self.current_version
                );
                return false;
            }
        }

        let policy = self.policy.read().await;

        // Check if channel is allowed
        if !policy.channel_allowed(&release_info.channel) {
            return false;
        }

        // Check download size (estimate based on downloads)
        let max_size = policy.max_download_size_mb();
        let has_large_download = release_info
            .downloads
            .values()
            .any(|d| d.size > (max_size as u64 * 1024 * 1024));

        if has_large_download {
            return false;
        }

        // Check compliance requirements
        let compliance_tags = vec![
            if release_info.compliance.soc2_compliant {
                "SOC2"
            } else {
                ""
            },
            if release_info.compliance.gdpr_compliant {
                "GDPR"
            } else {
                ""
            },
            if release_info.compliance.hipaa_compliant {
                "HIPAA"
            } else {
                ""
            },
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

        if !policy.compliance_requirements_met(&compliance_tags) {
            return false;
        }

        true
    }

    /// Force an immediate update check
    pub async fn force_check(&self) -> Result<UpdateCheckResult> {
        *self.last_check.write().await = None;
        self.check_for_updates().await
    }

    /// Update the policy configuration
    pub async fn update_policy(&self, new_policy: UpdatePolicy) {
        *self.policy.write().await = new_policy;
    }

    /// Get current policy
    pub async fn get_policy(&self) -> UpdatePolicy {
        self.policy.read().await.clone()
    }
}

/// Background update checker task
pub struct BackgroundChecker {
    checker: UpdateChecker,
    check_interval: Duration,
}

impl BackgroundChecker {
    /// Create a new background checker
    pub fn new(checker: UpdateChecker, check_interval: Duration) -> Self {
        Self {
            checker,
            check_interval,
        }
    }

    /// Start the background checking task
    pub async fn start(self) -> Result<()> {
        let mut interval = tokio::time::interval(self.check_interval.to_std().unwrap());

        loop {
            interval.tick().await;

            match self.checker.check_for_updates().await {
                Ok(result) => {
                    if result.update_available {
                        info!(
                            "Background update check found new version: {}",
                            result.latest_version.unwrap()
                        );
                        // Here we could trigger notifications or auto-download
                        // depending on policy settings
                    }
                }
                Err(e) => {
                    error!("Background update check failed: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::models::{ReleaseChannel, SecurityRequirements, UpdatePolicyConfig};

    #[tokio::test]
    async fn test_update_checker_creation() {
        let policy = UpdatePolicy::default();
        let version = Version::from_str("1.0.0").unwrap();
        let checker =
            UpdateChecker::new(policy, "https://updates.example.com".to_string(), version);

        assert_eq!(checker.current_version, version);
        assert_eq!(checker.update_server_url, "https://updates.example.com");
    }

    #[tokio::test]
    async fn test_policy_update() {
        let policy = UpdatePolicy::default();
        let version = Version::from_str("1.0.0").unwrap();
        let checker =
            UpdateChecker::new(policy, "https://updates.example.com".to_string(), version);

        let new_config = UpdatePolicyConfig {
            auto_update_enabled: false,
            ..Default::default()
        };
        let new_policy = UpdatePolicy::new(new_config);
        checker.update_policy(new_policy.clone()).await;

        let retrieved_policy = checker.get_policy().await;
        assert!(!retrieved_policy.auto_updates_allowed());
    }

    #[test]
    fn test_background_checker_creation() {
        let policy = UpdatePolicy::default();
        let version = Version::from_str("1.0.0").unwrap();
        let checker =
            UpdateChecker::new(policy, "https://updates.example.com".to_string(), version);
        let interval = Duration::hours(24);

        let background = BackgroundChecker::new(checker, interval);
        assert_eq!(background.check_interval, interval);
    }
}
