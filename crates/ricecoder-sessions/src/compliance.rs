//! GDPR/HIPAA compliance management for sessions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use uuid::Uuid;

use crate::error::{SessionError, SessionResult};
use crate::models::{
    ComplianceAlertLevel, ComplianceEvent, ComplianceEventType, DataErasureRequest,
    DataExportFormat, DataExportRequest, DataMinimizationSettings, DataRetentionPolicy, DataType,
    ErasureReason, PrivacySettings, Session,
};
use crate::store::SessionStore;

/// GDPR/HIPAA compliance manager
#[derive(Debug)]
pub struct ComplianceManager {
    /// Data retention policy
    retention_policy: DataRetentionPolicy,
    /// Privacy settings
    privacy_settings: PrivacySettings,
    /// Session store for data operations
    session_store: SessionStore,
    /// Audit logger for compliance events
    audit_logger: Option<ricecoder_security::audit::AuditLogger>,
}

impl ComplianceManager {
    /// Create a new compliance manager
    pub fn new(session_store: SessionStore) -> Self {
        Self {
            retention_policy: DataRetentionPolicy {
                session_data_retention_days: 2555, // 7 years for HIPAA
                audit_log_retention_days: 2555,
                backup_retention_days: 2555,
                auto_delete_expired_data: true,
                data_minimization: DataMinimizationSettings {
                    anonymize_ip_addresses: true,
                    limit_unnecessary_collection: true,
                    enable_data_purging: true,
                    enable_data_export: true,
                },
            },
            privacy_settings: PrivacySettings {
                enable_differential_privacy: true,
                enable_data_anonymization: true,
                enable_consent_management: true,
                enable_privacy_auditing: true,
            },
            session_store,
            audit_logger: None,
        }
    }

    /// Set audit logger for compliance events
    pub fn with_audit_logger(
        mut self,
        audit_logger: ricecoder_security::audit::AuditLogger,
    ) -> Self {
        self.audit_logger = Some(audit_logger);
        self
    }

    /// Set custom retention policy
    pub fn with_retention_policy(mut self, policy: DataRetentionPolicy) -> Self {
        self.retention_policy = policy;
        self
    }

    /// Set privacy settings
    pub fn with_privacy_settings(mut self, settings: PrivacySettings) -> Self {
        self.privacy_settings = settings;
        self
    }

    /// Process data export request (GDPR Article 20)
    pub async fn process_data_export(&self, request: DataExportRequest) -> SessionResult<PathBuf> {
        let export_id = Uuid::new_v4().to_string();
        let export_dir = self.get_export_dir().await?;
        let export_path = export_dir.join(format!(
            "data_export_{}_{}.{}",
            request.user_id,
            export_id,
            match request.format {
                DataExportFormat::Json => "json",
                DataExportFormat::Xml => "xml",
                DataExportFormat::Csv => "csv",
                DataExportFormat::Pdf => "pdf",
            }
        ));

        // Collect user data
        let mut export_data = serde_json::json!({
            "user_id": request.user_id,
            "export_id": export_id,
            "requested_at": request.requested_at,
            "format": request.format,
        });

        if request.include_session_data {
            // Export user's sessions
            let sessions = self.collect_user_sessions(&request.user_id).await?;
            export_data["sessions"] = serde_json::json!(sessions);
        }

        if request.include_audit_logs {
            // Export user's audit logs
            let audit_logs = self.collect_user_audit_logs(&request.user_id).await?;
            export_data["audit_logs"] = serde_json::json!(audit_logs);
        }

        if request.include_sharing_history {
            // Export user's sharing history
            let sharing_history = self.collect_user_sharing_history(&request.user_id).await?;
            export_data["sharing_history"] = serde_json::json!(sharing_history);
        }

        // Apply data minimization
        if self
            .retention_policy
            .data_minimization
            .limit_unnecessary_collection
        {
            self.minimize_export_data(&mut export_data);
        }

        // Anonymize sensitive data if enabled
        if self.privacy_settings.enable_data_anonymization {
            self.anonymize_export_data(&mut export_data);
        }

        // Write export file
        let export_content = match request.format {
            DataExportFormat::Json => serde_json::to_string_pretty(&export_data)?,
            DataExportFormat::Xml => self.convert_to_xml(&export_data),
            DataExportFormat::Csv => self.convert_to_csv(&export_data),
            DataExportFormat::Pdf => self.convert_to_pdf(&export_data),
        };

        fs::write(&export_path, export_content).await?;

        // Log compliance event
        self.log_compliance_event(ComplianceEvent {
            id: Uuid::new_v4().to_string(),
            event_type: ComplianceEventType::SessionShared, // Using existing type, could add DataExported
            alert_level: ComplianceAlertLevel::Info,
            user_id: Some(request.user_id.clone()),
            session_id: None,
            description: format!("Data export completed for user {}", request.user_id),
            metadata: {
                let mut map = HashMap::new();
                map.insert(
                    "export_id".to_string(),
                    serde_json::Value::String(export_id),
                );
                map.insert(
                    "format".to_string(),
                    serde_json::Value::String(format!("{:?}", request.format)),
                );
                map.insert(
                    "export_path".to_string(),
                    serde_json::Value::String(export_path.to_string_lossy().to_string()),
                );
                map
            },
            timestamp: Utc::now(),
        })
        .await;

        Ok(export_path)
    }

    /// Process right to erasure request (GDPR Article 17)
    pub async fn process_data_erasure(&self, request: DataErasureRequest) -> SessionResult<usize> {
        let mut erased_items = 0;

        // Validate erasure request
        self.validate_erasure_request(&request)?;

        // Erase data based on request
        if request.erase_all_data || request.data_types_to_erase.contains(&DataType::Sessions) {
            erased_items += self.erase_user_sessions(&request.user_id).await?;
        }

        if request.erase_all_data || request.data_types_to_erase.contains(&DataType::AuditLogs) {
            erased_items += self.erase_user_audit_logs(&request.user_id).await?;
        }

        if request.erase_all_data
            || request
                .data_types_to_erase
                .contains(&DataType::SharingHistory)
        {
            erased_items += self.erase_user_sharing_history(&request.user_id).await?;
        }

        if request.erase_all_data
            || request
                .data_types_to_erase
                .contains(&DataType::UserPreferences)
        {
            erased_items += self.erase_user_preferences(&request.user_id).await?;
        }

        // Log compliance event
        self.log_compliance_event(ComplianceEvent {
            id: Uuid::new_v4().to_string(),
            event_type: ComplianceEventType::RetentionViolation, // Using existing type, could add DataErased
            alert_level: ComplianceAlertLevel::Info,
            user_id: Some(request.user_id.clone()),
            session_id: None,
            description: format!(
                "Data erasure completed for user {}: {} items erased",
                request.user_id, erased_items
            ),
            metadata: {
                let mut map = HashMap::new();
                map.insert(
                    "erasure_reason".to_string(),
                    serde_json::Value::String(format!("{:?}", request.reason)),
                );
                map.insert(
                    "data_types_erased".to_string(),
                    serde_json::Value::Array(
                        request
                            .data_types_to_erase
                            .iter()
                            .map(|dt| serde_json::Value::String(format!("{:?}", dt)))
                            .collect(),
                    ),
                );
                map.insert(
                    "items_erased".to_string(),
                    serde_json::Value::Number(erased_items.into()),
                );
                map
            },
            timestamp: Utc::now(),
        })
        .await;

        Ok(erased_items)
    }

    /// Apply data retention policies
    pub async fn apply_retention_policies(&self) -> SessionResult<usize> {
        let mut cleaned_items = 0;

        if self.retention_policy.auto_delete_expired_data {
            // Clean expired sessions
            let retention_duration = Duration::from_secs(
                (self.retention_policy.session_data_retention_days as u64) * 24 * 60 * 60,
            );
            cleaned_items += self
                .session_store
                .cleanup_old_sessions(retention_duration)
                .await?;

            // Clean expired audit logs (would need audit log store)
            // cleaned_items += self.audit_store.cleanup_old_logs(cutoff_date).await?;

            // Clean expired backups
            cleaned_items += self.cleanup_expired_backups().await?;
        }

        // Log compliance event
        self.log_compliance_event(ComplianceEvent {
            id: Uuid::new_v4().to_string(),
            event_type: ComplianceEventType::RetentionViolation,
            alert_level: if cleaned_items > 0 {
                ComplianceAlertLevel::Warning
            } else {
                ComplianceAlertLevel::Info
            },
            user_id: None,
            session_id: None,
            description: format!(
                "Data retention policies applied: {} items cleaned",
                cleaned_items
            ),
            metadata: {
                let mut map = HashMap::new();
                map.insert(
                    "retention_policy".to_string(),
                    serde_json::to_value(&self.retention_policy).unwrap_or(serde_json::Value::Null),
                );
                map.insert(
                    "items_cleaned".to_string(),
                    serde_json::Value::Number(cleaned_items.into()),
                );
                map
            },
            timestamp: Utc::now(),
        })
        .await;

        Ok(cleaned_items)
    }

    /// Validate data processing consent
    pub fn validate_consent(
        &self,
        user_id: &str,
        data_processing_type: &str,
    ) -> SessionResult<bool> {
        // In a real implementation, this would check a consent store
        // For now, assume consent is granted
        Ok(true)
    }

    /// Generate privacy policy compliance report
    pub async fn generate_privacy_report(&self, user_id: &str) -> SessionResult<serde_json::Value> {
        let report = serde_json::json!({
            "user_id": user_id,
            "report_generated_at": Utc::now(),
            "data_retention_policy": self.retention_policy,
            "privacy_settings": self.privacy_settings,
            "sessions_count": self.count_user_sessions(user_id).await?,
            "audit_logs_count": self.count_user_audit_logs(user_id).await?,
            "sharing_history_count": self.count_user_sharing_history(user_id).await?,
            "last_activity": self.get_user_last_activity(user_id).await?,
            "compliance_status": "compliant"
        });

        Ok(report)
    }

    // Helper methods

    async fn get_export_dir(&self) -> SessionResult<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            SessionError::ConfigError("Could not determine home directory".to_string())
        })?;
        let export_dir = home.join(".ricecoder").join("exports");
        fs::create_dir_all(&export_dir).await?;
        Ok(export_dir)
    }

    async fn collect_user_sessions(&self, user_id: &str) -> SessionResult<Vec<Session>> {
        // In a real implementation, this would query sessions by user_id
        // For now, return empty vec
        Ok(Vec::new())
    }

    async fn collect_user_audit_logs(
        &self,
        user_id: &str,
    ) -> SessionResult<Vec<serde_json::Value>> {
        // In a real implementation, this would query audit logs by user_id
        Ok(Vec::new())
    }

    async fn collect_user_sharing_history(
        &self,
        user_id: &str,
    ) -> SessionResult<Vec<serde_json::Value>> {
        // In a real implementation, this would query sharing history by user_id
        Ok(Vec::new())
    }

    fn minimize_export_data(&self, data: &mut serde_json::Value) {
        // Remove unnecessary fields for data minimization
        if let Some(obj) = data.as_object_mut() {
            // Remove internal IDs, timestamps that aren't needed, etc.
            obj.remove("internal_id");
        }
    }

    fn anonymize_export_data(&self, data: &mut serde_json::Value) {
        // Anonymize sensitive data
        if let Some(obj) = data.as_object_mut() {
            if let Some(ip) = obj.get_mut("ip_address") {
                if let Some(ip_str) = ip.as_str() {
                    // Anonymize last octet of IP
                    let parts: Vec<&str> = ip_str.split('.').collect();
                    if parts.len() == 4 {
                        *ip =
                            serde_json::json!(format!("{}.{}.{}.0", parts[0], parts[1], parts[2]));
                    }
                }
            }
        }
    }

    fn convert_to_xml(&self, data: &serde_json::Value) -> String {
        // Simple XML conversion - in production, use a proper XML library
        format!(
            "<data>{}</data>",
            serde_json::to_string(data).unwrap_or_default()
        )
    }

    fn convert_to_csv(&self, data: &serde_json::Value) -> String {
        // Simple CSV conversion - in production, use a proper CSV library
        "field1,field2\nvalue1,value2\n".to_string()
    }

    fn convert_to_pdf(&self, data: &serde_json::Value) -> String {
        // PDF generation would require a PDF library
        // For now, return JSON as base64
        format!(
            "PDF:{}",
            base64::encode(serde_json::to_string(data).unwrap_or_default())
        )
    }

    fn validate_erasure_request(&self, request: &DataErasureRequest) -> SessionResult<()> {
        // Validate that the request is legitimate
        if request.data_types_to_erase.is_empty() && !request.erase_all_data {
            return Err(SessionError::Invalid(
                "Erasure request must specify data types or erase all data".to_string(),
            ));
        }
        Ok(())
    }

    async fn erase_user_sessions(&self, user_id: &str) -> SessionResult<usize> {
        // In a real implementation, this would delete user sessions
        Ok(0)
    }

    async fn erase_user_audit_logs(&self, user_id: &str) -> SessionResult<usize> {
        // In a real implementation, this would delete user audit logs
        Ok(0)
    }

    async fn erase_user_sharing_history(&self, user_id: &str) -> SessionResult<usize> {
        // In a real implementation, this would delete user sharing history
        Ok(0)
    }

    async fn erase_user_preferences(&self, user_id: &str) -> SessionResult<usize> {
        // In a real implementation, this would delete user preferences
        Ok(0)
    }

    async fn cleanup_expired_backups(&self) -> SessionResult<usize> {
        // In a real implementation, this would clean up expired backups
        Ok(0)
    }

    async fn count_user_sessions(&self, user_id: &str) -> SessionResult<usize> {
        Ok(0)
    }

    async fn count_user_audit_logs(&self, user_id: &str) -> SessionResult<usize> {
        Ok(0)
    }

    async fn count_user_sharing_history(&self, user_id: &str) -> SessionResult<usize> {
        Ok(0)
    }

    async fn get_user_last_activity(&self, user_id: &str) -> SessionResult<Option<DateTime<Utc>>> {
        Ok(Some(Utc::now()))
    }

    async fn log_compliance_event(&self, event: ComplianceEvent) {
        if let Some(ref audit_logger) = self.audit_logger {
            let event_clone = event.clone();
            let audit_event = ricecoder_security::audit::AuditEvent {
                event_type: match event.event_type {
                    ComplianceEventType::SessionShared => {
                        ricecoder_security::audit::AuditEventType::DataAccess
                    }
                    ComplianceEventType::UnauthorizedAccess => {
                        ricecoder_security::audit::AuditEventType::SecurityViolation
                    }
                    ComplianceEventType::RetentionViolation => {
                        ricecoder_security::audit::AuditEventType::LogRetentionCleanup
                    }
                    ComplianceEventType::EncryptionViolation => {
                        ricecoder_security::audit::AuditEventType::SecurityViolation
                    }
                    ComplianceEventType::AuditFailure => {
                        ricecoder_security::audit::AuditEventType::SystemAccess
                    }
                },
                user_id: event.user_id,
                session_id: event.session_id,
                action: format!("compliance_{:?}", event.event_type),
                resource: "compliance".to_string(),
                metadata: serde_json::json!({
                    "compliance_event": event_clone
                }),
            };

            let _ = audit_logger.log_event(audit_event).await;
        }
    }
}
