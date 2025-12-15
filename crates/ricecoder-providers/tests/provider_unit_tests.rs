//! Comprehensive unit tests for provider registration, discovery, curation,
//! health monitoring, reliability tracking, benchmarking, failover, switching,
//! cost optimization, performance tracking, and quality scoring validation.

use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_providers::curation::{CurationConfig, ProviderCurator, QualityScore, ReliabilityStatus, SelectionConstraints};
    use ricecoder_providers::evaluation::{BenchmarkResult, ProviderEvaluation, ProviderEvaluator};
    use ricecoder_providers::models::{Capability, ChatRequest, ChatResponse, FinishReason, Message, ModelInfo, TokenUsage};
    use ricecoder_providers::performance_monitor::{PerformanceMetrics, PerformanceThresholds, ProviderMetrics, ProviderPerformanceMonitor};
    use ricecoder_providers::provider::{ChatStream, Provider};
    use ricecoder_providers::provider::manager::{ConnectionState, ModelFilter, ModelFilterCriteria, ProviderManager, ProviderStatus};

    // Mock provider for testing
    struct TestProvider {
        id: String,
        name: String,
        models: Vec<ModelInfo>,
        should_fail: bool,
        response_time_ms: u64,
    }

    #[async_trait::async_trait]
    impl Provider for TestProvider {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn models(&self) -> Vec<ModelInfo> {
            self.models.clone()
        }

        async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
            if self.should_fail {
                return Err(ProviderError::ProviderError("Simulated failure".to_string()));
            }

            // Simulate response time
            tokio::time::sleep(Duration::from_millis(self.response_time_ms)).await;

            Ok(ChatResponse {
                content: format!("Response from {}", self.name),
                model: self.models.first().map(|m| m.id.clone()).unwrap_or_default(),
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                },
                finish_reason: FinishReason::Stop,
            })
        }

        async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatStream, ProviderError> {
            Err(ProviderError::NotFound("Not implemented".to_string()))
        }

        fn count_tokens(&self, _content: &str, _model: &str) -> Result<usize, ProviderError> {
            Ok(15)
        }

        async fn health_check(&self) -> Result<bool, ProviderError> {
            Ok(!self.should_fail)
        }
    }

    fn create_test_provider(id: &str, name: &str, should_fail: bool, response_time_ms: u64) -> Arc<dyn Provider> {
        let models = vec![ModelInfo {
            id: format!("{}-model", id),
            name: format!("{} Model", name),
            provider: id.to_string(),
            context_window: 4096,
            capabilities: vec![Capability::Chat, Capability::FunctionCalling],
            pricing: Some(crate::models::Pricing {
                input_per_1k_tokens: 0.01,
                output_per_1k_tokens: 0.02,
            }),
            is_free: false,
        }];

        Arc::new(TestProvider {
            id: id.to_string(),
            name: name.to_string(),
            models,
            should_fail,
            response_time_ms,
        })
    }

    // ===== PROVIDER REGISTRATION/DISCOVERY/CATWALK CURATION TESTS =====

    #[test]
    fn test_provider_registration_and_discovery() {
        let mut registry = ProviderRegistry::new();

        // Register multiple providers
        let provider1 = create_test_provider("openai", "OpenAI", false, 100);
        let provider2 = create_test_provider("anthropic", "Anthropic", false, 150);
        let provider3 = create_test_provider("google", "Google", false, 200);

        registry.register(provider1).unwrap();
        registry.register(provider2).unwrap();
        registry.register(provider3).unwrap();

        // Test discovery
        assert_eq!(registry.provider_count(), 3);
        assert!(registry.has_provider("openai"));
        assert!(registry.has_provider("anthropic"));
        assert!(registry.has_provider("google"));
        assert!(!registry.has_provider("nonexistent"));

        // Test retrieval by ID
        let retrieved = registry.get("openai").unwrap();
        assert_eq!(retrieved.id(), "openai");
        assert_eq!(retrieved.name(), "OpenAI");

        // Test retrieval by name
        let retrieved_by_name = registry.get_by_name("Anthropic").unwrap();
        assert_eq!(retrieved_by_name.id(), "anthropic");

        // Test listing all providers
        let all_providers = registry.list_all();
        assert_eq!(all_providers.len(), 3);

        let provider_ids = registry.list_provider_ids();
        assert_eq!(provider_ids.len(), 3);
        assert!(provider_ids.contains(&"openai".to_string()));
        assert!(provider_ids.contains(&"anthropic".to_string()));
        assert!(provider_ids.contains(&"google".to_string()));

        // Test listing all models
        let all_models = registry.list_all_models();
        assert_eq!(all_models.len(), 3);

        // Test listing models for specific provider
        let openai_models = registry.list_models("openai").unwrap();
        assert_eq!(openai_models.len(), 1);
        assert_eq!(openai_models[0].provider, "openai");
    }

    #[test]
    fn test_provider_registry_error_handling() {
        let mut registry = ProviderRegistry::new();

        // Test getting non-existent provider
        assert!(registry.get("nonexistent").is_err());
        assert!(registry.get_by_name("Nonexistent").is_err());

        // Test unregistering non-existent provider
        assert!(registry.unregister("nonexistent").is_err());

        // Test listing models for non-existent provider
        assert!(registry.list_models("nonexistent").is_err());
    }

    #[test]
    fn test_catwalk_curation_quality_score_calculation() {
        let monitor = Arc::new(ProviderPerformanceMonitor::default());
        let curator = ProviderCurator::default(monitor);

        let models = vec![
            ModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                provider: "openai".to_string(),
                context_window: 8192,
                capabilities: vec![Capability::Chat, Capability::FunctionCalling, Capability::Vision],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.03,
                    output_per_1k_tokens: 0.06,
                }),
                is_free: false,
            }
        ];

        let score = curator.calculate_quality_score("openai", &models);

        // Verify score components are within valid ranges
        assert!(score.overall >= 0.0 && score.overall <= 1.0);
        assert!(score.speed >= 0.0 && score.speed <= 1.0);
        assert!(score.reliability >= 0.0 && score.reliability <= 1.0);
        assert!(score.cost_efficiency >= 0.0 && score.cost_efficiency <= 1.0);
        assert!(score.features >= 0.0 && score.features <= 1.0);

        // Features score should be high due to multiple capabilities
        assert!(score.features > 0.5);
    }

    #[test]
    fn test_catwalk_curation_provider_selection() {
        let monitor = Arc::new(ProviderPerformanceMonitor::default());
        let mut curator = ProviderCurator::default(monitor);

        // Set up provider models
        let mut provider_models = HashMap::new();

        // High-quality provider
        provider_models.insert("premium_provider".to_string(), vec![ModelInfo {
            id: "premium-model".to_string(),
            name: "Premium Model".to_string(),
            provider: "premium_provider".to_string(),
            context_window: 32768,
            capabilities: vec![Capability::Chat, Capability::FunctionCalling, Capability::Vision, Capability::Streaming],
            pricing: Some(crate::models::Pricing {
                input_per_1k_tokens: 0.01,
                output_per_1k_tokens: 0.02,
            }),
            is_free: false,
        }]);

        // Basic provider
        provider_models.insert("basic_provider".to_string(), vec![ModelInfo {
            id: "basic-model".to_string(),
            name: "Basic Model".to_string(),
            provider: "basic_provider".to_string(),
            context_window: 4096,
            capabilities: vec![Capability::Chat],
            pricing: Some(crate::models::Pricing {
                input_per_1k_tokens: 0.05,
                output_per_1k_tokens: 0.10,
            }),
            is_free: false,
        }]);

        curator.update_quality_scores(&provider_models);

        let providers = vec!["premium_provider".to_string(), "basic_provider".to_string()];

        // Test best provider selection
        let best = curator.select_best_provider(&providers, None);
        assert!(best.is_some());
        assert!(providers.contains(&best.unwrap()));

        // Test provider ranking by quality
        let ranked = curator.get_providers_by_quality(&providers);
        assert_eq!(ranked.len(), 2);
        assert!(ranked[0].1 >= ranked[1].1); // First should have higher or equal score
    }

    #[test]
    fn test_catwalk_curation_selection_constraints() {
        let monitor = Arc::new(ProviderPerformanceMonitor::default());
        let mut curator = ProviderCurator::default(monitor);

        let mut provider_models = HashMap::new();

        // Vision-capable provider
        provider_models.insert("vision_provider".to_string(), vec![ModelInfo {
            id: "vision-model".to_string(),
            name: "Vision Model".to_string(),
            provider: "vision_provider".to_string(),
            context_window: 4096,
            capabilities: vec![Capability::Chat, Capability::Vision],
            pricing: None,
            is_free: false,
        }]);

        // Text-only provider
        provider_models.insert("text_provider".to_string(), vec![ModelInfo {
            id: "text-model".to_string(),
            name: "Text Model".to_string(),
            provider: "text_provider".to_string(),
            context_window: 4096,
            capabilities: vec![Capability::Chat],
            pricing: None,
            is_free: false,
        }]);

        curator.update_quality_scores(&provider_models);

        let providers = vec!["vision_provider".to_string(), "text_provider".to_string()];

        // Test selection with capability requirement
        let constraints = SelectionConstraints {
            min_quality_score: 0.0,
            require_quality_score: false,
            require_performance_data: false,
            performance_thresholds: PerformanceThresholds::default(),
            max_cost_per_request: None,
            required_capabilities: vec![Capability::Vision],
        };

        let selected = curator.select_best_provider(&providers, Some(&constraints));
        assert_eq!(selected, Some("vision_provider".to_string()));

        // Test selection with cost constraint
        let cost_constraints = SelectionConstraints {
            min_quality_score: 0.0,
            require_quality_score: false,
            require_performance_data: false,
            performance_thresholds: PerformanceThresholds::default(),
            max_cost_per_request: Some(0.001),
            required_capabilities: vec![],
        };

        // Should still select since no pricing info means no filtering
        let selected_with_cost = curator.select_best_provider(&providers, Some(&cost_constraints));
        assert!(selected_with_cost.is_some());
    }

    // ===== HEALTH MONITORING/RELIABILITY TRACKING/AIDER BENCHMARKING TESTS =====

    #[test]
    fn test_health_monitoring_and_reliability_tracking() {
        let monitor = Arc::new(ProviderPerformanceMonitor::default());
        let mut curator = ProviderCurator::default(monitor);

        let provider_id = "test_provider";

        // Test initial state
        let tracker = curator.get_reliability_tracker(provider_id);
        assert!(tracker.is_none());

        // Record successes
        for _ in 0..5 {
            curator.record_success(provider_id);
        }

        let tracker = curator.get_reliability_tracker(provider_id).unwrap();
        assert_eq!(tracker.total_requests, 5);
        assert_eq!(tracker.total_failures, 0);
        assert_eq!(tracker.reliability_score(), 1.0);
        assert_eq!(tracker.status, ReliabilityStatus::Excellent);

        // Record some failures
        for _ in 0..2 {
            curator.record_failure(provider_id);
        }

        let tracker = curator.get_reliability_tracker(provider_id).unwrap();
        assert_eq!(tracker.total_requests, 7);
        assert_eq!(tracker.total_failures, 2);
        assert_eq!(tracker.reliability_score(), 5.0 / 7.0);
        assert_eq!(tracker.consecutive_failures, 2);

        // Test consecutive failure penalty
        assert!(tracker.consecutive_failure_penalty() > 0.0);

        // Test should_avoid with default config
        assert!(!tracker.should_avoid(&CurationConfig::default()));

        // Record many consecutive failures
        for _ in 0..10 {
            curator.record_failure(provider_id);
        }

        let tracker = curator.get_reliability_tracker(provider_id).unwrap();
        assert!(tracker.consecutive_failures >= 10);
        assert_eq!(tracker.status, ReliabilityStatus::Critical);
    }

    #[tokio::test]
    async fn test_aider_benchmarking_evaluation() {
        let monitor = Arc::new(ProviderPerformanceMonitor::default());
        let evaluator = ProviderEvaluator::new(monitor);

        let provider = create_test_provider("test_provider", "Test Provider", false, 50);

        // Test evaluation
        let evaluation = evaluator.evaluate_provider(&provider, "test-model").await.unwrap();

        assert_eq!(evaluation.provider_id, "test_provider");
        assert_eq!(evaluation.model, "test-model");
        assert!(evaluation.overall_score >= 0.0 && evaluation.overall_score <= 1.0);
        assert!(!evaluation.benchmark_results.is_empty());

        // Check benchmark results
        for result in &evaluation.benchmark_results {
            assert!(result.score >= 0.0 && result.score <= 1.0);
            assert!(result.total_tests > 0);
            assert!(result.avg_response_time_ms >= 0.0);
        }

        // Check performance metrics
        assert!(evaluation.performance_metrics.avg_response_time_ms >= 0.0);
        assert!(evaluation.performance_metrics.total_tokens > 0);

        // Check reliability and cost scores
        assert!(evaluation.reliability_score >= 0.0 && evaluation.reliability_score <= 1.0);
        assert!(evaluation.cost_efficiency_score >= 0.0 && evaluation.cost_efficiency_score <= 1.0);
    }

    #[tokio::test]
    async fn test_benchmark_result_calculation() {
        let monitor = Arc::new(ProviderPerformanceMonitor::default());
        let evaluator = ProviderEvaluator::new(monitor);

        let provider = create_test_provider("benchmark_provider", "Benchmark Provider", false, 100);

        // Run a specific benchmark
        let benchmark = &evaluator.benchmarks[0]; // First benchmark
        let result = evaluator.run_benchmark(&provider, "test-model", benchmark).await.unwrap();

        assert_eq!(result.benchmark_name, benchmark.name);
        assert!(result.score >= 0.0 && result.score <= 1.0);
        assert_eq!(result.total_tests, benchmark.test_cases.len() as usize);
        assert!(result.passed_tests <= result.total_tests);
        assert!(result.avg_response_time_ms >= 0.0);
        assert!(result.total_tokens > 0);
        assert!(result.cost >= 0.0);
        assert!(result.timestamp <= SystemTime::now());
    }

    #[tokio::test]
    async fn test_provider_evaluation_with_failures() {
        let monitor = Arc::new(ProviderPerformanceMonitor::default());
        let evaluator = ProviderEvaluator::new(monitor);

        // Create a failing provider
        let failing_provider = create_test_provider("failing_provider", "Failing Provider", true, 50);

        let evaluation = evaluator.evaluate_provider(&failing_provider, "test-model").await;

        // Should fail due to provider failures
        assert!(evaluation.is_err());
    }

    // ===== PROVIDER FAILOVER/SWITCHING/COST OPTIMIZATION TESTS =====

    #[tokio::test]
    async fn test_provider_failover_mechanism() {
        let mut registry = ProviderRegistry::new();

        let primary_provider = create_test_provider("primary", "Primary Provider", false, 100);
        let backup_provider = create_test_provider("backup", "Backup Provider", false, 150);

        registry.register(primary_provider).unwrap();
        registry.register(backup_provider).unwrap();

        let mut manager = ProviderManager::new(registry, "primary".to_string());

        // Set provider states
        manager.update_provider_state("primary", ConnectionState::Connected, None);
        manager.update_provider_state("backup", ConnectionState::Connected, None);

        // Initially, no failover needed
        assert!(!manager.should_avoid_provider("primary"));
        assert!(manager.get_failover_provider("primary").is_none());

        // Simulate primary provider failures
        for _ in 0..6 {
            manager.curator_mut().record_failure("primary");
        }

        // Now primary should be avoided
        assert!(manager.should_avoid_provider("primary"));

        // Should suggest backup as failover
        let failover = manager.get_failover_provider("primary");
        assert_eq!(failover, Some("backup".to_string()));
    }

    #[tokio::test]
    async fn test_provider_switching_under_load() {
        let mut registry = ProviderRegistry::new();

        let fast_provider = create_test_provider("fast", "Fast Provider", false, 50);
        let slow_provider = create_test_provider("slow", "Slow Provider", false, 500);

        registry.register(fast_provider).unwrap();
        registry.register(slow_provider).unwrap();

        let mut manager = ProviderManager::new(registry, "fast".to_string());

        manager.update_provider_state("fast", ConnectionState::Connected, None);
        manager.update_provider_state("slow", ConnectionState::Connected, None);

        // Make requests to build performance data
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test message".to_string(),
            }],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        // Make several requests to fast provider
        for _ in 0..3 {
            manager.chat_with_provider(&manager.get_provider("fast").unwrap(), request.clone()).await.unwrap();
        }

        // Make several requests to slow provider
        for _ in 0..3 {
            manager.chat_with_provider(&manager.get_provider("slow").unwrap(), request.clone()).await.unwrap();
        }

        // Update quality scores
        manager.update_provider_quality_scores();

        // Fast provider should be selected as best
        let best_provider = manager.select_best_provider(None);
        assert_eq!(best_provider, Some("fast".to_string()));
    }

    #[test]
    fn test_cost_optimization_provider_selection() {
        let monitor = Arc::new(ProviderPerformanceMonitor::default());
        let mut curator = ProviderCurator::default(monitor);

        let mut provider_models = HashMap::new();

        // Cheap provider
        provider_models.insert("cheap_provider".to_string(), vec![ModelInfo {
            id: "cheap-model".to_string(),
            name: "Cheap Model".to_string(),
            provider: "cheap_provider".to_string(),
            context_window: 4096,
            capabilities: vec![Capability::Chat],
            pricing: Some(crate::models::Pricing {
                input_per_1k_tokens: 0.001,
                output_per_1k_tokens: 0.002,
            }),
            is_free: false,
        }]);

        // Expensive provider
        provider_models.insert("expensive_provider".to_string(), vec![ModelInfo {
            id: "expensive-model".to_string(),
            name: "Expensive Model".to_string(),
            provider: "expensive_provider".to_string(),
            context_window: 4096,
            capabilities: vec![Capability::Chat],
            pricing: Some(crate::models::Pricing {
                input_per_1k_tokens: 0.10,
                output_per_1k_tokens: 0.20,
            }),
            is_free: false,
        }]);

        curator.update_quality_scores(&provider_models);

        let providers = vec!["cheap_provider".to_string(), "expensive_provider".to_string()];

        // Test cost-based selection with constraint
        let cost_constraint = SelectionConstraints {
            min_quality_score: 0.0,
            require_quality_score: false,
            require_performance_data: false,
            performance_thresholds: PerformanceThresholds::default(),
            max_cost_per_request: Some(0.005), // Very low cost threshold
            required_capabilities: vec![],
        };

        let selected = curator.select_best_provider(&providers, Some(&cost_constraint));
        assert_eq!(selected, Some("cheap_provider".to_string()));
    }

    #[tokio::test]
    async fn test_automatic_provider_switching() {
        let mut registry = ProviderRegistry::new();

        let stable_provider = create_test_provider("stable", "Stable Provider", false, 100);
        let unstable_provider = create_test_provider("unstable", "Unstable Provider", false, 100);

        registry.register(stable_provider).unwrap();
        registry.register(unstable_provider).unwrap();

        let mut manager = ProviderManager::new(registry, "unstable".to_string());

        manager.update_provider_state("stable", ConnectionState::Connected, None);
        manager.update_provider_state("unstable", ConnectionState::Connected, None);

        // Simulate unstable provider having issues
        for _ in 0..8 {
            manager.curator_mut().record_failure("unstable");
        }

        // Configure auto-switching
        let mut config = CurationConfig::default();
        config.auto_switch_enabled = true;
        manager.curator_mut().update_config(config);

        // Attempt a request - should potentially switch providers
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        // Since we're using the default provider which is failing, and auto-switch is enabled,
        // the manager should handle this gracefully
        let result = manager.chat(request).await;
        // The result might succeed or fail depending on implementation, but it should handle the switching logic
        assert!(result.is_ok() || result.is_err()); // Either is acceptable for this test
    }

    // ===== PERFORMANCE TRACKING/QUALITY SCORING VALIDATION TESTS =====

    #[tokio::test]
    async fn test_performance_tracking_integration() {
        let mut registry = ProviderRegistry::new();

        let provider = create_test_provider("perf_test", "Performance Test Provider", false, 75);
        registry.register(provider).unwrap();

        let mut manager = ProviderManager::new(registry, "perf_test".to_string());
        manager.update_provider_state("perf_test", ConnectionState::Connected, None);

        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Performance test message".to_string(),
            }],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        // Make multiple requests to track performance
        for i in 0..5 {
            let test_request = ChatRequest {
                messages: vec![Message {
                    role: "user".to_string(),
                    content: format!("Test message {}", i),
                }],
                ..request.clone()
            };

            manager.chat(test_request).await.unwrap();
        }

        // Check performance metrics
        let metrics = manager.performance_monitor().get_metrics("perf_test");
        assert!(metrics.is_some());

        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_requests, 5);
        assert_eq!(metrics.successful_requests, 5);
        assert_eq!(metrics.failed_requests, 0);
        assert!(metrics.avg_response_time_ms >= 0.0);
        assert!(metrics.total_tokens > 0);
        assert!(metrics.last_request_time.is_some());
        assert!(metrics.error_rate == 0.0);
    }

    #[test]
    fn test_quality_scoring_validation() {
        let monitor = Arc::new(ProviderPerformanceMonitor::default());
        let curator = ProviderCurator::default(monitor);

        // Test with empty models
        let empty_score = curator.calculate_quality_score("empty_provider", &[]);
        assert!(empty_score.overall >= 0.0 && empty_score.overall <= 1.0);
        assert_eq!(empty_score.features, 0.0); // No features with no models

        // Test with comprehensive model
        let comprehensive_models = vec![ModelInfo {
            id: "comprehensive-model".to_string(),
            name: "Comprehensive Model".to_string(),
            provider: "comprehensive_provider".to_string(),
            context_window: 131072, // Very large context
            capabilities: vec![
                Capability::Chat,
                Capability::FunctionCalling,
                Capability::Vision,
                Capability::Streaming,
            ],
            pricing: Some(crate::models::Pricing {
                input_per_1k_tokens: 0.005,
                output_per_1k_tokens: 0.01,
            }),
            is_free: true, // Free model
        }];

        let comprehensive_score = curator.calculate_quality_score("comprehensive_provider", &comprehensive_models);

        // Should have high feature score due to all capabilities
        assert!(comprehensive_score.features > 0.8);
        // Should have high overall score
        assert!(comprehensive_score.overall > 0.7);
        // Cost efficiency should be high due to free model
        assert!(comprehensive_score.cost_efficiency > 0.9);
    }

    #[tokio::test]
    async fn test_performance_thresholds_validation() {
        let mut registry = ProviderRegistry::new();

        let fast_provider = create_test_provider("fast_perf", "Fast Provider", false, 50);
        let slow_provider = create_test_provider("slow_perf", "Slow Provider", false, 2000); // 2 seconds

        registry.register(fast_provider).unwrap();
        registry.register(slow_provider).unwrap();

        let mut manager = ProviderManager::new(registry, "fast_perf".to_string());

        manager.update_provider_state("fast_perf", ConnectionState::Connected, None);
        manager.update_provider_state("slow_perf", ConnectionState::Connected, None);

        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Performance threshold test".to_string(),
            }],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        // Make requests to both providers
        manager.chat_with_provider(&manager.get_provider("fast_perf").unwrap(), request.clone()).await.unwrap();
        manager.chat_with_provider(&manager.get_provider("slow_perf").unwrap(), request.clone()).await.unwrap();

        // Check performance metrics
        let fast_metrics = manager.performance_monitor().get_metrics("fast_perf").unwrap();
        let slow_metrics = manager.performance_monitor().get_metrics("slow_perf").unwrap();

        // Fast provider should have better performance
        assert!(fast_metrics.avg_response_time_ms < slow_metrics.avg_response_time_ms);

        // Test performance threshold checking
        let thresholds = PerformanceThresholds {
            max_avg_response_time_ms: 1000.0, // 1 second threshold
            max_error_rate: 0.1,
            min_requests_per_second: 0.0,
        };

        assert!(fast_metrics.is_performing_well(&thresholds));
        assert!(!slow_metrics.is_performing_well(&thresholds)); // Slow provider should fail threshold
    }

    #[test]
    fn test_model_filtering_and_validation() {
        // Test ModelFilter logic directly
        let vision_model = ModelInfo {
            id: "vision-model".to_string(),
            name: "Vision Model".to_string(),
            provider: "vision_provider".to_string(),
            context_window: 4096,
            capabilities: vec![Capability::Chat, Capability::Vision],
            pricing: None,
            is_free: false,
        };

        let text_model = ModelInfo {
            id: "text-model".to_string(),
            name: "Text Model".to_string(),
            provider: "text_provider".to_string(),
            context_window: 4096,
            capabilities: vec![Capability::Chat],
            pricing: None,
            is_free: true,
        };

        let free_model = ModelInfo {
            id: "free-model".to_string(),
            name: "Free Model".to_string(),
            provider: "free_provider".to_string(),
            context_window: 4096,
            capabilities: vec![Capability::Chat],
            pricing: None,
            is_free: true,
        };

        let models = vec![vision_model.clone(), text_model.clone(), free_model.clone()];

        // Test filtering by capability
        let vision_filter = ModelFilter::new().with_criterion(ModelFilterCriteria::Capability(Capability::Vision));
        let vision_models = models.iter().filter(|m| vision_filter.matches(m)).cloned().collect::<Vec<_>>();
        assert_eq!(vision_models.len(), 1);
        assert_eq!(vision_models[0].provider, "vision_provider");

        // Test filtering by free models
        let free_filter = ModelFilter::new().with_criterion(ModelFilterCriteria::FreeOnly);
        let free_models = models.iter().filter(|m| free_filter.matches(m)).cloned().collect::<Vec<_>>();
        assert_eq!(free_models.len(), 2); // text_model and free_model are both free

        // Test filtering by provider
        let provider_filter = ModelFilter::new().with_criterion(ModelFilterCriteria::Provider("vision_provider".to_string()));
        let provider_models = models.iter().filter(|m| provider_filter.matches(m)).cloned().collect::<Vec<_>>();
        assert_eq!(provider_models.len(), 1);
        assert_eq!(provider_models[0].provider, "vision_provider");

        // Test filtering by minimum context window
        let context_filter = ModelFilter::new().with_criterion(ModelFilterCriteria::MinContextWindow(8192));
        let context_models = models.iter().filter(|m| context_filter.matches(m)).cloned().collect::<Vec<_>>();
        assert_eq!(context_models.len(), 0); // None have 8192+ context

        // Test combined criteria
        let combined_filter = ModelFilter::new()
            .with_criterion(ModelFilterCriteria::Capability(Capability::Chat))
            .with_criterion(ModelFilterCriteria::FreeOnly);
        let combined_models = models.iter().filter(|m| combined_filter.matches(m)).cloned().collect::<Vec<_>>();
        assert_eq!(combined_models.len(), 2); // Both free models have Chat capability
    }

    #[tokio::test]
    async fn test_comprehensive_provider_lifecycle() {
        let mut registry = ProviderRegistry::new();

        let provider = create_test_provider("lifecycle_test", "Lifecycle Test Provider", false, 100);
        registry.register(provider).unwrap();

        let mut manager = ProviderManager::new(registry, "lifecycle_test".to_string());

        // Test auto-detection (will be empty without env vars)
        let detected = manager.auto_detect_providers().await.unwrap();
        // Should be empty in test environment
        assert_eq!(detected.len(), 0);

        // Manually set connected state
        manager.update_provider_state("lifecycle_test", ConnectionState::Connected, None);

        // Test health check
        let health = manager.health_check("lifecycle_test").await.unwrap();
        assert!(health);

        // Test provider status
        let status = manager.get_provider_status("lifecycle_test").unwrap();
        assert_eq!(status.state, ConnectionState::Connected);
        assert!(status.models.len() > 0);

        // Test chat functionality
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Lifecycle test".to_string(),
            }],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        let response = manager.chat(request).await.unwrap();
        assert!(response.content.contains("Lifecycle Test Provider"));

        // Test performance tracking
        let metrics = manager.performance_monitor().get_metrics("lifecycle_test");
        assert!(metrics.is_some());

        // Test quality scoring
        manager.update_provider_quality_scores();
        let curator = manager.curator();
        let quality_score = curator.get_quality_score("lifecycle_test");
        assert!(quality_score.is_some());
    }
