//! Core data models for the updates system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Release information from update server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    /// Release version
    pub version: semver::Version,
    /// Release channel (stable, beta, nightly)
    pub channel: ReleaseChannel,
    /// Release date
    pub release_date: DateTime<Utc>,
    /// Minimum required version for upgrade
    pub minimum_version: Option<semver::Version>,
    /// Release notes
    pub notes: String,
    /// Download URLs by platform
    pub downloads: HashMap<String, DownloadInfo>,
    /// Security advisories
    pub security_advisories: Vec<SecurityAdvisory>,
    /// Compliance information
    pub compliance: ComplianceInfo,
}

/// Release channel enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    /// Stable releases
    Stable,
    /// Beta releases
    Beta,
    /// Nightly/development releases
    Nightly,
}

/// Download information for a specific platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    /// Download URL
    pub url: String,
    /// File size in bytes
    pub size: u64,
    /// SHA-256 checksum
    pub sha256: String,
    /// Signature for verification
    pub signature: Option<String>,
}

/// Security advisory information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAdvisory {
    /// Advisory ID
    pub id: String,
    /// Severity level
    pub severity: SecuritySeverity,
    /// Description
    pub description: String,
    /// Affected versions
    pub affected_versions: String,
    /// Fixed in version
    pub fixed_in: semver::Version,
}

/// Security severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum SecuritySeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Compliance information for enterprise requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceInfo {
    /// SOC 2 compliance status
    pub soc2_compliant: bool,
    /// GDPR compliance status
    pub gdpr_compliant: bool,
    /// HIPAA compliance status
    pub hipaa_compliant: bool,
    /// Security audit status
    pub security_audited: bool,
    /// Last compliance review date
    pub last_review: DateTime<Utc>,
}

/// Update check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    /// Whether an update is available
    pub update_available: bool,
    /// Latest available version
    pub latest_version: Option<semver::Version>,
    /// Current installed version
    pub current_version: semver::Version,
    /// Release information if update available
    pub release_info: Option<ReleaseInfo>,
    /// Last check timestamp
    pub checked_at: DateTime<Utc>,
    /// Next check recommended time
    pub next_check: DateTime<Utc>,
}

/// Update installation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UpdateStatus {
    /// Update is pending
    Pending,
    /// Update is downloading
    Downloading,
    /// Update is downloaded and ready
    Downloaded,
    /// Update is installing
    Installing,
    /// Update installed successfully
    Installed,
    /// Update failed
    Failed,
    /// Update rolled back
    RolledBack,
}

/// Update operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOperation {
    /// Unique operation ID
    pub id: Uuid,
    /// Target version
    pub version: semver::Version,
    /// Operation status
    pub status: UpdateStatus,
    /// Start timestamp
    pub started_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Rollback information
    pub rollback_info: Option<RollbackInfo>,
    /// Security validation results
    pub security_validation: SecurityValidationResult,
}

/// Rollback information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackInfo {
    /// Previous version before update
    pub previous_version: semver::Version,
    /// Backup location
    pub backup_path: String,
    /// Rollback reason
    pub reason: String,
    /// Rollback timestamp
    pub rolled_back_at: DateTime<Utc>,
}

/// Security validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Validation timestamp
    pub validated_at: DateTime<Utc>,
    /// Checksum verification result
    pub checksum_valid: bool,
    /// Signature verification result
    pub signature_valid: Option<bool>,
    /// Additional validation details
    pub details: HashMap<String, String>,
}

/// Usage analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnalytics {
    /// Installation ID
    pub installation_id: Uuid,
    /// Session ID
    pub session_id: Uuid,
    /// Version being used
    pub version: semver::Version,
    /// Platform information
    pub platform: String,
    /// Usage start time
    pub started_at: DateTime<Utc>,
    /// Usage duration in seconds
    pub duration_seconds: u64,
    /// Commands executed
    pub commands_executed: Vec<String>,
    /// Features used
    pub features_used: Vec<String>,
    /// Error count
    pub error_count: u32,
    /// Performance metrics
    pub performance_metrics: HashMap<String, f64>,
}

/// Enterprise usage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseUsageReport {
    /// Organization ID
    pub organization_id: String,
    /// Report period start
    pub period_start: DateTime<Utc>,
    /// Report period end
    pub period_end: DateTime<Utc>,
    /// Total installations
    pub total_installations: u32,
    /// Active installations
    pub active_installations: u32,
    /// Version distribution
    pub version_distribution: HashMap<String, u32>,
    /// Platform distribution
    pub platform_distribution: HashMap<String, u32>,
    /// Feature usage statistics
    pub feature_usage: HashMap<String, u32>,
    /// Performance metrics
    pub performance_metrics: HashMap<String, f64>,
    /// Security incidents
    pub security_incidents: Vec<SecurityIncident>,
}

/// Security incident report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncident {
    /// Incident ID
    pub id: Uuid,
    /// Incident timestamp
    pub timestamp: DateTime<Utc>,
    /// Incident severity
    pub severity: SecuritySeverity,
    /// Incident description
    pub description: String,
    /// Affected installations
    pub affected_installations: u32,
    /// Resolution status
    pub resolved: bool,
}

/// Update policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicyConfig {
    /// Whether automatic updates are enabled
    pub auto_update_enabled: bool,
    /// Update check interval in hours
    pub check_interval_hours: u32,
    /// Allowed release channels
    pub allowed_channels: Vec<ReleaseChannel>,
    /// Require manual approval for updates
    pub require_approval: bool,
    /// Maximum download size in MB
    pub max_download_size_mb: u32,
    /// Security validation requirements
    pub security_requirements: SecurityRequirements,
    /// Enterprise-specific settings
    pub enterprise_settings: Option<EnterpriseSettings>,
}

/// Security requirements for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// Require signature verification
    pub require_signature: bool,
    /// Require checksum verification
    pub require_checksum: bool,
    /// Allowed certificate authorities
    pub allowed_cas: Vec<String>,
    /// Minimum security level
    pub minimum_security_level: SecuritySeverity,
}

/// Enterprise-specific update settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseSettings {
    /// Organization ID
    pub organization_id: String,
    /// Compliance requirements
    pub compliance_requirements: Vec<String>,
    /// Custom update server URL
    pub custom_update_server: Option<String>,
    /// Proxy settings
    pub proxy_settings: Option<ProxySettings>,
    /// Audit logging level
    pub audit_level: String,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    /// Proxy URL
    pub url: String,
    /// Authentication credentials
    pub auth: Option<ProxyAuth>,
}

/// Proxy authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAuth {
    /// Username
    pub username: String,
    /// Password (should be encrypted)
    pub password: String,
}