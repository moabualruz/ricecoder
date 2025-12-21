//! # RiceCoder Security
#![forbid(unsafe_code)]

//!
//! Security utilities and cryptographic operations for RiceCoder.
//!
//! This crate provides:
//! - API key encryption and secure storage
//! - Input validation and sanitization
//! - Authentication helpers
//! - Audit logging system
//! - Access control and permission management
//! - Compliance features (SOC 2, GDPR, HIPAA)

use std::sync::Arc;

pub mod access_control;
pub mod audit;
pub mod compliance;
pub mod encryption;
pub mod error;
pub mod monitoring;
pub mod oauth;
pub mod penetration_testing;
pub mod reporting;
pub mod testing;
pub mod validation;
pub mod vulnerability;

pub use access_control::{
    AbacPolicy, AccessControl, AttributeBasedAccessControl, Permission, PermissionCheck,
    ResourceType,
};
pub use audit::{AuditEvent, AuditLogger, AuditRecord, MemoryAuditStorage};
pub use compliance::{
    ComplianceManager, ComplianceResult, ComplianceValidator, DataClassification, DataErasure,
    DataPortability, DefaultComplianceChecker, PrivacyAnalytics,
};
pub use encryption::{CustomerKeyManager, EncryptedData, KeyManager};
pub use error::SecurityError;
pub use monitoring::{SecurityEvent, SecurityMonitor, ThreatDetector, ThreatLevel};
pub use oauth::{OAuthProvider, OAuthToken, OidcProvider, TokenManager, UserInfo};
pub use penetration_testing::{DefaultPenetrationTester, PenetrationTestResult, PenetrationTester};
pub use reporting::{ComplianceReport, ComplianceReporter, ReportType};
pub use testing::{
    AuthResult, DefaultSecurityValidator, EncryptionResult, InputValidationResult,
    SecurityValidator,
};
pub use validation::{validate_input, ValidatedInput, ValidationEngine, ValidationError};
pub use vulnerability::{
    CodeSecurityScanResult, ConfigSecurityScanResult, DefaultVulnerabilityScanner,
    LicenseScanResult, VulnerabilityScanResult, VulnerabilityScanner,
};

// Service wrappers for DI integration
pub struct EncryptionService {
    key_manager: KeyManager,
}

impl EncryptionService {
    pub fn new() -> Self {
        // Use a default password for service initialization
        // In production, this should be configured securely
        let key_manager = KeyManager::new("default-service-password").unwrap();
        Self { key_manager }
    }

    pub fn key_manager(&self) -> &KeyManager {
        &self.key_manager
    }

    /// Encrypt data using the service's key manager
    pub fn encrypt(&self, data: &str) -> crate::Result<String> {
        let encrypted = self.key_manager.encrypt_api_key(data)?;
        Ok(
            serde_json::to_string(&encrypted).map_err(|e| crate::SecurityError::Serialization {
                message: e.to_string(),
            })?,
        )
    }

    /// Decrypt data using the service's key manager
    pub fn decrypt(&self, encrypted_json: &str) -> crate::Result<String> {
        let encrypted: crate::encryption::EncryptedData = serde_json::from_str(encrypted_json)
            .map_err(|e| crate::SecurityError::Deserialization {
                message: e.to_string(),
            })?;
        self.key_manager.decrypt_api_key(&encrypted)
    }
}

pub struct ValidationService {
    validator: ValidationEngine,
}

impl ValidationService {
    pub fn new() -> Self {
        let audit_logger = Arc::new(AuditLogger::new(Arc::new(MemoryAuditStorage::new())));
        let validator = ValidationEngine::new(audit_logger);
        Self { validator }
    }

    pub fn validator(&self) -> &ValidationEngine {
        &self.validator
    }

    /// Validate input data
    pub async fn validate(&self, input: &str) -> crate::Result<crate::validation::ValidatedInput> {
        crate::validation::validate_input(input)
    }

    /// Validate SQL input
    pub async fn validate_sql(&self, input: &str) -> crate::Result<String> {
        self.validator.validate_sql_input(input).await.map_err(|e| {
            crate::SecurityError::Validation {
                message: e.to_string(),
            }
        })
    }

    /// Validate HTML input
    pub async fn validate_html(&self, input: &str) -> crate::Result<String> {
        self.validator
            .validate_html_input(input)
            .await
            .map_err(|e| crate::SecurityError::Validation {
                message: e.to_string(),
            })
    }
}

/// Re-export commonly used types
pub type Result<T> = std::result::Result<T, SecurityError>;

// Property-based tests can be added here in the future
// For now, unit tests provide comprehensive coverage
