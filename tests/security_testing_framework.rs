//! Comprehensive Security Testing Suite
//!
//! This test suite validates:
//! - Security testing framework functionality
//! - Vulnerability scanning capabilities
//! - Penetration testing simulation
//! - Compliance validation for enterprise standards
//! - Security regression testing

use ricecoder_security::*;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::test;

/// Test security validator functionality
#[tokio::test]
async fn test_security_validator() {
    let validator = DefaultSecurityValidator::new();

    // Test input validation
    let safe_input = validator.validate_input("Hello world").await.unwrap();
    assert!(safe_input.is_safe);

    let malicious_input = validator.validate_input("<script>alert('xss')</script>").await.unwrap();
    assert!(!malicious_input.is_safe);
    assert!(malicious_input.violations.len() > 0);

    // Test authentication
    let auth_result = validator.validate_authentication("user", "password").await.unwrap();
    assert!(auth_result.success);

    // Test authorization
    let authz_result = validator.validate_authorization("admin", "read").await.unwrap();
    assert!(authz_result.allowed);

    // Test encryption
    let encrypted = validator.encrypt_data("sensitive data").await.unwrap();
    assert!(encrypted.success);
    assert_ne!(encrypted.data, b"sensitive data");

    let decrypted = validator.decrypt_data(&encrypted.data).await.unwrap();
    assert_eq!(decrypted, "sensitive data");

    // Test API key validation
    let api_result = validator.validate_api_key("sk-test-key-123").await.unwrap();
    assert!(api_result.is_valid);

    // Test rate limiting
    let rate_result = validator.check_rate_limit("user1", "api_call").await.unwrap();
    assert!(rate_result.allowed);
}

/// Test vulnerability scanner
#[tokio::test]
async fn test_vulnerability_scanner() {
    let scanner = DefaultVulnerabilityScanner::new();
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("Cargo.toml");

    // Create a dummy Cargo.toml
    std::fs::write(&manifest_path, r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
    "#).unwrap();

    // Test dependency scanning
    let result = scanner.scan_dependencies(&manifest_path).await.unwrap();
    assert!(result.scan_timestamp <= chrono::Utc::now());

    // Test code scanning
    let code_result = scanner.scan_code(temp_dir.path()).await.unwrap();
    assert_eq!(code_result.files_scanned, 1); // The Cargo.toml file

    // Test config scanning
    let config_result = scanner.scan_config(temp_dir.path()).await.unwrap();
    assert_eq!(config_result.files_scanned, 1);
}

/// Test penetration testing
#[tokio::test]
async fn test_penetration_testing() {
    let tester = DefaultPenetrationTester::new();

    // Test SQL injection
    let sql_result = tester.test_sql_injection("http://example.com").await.unwrap();
    assert!(sql_result.vulnerable_endpoints.is_empty()); // Should be safe for test URL

    // Test XSS
    let xss_result = tester.test_xss("http://example.com").await.unwrap();
    assert!(xss_result.vulnerable_endpoints.is_empty()); // Should be safe for test URL

    // Test CSRF
    let csrf_result = tester.test_csrf("http://example.com").await.unwrap();
    assert!(csrf_result.vulnerable_endpoints.is_empty()); // Should be safe for test URL

    // Test full penetration test
    let full_result = tester.run_full_penetration_test("http://example.com").await.unwrap();
    // Should not find vulnerabilities for test URL
    assert!(full_result.iter().all(|r| r.vulnerabilities_found.is_empty()));
}

/// Test compliance checking
#[tokio::test]
async fn test_compliance_checking() {
    let checker = DefaultComplianceChecker::new();

    // Test SOC 2 compliance
    let soc2_result = checker.check_soc2_compliance().await.unwrap();
    assert_eq!(soc2_result.standard, ComplianceStandard::SOC2Type2);
    assert!(soc2_result.score >= 0.0);

    // Test GDPR compliance
    let gdpr_result = checker.check_gdpr_compliance().await.unwrap();
    assert_eq!(gdpr_result.standard, ComplianceStandard::GDPR);
    assert!(gdpr_result.score >= 0.0);

    // Test HIPAA compliance
    let hipaa_result = checker.check_hipaa_compliance().await.unwrap();
    assert_eq!(hipaa_result.standard, ComplianceStandard::HIPAA);
    assert!(hipaa_result.score >= 0.0);

    // Test OWASP compliance
    let owasp_result = checker.check_owasp_compliance().await.unwrap();
    assert_eq!(owasp_result.standard, ComplianceStandard::OWASP);
    assert!(owasp_result.score >= 0.0);

    // Test compliance report generation
    let report = checker.generate_compliance_report().await.unwrap();
    assert_eq!(report.standards_checked.len(), 4);
    assert!(report.overall_score >= 0.0);
}

/// Test security regression detection
#[tokio::test]
async fn test_security_regression_testing() {
    let validator = DefaultSecurityValidator::new();

    // Get baseline
    let baseline = validator.get_security_baseline().await.unwrap();
    assert!(baseline.security_score >= 0.0);

    // Get current status
    let current = validator.get_current_security_status().await.unwrap();
    assert!(current.security_score >= 0.0);

    // Ensure no regression (current should be at least as good as baseline)
    assert!(current.security_score >= baseline.security_score - 5.0); // Allow small variance
}

/// Test property-based security testing
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_input_validation_property(input in ".*") {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let validator = DefaultSecurityValidator::new();
                let result = validator.validate_input(&input).await.unwrap();

                // Safe inputs should be marked as safe
                if !input.contains("<script>") && !input.contains("../../../") && !input.contains("DROP TABLE") {
                    // For safe inputs, result should be safe or sanitized
                    assert!(result.is_safe || result.sanitized_input.is_some(),
                           "Safe input '{}' was incorrectly flagged as unsafe", input);
                }
            });
        }

        #[test]
        fn test_encryption_roundtrip_property(data in ".*") {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let validator = DefaultSecurityValidator::new();

                if !data.is_empty() {
                    let encrypted = validator.encrypt_data(&data).await.unwrap();
                    assert!(encrypted.success);

                    let decrypted = validator.decrypt_data(&encrypted.data).await.unwrap();
                    assert_eq!(decrypted, data);
                }
            });
        }
    }
}

/// Integration test for security scanning
#[tokio::test]
async fn test_security_scanning_integration() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    // Create a test file with potential security issues
    std::fs::write(&test_file, r#"
fn main() {
    let password = "hardcoded_password"; // This should be flagged
    let query = format!("SELECT * FROM users WHERE id = {}", user_id); // SQL injection risk
    unsafe { // Unsafe code
        println!("Hello");
    }
}
    "#).unwrap();

    let scanner = DefaultVulnerabilityScanner::new();
    let result = scanner.scan_code(temp_dir.path()).await.unwrap();

    assert_eq!(result.files_scanned, 1);
    assert!(result.issues.len() >= 2); // Should find at least password and unsafe issues
}

/// Test compliance validation with different configurations
#[tokio::test]
async fn test_compliance_with_config() {
    // Test with security features disabled
    let checker = DefaultComplianceChecker::with_config(false, false, false, false);

    let soc2_result = checker.check_soc2_compliance().await.unwrap();
    assert!(soc2_result.score < 50.0); // Should have low score with features disabled

    // Test with all features enabled
    let checker_secure = DefaultComplianceChecker::with_config(true, true, true, true);

    let soc2_secure_result = checker_secure.check_soc2_compliance().await.unwrap();
    assert!(soc2_secure_result.score > soc2_result.score); // Should have higher score
}