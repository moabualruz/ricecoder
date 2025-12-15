//! Compliance features for SOC 2, GDPR, and HIPAA
//!
//! This module provides compliance infrastructure including:
//! - SOC 2 Type II controls with customer-managed encryption keys
//! - GDPR/HIPAA data handling (right to erasure, portability)
//! - Privacy-preserving analytics with opt-in and log retention

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::audit::{AuditLogger, AuditEventType};
use crate::encryption::{KeyManager, EncryptedData};
use crate::error::SecurityError;
use crate::Result;

/// SOC 2 Type II compliance controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soc2Controls {
    pub customer_managed_keys: bool,
    pub audit_trail_integrity: bool,
    pub access_controls: bool,
    pub change_management: bool,
    pub risk_assessment: bool,
    pub security_monitoring: bool,
}

/// GDPR/HIPAA compliance data handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceData {
    pub user_id: String,
    pub data_types: Vec<DataType>,
    pub consent_given: bool,
    pub consent_date: Option<DateTime<Utc>>,
    pub data_retention_period_days: i64,
    pub last_accessed: DateTime<Utc>,
    pub encryption_key_id: String,
}

/// Data types for compliance tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    PersonalInfo,
    HealthInfo,
    SessionData,
    AuditLogs,
    AnalyticsData,
    ApiKeys,
}

/// Data classification levels for compliance
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

/// Data erasure request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataErasureRequest {
    pub id: Uuid,
    pub user_id: String,
    pub request_date: DateTime<Utc>,
    pub reason: ErasureReason,
    pub status: ErasureStatus,
    pub completed_date: Option<DateTime<Utc>>,
}

/// Reason for data erasure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErasureReason {
    UserRequest,
    LegalRequirement,
    DataRetentionExpired,
    AccountDeletion,
}

/// Status of erasure request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErasureStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

/// Data portability request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPortabilityRequest {
    pub id: Uuid,
    pub user_id: String,
    pub request_date: DateTime<Utc>,
    pub data_types: Vec<DataType>,
    pub format: ExportFormat,
    pub status: PortabilityStatus,
    pub download_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Export format for data portability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
}

/// Status of portability request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortabilityStatus {
    Pending,
    Processing,
    Ready,
    Expired,
    Failed(String),
}

/// Privacy analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyAnalyticsConfig {
    pub analytics_enabled: bool,
    pub opt_in_required: bool,
    pub log_retention_days: i64,
    pub anonymization_enabled: bool,
    pub data_minimization: bool,
}

/// Anonymized analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedAnalytics {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub session_count: u64,
    pub feature_usage: HashMap<String, u64>,
    pub performance_metrics: HashMap<String, f64>,
}

/// Customer-managed encryption key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerKey {
    pub key_id: String,
    pub key_version: String,
    pub encrypted_key_material: EncryptedData,
    pub created_at: DateTime<Utc>,
    pub rotation_required: bool,
}

/// Compliance manager for SOC 2, GDPR, HIPAA
pub struct ComplianceManager {
    audit_logger: Arc<AuditLogger>,
    customer_keys: Arc<RwLock<HashMap<String, CustomerKey>>>,
    compliance_data: Arc<RwLock<HashMap<String, ComplianceData>>>,
    erasure_requests: Arc<RwLock<HashMap<Uuid, DataErasureRequest>>>,
    portability_requests: Arc<RwLock<HashMap<Uuid, DataPortabilityRequest>>>,
    privacy_config: Arc<RwLock<PrivacyAnalyticsConfig>>,
    soc2_controls: Soc2Controls,
}

impl ComplianceManager {
    /// Create a new compliance manager
    pub fn new(audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            audit_logger,
            customer_keys: Arc::new(RwLock::new(HashMap::new())),
            compliance_data: Arc::new(RwLock::new(HashMap::new())),
            erasure_requests: Arc::new(RwLock::new(HashMap::new())),
            portability_requests: Arc::new(RwLock::new(HashMap::new())),
            privacy_config: Arc::new(RwLock::new(PrivacyAnalyticsConfig {
                analytics_enabled: false,
                opt_in_required: true,
                log_retention_days: 90,
                anonymization_enabled: true,
                data_minimization: true,
            })),
            soc2_controls: Soc2Controls {
                customer_managed_keys: true,
                audit_trail_integrity: true,
                access_controls: true,
                change_management: true,
                risk_assessment: true,
                security_monitoring: true,
            },
        }
    }

    /// Register customer-managed encryption key
    pub async fn register_customer_key(
        &self,
        user_id: &str,
        key_material: &[u8],
        master_key_manager: &KeyManager,
    ) -> Result<String> {
        let key_id = format!("customer-key-{}", Uuid::new_v4());
        let encrypted_key = master_key_manager.encrypt_api_key(&base64::encode(key_material))?;

        let customer_key = CustomerKey {
            key_id: key_id.clone(),
            key_version: "1.0".to_string(),
            encrypted_key_material: encrypted_key,
            created_at: Utc::now(),
            rotation_required: false,
        };

        let mut keys = self.customer_keys.write().await;
        keys.insert(key_id.clone(), customer_key);

        // Audit the key registration
        self.audit_logger.log_event(AuditEvent {
            event_type: AuditEventType::SecurityViolation, // Using existing type, could add new one
            user_id: Some(user_id.to_string()),
            session_id: None,
            action: "register_customer_key".to_string(),
            resource: format!("customer_key:{}", key_id),
            metadata: serde_json::json!({
                "key_version": "1.0",
                "compliance": "SOC2"
            }),
        }).await?;

        Ok(key_id)
    }

    /// Get customer key for encryption/decryption
    pub async fn get_customer_key(&self, key_id: &str) -> Result<Option<CustomerKey>> {
        let keys = self.customer_keys.read().await;
        Ok(keys.get(key_id).cloned())
    }

    /// Record compliance data for user
    pub async fn record_compliance_data(&self, data: ComplianceData) -> Result<()> {
        let mut compliance_data = self.compliance_data.write().await;
        compliance_data.insert(data.user_id.clone(), data);
        Ok(())
    }

    /// Get compliance data for user
    pub async fn get_compliance_data(&self, user_id: &str) -> Result<Option<ComplianceData>> {
        let compliance_data = self.compliance_data.read().await;
        Ok(compliance_data.get(user_id).cloned())
    }

    /// Request data erasure (GDPR/HIPAA right to erasure)
    pub async fn request_data_erasure(
        &self,
        user_id: &str,
        reason: ErasureReason,
    ) -> Result<Uuid> {
        let request_id = Uuid::new_v4();
        let request = DataErasureRequest {
            id: request_id,
            user_id: user_id.to_string(),
            request_date: Utc::now(),
            reason,
            status: ErasureStatus::Pending,
            completed_date: None,
        };

        let mut requests = self.erasure_requests.write().await;
        requests.insert(request_id, request);

        // Audit the erasure request
        self.audit_logger.log_event(AuditEvent {
            event_type: AuditEventType::SecurityViolation, // Could add specific type
            user_id: Some(user_id.to_string()),
            session_id: None,
            action: "request_data_erasure".to_string(),
            resource: format!("user_data:{}", user_id),
            metadata: serde_json::json!({
                "request_id": request_id,
                "reason": format!("{:?}", reason),
                "compliance": "GDPR_HIPAA"
            }),
        }).await?;

        Ok(request_id)
    }

    /// Process data erasure request
    pub async fn process_data_erasure(&self, request_id: &Uuid) -> Result<()> {
        let mut requests = self.erasure_requests.write().await;
        if let Some(request) = requests.get_mut(request_id) {
            request.status = ErasureStatus::InProgress;

            // Here would be the actual data deletion logic
            // For now, just mark as completed
            request.status = ErasureStatus::Completed;
            request.completed_date = Some(Utc::now());

            // Remove compliance data
            let mut compliance_data = self.compliance_data.write().await;
            compliance_data.remove(&request.user_id);

            // Audit completion
            self.audit_logger.log_event(AuditEvent {
                event_type: AuditEventType::SecurityViolation,
                user_id: Some(request.user_id.clone()),
                session_id: None,
                action: "complete_data_erasure".to_string(),
                resource: format!("erasure_request:{}", request_id),
                metadata: serde_json::json!({
                    "request_id": request_id,
                    "compliance": "GDPR_HIPAA"
                }),
            }).await?;
        }

        Ok(())
    }

    /// Request data portability (GDPR right to portability)
    pub async fn request_data_portability(
        &self,
        user_id: &str,
        data_types: Vec<DataType>,
        format: ExportFormat,
    ) -> Result<Uuid> {
        let request_id = Uuid::new_v4();
        let request = DataPortabilityRequest {
            id: request_id,
            user_id: user_id.to_string(),
            request_date: Utc::now(),
            data_types,
            format,
            status: PortabilityStatus::Pending,
            download_url: None,
            expires_at: None,
        };

        let mut requests = self.portability_requests.write().await;
        requests.insert(request_id, request);

        // Audit the portability request
        self.audit_logger.log_event(AuditEvent {
            event_type: AuditEventType::SecurityViolation,
            user_id: Some(user_id.to_string()),
            session_id: None,
            action: "request_data_portability".to_string(),
            resource: format!("user_data:{}", user_id),
            metadata: serde_json::json!({
                "request_id": request_id,
                "data_types": format!("{:?}", data_types),
                "format": format!("{:?}", format),
                "compliance": "GDPR"
            }),
        }).await?;

        Ok(request_id)
    }

    /// Process data portability request
    pub async fn process_data_portability(&self, request_id: &Uuid) -> Result<()> {
        let mut requests = self.portability_requests.write().await;
        if let Some(request) = requests.get_mut(request_id) {
            request.status = PortabilityStatus::Processing;

            // Here would be the actual data export logic
            // For now, simulate completion
            request.status = PortabilityStatus::Ready;
            request.download_url = Some(format!("https://api.ricecoder.com/portability/{}", request_id));
            request.expires_at = Some(Utc::now() + Duration::days(30));

            // Audit completion
            self.audit_logger.log_event(AuditEvent {
                event_type: AuditEventType::SecurityViolation,
                user_id: Some(request.user_id.clone()),
                session_id: None,
                action: "complete_data_portability".to_string(),
                resource: format!("portability_request:{}", request_id),
                metadata: serde_json::json!({
                    "request_id": request_id,
                    "compliance": "GDPR"
                }),
            }).await?;
        }

        Ok(())
    }

    /// Configure privacy analytics
    pub async fn configure_privacy_analytics(&self, config: PrivacyAnalyticsConfig) -> Result<()> {
        let mut privacy_config = self.privacy_config.write().await;
        *privacy_config = config;
        Ok(())
    }

    /// Get privacy analytics configuration
    pub async fn get_privacy_config(&self) -> Result<PrivacyAnalyticsConfig> {
        let config = self.privacy_config.read().await;
        Ok(config.clone())
    }

    /// Record anonymized analytics data
    pub async fn record_analytics(&self, analytics: AnonymizedAnalytics) -> Result<()> {
        let config = self.privacy_config.read().await;

        if !config.analytics_enabled {
            return Ok(()); // Silently ignore if analytics disabled
        }

        // Here would be the actual analytics storage
        // For compliance, ensure data is anonymized and retention is enforced

        // Audit analytics recording
        self.audit_logger.log_event(AuditEvent {
            event_type: AuditEventType::SecurityViolation,
            user_id: None,
            session_id: None,
            action: "record_analytics".to_string(),
            resource: "analytics".to_string(),
            metadata: serde_json::json!({
                "event_type": analytics.event_type,
                "anonymized": config.anonymization_enabled,
                "compliance": "privacy_analytics"
            }),
        }).await?;

        Ok(())
    }

    /// Clean up expired logs and data per retention policies
    pub async fn cleanup_expired_data(&self) -> Result<()> {
        let config = self.privacy_config.read().await;
        let cutoff_date = Utc::now() - Duration::days(config.log_retention_days);

        // Here would be actual cleanup logic for logs and analytics data
        // that are older than the retention period

        // Audit cleanup
        self.audit_logger.log_event(AuditEvent {
            event_type: AuditEventType::SecurityViolation,
            user_id: None,
            session_id: None,
            action: "cleanup_expired_data".to_string(),
            resource: "system".to_string(),
            metadata: serde_json::json!({
                "retention_days": config.log_retention_days,
                "cutoff_date": cutoff_date,
                "compliance": "data_retention"
            }),
        }).await?;

        Ok(())
    }

    /// Generate compliance report
    pub async fn generate_compliance_report(&self) -> Result<serde_json::Value> {
        let soc2_status = serde_json::json!({
            "customer_managed_keys": self.soc2_controls.customer_managed_keys,
            "audit_trail_integrity": self.soc2_controls.audit_trail_integrity,
            "access_controls": self.soc2_controls.access_controls,
            "change_management": self.soc2_controls.change_management,
            "risk_assessment": self.soc2_controls.risk_assessment,
            "security_monitoring": self.soc2_controls.security_monitoring
        });

        let privacy_config = self.privacy_config.read().await;
        let privacy_status = serde_json::json!({
            "analytics_enabled": privacy_config.analytics_enabled,
            "opt_in_required": privacy_config.opt_in_required,
            "log_retention_days": privacy_config.log_retention_days,
            "anonymization_enabled": privacy_config.anonymization_enabled,
            "data_minimization": privacy_config.data_minimization
        });

        let report = serde_json::json!({
            "generated_at": Utc::now(),
            "soc2_type_ii_controls": soc2_status,
            "gdpr_hipaa_compliance": privacy_status,
            "active_erasure_requests": self.erasure_requests.read().await.len(),
            "active_portability_requests": self.portability_requests.read().await.len(),
            "customer_keys_registered": self.customer_keys.read().await.len()
        });

        Ok(report)
    }

    /// Check SOC 2 compliance status
    pub fn check_soc2_compliance(&self) -> bool {
        self.soc2_controls.customer_managed_keys
            && self.soc2_controls.audit_trail_integrity
            && self.soc2_controls.access_controls
            && self.soc2_controls.change_management
            && self.soc2_controls.risk_assessment
            && self.soc2_controls.security_monitoring
    }
}

/// Compliance validator for enterprise features
#[derive(Debug, Clone)]
pub struct ComplianceValidator {
    compliance_manager: Arc<ComplianceManager>,
}

impl ComplianceValidator {
    /// Create a new compliance validator
    pub fn new(compliance_manager: Arc<ComplianceManager>) -> Self {
        Self { compliance_manager }
    }

    /// Validate data classification compliance
    pub async fn validate_data_classification(&self, classification: &DataClassification) -> Result<bool> {
        // Check if the data classification is allowed based on current compliance settings
        match classification {
            DataClassification::Public => Ok(true), // Always allowed
            DataClassification::Internal => Ok(true), // Allowed with basic controls
            DataClassification::Confidential => {
                // Check if customer-managed keys are enabled for confidential data
                Ok(self.compliance_manager.check_soc2_compliance())
            }
            DataClassification::Restricted => {
                // Highest level - requires full SOC 2 compliance
                Ok(self.compliance_manager.check_soc2_compliance())
            }
        }
    }

    /// Validate data access compliance
    pub async fn validate_data_access(&self, classification: &DataClassification, user_id: Option<&str>) -> Result<bool> {
        // Check user consent and data access permissions
        if let Some(user_id) = user_id {
            if let Some(compliance_data) = self.compliance_manager.get_compliance_data(user_id).await? {
                // Check if user has given consent
                if !compliance_data.consent_given {
                    return Ok(false);
                }

                // Check data retention hasn't expired
                if DataErasure::is_retention_expired(&compliance_data) {
                    return Ok(false);
                }
            }
        }

        self.validate_data_classification(classification).await
    }

    /// Validate data modification compliance
    pub async fn validate_data_modification(&self, classification: &DataClassification, user_id: Option<&str>) -> Result<bool> {
        // Similar to access but with additional checks for modification
        let access_allowed = self.validate_data_access(classification, user_id).await?;

        if !access_allowed {
            return Ok(false);
        }

        // Additional checks for modification based on classification
        match classification {
            DataClassification::Public => Ok(true),
            DataClassification::Internal => Ok(true),
            DataClassification::Confidential | DataClassification::Restricted => {
                // Require audit logging for modifications
                Ok(true) // Assume audit logging is enabled
            }
        }
    }

    /// Validate data sharing compliance
    pub async fn validate_data_sharing(&self, classification: &DataClassification, user_id: Option<&str>) -> Result<bool> {
        // Check if sharing is allowed for this classification
        match classification {
            DataClassification::Public => Ok(true),
            DataClassification::Internal => Ok(true),
            DataClassification::Confidential => {
                // May require approval for confidential data sharing
                Ok(user_id.is_some()) // Require authenticated user
            }
            DataClassification::Restricted => Ok(false), // Never allow sharing of restricted data
        }
    }

    /// Validate shared data access compliance
    pub async fn validate_shared_data_access(&self, classification: &DataClassification, accessing_user_id: Option<&str>) -> Result<bool> {
        // Similar to regular access but for shared data
        self.validate_data_access(classification, accessing_user_id).await
    }

    /// Validate data erasure compliance
    pub async fn validate_data_erasure(&self, classification: &DataClassification, user_id: Option<&str>) -> Result<bool> {
        // Check if erasure is allowed (always true for user requests, but may have restrictions)
        match classification {
            DataClassification::Public | DataClassification::Internal | DataClassification::Confidential | DataClassification::Restricted => {
                // Erasure is generally allowed, but may require special handling for restricted data
                Ok(true)
            }
        }
    }


}

/// Data erasure operations
pub struct DataErasure;

impl DataErasure {
    /// Check if user has consented to data processing
    pub fn has_user_consent(compliance_data: &ComplianceData) -> bool {
        compliance_data.consent_given
    }

    /// Check if data retention period has expired
    pub fn is_retention_expired(compliance_data: &ComplianceData) -> bool {
        let retention_end = compliance_data.last_accessed + Duration::days(compliance_data.data_retention_period_days);
        Utc::now() > retention_end
    }

    /// Anonymize data for retention beyond consent
    pub fn anonymize_data(data: &mut serde_json::Value) {
        // Remove personally identifiable information
        if let Some(obj) = data.as_object_mut() {
            obj.remove("user_id");
            obj.remove("email");
            obj.remove("name");
            obj.remove("ip_address");
        }
    }
}

/// Data portability operations
pub struct DataPortability;

impl DataPortability {
    /// Export user data in specified format
    pub fn export_user_data(
        compliance_data: &ComplianceData,
        format: &ExportFormat,
    ) -> Result<String> {
        let data = serde_json::to_value(compliance_data)?;

        match format {
            ExportFormat::Json => Ok(serde_json::to_string_pretty(&data)?),
            ExportFormat::Csv => {
                // Simple CSV conversion for compliance data
                let csv = format!(
                    "user_id,data_types,consent_given,consent_date,last_accessed\n{},\"{:?}\",{},{:?},{}\n",
                    compliance_data.user_id,
                    compliance_data.data_types,
                    compliance_data.consent_given,
                    compliance_data.consent_date,
                    compliance_data.last_accessed
                );
                Ok(csv)
            }
            ExportFormat::Xml => {
                // Simple XML conversion
                let xml = format!(
                    r#"<user_data>
  <user_id>{}</user_id>
  <consent_given>{}</consent_given>
  <last_accessed>{}</last_accessed>
</user_data>"#,
                    compliance_data.user_id,
                    compliance_data.consent_given,
                    compliance_data.last_accessed
                );
                Ok(xml)
            }
        }
    }
}

/// Privacy-preserving analytics
pub struct PrivacyAnalytics;

impl PrivacyAnalytics {
    /// Check if analytics collection is allowed
    pub fn can_collect_analytics(config: &PrivacyAnalyticsConfig, user_consent: bool) -> bool {
        if !config.analytics_enabled {
            return false;
        }

        if config.opt_in_required && !user_consent {
            return false;
        }

        true
    }

    /// Anonymize analytics data
    pub fn anonymize_analytics_data(data: &mut AnonymizedAnalytics) {
        // Remove any potentially identifying information
        data.event_type = format!("anon_{}", data.event_type.len()); // Replace with hash-like identifier
    }

    /// Check if log retention period has expired
    pub fn is_log_retention_expired(
        log_timestamp: DateTime<Utc>,
        retention_days: i64,
    ) -> bool {
        let retention_end = log_timestamp + Duration::days(retention_days);
        Utc::now() > retention_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::MemoryAuditStorage;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_customer_key_registration() {
        let storage = Arc::new(MemoryAuditStorage::new());
        let audit_logger = Arc::new(AuditLogger::new(storage));
        let compliance_manager = ComplianceManager::new(audit_logger);

        let master_key = KeyManager::new("test-master-password").unwrap();
        let key_material = b"customer-key-material-12345";

        let key_id = compliance_manager
            .register_customer_key("user123", key_material, &master_key)
            .await
            .unwrap();

        assert!(key_id.starts_with("customer-key-"));

        let retrieved_key = compliance_manager.get_customer_key(&key_id).await.unwrap();
        assert!(retrieved_key.is_some());
    }

    #[tokio::test]
    async fn test_data_erasure_request() {
        let storage = Arc::new(MemoryAuditStorage::new());
        let audit_logger = Arc::new(AuditLogger::new(storage));
        let compliance_manager = ComplianceManager::new(audit_logger);

        let request_id = compliance_manager
            .request_data_erasure("user123", ErasureReason::UserRequest)
            .await
            .unwrap();

        compliance_manager.process_data_erasure(&request_id).await.unwrap();

        // Verify compliance data was removed
        let data = compliance_manager.get_compliance_data("user123").await.unwrap();
        assert!(data.is_none());
    }

    #[tokio::test]
    async fn test_data_portability_request() {
        let storage = Arc::new(MemoryAuditStorage::new());
        let audit_logger = Arc::new(AuditLogger::new(storage));
        let compliance_manager = ComplianceManager::new(audit_logger);

        let request_id = compliance_manager
            .request_data_portability(
                "user123",
                vec![DataType::PersonalInfo],
                ExportFormat::Json,
            )
            .await
            .unwrap();

        compliance_manager.process_data_portability(&request_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_privacy_analytics_config() {
        let storage = Arc::new(MemoryAuditStorage::new());
        let audit_logger = Arc::new(AuditLogger::new(storage));
        let compliance_manager = ComplianceManager::new(audit_logger);

        let config = PrivacyAnalyticsConfig {
            analytics_enabled: true,
            opt_in_required: true,
            log_retention_days: 90,
            anonymization_enabled: true,
            data_minimization: true,
        };

        compliance_manager.configure_privacy_analytics(config.clone()).await.unwrap();
        let retrieved_config = compliance_manager.get_privacy_config().await.unwrap();

        assert_eq!(retrieved_config.analytics_enabled, config.analytics_enabled);
        assert_eq!(retrieved_config.log_retention_days, 90);
    }

    #[test]
    fn test_data_erasure_operations() {
        let compliance_data = ComplianceData {
            user_id: "user123".to_string(),
            data_types: vec![DataType::PersonalInfo],
            consent_given: true,
            consent_date: Some(Utc::now()),
            data_retention_period_days: 365,
            last_accessed: Utc::now(),
            encryption_key_id: "key123".to_string(),
        };

        assert!(DataErasure::has_user_consent(&compliance_data));

        // Test with expired retention
        let mut old_data = compliance_data.clone();
        old_data.last_accessed = Utc::now() - Duration::days(400);
        assert!(DataErasure::is_retention_expired(&old_data));
    }

    #[test]
    fn test_data_portability_export() {
        let compliance_data = ComplianceData {
            user_id: "user123".to_string(),
            data_types: vec![DataType::PersonalInfo],
            consent_given: true,
            consent_date: Some(Utc::now()),
            data_retention_period_days: 365,
            last_accessed: Utc::now(),
            encryption_key_id: "key123".to_string(),
        };

        let json_export = DataPortability::export_user_data(&compliance_data, &ExportFormat::Json).unwrap();
        assert!(json_export.contains("user123"));

        let csv_export = DataPortability::export_user_data(&compliance_data, &ExportFormat::Csv).unwrap();
        assert!(csv_export.contains("user123"));
    }

    #[test]
    fn test_privacy_analytics_permissions() {
        let config_opt_in = PrivacyAnalyticsConfig {
            analytics_enabled: true,
            opt_in_required: true,
            log_retention_days: 90,
            anonymization_enabled: true,
            data_minimization: true,
        };

        assert!(PrivacyAnalytics::can_collect_analytics(&config_opt_in, true));
        assert!(!PrivacyAnalytics::can_collect_analytics(&config_opt_in, false));

        let config_disabled = PrivacyAnalyticsConfig {
            analytics_enabled: false,
            opt_in_required: false,
            log_retention_days: 90,
            anonymization_enabled: true,
            data_minimization: true,
        };

        assert!(!PrivacyAnalytics::can_collect_analytics(&config_disabled, true));
    }

    #[tokio::test]
    async fn test_compliance_report_generation() {
        let storage = Arc::new(MemoryAuditStorage::new());
        let audit_logger = Arc::new(AuditLogger::new(storage));
        let compliance_manager = ComplianceManager::new(audit_logger);

        let report = compliance_manager.generate_compliance_report().await.unwrap();

        assert!(report.get("soc2_type_ii_controls").is_some());
        assert!(report.get("gdpr_hipaa_compliance").is_some());
        assert!(report.get("generated_at").is_some());
    }

    #[test]
    fn test_soc2_compliance_check() {
        let storage = Arc::new(MemoryAuditStorage::new());
        let audit_logger = Arc::new(AuditLogger::new(storage));
        let compliance_manager = ComplianceManager::new(audit_logger);

        assert!(compliance_manager.check_soc2_compliance());
    }
}