//! Session entity for AI interaction sessions with security features

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_objects::{ProjectId, SessionId};

use super::{
    AccessEvent, ConsentRecord, ConsentType, DataDeletionRequest, DataExport, DataMinimizationPolicy,
    DeletionReason, ExportFormat, GdprConsent, PerformanceMetric, PrivacyPolicy, SecurityAlert,
    SecurityContext, SecurityEvent,
};

/// Session entity representing an AI interaction session with security features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub project_id: Option<ProjectId>,
    pub name: Option<String>,
    pub provider_id: String,
    pub model_id: String,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub timeout_minutes: u32,
    pub security_context: SecurityContext,
    pub access_log: Vec<AccessEvent>,
    pub security_events: Vec<SecurityEvent>,
    pub security_alerts: Vec<SecurityAlert>,
    pub performance_metrics: Vec<PerformanceMetric>,
    pub gdpr_consents: Vec<GdprConsent>,
    pub data_minimization_policy: Option<DataMinimizationPolicy>,
    pub data_exports: Vec<DataExport>,
    pub deletion_requests: Vec<DataDeletionRequest>,
    pub privacy_policy: Option<PrivacyPolicy>,
    pub consent_records: Vec<ConsentRecord>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Session {
    /// Create a new session with security defaults
    pub fn new(provider_id: String, model_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            project_id: None,
            name: None,
            provider_id,
            model_id,
            status: SessionStatus::Active,
            created_at: now,
            updated_at: now,
            last_activity: now,
            timeout_minutes: 30, // Default 30 minutes
            security_context: SecurityContext::default(),
            access_log: Vec::new(),
            security_events: Vec::new(),
            security_alerts: Vec::new(),
            performance_metrics: Vec::new(),
            gdpr_consents: Vec::new(),
            data_minimization_policy: None,
            data_exports: Vec::new(),
            deletion_requests: Vec::new(),
            privacy_policy: None,
            consent_records: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Associate session with a project
    pub fn set_project(&mut self, project_id: ProjectId) {
        self.project_id = Some(project_id);
        self.updated_at = Utc::now();
    }

    /// Update session name
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
        self.updated_at = Utc::now();
    }

    /// End the session
    pub fn end(&mut self) {
        self.status = SessionStatus::Ended;
        self.updated_at = Utc::now();
    }

    /// Pause the session
    pub fn pause(&mut self) {
        self.status = SessionStatus::Paused;
        self.updated_at = Utc::now();
    }

    /// Resume the session
    pub fn resume(&mut self) {
        if self.status == SessionStatus::Paused {
            self.status = SessionStatus::Active;
            self.updated_at = Utc::now();
        }
    }

    /// Check if session is active and not timed out
    pub fn is_active(&self) -> bool {
        if self.status != SessionStatus::Active {
            return false;
        }
        let timeout_duration = chrono::Duration::minutes(self.timeout_minutes as i64);
        Utc::now() < self.last_activity + timeout_duration
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
        self.updated_at = Utc::now();
    }

    /// Log access event
    pub fn log_access(&mut self, event: AccessEvent) {
        self.access_log.push(event);
        self.update_activity();
    }

    /// Log security event for SOC 2 compliance
    pub fn log_security_event(&mut self, event: SecurityEvent) {
        self.security_events.push(event);
        self.update_activity();
    }

    /// Log security alert
    pub fn log_security_alert(&mut self, alert: SecurityAlert) {
        self.security_alerts.push(alert);
        self.update_activity();
    }

    /// Record performance metric
    pub fn record_performance_metric(&mut self, metric: PerformanceMetric) {
        self.performance_metrics.push(metric);
        self.update_activity();
    }

    /// Add GDPR consent
    pub fn add_gdpr_consent(&mut self, consent: GdprConsent) {
        self.gdpr_consents.push(consent);
        self.update_activity();
    }

    /// Check if has valid consent for purpose
    pub fn has_gdpr_consent(&self, consent_type: &ConsentType) -> bool {
        self.gdpr_consents
            .iter()
            .any(|c| c.consent_type == *consent_type && c.is_valid())
    }

    /// Set data minimization policy
    pub fn set_data_minimization_policy(&mut self, policy: DataMinimizationPolicy) {
        self.data_minimization_policy = Some(policy);
        self.update_activity();
    }

    /// Request data export for GDPR portability
    pub fn request_data_export(&mut self, user_id: String, format: ExportFormat) -> &mut DataExport {
        let export = DataExport::new(user_id, format);
        self.data_exports.push(export);
        self.data_exports.last_mut().unwrap()
    }

    /// Get exportable data as JSON
    pub fn export_data(&self) -> serde_json::Value {
        serde_json::json!({
            "session_id": self.id.to_string(),
            "project_id": self.project_id.map(|id| id.to_string()),
            "name": self.name,
            "provider_id": self.provider_id,
            "model_id": self.model_id,
            "status": self.status,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
            "last_activity": self.last_activity,
            "timeout_minutes": self.timeout_minutes,
            "access_log": self.access_log,
            "security_events": self.security_events,
            "performance_metrics": self.performance_metrics,
            "gdpr_consents": self.gdpr_consents,
            "metadata": self.metadata,
        })
    }

    /// Request data deletion for GDPR right to erasure
    pub fn request_data_deletion(
        &mut self,
        user_id: String,
        reason: DeletionReason,
        data_categories: Vec<String>,
    ) -> &mut DataDeletionRequest {
        let request = DataDeletionRequest::new(user_id, reason, data_categories);
        self.deletion_requests.push(request);
        self.deletion_requests.last_mut().unwrap()
    }

    /// Anonymize data for retention
    pub fn anonymize_data(&mut self) {
        // Remove personally identifiable information
        self.security_context.user_id = None;
        self.name = None;
        // Clear sensitive metadata
        self.metadata.clear();
        // Note: In real implementation, would hash or remove PII
    }

    /// Set privacy policy
    pub fn set_privacy_policy(&mut self, policy: PrivacyPolicy) {
        self.privacy_policy = Some(policy);
        self.update_activity();
    }

    /// Record user consent
    pub fn record_consent(
        &mut self,
        user_id: String,
        policy_version: String,
        consent_details: HashMap<String, bool>,
    ) -> &mut ConsentRecord {
        let record = ConsentRecord::new(user_id, policy_version, consent_details);
        self.consent_records.push(record);
        self.consent_records.last_mut().unwrap()
    }

    /// Check if user has consented to specific purpose
    pub fn has_consent(&self, user_id: &str, purpose: &str) -> bool {
        self.consent_records
            .iter()
            .filter(|r| r.user_id == user_id && r.is_valid())
            .any(|r| r.consent_details.get(purpose).copied().unwrap_or(false))
    }

    /// Check if session has timed out
    pub fn is_timed_out(&self) -> bool {
        let timeout_duration = chrono::Duration::minutes(self.timeout_minutes as i64);
        Utc::now() >= self.last_activity + timeout_duration
    }

    /// Force timeout the session
    pub fn timeout(&mut self) {
        self.status = SessionStatus::TimedOut;
        self.updated_at = Utc::now();
    }
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Paused,
    Ended,
    TimedOut,
}
