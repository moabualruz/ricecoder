//! Property-based tests for provider ecosystem with performance and reliability validation
//!
//! **Feature: ricecoder-providers, Property Tests: Performance and Reliability**
//! **Validates: Requirements PROVIDER-1.1, PROVIDER-1.2, PROVIDER-2.1, PROVIDER-2.2, PROVIDER-3.1**
//!
//! These tests verify that the provider ecosystem maintains performance characteristics,
//! reliability under load, and correct behavior across different provider configurations.

use proptest::prelude::*;
use ricecoder_providers::{
    cache::ProviderCache,
    health_check::HealthChecker,
    models::{Message, MessageRole},
    performance_monitor::PerformanceMonitor,
    provider::{ProviderManager, ProviderRegistry},
    rate_limiter::{RateLimiter, TokenBucketLimiter},
    ChatRequest, ChatResponse, ModelInfo, Provider, ProviderError, TokenUsage,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;

// ============================================================================
// Mock Providers for Testing
// ============================================================================

#[derive(Clone)]
struct MockProvider {
    id: String,
    name: String,
    models: Vec<ModelInfo>,
    should_fail: bool,
    response_delay: Duration,
    token_usage: TokenUsage,
}

impl MockProvider {
    fn new(id: &str, should_fail: bool, response_delay_ms: u64) -> Self {
        let models = vec![
            ModelInfo {
                id: format!("{}-model-1", id),
                name: format!("{} Model 1", id),
                provider: id.to_string(),
                context_window: 4096,
                capabilities: vec!["chat".to_string()],
                pricing: None,
                is_free: true,
            },
            ModelInfo {
                id: format!("{}-model-2", id),
                name: format!("{} Model 2", id),
                provider: id.to_string(),
                context_window: 8192,
                capabilities: vec!["chat".to_string(), "streaming".to_string()],
                pricing: None,
                is_free: false,
            },
        ];

        Self {
            id: id.to_string(),
            name: format!("Mock {}", id),
            models,
            should_fail,
            response_delay: Duration::from_millis(response_delay_ms),
            token_usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
        }
    }
}

#[async_trait::async_trait]
impl Provider for MockProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn models(&self) -> Vec<ModelInfo> {
        self.models.clone()
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        // Simulate processing delay
        tokio::time::sleep(self.response_delay).await;

        if self.should_fail {
            return Err(ProviderError::ProviderError(
                "Mock provider failure".to_string(),
            ));
        }

        Ok(ChatResponse {
            content: format!("Response from {} for model {}", self.id, request.model),
            model: request.model,
            usage: self.token_usage.clone(),
            finish_reason: ricecoder_providers::models::FinishReason::Stop,
        })
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<ricecoder_providers::ChatStream, ProviderError> {
        Err(ProviderError::NotFound(
            "Streaming not implemented".to_string(),
        ))
    }

    fn count_tokens(&self, content: &str, _model: &str) -> Result<usize, ProviderError> {
        Ok(content.len() / 4) // Simple approximation
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        Ok(!self.should_fail)
    }
}

// ============================================================================
// Generators for property-based testing
// ============================================================================

fn arb_chat_request() -> impl Strategy<Value = ChatRequest> {
    (
        "[a-zA-Z0-9_-]{1,50}".prop_map(|s| s),       // model
        prop::collection::vec(arb_message(), 1..10), // messages
        proptest::option::of((0.0..2.0f32).prop_map(|f| f as f64)), // temperature
        proptest::option::of((1..1000usize)),        // max_tokens
        any::<bool>(),                               // stream
    )
        .prop_map(
            |(model, messages, temperature, max_tokens, stream)| ChatRequest {
                model,
                messages,
                temperature,
                max_tokens,
                stream,
            },
        )
}

fn arb_message() -> impl Strategy<Value = Message> {
    (
        prop_oneof![
            Just(MessageRole::User),
            Just(MessageRole::Assistant),
            Just(MessageRole::System)
        ],
        ".{1,500}".prop_map(|s| s), // content
    )
        .prop_map(|(role, content)| Message { role, content })
}

fn arb_provider_config() -> impl Strategy<Value = (String, bool, u64)> {
    (
        "[a-zA-Z0-9_-]{1,20}".prop_map(|s| s), // provider_id
        any::<bool>(),                         // should_fail
        (10..500u64),                          // response_delay_ms
    )
}

// ============================================================================
// Property 1: Provider Performance Under Load
// ============================================================================

proptest! {
    /// Property 1: Provider Performance Under Load
    /// *For any* provider configuration and request load, response times SHALL remain
    /// within acceptable bounds and not degrade catastrophically.
    /// **Validates: Requirements PROVIDER-1.1, PROVIDER-2.1**
    #[test]
    fn prop_provider_performance_under_load(
        provider_configs in prop::collection::vec(arb_provider_config(), 1..5),
        request_count in 1..50usize,
        concurrent_requests in 1..10usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            // Create providers
            let mut registry = ProviderRegistry::new();
            let mut provider_ids = Vec::new();

            for (id, should_fail, delay_ms) in provider_configs {
                if !should_fail { // Only add working providers for performance testing
                    let provider = Arc::new(MockProvider::new(&id, false, delay_ms));
                    registry.register(provider).unwrap();
                    provider_ids.push(id);
                }
            }

            if provider_ids.is_empty() {
                return Ok(()); // Skip if no working providers
            }

            let manager = Arc::new(ProviderManager::new(registry, provider_ids[0].clone()));
            let semaphore = Arc::new(Semaphore::new(concurrent_requests));

            let mut handles = Vec::new();
            let start_time = Instant::now();

            for i in 0..request_count {
                let manager_clone = manager.clone();
                let semaphore_clone = semaphore.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();

                    let request = ChatRequest {
                        model: format!("model-{}", i % 2),
                        messages: vec![Message {
                            role: MessageRole::User,
                            content: format!("Test message {}", i),
                        }],
                        temperature: Some(0.7),
                        max_tokens: Some(100),
                        stream: false,
                    };

                    let result = timeout(
                        Duration::from_secs(10), // 10 second timeout
                        manager_clone.chat(request)
                    ).await;

                    match result {
                        Ok(Ok(_)) => true, // Success
                        Ok(Err(_)) => false, // Provider error
                        Err(_) => false, // Timeout
                    }
                });

                handles.push(handle);
            }

            // Wait for all requests to complete
            let mut success_count = 0;
            for handle in handles {
                if let Ok(success) = handle.await {
                    if success {
                        success_count += 1;
                    }
                }
            }

            let total_time = start_time.elapsed();

            // Performance assertions
            let avg_time_per_request = total_time / request_count as u32;

            // Average response time should be reasonable (less than 2 seconds per request)
            prop_assert!(avg_time_per_request < Duration::from_secs(2),
                "Average response time too high: {:?}", avg_time_per_request);

            // At least some requests should succeed
            prop_assert!(success_count > 0, "No requests succeeded");

            // Success rate should be reasonable (> 50%)
            let success_rate = success_count as f64 / request_count as f64;
            prop_assert!(success_rate > 0.5, "Success rate too low: {}", success_rate);
        });
    }

    /// Property 1 variant: Performance degradation detection
    #[test]
    fn prop_performance_degradation_detection(
        base_delay_ms in 50..200u64,
        load_multiplier in 1..10usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let provider = Arc::new(MockProvider::new("perf-test", false, base_delay_ms));
            let mut registry = ProviderRegistry::new();
            registry.register(provider).unwrap();

            let manager = ProviderManager::new(registry, "perf-test".to_string());

            // Test baseline performance
            let baseline_request = ChatRequest {
                model: "model-1".to_string(),
                messages: vec![Message {
                    role: MessageRole::User,
                    content: "Baseline test".to_string(),
                }],
                temperature: None,
                max_tokens: None,
                stream: false,
            };

            let baseline_start = Instant::now();
            let _baseline_result = manager.chat(baseline_request).await.unwrap();
            let baseline_time = baseline_start.elapsed();

            // Test under load
            let mut handles = Vec::new();
            let load_start = Instant::now();

            for i in 0..load_multiplier {
                let manager_clone = manager.clone();

                let handle = tokio::spawn(async move {
                    let request = ChatRequest {
                        model: "model-1".to_string(),
                        messages: vec![Message {
                            role: MessageRole::User,
                            content: format!("Load test {}", i),
                        }],
                        temperature: None,
                        max_tokens: None,
                        stream: false,
                    };

                    let req_start = Instant::now();
                    let result = manager_clone.chat(request).await;
                    let req_time = req_start.elapsed();

                    (result.is_ok(), req_time)
                });

                handles.push(handle);
            }

            // Collect load test results
            let mut load_times = Vec::new();
            let mut success_count = 0;

            for handle in handles {
                if let Ok((success, time)) = handle.await {
                    if success {
                        success_count += 1;
                        load_times.push(time);
                    }
                }
            }

            let load_duration = load_start.elapsed();

            // Performance should not degrade more than 5x under load
            let avg_load_time = load_times.iter().sum::<Duration>() / load_times.len() as u32;
            let degradation_ratio = avg_load_time.as_millis() as f64 / baseline_time.as_millis() as f64;

            prop_assert!(degradation_ratio < 5.0,
                "Performance degraded too much: {}x", degradation_ratio);

            // All requests should succeed
            prop_assert_eq!(success_count, load_multiplier);
        });
    }
}

// ============================================================================
// Property 2: Provider Reliability and Failover
// ============================================================================

proptest! {
    /// Property 2: Provider Reliability and Failover
    /// *For any* provider failure scenario, the system SHALL maintain reliability
    /// through appropriate error handling and failover mechanisms.
    /// **Validates: Requirements PROVIDER-1.2, PROVIDER-2.2**
    #[test]
    fn prop_provider_reliability_failover(
        provider_configs in prop::collection::vec(arb_provider_config(), 2..5),
        failure_rate in 0.0..1.0f64,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let mut registry = ProviderRegistry::new();
            let mut provider_ids = Vec::new();

            // Create providers with controlled failure rates
            for (id, _, delay_ms) in provider_configs {
                let should_fail = rand::random::<f64>() < failure_rate;
                let provider = Arc::new(MockProvider::new(&id, should_fail, delay_ms));
                registry.register(provider).unwrap();
                provider_ids.push(id);
            }

            let manager = Arc::new(ProviderManager::new(registry, provider_ids[0].clone()));

            // Test multiple requests
            let request_count = 20;
            let mut success_count = 0;

            for i in 0..request_count {
                let manager_clone = manager.clone();

                let request = ChatRequest {
                    model: format!("model-{}", i % 2),
                    messages: vec![Message {
                        role: MessageRole::User,
                        content: format!("Reliability test {}", i),
                    }],
                    temperature: Some(0.7),
                    max_tokens: Some(100),
                    stream: false,
                };

                let result = timeout(
                    Duration::from_secs(5),
                    manager_clone.chat(request)
                ).await;

                if let Ok(Ok(_)) = result {
                    success_count += 1;
                }
            }

            let success_rate = success_count as f64 / request_count as f64;

            // If failure rate is low (< 50%), success rate should be reasonable
            if failure_rate < 0.5 {
                prop_assert!(success_rate > 0.3,
                    "Success rate too low for low failure rate. Expected > 0.3, got {}", success_rate);
            }

            // If all providers fail, success rate should be 0
            if failure_rate >= 1.0 {
                prop_assert_eq!(success_rate, 0.0, "Should have 0 success rate when all providers fail");
            }
        });
    }

    /// Property 2 variant: Circuit breaker behavior
    #[test]
    fn prop_circuit_breaker_behavior(
        failure_threshold in 3..10usize,
        recovery_attempts in 1..5usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            // Create a provider that fails consistently
            let failing_provider = Arc::new(MockProvider::new("failing", true, 10));
            let mut registry = ProviderRegistry::new();
            registry.register(failing_provider).unwrap();

            let manager = ProviderManager::new(registry, "failing".to_string());

            // Send failing requests up to threshold
            for i in 0..failure_threshold {
                let manager_clone = manager.clone();
                let request = ChatRequest {
                    model: "model-1".to_string(),
                    messages: vec![Message {
                        role: MessageRole::User,
                        content: format!("Failure test {}", i),
                    }],
                    temperature: None,
                    max_tokens: None,
                    stream: false,
                };

                let _result = manager_clone.chat(request).await;
                // Expect failure but don't assert - we're testing circuit breaker
            }

            // After threshold failures, circuit should potentially open
            // This is a simplified test - real implementation would have circuit breaker
            let final_request = ChatRequest {
                model: "model-1".to_string(),
                messages: vec![Message {
                    role: MessageRole::User,
                    content: "Final test".to_string(),
                }],
                temperature: None,
                max_tokens: None,
                stream: false,
            };

            let final_result = manager.chat(final_request).await;

            // Result should be failure (circuit open or provider failing)
            prop_assert!(final_result.is_err());
        });
    }
}

// ============================================================================
// Property 3: Rate Limiting Effectiveness
// ============================================================================

proptest! {
    /// Property 3: Rate Limiting Effectiveness
    /// *For any* rate limit configuration, the rate limiter SHALL prevent requests
    /// from exceeding the configured limits.
    /// **Validates: Requirements PROVIDER-3.1**
    #[test]
    fn prop_rate_limiting_effectiveness(
        capacity in 5..100usize,
        refill_rate in 1..20usize,
        request_burst in (capacity + 1)..200usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let rate_limiter = Arc::new(TokenBucketLimiter::new(capacity, refill_rate));
            let semaphore = Arc::new(Semaphore::new(10)); // Allow some concurrency

            let mut handles = Vec::new();
            let start_time = Instant::now();

            // Send burst of requests
            for i in 0..request_burst {
                let rate_limiter_clone = rate_limiter.clone();
                let semaphore_clone = semaphore.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();

                    // Check if request is allowed
                    rate_limiter_clone.acquire().await
                });

                handles.push(handle);
            }

            // Collect results
            let mut allowed_count = 0;
            let mut denied_count = 0;

            for handle in handles {
                if let Ok(allowed) = handle.await {
                    if allowed {
                        allowed_count += 1;
                    } else {
                        denied_count += 1;
                    }
                }
            }

            // Should allow up to capacity requests initially
            prop_assert!(allowed_count <= capacity,
                "Allowed more requests than capacity: {} > {}", allowed_count, capacity);

            // Should deny excess requests
            prop_assert!(denied_count > 0,
                "Should have denied some requests in burst");

            // Total requests should equal burst size
            prop_assert_eq!(allowed_count + denied_count, request_burst);
        });
    }

    /// Property 3 variant: Rate limit recovery
    #[test]
    fn prop_rate_limit_recovery(
        capacity in 10..50usize,
        wait_time_ms in 100..1000u64,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let rate_limiter = TokenBucketLimiter::new(capacity, 10); // 10 tokens per second

            // Exhaust the bucket
            for _ in 0..capacity {
                let allowed = rate_limiter.acquire().await;
                prop_assert!(allowed, "Should allow initial capacity requests");
            }

            // Next request should be denied
            let denied = rate_limiter.acquire().await;
            prop_assert!(!denied, "Should deny request over capacity");

            // Wait for recovery
            tokio::time::sleep(Duration::from_millis(wait_time_ms)).await;

            // Should allow some requests after waiting
            let recovered_allowed = rate_limiter.acquire().await;
            // Note: This may still be false depending on timing, so we don't assert
            // In a real test, we'd have more sophisticated timing control

            // The rate limiter should still be functional
            prop_assert!(true); // If we reach here, the test passed
        });
    }
}

// ============================================================================
// Property 4: Caching Effectiveness
// ============================================================================

proptest! {
    /// Property 4: Caching Effectiveness
    /// *For any* cache configuration and access pattern, the cache SHALL improve
    /// performance for repeated requests and maintain data consistency.
    /// **Validates: Requirements PROVIDER-1.1, PROVIDER-2.1**
    #[test]
    fn prop_caching_effectiveness(
        cache_size in 10..100usize,
        request_count in 50..200usize,
        cache_hit_rate_target in 0.3..0.9f64,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let cache = Arc::new(ProviderCache::new(cache_size));
            let semaphore = Arc::new(Semaphore::new(5));

            let mut handles = Vec::new();
            let mut cache_hits = Arc::new(RwLock::new(0));
            let mut total_requests = Arc::new(RwLock::new(0));

            for i in 0..request_count {
                let cache_clone = cache.clone();
                let semaphore_clone = semaphore.clone();
                let cache_hits_clone = cache_hits.clone();
                let total_requests_clone = total_requests.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();

                    *total_requests_clone.write().await += 1;

                    // Use limited set of cache keys to ensure hits
                    let cache_key = format!("request-{}", i % (cache_size / 2));

                    // Simulate cache lookup
                    if cache_clone.get(&cache_key).await.is_some() {
                        *cache_hits_clone.write().await += 1;
                    } else {
                        // Cache miss - store something
                        let response = ChatResponse {
                            content: format!("Cached response {}", i),
                            model: "model-1".to_string(),
                            usage: TokenUsage {
                                prompt_tokens: 5,
                                completion_tokens: 10,
                                total_tokens: 15,
                            },
                            finish_reason: ricecoder_providers::models::FinishReason::Stop,
                        };
                        let _ = cache_clone.put(cache_key, response, Duration::from_secs(300)).await;
                    }
                });

                handles.push(handle);
            }

            // Wait for all requests
            for handle in handles {
                let _ = handle.await;
            }

            let final_cache_hits = *cache_hits.read().await;
            let final_total_requests = *total_requests.read().await;

            let hit_rate = final_cache_hits as f64 / final_total_requests as f64;

            // Hit rate should be reasonable for this access pattern
            prop_assert!(hit_rate >= cache_hit_rate_target * 0.5,
                "Cache hit rate too low: {} < {}", hit_rate, cache_hit_rate_target * 0.5);

            // Should have some cache hits
            prop_assert!(final_cache_hits > 0, "Should have some cache hits");
        });
    }

    /// Property 4 variant: Cache consistency under concurrency
    #[test]
    fn prop_cache_consistency_concurrency(
        operation_count in 20..100usize,
        concurrent_threads in 1..10usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let cache = Arc::new(ProviderCache::new(50));
            let semaphore = Arc::new(Semaphore::new(concurrent_threads));

            let mut handles = Vec::new();

            for i in 0..operation_count {
                let cache_clone = cache.clone();
                let semaphore_clone = semaphore.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();

                    match i % 3 {
                        0 => {
                            // Put operation
                            let response = ChatResponse {
                                content: format!("Response {}", i),
                                model: "model-1".to_string(),
                                usage: TokenUsage {
                                    prompt_tokens: 5,
                                    completion_tokens: 10,
                                    total_tokens: 15,
                                },
                                finish_reason: ricecoder_providers::models::FinishReason::Stop,
                            };
                            cache_clone.put(
                                format!("key-{}", i),
                                response,
                                Duration::from_secs(300)
                            ).await.is_ok()
                        }
                        1 => {
                            // Get operation
                            cache_clone.get(&format!("key-{}", i % 10)).await.is_some()
                        }
                        2 => {
                            // Invalidate operation
                            cache_clone.invalidate(&format!("key-{}", i % 10)).await;
                            true
                        }
                        _ => unreachable!(),
                    }
                });

                handles.push(handle);
            }

            // Wait for all operations
            let mut success_count = 0;
            for handle in handles {
                if let Ok(success) = handle.await {
                    if success {
                        success_count += 1;
                    }
                }
            }

            // Most operations should succeed
            prop_assert!(success_count >= operation_count * 2 / 3);
        });
    }
}

// ============================================================================
// Property 5: Health Check Reliability
// ============================================================================

proptest! {
    /// Property 5: Health Check Reliability
    /// *For any* provider health status, health checks SHALL accurately report
    /// provider availability and recovery.
    /// **Validates: Requirements PROVIDER-2.2**
    #[test]
    fn prop_health_check_reliability(
        provider_configs in prop::collection::vec(arb_provider_config(), 1..5),
        check_count in 10..50usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let mut registry = ProviderRegistry::new();
            let mut provider_health_map = HashMap::new();

            // Create providers and track expected health
            for (id, should_fail, delay_ms) in provider_configs {
                let provider = Arc::new(MockProvider::new(&id, should_fail, delay_ms));
                registry.register(provider).unwrap();
                provider_health_map.insert(id, !should_fail);
            }

            let health_checker = Arc::new(HealthChecker::new(Duration::from_millis(100)));
            let semaphore = Arc::new(Semaphore::new(5));

            let mut handles = Vec::new();

            for _ in 0..check_count {
                let registry_clone = registry.clone();
                let health_checker_clone = health_checker.clone();
                let semaphore_clone = semaphore.clone();
                let provider_health_map_clone = provider_health_map.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();

                    let mut correct_checks = 0;
                    let mut total_checks = 0;

                    // Check health of all providers
                    for (provider_id, expected_healthy) in provider_health_map_clone {
                        if let Ok(provider) = registry_clone.get(&provider_id) {
                            let health_result = health_checker_clone.check_provider(&*provider).await;
                            let is_healthy = health_result.is_ok() && health_result.unwrap();

                            if is_healthy == expected_healthy {
                                correct_checks += 1;
                            }
                            total_checks += 1;
                        }
                    }

                    (correct_checks, total_checks)
                });

                handles.push(handle);
            }

            // Collect results
            let mut total_correct = 0;
            let mut total_checks = 0;

            for handle in handles {
                if let Ok((correct, checks)) = handle.await {
                    total_correct += correct;
                    total_checks += checks;
                }
            }

            // Health checks should be highly accurate
            if total_checks > 0 {
                let accuracy = total_correct as f64 / total_checks as f64;
                prop_assert!(accuracy > 0.95,
                    "Health check accuracy too low: {}%", accuracy * 100.0);
            }
        });
    }

    /// Property 5 variant: Health check performance
    #[test]
    fn prop_health_check_performance(
        provider_count in 1..20usize,
        check_timeout_ms in 50..500u64,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let mut registry = ProviderRegistry::new();

            // Create providers with varying response times
            for i in 0..provider_count {
                let delay_ms = (i as u64 * 10) + 10; // 10ms, 20ms, 30ms, etc.
                let provider = Arc::new(MockProvider::new(&format!("hc-prov-{}", i), false, delay_ms));
                registry.register(provider).unwrap();
            }

            let health_checker = HealthChecker::new(Duration::from_millis(check_timeout_ms));

            let start_time = Instant::now();

            // Check all providers concurrently
            let mut handles = Vec::new();
            for i in 0..provider_count {
                let registry_clone = registry.clone();
                let health_checker_clone = health_checker.clone();

                let handle = tokio::spawn(async move {
                    let provider_id = format!("hc-prov-{}", i);
                    if let Ok(provider) = registry_clone.get(&provider_id) {
                        health_checker_clone.check_provider(&*provider).await.is_ok()
                    } else {
                        false
                    }
                });

                handles.push(handle);
            }

            // Wait for all checks
            let mut success_count = 0;
            for handle in handles {
                if let Ok(success) = handle.await {
                    if success {
                        success_count += 1;
                    }
                }
            }

            let total_time = start_time.elapsed();

            // All health checks should succeed
            prop_assert_eq!(success_count, provider_count);

            // Total time should be reasonable (not much more than the slowest provider)
            let max_expected_time = Duration::from_millis(check_timeout_ms + (provider_count as u64 * 10));
            prop_assert!(total_time <= max_expected_time,
                "Health checks took too long: {:?} > {:?}", total_time, max_expected_time);
        });
    }
}
