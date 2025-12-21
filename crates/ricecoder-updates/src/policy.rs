//! Enterprise policy management for updates

use crate::error::{Result, UpdateError};
use crate::models::{ReleaseChannel, SecurityRequirements, UpdatePolicyConfig};
use ricecoder_security::access_control::{Permission, Role};
use std::collections::HashSet;

/// Update policy manager with enterprise controls
#[derive(Debug, Clone)]
pub struct UpdatePolicy {
    config: UpdatePolicyConfig,
    user_permissions: HashSet<Permission>,
    user_roles: HashSet<Role>,
}

impl UpdatePolicy {
    /// Create a new policy manager
    pub fn new(config: UpdatePolicyConfig) -> Self {
        Self {
            config,
            user_permissions: HashSet::new(),
            user_roles: HashSet::new(),
        }
    }

    /// Set user permissions for policy evaluation
    pub fn with_permissions(mut self, permissions: HashSet<Permission>) -> Self {
        self.user_permissions = permissions;
        self
    }

    /// Set user roles for policy evaluation
    pub fn with_roles(mut self, roles: HashSet<Role>) -> Self {
        self.user_roles = roles;
        self
    }

    /// Check if automatic updates are allowed
    pub fn auto_updates_allowed(&self) -> bool {
        self.config.auto_update_enabled && self.has_permission(Permission::UpdateAuto)
    }

    /// Check if a release channel is allowed
    pub fn channel_allowed(&self, channel: &ReleaseChannel) -> bool {
        self.config.allowed_channels.contains(channel)
    }

    /// Check if manual approval is required for updates
    pub fn requires_approval(&self) -> bool {
        self.config.require_approval || !self.has_role(Role::Admin)
    }

    /// Get maximum allowed download size in MB
    pub fn max_download_size_mb(&self) -> u32 {
        self.config.max_download_size_mb
    }

    /// Check if a download size is within limits
    pub fn download_size_allowed(&self, size_mb: u32) -> bool {
        size_mb <= self.config.max_download_size_mb
    }

    /// Check if signature verification is required
    pub fn signature_required(&self) -> bool {
        self.config.security_requirements.require_signature
    }

    /// Validate security requirements for an update
    pub fn validate_security_requirements(
        &self,
        has_signature: bool,
        has_checksum: bool,
    ) -> Result<()> {
        let reqs = &self.config.security_requirements;

        if reqs.require_signature && !has_signature {
            return Err(UpdateError::policy_violation(
                "Update requires signature verification but none provided",
            ));
        }

        if reqs.require_checksum && !has_checksum {
            return Err(UpdateError::policy_violation(
                "Update requires checksum verification but none provided",
            ));
        }

        Ok(())
    }

    /// Check if enterprise features are enabled
    pub fn enterprise_enabled(&self) -> bool {
        self.config.enterprise_settings.is_some()
    }

    /// Get organization ID if enterprise features enabled
    pub fn organization_id(&self) -> Option<&str> {
        self.config
            .enterprise_settings
            .as_ref()
            .map(|e| e.organization_id.as_str())
    }

    /// Check if compliance requirements are met
    pub fn compliance_requirements_met(&self, compliance_tags: &[String]) -> bool {
        if let Some(enterprise) = &self.config.enterprise_settings {
            enterprise
                .compliance_requirements
                .iter()
                .all(|req| compliance_tags.contains(req))
        } else {
            true // No enterprise requirements
        }
    }

    /// Get update check interval in hours
    pub fn check_interval_hours(&self) -> u32 {
        self.config.check_interval_hours
    }

    /// Check if user has specific permission
    fn has_permission(&self, permission: Permission) -> bool {
        self.user_permissions.contains(&permission)
            || self
                .user_roles
                .iter()
                .any(|role| role.has_permission(&permission))
    }

    /// Check if user has specific role
    fn has_role(&self, role: Role) -> bool {
        self.user_roles.contains(&role)
    }
}

impl Default for UpdatePolicy {
    fn default() -> Self {
        Self {
            config: UpdatePolicyConfig {
                auto_update_enabled: true,
                check_interval_hours: 24,
                allowed_channels: vec![ReleaseChannel::Stable],
                require_approval: false,
                max_download_size_mb: 100,
                security_requirements: SecurityRequirements {
                    require_signature: true,
                    require_checksum: true,
                    allowed_cas: vec![],
                    minimum_security_level: crate::models::SecuritySeverity::Medium,
                },
                enterprise_settings: None,
            },
            user_permissions: HashSet::new(),
            user_roles: HashSet::new(),
        }
    }
}

/// Policy enforcement result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyResult {
    /// Policy allows the operation
    Allowed,
    /// Policy denies the operation
    Denied(String),
    /// Policy requires approval
    RequiresApproval,
}

impl UpdatePolicy {
    /// Evaluate policy for an update operation
    pub fn evaluate_update(
        &self,
        channel: &ReleaseChannel,
        size_mb: u32,
        compliance_tags: &[String],
    ) -> PolicyResult {
        // Check if channel is allowed
        if !self.channel_allowed(channel) {
            return PolicyResult::Denied(format!("Release channel {:?} is not allowed", channel));
        }

        // Check download size
        if !self.download_size_allowed(size_mb) {
            return PolicyResult::Denied(format!(
                "Download size {}MB exceeds limit {}MB",
                size_mb, self.config.max_download_size_mb
            ));
        }

        // Check compliance requirements
        if !self.compliance_requirements_met(compliance_tags) {
            return PolicyResult::Denied("Compliance requirements not met".to_string());
        }

        // Check if approval is required
        if self.requires_approval() {
            return PolicyResult::RequiresApproval;
        }

        PolicyResult::Allowed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_security::access_control::Role;

    #[test]
    fn test_default_policy() {
        let policy = UpdatePolicy::default();
        assert!(policy.auto_updates_allowed());
        assert!(policy.channel_allowed(&ReleaseChannel::Stable));
        assert!(!policy.channel_allowed(&ReleaseChannel::Beta));
        assert!(!policy.requires_approval());
    }

    #[test]
    fn test_enterprise_policy() {
        let config = UpdatePolicyConfig {
            auto_update_enabled: false,
            check_interval_hours: 12,
            allowed_channels: vec![ReleaseChannel::Stable, ReleaseChannel::Beta],
            require_approval: true,
            max_download_size_mb: 50,
            security_requirements: SecurityRequirements {
                require_signature: true,
                require_checksum: true,
                allowed_cas: vec!["TestCA".to_string()],
                minimum_security_level: crate::models::SecuritySeverity::High,
            },
            enterprise_settings: Some(crate::models::EnterpriseSettings {
                organization_id: "test-org".to_string(),
                compliance_requirements: vec!["SOC2".to_string(), "GDPR".to_string()],
                custom_update_server: None,
                proxy_settings: None,
                audit_level: "detailed".to_string(),
            }),
        };

        let mut policy = UpdatePolicy::new(config);
        policy = policy.with_roles([Role::Admin].into());

        assert!(!policy.auto_updates_allowed());
        assert!(policy.channel_allowed(&ReleaseChannel::Beta));
        assert!(policy.download_size_allowed(40));
        assert!(!policy.download_size_allowed(60));
        assert!(policy.enterprise_enabled());
        assert_eq!(policy.organization_id(), Some("test-org"));
        assert!(policy.compliance_requirements_met(&["SOC2".to_string(), "GDPR".to_string()]));
        assert!(!policy.compliance_requirements_met(&["SOC2".to_string()]));
    }

    #[test]
    fn test_policy_evaluation() {
        let config = UpdatePolicyConfig {
            require_approval: true,
            max_download_size_mb: 50,
            ..Default::default()
        };

        let policy = UpdatePolicy::new(config);

        // Should require approval
        assert_eq!(
            policy.evaluate_update(&ReleaseChannel::Stable, 40, &[]),
            PolicyResult::RequiresApproval
        );

        // Should deny oversized download
        assert_eq!(
            policy.evaluate_update(&ReleaseChannel::Stable, 60, &[]),
            PolicyResult::Denied("Download size 60MB exceeds limit 50MB".to_string())
        );
    }
}
