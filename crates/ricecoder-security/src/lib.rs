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

pub mod encryption;
pub mod validation;
pub mod audit;
pub mod access_control;
pub mod compliance;
pub mod reporting;
pub mod oauth;
pub mod monitoring;
pub mod testing;
pub mod error;
pub mod vulnerability;
pub mod penetration_testing;

pub use encryption::{KeyManager, CustomerKeyManager, EncryptedData};
pub use validation::{validate_input, ValidatedInput, ValidationError, ValidationEngine};
pub use audit::{AuditLogger, AuditEvent, AuditRecord, MemoryAuditStorage};
pub use access_control::{Permission, ResourceType, AccessControl, PermissionCheck, AbacPolicy, AttributeBasedAccessControl};
pub use compliance::{ComplianceManager, ComplianceValidator, DataErasure, DataPortability, PrivacyAnalytics, DataClassification, DefaultComplianceChecker, ComplianceResult};
pub use vulnerability::{VulnerabilityScanner, DefaultVulnerabilityScanner, VulnerabilityScanResult, CodeSecurityScanResult, ConfigSecurityScanResult, LicenseScanResult};
pub use penetration_testing::{PenetrationTester, DefaultPenetrationTester, PenetrationTestResult};
pub use testing::{SecurityValidator, DefaultSecurityValidator, InputValidationResult, AuthResult, EncryptionResult};
pub use reporting::{ComplianceReporter, ComplianceReport, ReportType};
pub use oauth::{TokenManager, OAuthProvider, OidcProvider, OAuthToken, UserInfo};
pub use monitoring::{SecurityMonitor, ThreatDetector, SecurityEvent, ThreatLevel};
pub use error::SecurityError;

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
        Ok(serde_json::to_string(&encrypted).map_err(|e| crate::SecurityError::Serialization {
            message: e.to_string(),
        })?)
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
        self.validator.validate_sql_input(input).await
            .map_err(|e| crate::SecurityError::Validation {
                message: e.to_string(),
            })
    }

    /// Validate HTML input
    pub async fn validate_html(&self, input: &str) -> crate::Result<String> {
        self.validator.validate_html_input(input).await
            .map_err(|e| crate::SecurityError::Validation {
                message: e.to_string(),
            })
    }
}

/// Re-export commonly used types
pub type Result<T> = std::result::Result<T, SecurityError>;

// Property-based tests can be added here in the future
// For now, unit tests provide comprehensive coverage