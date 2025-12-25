//! GDPR data protection and privacy entities

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub fn new(
        user_id: String,
        consent_type: ConsentType,
        purpose: String,
        data_categories: Vec<String>,
    ) -> Self {
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

/// Retention rule for data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionRule {
    pub max_age_days: u32,
    pub auto_delete: bool,
    pub justification: String,
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
        let severity = breach.severity;
        self.breach_incidents.push(breach);
        if severity == BreachSeverity::High || severity == BreachSeverity::Critical {
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
    pub fn new(
        description: String,
        affected_data_subjects: usize,
        data_categories_affected: Vec<String>,
        severity: BreachSeverity,
    ) -> Self {
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
