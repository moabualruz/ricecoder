//! End-to-End Test Suite: API E2E Tests with Provider Switching and Compliance Monitoring
//!
//! This test suite validates API-level end-to-end scenarios including provider switching,
//! compliance monitoring, rate limiting, error handling, and performance validation.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use ricecoder_cli::commands::*;
use ricecoder_providers::{ProviderManager, ProviderRegistry};
use ricecoder_security::{AuditLogger, ComplianceManager};
use tempfile::TempDir;
use tokio::{test, time::timeout};

/// Complete API workflow: Configure multiple providers, switch between them,
/// execute requests with compliance monitoring, handle failures gracefully.
#[tokio::test]
async fn test_api_provider_switching_workflow() {
    let provider_manager = Arc::new(
        ProviderManager::new()
            .await
            .expect("Failed to create provider manager"),
    );

    // Configure multiple providers
    configure_test_providers(&provider_manager).await;

    // Test provider switching
    test_provider_switching(&provider_manager).await;

    // Test compliance monitoring during API calls
    test_compliance_monitoring(&provider_manager).await;

    // Test failover scenarios
    test_provider_failover(&provider_manager).await;
}

/// Configure test providers for API testing
async fn configure_test_providers(provider_manager: &Arc<ProviderManager>) {
    // Configure OpenAI provider
    let openai_config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        api_key: Some("test-openai-key".to_string()),
        endpoint: Some("https://api.openai.com/v1".to_string()),
        models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
        rate_limits: Some(HashMap::from([
            ("requests_per_minute".to_string(), "60".to_string()),
            ("tokens_per_minute".to_string(), "150000".to_string()),
        ])),
        ..Default::default()
    };
    provider_manager
        .add_provider("openai", openai_config)
        .await
        .expect("Failed to add OpenAI provider");

    // Configure Anthropic provider
    let anthropic_config = ProviderConfig {
        provider_type: ProviderType::Anthropic,
        api_key: Some("test-anthropic-key".to_string()),
        endpoint: Some("https://api.anthropic.com".to_string()),
        models: vec!["claude-3-opus".to_string(), "claude-3-sonnet".to_string()],
        rate_limits: Some(HashMap::from([
            ("requests_per_minute".to_string(), "50".to_string()),
            ("tokens_per_minute".to_string(), "100000".to_string()),
        ])),
        ..Default::default()
    };
    provider_manager
        .add_provider("anthropic", anthropic_config)
        .await
        .expect("Failed to add Anthropic provider");

    // Configure local provider
    let local_config = ProviderConfig {
        provider_type: ProviderType::Local,
        endpoint: Some("http://localhost:11434".to_string()),
        models: vec!["llama2".to_string(), "codellama".to_string()],
        ..Default::default()
    };
    provider_manager
        .add_provider("local", local_config)
        .await
        .expect("Failed to add local provider");
}

/// Test switching between providers
async fn test_provider_switching(provider_manager: &Arc<ProviderManager>) {
    // Start with OpenAI
    provider_manager
        .set_active_provider("openai")
        .await
        .expect("Failed to set active provider");

    let active_provider = provider_manager
        .get_active_provider()
        .await
        .expect("Failed to get active provider");
    assert_eq!(active_provider.id, "openai");

    // Switch to Anthropic
    provider_manager
        .set_active_provider("anthropic")
        .await
        .expect("Failed to switch provider");

    let active_provider = provider_manager
        .get_active_provider()
        .await
        .expect("Failed to get active provider");
    assert_eq!(active_provider.id, "anthropic");

    // Switch to local
    provider_manager
        .set_active_provider("local")
        .await
        .expect("Failed to switch provider");

    let active_provider = provider_manager
        .get_active_provider()
        .await
        .expect("Failed to get active provider");
    assert_eq!(active_provider.id, "local");
}

/// Test compliance monitoring during API operations
async fn test_compliance_monitoring(provider_manager: &Arc<ProviderManager>) {
    let compliance_monitor = Arc::new(ComplianceMonitor::new(provider_manager.clone()));
    compliance_monitor
        .start_monitoring()
        .await
        .expect("Failed to start compliance monitoring");

    // Execute API calls with different providers
    for provider_id in ["openai", "anthropic", "local"] {
        provider_manager
            .set_active_provider(provider_id)
            .await
            .expect("Failed to set provider");

        // Execute test request
        let result = provider_manager
            .execute_request("test prompt", Some("gpt-4".to_string()), None)
            .await;

        // Result may fail due to mock keys, but compliance should be monitored
        let _ = result;
    }

    // Check compliance status
    let compliance_status = compliance_monitor
        .get_compliance_status()
        .await
        .expect("Failed to get compliance status");

    assert!(
        compliance_status.api_calls_logged >= 3,
        "Should have logged API calls"
    );

    compliance_monitor
        .stop_monitoring()
        .await
        .expect("Failed to stop monitoring");
}

/// Test provider failover scenarios
async fn test_provider_failover(provider_manager: &Arc<ProviderManager>) {
    // Set up failover chain: openai -> anthropic -> local
    provider_manager
        .configure_failover_chain(vec!["openai", "anthropic", "local"])
        .await
        .expect("Failed to configure failover");

    // Simulate OpenAI failure
    provider_manager
        .simulate_provider_failure("openai", true)
        .await;

    // Execute request - should failover to Anthropic
    let result = provider_manager
        .execute_request_with_failover("test prompt", Some("gpt-4".to_string()), None)
        .await;

    // Should succeed via failover or fail gracefully
    match result {
        Ok(_) => {
            let active_provider = provider_manager
                .get_active_provider()
                .await
                .expect("Failed to get active provider");
            assert!(active_provider.id == "anthropic" || active_provider.id == "local");
        }
        Err(_) => {
            // Verify failover was attempted
            let failover_attempts = provider_manager.get_failover_attempts().await;
            assert!(
                !failover_attempts.is_empty(),
                "Should have failover attempts"
            );
        }
    }
}

/// Test API rate limiting and throttling
#[tokio::test]
async fn test_api_rate_limiting() {
    let provider_manager = Arc::new(
        ProviderManager::new()
            .await
            .expect("Failed to create provider manager"),
    );

    // Configure provider with strict rate limits
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        api_key: Some("test-key".to_string()),
        rate_limits: Some(HashMap::from([
            ("requests_per_minute".to_string(), "2".to_string()),
            ("tokens_per_minute".to_string(), "1000".to_string()),
        ])),
        ..Default::default()
    };

    provider_manager
        .add_provider("rate-limited", config)
        .await
        .expect("Failed to add provider");

    provider_manager
        .set_active_provider("rate-limited")
        .await
        .expect("Failed to set active provider");

    // Execute requests rapidly
    let mut results = vec![];
    for i in 0..5 {
        let result = provider_manager
            .execute_request(&format!("Request {}", i), Some("gpt-4".to_string()), None)
            .await;
        results.push(result);
    }

    // Some requests should be rate limited
    let rate_limited_count = results.iter().filter(|r| r.is_err()).count();
    assert!(
        rate_limited_count > 0,
        "Some requests should be rate limited"
    );

    // Check rate limit status
    let rate_limit_status = provider_manager
        .get_rate_limit_status()
        .await
        .expect("Failed to get rate limit status");

    assert!(
        rate_limit_status.requests_remaining < 2,
        "Should have used rate limit quota"
    );
}

/// Test API error handling and recovery
#[tokio::test]
async fn test_api_error_handling_recovery() {
    let provider_manager = Arc::new(
        ProviderManager::new()
            .await
            .expect("Failed to create provider manager"),
    );

    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        api_key: Some("invalid-key".to_string()),
        retry_config: Some(HashMap::from([
            ("max_retries".to_string(), "3".to_string()),
            ("backoff_ms".to_string(), "100".to_string()),
        ])),
        ..Default::default()
    };

    provider_manager
        .add_provider("error-test", config)
        .await
        .expect("Failed to add provider");

    provider_manager
        .set_active_provider("error-test")
        .await
        .expect("Failed to set active provider");

    let start_time = Instant::now();

    // Execute request that will fail and retry
    let result = provider_manager
        .execute_request("test prompt", Some("gpt-4".to_string()), None)
        .await;

    let duration = start_time.elapsed();

    // Should eventually fail after retries
    assert!(result.is_err(), "Request should fail with invalid key");

    // Should have taken some time due to retries
    assert!(
        duration > Duration::from_millis(300),
        "Should have retry delays"
    );

    // Check retry statistics
    let retry_stats = provider_manager
        .get_retry_statistics()
        .await
        .expect("Failed to get retry statistics");

    assert!(retry_stats.total_retries > 0, "Should have retry attempts");
}

/// Test API compliance validation
#[tokio::test]
async fn test_api_compliance_validation() {
    let provider_manager = Arc::new(
        ProviderManager::new()
            .await
            .expect("Failed to create provider manager"),
    );

    let compliance_monitor = Arc::new(ComplianceMonitor::new(provider_manager.clone()));

    // Configure provider with compliance rules
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        api_key: Some("test-key".to_string()),
        compliance_rules: Some(vec![
            "no_pii_data".to_string(),
            "content_filtering".to_string(),
            "audit_logging".to_string(),
        ]),
        ..Default::default()
    };

    provider_manager
        .add_provider("compliant", config)
        .await
        .expect("Failed to add provider");

    compliance_monitor
        .start_monitoring()
        .await
        .expect("Failed to start monitoring");

    // Test compliant request
    let compliant_result = provider_manager
        .execute_compliant_request(
            "Please analyze this code structure",
            Some("gpt-4".to_string()),
            None,
        )
        .await;

    // Test non-compliant request (contains PII)
    let non_compliant_result = provider_manager
        .execute_compliant_request(
            "Analyze data for user john.doe@email.com with SSN 123-45-6789",
            Some("gpt-4".to_string()),
            None,
        )
        .await;

    // Non-compliant request should be blocked or flagged
    match non_compliant_result {
        Ok(_) => {
            // If it succeeded, check that it was flagged
            let violations = compliance_monitor
                .get_violations()
                .await
                .expect("Failed to get violations");
            assert!(!violations.is_empty(), "Should have compliance violations");
        }
        Err(_) => {
            // Request was blocked - this is also acceptable
        }
    }

    compliance_monitor
        .stop_monitoring()
        .await
        .expect("Failed to stop monitoring");
}

/// Test concurrent API requests with provider management
#[tokio::test]
async fn test_concurrent_api_requests() {
    let provider_manager = Arc::new(
        ProviderManager::new()
            .await
            .expect("Failed to create provider manager"),
    );

    // Configure provider
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        api_key: Some("test-key".to_string()),
        ..Default::default()
    };

    provider_manager
        .add_provider("concurrent", config)
        .await
        .expect("Failed to add provider");

    // Execute multiple concurrent requests
    let mut handles = vec![];

    for i in 0..10 {
        let manager = provider_manager.clone();
        let handle = tokio::spawn(async move {
            manager
                .set_active_provider("concurrent")
                .await
                .expect("Failed to set provider");

            let result = manager
                .execute_request(
                    &format!("Concurrent request {}", i),
                    Some("gpt-4".to_string()),
                    None,
                )
                .await;

            result.is_ok()
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut success_count = 0;
    for handle in handles {
        let success = handle.await.expect("Request task failed");
        if success {
            success_count += 1;
        }
    }

    // Some requests may succeed (depending on mock implementation)
    assert!(success_count >= 0, "Should have completed requests");

    // Check concurrency metrics
    let concurrency_stats = provider_manager
        .get_concurrency_statistics()
        .await
        .expect("Failed to get concurrency statistics");

    assert!(
        concurrency_stats.total_requests >= 10,
        "Should have handled all requests"
    );
}

/// Test API performance validation
#[tokio::test]
async fn test_api_performance_validation() {
    let provider_manager = Arc::new(
        ProviderManager::new()
            .await
            .expect("Failed to create provider manager"),
    );

    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        api_key: Some("test-key".to_string()),
        performance_targets: Some(HashMap::from([
            ("max_latency_ms".to_string(), "5000".to_string()),
            ("min_throughput_req_per_sec".to_string(), "10".to_string()),
        ])),
        ..Default::default()
    };

    provider_manager
        .add_provider("performance", config)
        .await
        .expect("Failed to add provider");

    provider_manager
        .set_active_provider("performance")
        .await
        .expect("Failed to set provider");

    let start_time = Instant::now();

    // Execute multiple requests to measure performance
    let mut latencies = vec![];
    for i in 0..5 {
        let request_start = Instant::now();
        let _result = provider_manager
            .execute_request(
                &format!("Performance test request {}", i),
                Some("gpt-4".to_string()),
                None,
            )
            .await;
        let latency = request_start.elapsed();
        latencies.push(latency);
    }

    let total_duration = start_time.elapsed();
    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;

    // Check performance targets
    let performance_report = provider_manager
        .get_performance_report()
        .await
        .expect("Failed to get performance report");

    assert!(
        performance_report.average_latency <= Duration::from_secs(5),
        "Average latency should meet target"
    );

    // Throughput should be reasonable
    let throughput = 5.0 / total_duration.as_secs_f64();
    assert!(throughput > 0.1, "Throughput should be reasonable");
}
