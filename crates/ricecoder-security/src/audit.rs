//! Audit logging system for security events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

use crate::Result;

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    ApiKeyAccess,
    ApiKeyRotation,
    Authentication,
    Authorization,
    DataAccess,
    SecurityViolation,
    SystemAccess,
    // Compliance-specific events
    DataErasure,
    DataPortability,
    ConsentChange,
    PrivacySettingsChange,
    CustomerKeyRotation,
    ComplianceReportGenerated,
    LogRetentionCleanup,
    AnalyticsOptIn,
    AnalyticsOptOut,
}

/// Audit event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub action: String,
    pub resource: String,
    pub metadata: serde_json::Value,
}

/// Audit record for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub action: String,
    pub resource: String,
    pub metadata: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Audit storage trait
#[async_trait::async_trait]
pub trait AuditStorage: Send + Sync + std::fmt::Debug {
    async fn store_record(&self, record: &AuditRecord) -> Result<()>;
    async fn query_records(
        &self,
        filter: AuditQuery,
        limit: usize,
    ) -> Result<Vec<AuditRecord>>;
}

/// Audit query for filtering records
#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    pub event_type: Option<AuditEventType>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub resource: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

/// In-memory audit storage for testing/development
#[derive(Debug)]
pub struct MemoryAuditStorage {
    records: Arc<Mutex<Vec<AuditRecord>>>,
}

impl MemoryAuditStorage {
    pub fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl AuditStorage for MemoryAuditStorage {
    async fn store_record(&self, record: &AuditRecord) -> Result<()> {
        let mut records = self.records.lock().unwrap();
        records.push(record.clone());
        Ok(())
    }

    async fn query_records(
        &self,
        filter: AuditQuery,
        limit: usize,
    ) -> Result<Vec<AuditRecord>> {
        let records = self.records.lock().unwrap();
        let mut filtered: Vec<AuditRecord> = records
            .iter()
            .filter(|record| {
                if let Some(ref event_type) = filter.event_type {
                    if std::mem::discriminant(event_type) != std::mem::discriminant(&record.event_type) {
                        return false;
                    }
                }
                if let Some(ref user_id) = filter.user_id {
                    if record.user_id.as_ref() != Some(user_id) {
                        return false;
                    }
                }
                if let Some(ref session_id) = filter.session_id {
                    if record.session_id.as_ref() != Some(session_id) {
                        return false;
                    }
                }
                if let Some(ref resource) = filter.resource {
                    if &record.resource != resource {
                        return false;
                    }
                }
                if let Some(start_time) = filter.start_time {
                    if record.timestamp < start_time {
                        return false;
                    }
                }
                if let Some(end_time) = filter.end_time {
                    if record.timestamp > end_time {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        filtered.truncate(limit);
        Ok(filtered)
    }
}

/// Audit logger for recording security events
#[derive(Debug)]
pub struct AuditLogger {
    storage: Arc<dyn AuditStorage>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(storage: Arc<dyn AuditStorage>) -> Self {
        Self { storage }
    }

    /// Log an audit event
    pub async fn log_event(&self, event: AuditEvent) -> Result<()> {
        let record = AuditRecord {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: event.event_type,
            user_id: event.user_id,
            session_id: event.session_id,
            action: event.action,
            resource: event.resource,
            metadata: event.metadata,
            ip_address: None, // Would be set by middleware in real implementation
            user_agent: None, // Would be set by middleware in real implementation
        };

        self.storage.store_record(&record).await
    }

    /// Log API key access
    pub async fn log_api_key_access(
        &self,
        user_id: Option<String>,
        session_id: Option<String>,
        provider: &str,
    ) -> Result<()> {
        let event = AuditEvent {
            event_type: AuditEventType::ApiKeyAccess,
            user_id,
            session_id,
            action: "access".to_string(),
            resource: format!("api_key:{}", provider),
            metadata: serde_json::json!({
                "provider": provider,
                "access_type": "read"
            }),
        };
        self.log_event(event).await
    }

    /// Log API key rotation
    pub async fn log_api_key_rotation(
        &self,
        user_id: Option<String>,
        session_id: Option<String>,
        provider: &str,
    ) -> Result<()> {
        let event = AuditEvent {
            event_type: AuditEventType::ApiKeyRotation,
            user_id,
            session_id,
            action: "rotate".to_string(),
            resource: format!("api_key:{}", provider),
            metadata: serde_json::json!({
                "provider": provider,
                "rotation_type": "manual"
            }),
        };
        self.log_event(event).await
    }

    /// Log security violation
    pub async fn log_security_violation(
        &self,
        violation_type: &str,
        details: serde_json::Value,
    ) -> Result<()> {
        let event = AuditEvent {
            event_type: AuditEventType::SecurityViolation,
            user_id: None,
            session_id: None,
            action: "violation".to_string(),
            resource: "security".to_string(),
            metadata: serde_json::json!({
                "violation_type": violation_type,
                "details": details
            }),
        };
        self.log_event(event).await
    }

    /// Log data erasure event
    pub async fn log_data_erasure(
        &self,
        user_id: &str,
        reason: &str,
        request_id: &Uuid,
    ) -> Result<()> {
        let event = AuditEvent {
            event_type: AuditEventType::DataErasure,
            user_id: Some(user_id.to_string()),
            session_id: None,
            action: "erasure_requested".to_string(),
            resource: format!("user_data:{}", user_id),
            metadata: serde_json::json!({
                "reason": reason,
                "request_id": request_id,
                "compliance": "GDPR_HIPAA"
            }),
        };
        self.log_event(event).await
    }

    /// Log data portability event
    pub async fn log_data_portability(
        &self,
        user_id: &str,
        data_types: &[String],
        format: &str,
        request_id: &Uuid,
    ) -> Result<()> {
        let event = AuditEvent {
            event_type: AuditEventType::DataPortability,
            user_id: Some(user_id.to_string()),
            session_id: None,
            action: "portability_requested".to_string(),
            resource: format!("user_data:{}", user_id),
            metadata: serde_json::json!({
                "data_types": data_types,
                "format": format,
                "request_id": request_id,
                "compliance": "GDPR"
            }),
        };
        self.log_event(event).await
    }

    /// Log consent change
    pub async fn log_consent_change(
        &self,
        user_id: &str,
        consent_type: &str,
        granted: bool,
    ) -> Result<()> {
        let event = AuditEvent {
            event_type: AuditEventType::ConsentChange,
            user_id: Some(user_id.to_string()),
            session_id: None,
            action: if granted { "consent_granted" } else { "consent_revoked" }.to_string(),
            resource: format!("user_consent:{}", user_id),
            metadata: serde_json::json!({
                "consent_type": consent_type,
                "granted": granted,
                "compliance": "GDPR"
            }),
        };
        self.log_event(event).await
    }

    /// Log customer key rotation
    pub async fn log_customer_key_rotation(
        &self,
        user_id: &str,
        key_id: &str,
        new_version: &str,
    ) -> Result<()> {
        let event = AuditEvent {
            event_type: AuditEventType::CustomerKeyRotation,
            user_id: Some(user_id.to_string()),
            session_id: None,
            action: "key_rotated".to_string(),
            resource: format!("customer_key:{}", key_id),
            metadata: serde_json::json!({
                "new_version": new_version,
                "compliance": "SOC2"
            }),
        };
        self.log_event(event).await
    }

    /// Log analytics opt-in/opt-out
    pub async fn log_analytics_consent(
        &self,
        user_id: &str,
        opted_in: bool,
    ) -> Result<()> {
        let event = AuditEvent {
            event_type: if opted_in { AuditEventType::AnalyticsOptIn } else { AuditEventType::AnalyticsOptOut },
            user_id: Some(user_id.to_string()),
            session_id: None,
            action: if opted_in { "analytics_opt_in" } else { "analytics_opt_out" }.to_string(),
            resource: format!("user_analytics:{}", user_id),
            metadata: serde_json::json!({
                "opted_in": opted_in,
                "compliance": "privacy"
            }),
        };
        self.log_event(event).await
    }

    /// Log compliance report generation
    pub async fn log_compliance_report(
        &self,
        report_type: &str,
        generated_by: &str,
    ) -> Result<()> {
        let event = AuditEvent {
            event_type: AuditEventType::ComplianceReportGenerated,
            user_id: Some(generated_by.to_string()),
            session_id: None,
            action: "report_generated".to_string(),
            resource: "compliance_reports".to_string(),
            metadata: serde_json::json!({
                "report_type": report_type,
                "compliance": "SOC2_GDPR_HIPAA"
            }),
        };
        self.log_event(event).await
    }

    /// Query audit records
    pub async fn query_records(
        &self,
        filter: AuditQuery,
        limit: usize,
    ) -> Result<Vec<AuditRecord>> {
        self.storage.query_records(filter, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_audit_logging() {
        let storage = Arc::new(MemoryAuditStorage::new());
        let logger = AuditLogger::new(storage.clone());

        // Log an API key access event
        logger
            .log_api_key_access(
                Some("user123".to_string()),
                Some("session456".to_string()),
                "openai",
            )
            .await
            .unwrap();

        // Query the records
        let filter = AuditQuery {
            event_type: Some(AuditEventType::ApiKeyAccess),
            ..Default::default()
        };
        let records = logger.query_records(filter, 10).await.unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].user_id, Some("user123".to_string()));
        assert_eq!(records[0].resource, "api_key:openai");
    }

    #[tokio::test]
    async fn test_security_violation_logging() {
        let storage = Arc::new(MemoryAuditStorage::new());
        let logger = AuditLogger::new(storage.clone());

        let violation_details = serde_json::json!({
            "attempted_action": "path_traversal",
            "input": "../../../etc/passwd"
        });

        logger
            .log_security_violation("path_traversal", violation_details)
            .await
            .unwrap();

        let filter = AuditQuery {
            event_type: Some(AuditEventType::SecurityViolation),
            ..Default::default()
        };
        let records = logger.query_records(filter, 10).await.unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].action, "violation");
    }
}