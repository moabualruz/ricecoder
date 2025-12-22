//! Core domain entities with business logic and validation

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{errors::*, value_objects::*};
use crate::value_objects::{UserRole, Permission};

/// Core project entity representing a code project being analyzed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub language: ProgrammingLanguage,
    pub root_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl Project {
    /// Create a new project with validation
    pub fn new(
        name: String,
        language: ProgrammingLanguage,
        root_path: String,
    ) -> DomainResult<Self> {
        Self::validate_name(&name)?;
        Self::validate_path(&root_path)?;

        let now = Utc::now();
        Ok(Self {
            id: ProjectId::new(),
            name,
            description: None,
            language,
            root_path,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        })
    }

    /// Update project name with validation
    pub fn update_name(&mut self, name: String) -> DomainResult<()> {
        Self::validate_name(&name)?;
        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update project description
    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Validate project name
    fn validate_name(name: &str) -> DomainResult<()> {
        if name.trim().is_empty() {
            return Err(DomainError::InvalidProjectName {
                reason: "Project name cannot be empty".to_string(),
            });
        }

        if name.len() > 100 {
            return Err(DomainError::InvalidProjectName {
                reason: "Project name cannot exceed 100 characters".to_string(),
            });
        }

        // Check for valid characters (alphanumeric, dash, underscore)
        if !regex::Regex::new(r"^[a-zA-Z0-9_-]+$")
            .unwrap()
            .is_match(name)
        {
            return Err(DomainError::InvalidProjectName {
                reason: "Project name can only contain letters, numbers, dashes, and underscores"
                    .to_string(),
            });
        }

        Ok(())
    }

    /// Validate project path
    fn validate_path(path: &str) -> DomainResult<()> {
        if path.trim().is_empty() {
            return Err(DomainError::InvalidFilePath {
                reason: "Project path cannot be empty".to_string(),
            });
        }

        // Basic path validation - could be enhanced
        if path.contains("..") {
            return Err(DomainError::InvalidFilePath {
                reason: "Project path cannot contain '..'".to_string(),
            });
        }

        Ok(())
    }
}

/// File entity representing a source code file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFile {
    pub id: FileId,
    pub project_id: ProjectId,
    pub relative_path: String,
    pub language: ProgrammingLanguage,
    pub content: String,
    pub size_bytes: usize,
    pub mime_type: MimeType,
    pub last_modified: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl CodeFile {
    /// Create a new code file
    pub fn new(
        project_id: ProjectId,
        relative_path: String,
        content: String,
        language: ProgrammingLanguage,
    ) -> DomainResult<Self> {
        let id = FileId::from_path(&relative_path);

        Ok(Self {
            id,
            project_id,
            relative_path: relative_path.clone(),
            language,
            content: content.clone(),
            size_bytes: content.len(),
            mime_type: MimeType::from_path(&relative_path),
            last_modified: Utc::now(),
            metadata: HashMap::new(),
        })
    }

    /// Update file content
    pub fn update_content(&mut self, content: String) {
        self.content = content.clone();
        self.size_bytes = content.len();
        self.last_modified = Utc::now();
    }

    /// Check if file is empty
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        std::path::Path::new(&self.relative_path)
            .extension()
            .and_then(|ext| ext.to_str())
    }
}

/// Analysis result entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub id: String,
    pub project_id: ProjectId,
    pub file_id: Option<FileId>,
    pub analysis_type: AnalysisType,
    pub status: AnalysisStatus,
    pub results: serde_json::Value,
    pub metrics: AnalysisMetrics,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl AnalysisResult {
    /// Create a new analysis result
    pub fn new(
        project_id: ProjectId,
        file_id: Option<FileId>,
        analysis_type: AnalysisType,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            file_id,
            analysis_type,
            status: AnalysisStatus::Pending,
            results: serde_json::Value::Null,
            metrics: AnalysisMetrics::default(),
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Mark analysis as completed
    pub fn complete(&mut self, results: serde_json::Value, metrics: AnalysisMetrics) {
        self.status = AnalysisStatus::Completed;
        self.results = results;
        self.metrics = metrics;
        self.completed_at = Some(Utc::now());
    }

    /// Mark analysis as failed
    pub fn fail(&mut self, error: String) {
        self.status = AnalysisStatus::Failed;
        self.results = serde_json::Value::String(error);
        self.completed_at = Some(Utc::now());
    }

    /// Check if analysis is complete
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            AnalysisStatus::Completed | AnalysisStatus::Failed
        )
    }
}

/// Analysis type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisType {
    Syntax,
    Semantic,
    Complexity,
    Dependencies,
    Patterns,
    Security,
    Performance,
}

/// Analysis status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Analysis metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    pub lines_of_code: usize,
    pub cyclomatic_complexity: f64,
    pub maintainability_index: f64,
    pub technical_debt_ratio: f64,
    pub execution_time_ms: u64,
}

impl Default for AnalysisMetrics {
    fn default() -> Self {
        Self {
            lines_of_code: 0,
            cyclomatic_complexity: 0.0,
            maintainability_index: 100.0,
            technical_debt_ratio: 0.0,
            execution_time_ms: 0,
        }
    }
}

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
        self.gdpr_consents.iter().any(|c| c.consent_type == *consent_type && c.is_valid())
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
    pub fn request_data_deletion(&mut self, user_id: String, reason: DeletionReason, data_categories: Vec<String>) -> &mut DataDeletionRequest {
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
    pub fn record_consent(&mut self, user_id: String, policy_version: String, consent_details: HashMap<String, bool>) -> &mut ConsentRecord {
        let record = ConsentRecord::new(user_id, policy_version, consent_details);
        self.consent_records.push(record);
        self.consent_records.last_mut().unwrap()
    }

    /// Check if user has consented to specific purpose
    pub fn has_consent(&self, user_id: &str, purpose: &str) -> bool {
        self.consent_records.iter()
            .filter(|r| r.user_id == user_id && r.is_valid())
            .any(|r| r.consent_details.get(purpose).copied().unwrap_or(false))
    }

/// GDPR compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprComplianceReport {
    pub id: String,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub compliance_status: GdprComplianceStatus,
    pub data_processing_inventory: Vec<DataProcessingActivity>,
    pub consent_compliance: ConsentComplianceSummary,
    pub breach_incidents: Vec<BreachIncident>,
    pub recommendations: Vec<String>,
}

impl GdprComplianceReport {
    /// Create a new GDPR compliance report
    pub fn new(period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            generated_at: Utc::now(),
            period_start,
            period_end,
            compliance_status: GdprComplianceStatus::Compliant,
            data_processing_inventory: Vec::new(),
            consent_compliance: ConsentComplianceSummary::default(),
            breach_incidents: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// Add data processing activity
    pub fn add_data_processing(&mut self, activity: DataProcessingActivity) {
        self.data_processing_inventory.push(activity);
    }

    /// Add breach incident
    pub fn add_breach(&mut self, breach: BreachIncident) {
        self.breach_incidents.push(breach);
        if breach.severity == BreachSeverity::High || breach.severity == BreachSeverity::Critical {
            self.compliance_status = GdprComplianceStatus::NonCompliant;
        }
    }
}

/// GDPR compliance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GdprComplianceStatus {
    Compliant,
    NonCompliant,
    UnderReview,
}

/// Data processing activity for GDPR inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProcessingActivity {
    pub id: String,
    pub purpose: String,
    pub data_categories: Vec<String>,
    pub legal_basis: String,
    pub data_subjects: Vec<String>,
    pub retention_period: u32,
    pub processors: Vec<String>,
    pub security_measures: Vec<String>,
}

/// Consent compliance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentComplianceSummary {
    pub total_consents: usize,
    pub valid_consents: usize,
    pub withdrawn_consents: usize,
    pub consent_by_purpose: HashMap<String, usize>,
}

impl Default for ConsentComplianceSummary {
    fn default() -> Self {
        Self {
            total_consents: 0,
            valid_consents: 0,
            withdrawn_consents: 0,
            consent_by_purpose: HashMap::new(),
        }
    }
}

/// Breach incident for GDPR reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachIncident {
    pub id: String,
    pub reported_at: DateTime<Utc>,
    pub discovered_at: DateTime<Utc>,
    pub description: String,
    pub affected_data_subjects: usize,
    pub data_categories_affected: Vec<String>,
    pub severity: BreachSeverity,
    pub containment_measures: Vec<String>,
    pub notification_sent: bool,
}

impl BreachIncident {
    /// Create a new breach incident
    pub fn new(description: String, affected_data_subjects: usize, data_categories_affected: Vec<String>, severity: BreachSeverity) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            reported_at: now,
            discovered_at: now,
            description,
            affected_data_subjects,
            data_categories_affected,
            severity,
            containment_measures: Vec::new(),
            notification_sent: false,
        }
    }
}

/// Breach severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreachSeverity {
    Low,
    Medium,
    High,
    Critical,
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

/// Security context for session isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub user_id: Option<String>,
    pub role: Option<UserRole>,
    pub permissions: Vec<Permission>,
    pub isolation_level: IsolationLevel,
    pub encryption_enabled: bool,
    pub confidentiality_level: ConfidentialityLevel,
}

impl SecurityContext {
    /// Check if the context has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| p.implies(permission))
    }

    /// Check if the context has a specific role
    pub fn has_role(&self, role: &UserRole) -> bool {
        self.role.as_ref() == Some(role)
    }

    /// Grant additional permissions
    pub fn grant_permission(&mut self, permission: Permission) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
    }

    /// Revoke a permission
    pub fn revoke_permission(&mut self, permission: &Permission) {
        self.permissions.retain(|p| p != permission);
    }

    /// Check if can access data of given confidentiality level
    pub fn can_access_confidentiality(&self, level: &ConfidentialityLevel) -> bool {
        match (self.role, level) {
            (Some(UserRole::Admin), _) => true,
            (_, ConfidentialityLevel::Public) => true,
            (Some(UserRole::Developer), ConfidentialityLevel::Internal) => true,
            (Some(UserRole::Developer), ConfidentialityLevel::Confidential) => true,
            (Some(UserRole::Analyst), ConfidentialityLevel::Internal) => true,
            _ => false,
        }
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            user_id: None,
            role: Some(UserRole::Guest),
            permissions: UserRole::Guest.default_permissions(),
            isolation_level: IsolationLevel::Standard,
            encryption_enabled: false,
            confidentiality_level: ConfidentialityLevel::Public,
        }
    }
}

/// Isolation level for micro-segmentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    Standard,
    High,
    Critical,
}

/// Confidentiality level for data protection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidentialityLevel {
    Public,
    Internal,
    Confidential,
    Restricted,
}

/// Access event for auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessEvent {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub resource: String,
    pub success: bool,
    pub details: Option<String>,
}

/// Security event for comprehensive SOC 2 audit trails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub subject: Option<SecuritySubject>,
    pub resource: Option<SecurityResource>,
    pub action: Option<String>,
    pub success: bool,
    pub details: HashMap<String, String>,
    pub compliance_flags: Vec<ComplianceFlag>,
}

impl SecurityEvent {
    /// Create a new security event
    pub fn new(event_type: SecurityEventType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            subject: None,
            resource: None,
            action: None,
            success: true,
            details: HashMap::new(),
            compliance_flags: Vec::new(),
        }
    }

    /// Set subject
    pub fn with_subject(mut self, subject: SecuritySubject) -> Self {
        self.subject = Some(subject);
        self
    }

    /// Set resource
    pub fn with_resource(mut self, resource: SecurityResource) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Set action
    pub fn with_action(mut self, action: String) -> Self {
        self.action = Some(action);
        self
    }

    /// Mark as failed
    pub fn failed(mut self) -> Self {
        self.success = false;
        self
    }

    /// Add detail
    pub fn add_detail(mut self, key: String, value: String) -> Self {
        self.details.insert(key, value);
        self
    }

    /// Add compliance flag
    pub fn add_compliance_flag(mut self, flag: ComplianceFlag) -> Self {
        self.compliance_flags.push(flag);
        self
    }
}

/// Security event types for SOC 2 compliance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityEventType {
    AccessGranted,
    AccessDenied,
    AuthenticationSuccess,
    AuthenticationFailure,
    AuthorizationSuccess,
    AuthorizationFailure,
    SessionCreated,
    SessionEnded,
    SessionTimeout,
    DataAccess,
    DataModification,
    SecurityViolation,
    ComplianceCheck,
}

/// Security subject (who performed the action)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySubject {
    pub id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Security resource (what was accessed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityResource {
    pub resource_type: String,
    pub resource_id: String,
    pub attributes: HashMap<String, String>,
}

/// Compliance flags for SOC 2 requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceFlag {
    Soc2AccessControl,
    Soc2AuditTrail,
    GdprDataAccess,
    HipaaProtectedData,
}

/// Security alert for monitoring and alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub alert_type: SecurityAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub details: HashMap<String, String>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

impl SecurityAlert {
    /// Create a new security alert
    pub fn new(alert_type: SecurityAlertType, severity: AlertSeverity, message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            alert_type,
            severity,
            message,
            details: HashMap::new(),
            resolved: false,
            resolved_at: None,
        }
    }

    /// Add detail
    pub fn add_detail(mut self, key: String, value: String) -> Self {
        self.details.insert(key, value);
        self
    }

    /// Mark as resolved
    pub fn resolve(mut self) -> Self {
        self.resolved = true;
        self.resolved_at = Some(Utc::now());
        self
    }
}

/// Types of security alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityAlertType {
    UnauthorizedAccess,
    SuspiciousActivity,
    ComplianceViolation,
    SecurityBreach,
    AuditFailure,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Performance metric for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub id: String,
    pub name: String,
    pub value: f64,
    pub unit: MetricUnit,
    pub timestamp: DateTime<Utc>,
    pub context: HashMap<String, String>,
}

impl PerformanceMetric {
    /// Create a new performance metric
    pub fn new(name: String, value: f64, unit: MetricUnit) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            value,
            unit,
            timestamp: Utc::now(),
            context: HashMap::new(),
        }
    }

    /// Add context
    pub fn add_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }
}

/// Units for performance metrics
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MetricUnit {
    Milliseconds,
    Seconds,
    Bytes,
    Kilobytes,
    Megabytes,
    Count,
    Percentage,
}

/// Performance benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmark {
    pub id: String,
    pub name: String,
    pub target_value: f64,
    pub tolerance: f64,
    pub unit: MetricUnit,
    pub description: String,
}

impl PerformanceBenchmark {
    /// Create a new benchmark
    pub fn new(name: String, target_value: f64, tolerance: f64, unit: MetricUnit, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            target_value,
            tolerance,
            unit,
            description,
        }
    }

    /// Check if a metric passes this benchmark
    pub fn check(&self, metric: &PerformanceMetric) -> BenchmarkResult {
        if metric.unit != self.unit {
            return BenchmarkResult::Error("Unit mismatch".to_string());
        }

        let deviation = (metric.value - self.target_value).abs() / self.target_value;
        if deviation <= self.tolerance {
            BenchmarkResult::Pass
        } else if deviation <= self.tolerance * 2.0 {
            BenchmarkResult::Warning
        } else {
            BenchmarkResult::Fail
        }
    }
}

/// Benchmark check result
#[derive(Debug, Clone, PartialEq)]
pub enum BenchmarkResult {
    Pass,
    Warning,
    Fail,
    Error(String),
}

/// SOC 2 compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: String,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub compliance_status: ComplianceStatus,
    pub findings: Vec<ComplianceFinding>,
    pub recommendations: Vec<String>,
    pub audit_trail_count: usize,
    pub alert_count: usize,
    pub benchmark_results: Vec<BenchmarkResult>,
}

impl ComplianceReport {
    /// Create a new compliance report
    pub fn new(period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            generated_at: Utc::now(),
            period_start,
            period_end,
            compliance_status: ComplianceStatus::Compliant,
            findings: Vec::new(),
            recommendations: Vec::new(),
            audit_trail_count: 0,
            alert_count: 0,
            benchmark_results: Vec::new(),
        }
    }

    /// Add a finding
    pub fn add_finding(&mut self, finding: ComplianceFinding) {
        self.findings.push(finding);
        if finding.severity == FindingSeverity::High || finding.severity == FindingSeverity::Critical {
            self.compliance_status = ComplianceStatus::NonCompliant;
        }
    }

    /// Add recommendation
    pub fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
    }
}

/// Compliance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    UnderReview,
}

/// Compliance finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: FindingSeverity,
    pub soc2_principle: Soc2Principle,
    pub remediation: String,
}

impl ComplianceFinding {
    /// Create a new finding
    pub fn new(title: String, description: String, severity: FindingSeverity, principle: Soc2Principle, remediation: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            description,
            severity,
            soc2_principle: principle,
            remediation,
        }
    }
}

/// Finding severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// SOC 2 principles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Soc2Principle {
    Security,
    Availability,
    ProcessingIntegrity,
    Confidentiality,
    Privacy,
}

/// GDPR consent for data processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprConsent {
    pub id: String,
    pub user_id: String,
    pub consent_given: bool,
    pub consent_date: DateTime<Utc>,
    pub consent_type: ConsentType,
    pub purpose: String,
    pub data_categories: Vec<String>,
    pub retention_period_days: Option<u32>,
    pub withdrawn: bool,
    pub withdrawn_date: Option<DateTime<Utc>>,
}

impl GdprConsent {
    /// Create a new consent record
    pub fn new(user_id: String, consent_type: ConsentType, purpose: String, data_categories: Vec<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            consent_given: false,
            consent_date: Utc::now(),
            consent_type,
            purpose,
            data_categories,
            retention_period_days: None,
            withdrawn: false,
            withdrawn_date: None,
        }
    }

    /// Give consent
    pub fn give_consent(&mut self) {
        self.consent_given = true;
        self.consent_date = Utc::now();
    }

    /// Withdraw consent
    pub fn withdraw_consent(&mut self) {
        self.withdrawn = true;
        self.withdrawn_date = Some(Utc::now());
    }

    /// Check if consent is valid
    pub fn is_valid(&self) -> bool {
        self.consent_given && !self.withdrawn
    }
}

/// Types of consent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsentType {
    Essential,
    Analytics,
    Marketing,
    ThirdParty,
}

/// Data minimization policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMinimizationPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub data_fields: Vec<String>,
    pub retention_rules: HashMap<String, RetentionRule>,
    pub anonymization_required: bool,
}

impl DataMinimizationPolicy {
    /// Create a new policy
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            data_fields: Vec::new(),
            retention_rules: HashMap::new(),
            anonymization_required: false,
        }
    }

    /// Add data field
    pub fn add_data_field(&mut self, field: String) {
        self.data_fields.push(field);
    }

    /// Add retention rule
    pub fn add_retention_rule(&mut self, field: String, rule: RetentionRule) {
        self.retention_rules.insert(field, rule);
    }
}

/// Data export for GDPR portability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExport {
    pub id: String,
    pub user_id: String,
    pub requested_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub format: ExportFormat,
    pub data: serde_json::Value,
    pub status: ExportStatus,
}

impl DataExport {
    /// Create a new data export request
    pub fn new(user_id: String, format: ExportFormat) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            requested_at: Utc::now(),
            completed_at: None,
            format,
            data: serde_json::Value::Null,
            status: ExportStatus::Pending,
        }
    }

    /// Complete the export
    pub fn complete(&mut self, data: serde_json::Value) {
        self.data = data;
        self.completed_at = Some(Utc::now());
        self.status = ExportStatus::Completed;
    }

    /// Fail the export
    pub fn fail(&mut self) {
        self.status = ExportStatus::Failed;
        self.completed_at = Some(Utc::now());
    }
}

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Xml,
    Csv,
}

/// Export status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Data deletion request for GDPR right to erasure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDeletionRequest {
    pub id: String,
    pub user_id: String,
    pub requested_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub reason: DeletionReason,
    pub status: DeletionStatus,
    pub data_categories: Vec<String>,
    pub retention_override: bool,
}

impl DataDeletionRequest {
    /// Create a new deletion request
    pub fn new(user_id: String, reason: DeletionReason, data_categories: Vec<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            requested_at: Utc::now(),
            completed_at: None,
            reason,
            status: DeletionStatus::Pending,
            data_categories,
            retention_override: false,
        }
    }

    /// Complete the deletion
    pub fn complete(&mut self) {
        self.status = DeletionStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// Reject the deletion
    pub fn reject(&mut self) {
        self.status = DeletionStatus::Rejected;
        self.completed_at = Some(Utc::now());
    }
}

/// Reason for data deletion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionReason {
    UserRequest,
    ConsentWithdrawn,
    LegalObligation,
    DataNoLongerNeeded,
    Other,
}

/// Deletion status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionStatus {
    Pending,
    Processing,
    Completed,
    Rejected,
}

/// Privacy policy for GDPR compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPolicy {
    pub id: String,
    pub version: String,
    pub effective_date: DateTime<Utc>,
    pub content: String,
    pub data_processing_purposes: Vec<String>,
    pub legal_basis: Vec<String>,
    pub retention_periods: HashMap<String, u32>, // data category -> days
    pub contact_information: String,
    pub last_updated: DateTime<Utc>,
}

impl PrivacyPolicy {
    /// Create a new privacy policy
    pub fn new(version: String, content: String, contact_information: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            version,
            effective_date: now,
            content,
            data_processing_purposes: Vec::new(),
            legal_basis: Vec::new(),
            retention_periods: HashMap::new(),
            contact_information,
            last_updated: now,
        }
    }

    /// Add data processing purpose
    pub fn add_purpose(&mut self, purpose: String) {
        self.data_processing_purposes.push(purpose);
        self.last_updated = Utc::now();
    }

    /// Add legal basis
    pub fn add_legal_basis(&mut self, basis: String) {
        self.legal_basis.push(basis);
        self.last_updated = Utc::now();
    }

    /// Set retention period for data category
    pub fn set_retention_period(&mut self, category: String, days: u32) {
        self.retention_periods.insert(category, days);
        self.last_updated = Utc::now();
    }
}

/// Consent tracking record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub id: String,
    pub user_id: String,
    pub policy_version: String,
    pub consented_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub consent_details: HashMap<String, bool>, // purpose -> consented
    pub withdrawn_at: Option<DateTime<Utc>>,
}

impl ConsentRecord {
    /// Create a new consent record
    pub fn new(user_id: String, policy_version: String, consent_details: HashMap<String, bool>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            policy_version,
            consented_at: Utc::now(),
            ip_address: None,
            user_agent: None,
            consent_details,
            withdrawn_at: None,
        }
    }

    /// Withdraw consent
    pub fn withdraw(&mut self) {
        self.withdrawn_at = Some(Utc::now());
    }

    /// Check if consent is valid
    pub fn is_valid(&self) -> bool {
        self.withdrawn_at.is_none()
    }
}

/// User entity representing a system user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl User {
    /// Create a new user
    pub fn new(id: String, username: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            username,
            email: None,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Update username
    pub fn update_username(&mut self, username: String) {
        self.username = username;
        self.updated_at = Utc::now();
    }

    /// Set email
    pub fn set_email(&mut self, email: Option<String>) {
        self.email = email;
        self.updated_at = Utc::now();
    }
}

/// Provider configuration entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub base_url: Option<String>,
    pub models: Vec<ModelInfo>,
    pub config: HashMap<String, serde_json::Value>,
    pub is_active: bool,
}

impl Provider {
    /// Create a new provider
    pub fn new(id: String, name: String, provider_type: ProviderType) -> Self {
        Self {
            id,
            name,
            provider_type,
            base_url: None,
            models: Vec::new(),
            config: HashMap::new(),
            is_active: true,
        }
    }

    /// Add a model to the provider
    pub fn add_model(&mut self, model: ModelInfo) {
        self.models.push(model);
    }

    /// Get a model by ID
    pub fn get_model(&self, model_id: &str) -> Option<&ModelInfo> {
        self.models.iter().find(|m| m.id == model_id)
    }

    /// Check if provider supports a model
    pub fn supports_model(&self, model_id: &str) -> bool {
        self.models.iter().any(|m| m.id == model_id)
    }
}

/// Provider type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Google,
    Local,
    Custom,
}

/// Model information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: usize,
    pub supports_function_calling: bool,
    pub supports_vision: bool,
    pub cost_per_1m_input: Option<f64>,
    pub cost_per_1m_output: Option<f64>,
}

impl ModelInfo {
    /// Create a new model info
    pub fn new(id: String, name: String, context_window: usize) -> Self {
        Self {
            id,
            name,
            context_window,
            supports_function_calling: false,
            supports_vision: false,
            cost_per_1m_input: None,
            cost_per_1m_output: None,
        }
    }

    /// Enable function calling
    pub fn with_function_calling(mut self) -> Self {
        self.supports_function_calling = true;
        self
    }

    /// Enable vision
    pub fn with_vision(mut self) -> Self {
        self.supports_vision = true;
        self
    }

    /// Set pricing
    pub fn with_pricing(mut self, input_cost: f64, output_cost: f64) -> Self {
        self.cost_per_1m_input = Some(input_cost);
        self.cost_per_1m_output = Some(output_cost);
        self
    }
}
