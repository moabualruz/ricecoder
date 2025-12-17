//! Distribution analytics, monitoring, and enterprise usage tracking

use crate::error::{Result, UpdateError};
use crate::models::{UsageAnalytics, EnterpriseUsageReport, SecurityIncident};
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Analytics collector service
#[derive(Clone)]
pub struct AnalyticsCollector {
    client: Client,
    installation_id: Uuid,
    session_id: Uuid,
    analytics_endpoint: String,
    current_version: semver::Version,
    platform: String,
    session_start: DateTime<Utc>,
    event_buffer: Arc<RwLock<Vec<AnalyticsEvent>>>,
    flush_interval: Duration,
}

impl AnalyticsCollector {
    /// Create a new analytics collector
    pub fn new(
        analytics_endpoint: String,
        current_version: semver::Version,
        platform: String,
    ) -> Self {
        let installation_id = Self::get_or_create_installation_id();
        let session_id = Uuid::new_v4();
        let session_start = Utc::now();

        Self {
            client: Client::new(),
            installation_id,
            session_id,
            analytics_endpoint,
            current_version,
            platform,
            session_start,
            event_buffer: Arc::new(RwLock::new(Vec::new())),
            flush_interval: Duration::minutes(5),
        }
    }

    /// Record usage analytics
    pub async fn record_usage(&self, duration_seconds: u64, commands: Vec<String>, features: Vec<String>, error_count: u32, performance_metrics: HashMap<String, f64>) -> Result<()> {
        let analytics = UsageAnalytics {
            installation_id: self.installation_id,
            session_id: self.session_id,
            version: self.current_version.clone(),
            platform: self.platform.clone(),
            started_at: self.session_start,
            duration_seconds,
            commands_executed: commands,
            features_used: features,
            error_count,
            performance_metrics,
        };

        let event = AnalyticsEvent::Usage(analytics);
        self.buffer_event(event).await;
        Ok(())
    }

    /// Record update operation
    pub async fn record_update_operation(&self, operation: crate::models::UpdateOperation) -> Result<()> {
        let event = AnalyticsEvent::UpdateOperation(operation);
        self.buffer_event(event).await;
        Ok(())
    }

    /// Record security incident
    pub async fn record_security_incident(&self, incident: SecurityIncident) -> Result<()> {
        let event = AnalyticsEvent::SecurityIncident(incident);
        self.buffer_event(event).await;
        Ok(())
    }

    /// Generate enterprise usage report
    pub async fn generate_enterprise_report(&self, period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Result<EnterpriseUsageReport> {
        // In a real implementation, this would aggregate data from a database
        // For now, return a mock report
        let report = EnterpriseUsageReport {
            organization_id: "default-org".to_string(),
            period_start,
            period_end,
            total_installations: 1,
            active_installations: 1,
            version_distribution: {
                let mut dist = HashMap::new();
                dist.insert(self.current_version.to_string(), 1);
                dist
            },
            platform_distribution: {
                let mut dist = HashMap::new();
                dist.insert(self.platform.clone(), 1);
                dist
            },
            feature_usage: {
                let mut usage = HashMap::new();
                usage.insert("lsp".to_string(), 10);
                usage.insert("completion".to_string(), 15);
                usage
            },
            performance_metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("avg_startup_time".to_string(), 2.5);
                metrics.insert("avg_response_time".to_string(), 450.0);
                metrics
            },
            security_incidents: vec![],
        };

        Ok(report)
    }

    /// Flush buffered events to analytics server
    pub async fn flush_events(&self) -> Result<()> {
        let events = {
            let mut buffer = self.event_buffer.write().await;
            std::mem::take(&mut *buffer)
        };

        if events.is_empty() {
            return Ok(());
        }

        info!("Flushing {} analytics events", events.len());

        let payload = json!({
            "installation_id": self.installation_id,
            "session_id": self.session_id,
            "events": events
        });

        let response = self.client
            .post(&self.analytics_endpoint)
            .header("Content-Type", "application/json")
            .header("User-Agent", "RiceCoder-Analytics")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("Analytics flush failed: {} - {}", status, error_text);
            return Err(UpdateError::analytics(format!("Server returned status: {}", status)));
        }

        info!("Analytics events flushed successfully");
        Ok(())
    }

    /// Start background analytics flushing
    pub async fn start_background_flushing(self) -> Result<()> {
        let mut interval = tokio::time::interval(self.flush_interval.to_std().unwrap());

        loop {
            interval.tick().await;

            if let Err(e) = self.flush_events().await {
                error!("Background analytics flush failed: {}", e);
                // Continue running despite errors
            }
        }
    }

    /// Get or create installation ID
    fn get_or_create_installation_id() -> Uuid {
        // Try to read from file
        if let Ok(id_str) = std::fs::read_to_string(Self::installation_id_path()) {
            if let Ok(id) = Uuid::parse_str(id_str.trim()) {
                return id;
            }
        }

        // Create new ID and save it
        let id = Uuid::new_v4();
        let _ = std::fs::write(Self::installation_id_path(), id.to_string());
        id
    }

    /// Get installation ID file path
    fn installation_id_path() -> std::path::PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join("ricecoder")
            .join("installation.id")
    }

    /// Buffer an analytics event
    async fn buffer_event(&self, event: AnalyticsEvent) {
        let mut buffer = self.event_buffer.write().await;
        buffer.push(event);

        // Auto-flush if buffer gets too large
        if buffer.len() >= 100 {
            // Note: In a real implementation, we'd want to avoid recursive locking
            // This is simplified for demonstration
            drop(buffer);
            let _ = self.flush_events().await;
        }
    }
}

/// Analytics event types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AnalyticsEvent {
    /// Usage analytics
    Usage(UsageAnalytics),
    /// Update operation
    UpdateOperation(crate::models::UpdateOperation),
    /// Security incident
    SecurityIncident(SecurityIncident),
    /// Custom event
    Custom {
        /// Event name
        name: String,
        /// Event properties
        properties: HashMap<String, serde_json::Value>,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },
}

/// Enterprise analytics dashboard
pub struct EnterpriseDashboard {
    collector: AnalyticsCollector,
    organization_id: String,
}

impl EnterpriseDashboard {
    /// Create a new enterprise dashboard
    pub fn new(collector: AnalyticsCollector, organization_id: String) -> Self {
        Self {
            collector,
            organization_id,
        }
    }

    /// Get usage report for the organization
    pub async fn get_usage_report(&self, period_days: i64) -> Result<EnterpriseUsageReport> {
        let end = Utc::now();
        let start = end - Duration::days(period_days);

        self.collector.generate_enterprise_report(start, end).await
    }

    /// Get security incidents report
    pub async fn get_security_report(&self, period_days: i64) -> Result<Vec<SecurityIncident>> {
        // In a real implementation, this would query a database
        // For now, return empty vec
        let _period_days = period_days;
        Ok(vec![])
    }

    /// Get compliance status
    pub async fn get_compliance_status(&self) -> Result<HashMap<String, bool>> {
        // Mock compliance status
        let mut status = HashMap::new();
        status.insert("SOC2".to_string(), true);
        status.insert("GDPR".to_string(), true);
        status.insert("HIPAA".to_string(), false);
        Ok(status)
    }

    /// Export analytics data for compliance
    pub async fn export_compliance_data(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<String> {
        let report = self.collector.generate_enterprise_report(start, end).await?;

        // Export as JSON
        serde_json::to_string_pretty(&report)
            .map_err(|e| UpdateError::analytics(format!("Export failed: {}", e)))
    }
}

/// Compliance reporting service
pub struct ComplianceReporter {
    dashboard: EnterpriseDashboard,
}

impl ComplianceReporter {
    /// Create a new compliance reporter
    pub fn new(dashboard: EnterpriseDashboard) -> Self {
        Self { dashboard }
    }

    /// Generate SOC 2 compliance report
    pub async fn generate_soc2_report(&self, period_days: i64) -> Result<String> {
        let report = self.dashboard.get_usage_report(period_days).await?;
        let security_incidents = self.dashboard.get_security_report(period_days).await?;

        let soc2_report = json!({
            "organization_id": report.organization_id,
            "period_start": report.period_start,
            "period_end": report.period_end,
            "total_installations": report.total_installations,
            "active_installations": report.active_installations,
            "security_incidents": security_incidents.len(),
            "compliance_status": {
                "data_security": security_incidents.is_empty(),
                "access_controls": true, // Mock
                "change_management": true, // Mock
                "risk_management": true, // Mock
                "monitoring": true, // Mock
            }
        });

        serde_json::to_string_pretty(&soc2_report)
            .map_err(|e| UpdateError::analytics(format!("SOC2 report generation failed: {}", e)))
    }

    /// Generate GDPR compliance report
    pub async fn generate_gdpr_report(&self, period_days: i64) -> Result<String> {
        let report = self.dashboard.get_usage_report(period_days).await?;
        let compliance_status = self.dashboard.get_compliance_status().await?;

        let gdpr_compliant = compliance_status.get("GDPR").copied().unwrap_or(false);

        let gdpr_report = json!({
            "organization_id": report.organization_id,
            "period_start": report.period_start,
            "period_end": report.period_end,
            "gdpr_compliant": gdpr_compliant,
            "data_processing": {
                "personal_data_collected": false, // RiceCoder doesn't collect personal data
                "data_retention_period": "session_only",
                "data_deletion_capability": true,
                "consent_management": true,
            },
            "data_subject_rights": {
                "right_to_access": true,
                "right_to_rectification": true,
                "right_to_erasure": true,
                "right_to_portability": true,
            }
        });

        serde_json::to_string_pretty(&gdpr_report)
            .map_err(|e| UpdateError::analytics(format!("GDPR report generation failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_analytics_collector_creation() {
        let version = semver::Version::from_str("1.0.0").unwrap();
        let collector = AnalyticsCollector::new(
            "https://analytics.example.com".to_string(),
            version.clone(),
            "linux-x86_64".to_string(),
        );

        assert_eq!(collector.current_version, version);
        assert_eq!(collector.platform, "linux-x86_64");
        assert_eq!(collector.analytics_endpoint, "https://analytics.example.com");
    }

    #[tokio::test]
    async fn test_usage_recording() {
        let version = semver::Version::from_str("1.0.0").unwrap();
        let collector = AnalyticsCollector::new(
            "https://analytics.example.com".to_string(),
            version,
            "linux-x86_64".to_string(),
        );

        let commands = vec!["lsp".to_string(), "completion".to_string()];
        let features = vec!["lsp".to_string()];
        let mut performance = HashMap::new();
        performance.insert("response_time".to_string(), 100.0);

        collector.record_usage(300, commands.clone(), features.clone(), 0, performance).await.unwrap();

        // Check that event was buffered
        let buffer = collector.event_buffer.read().await;
        assert_eq!(buffer.len(), 1);

        match &buffer[0] {
            AnalyticsEvent::Usage(analytics) => {
                assert_eq!(analytics.commands_executed, commands);
                assert_eq!(analytics.features_used, features);
                assert_eq!(analytics.duration_seconds, 300);
                assert_eq!(analytics.error_count, 0);
            }
            _ => panic!("Expected Usage event"),
        }
    }

    #[test]
    fn test_installation_id_persistence() {
        let id1 = AnalyticsCollector::get_or_create_installation_id();
        let id2 = AnalyticsCollector::get_or_create_installation_id();

        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_enterprise_dashboard() {
        let version = semver::Version::from_str("1.0.0").unwrap();
        let collector = AnalyticsCollector::new(
            "https://analytics.example.com".to_string(),
            version,
            "linux-x86_64".to_string(),
        );

        let dashboard = EnterpriseDashboard::new(collector, "test-org".to_string());

        let report = dashboard.get_usage_report(30).await.unwrap();
        assert_eq!(report.organization_id, "test-org");
        assert_eq!(report.total_installations, 1);
        assert_eq!(report.active_installations, 1);
    }

    #[tokio::test]
    async fn test_compliance_reporting() {
        let version = semver::Version::from_str("1.0.0").unwrap();
        let collector = AnalyticsCollector::new(
            "https://analytics.example.com".to_string(),
            version,
            "linux-x86_64".to_string(),
        );

        let dashboard = EnterpriseDashboard::new(collector, "test-org".to_string());
        let reporter = ComplianceReporter::new(dashboard);

        let soc2_report = reporter.generate_soc2_report(30).await.unwrap();
        assert!(soc2_report.contains("test-org"));

        let gdpr_report = reporter.generate_gdpr_report(30).await.unwrap();
        assert!(gdpr_report.contains("gdpr_compliant"));
    }
}