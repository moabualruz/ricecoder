use ricecoder_providers::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

#[cfg(test)]
mod provider_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_curation_workflow_integration() {
        // Test the complete provider curation workflow
        let performance_monitor = Arc::new(ProviderPerformanceMonitor::default());
        let mut curator = ProviderCurator::default(performance_monitor.clone());

        // Set up provider models
        let mut provider_models = HashMap::new();
        provider_models.insert("openai".to_string(), vec![
            ModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                provider: "openai".to_string(),
                context_window: 8192,
                capabilities: vec![Capability::Chat, Capability::FunctionCalling],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.03,
                    output_per_1k_tokens: 0.06,
                }),
                is_free: false,
            }
        ]);

        provider_models.insert("anthropic".to_string(), vec![
            ModelInfo {
                id: "claude-3".to_string(),
                name: "Claude 3".to_string(),
                provider: "anthropic".to_string(),
                context_window: 100000,
                capabilities: vec![Capability::Chat, Capability::FunctionCalling, Capability::Vision],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.015,
                    output_per_1k_tokens: 0.075,
                }),
                is_free: false,
            }
        ]);

        // Update quality scores
        curator.update_quality_scores(&provider_models);

        // Record performance data
        performance_monitor.record_success("openai", 500, 100, 0.003);
        performance_monitor.record_success("openai", 600, 120, 0.0036);
        performance_monitor.record_success("anthropic", 800, 150, 0.00225);

        // Test provider selection
        let providers = vec!["openai".to_string(), "anthropic".to_string()];
        let best_provider = curator.select_best_provider(&providers, None);

        assert!(best_provider.is_some());
        let best = best_provider.unwrap();

        // Should select based on quality score (anthropic has better features)
        let openai_score = curator.get_quality_score("openai").unwrap();
        let anthropic_score = curator.get_quality_score("anthropic").unwrap();

        // Anthropic should have higher score due to better features and context window
        assert!(anthropic_score.overall >= openai_score.overall);
    }

    #[tokio::test]
    async fn test_community_contribution_validation_integration() {
        // Test community contribution submission and validation
        let registry = Arc::new(RwLock::new(CommunityProviderRegistry::new()));
        let config = CommunityDatabaseConfig::default();
        let sync = CommunityDatabaseSync::new(config, registry.clone());

        // Create a valid community contribution
        let contribution = CommunityProviderConfig {
            id: "".to_string(),
            provider_id: "test_provider".to_string(),
            name: "Test Provider".to_string(),
            description: "A test community provider".to_string(),
            base_url: Some("https://api.test-provider.com".to_string()),
            models: vec![
                ModelInfo {
                    id: "test-model".to_string(),
                    name: "Test Model".to_string(),
                    provider: "test_provider".to_string(),
                    context_window: 4096,
                    capabilities: vec![Capability::Chat],
                    pricing: Some(Pricing {
                        input_per_1k_tokens: 0.01,
                        output_per_1k_tokens: 0.02,
                    }),
                    is_free: false,
                }
            ],
            default_config: ProviderSettings {
                timeout: Some(Duration::from_secs(30)),
                retry_count: Some(3),
                rate_limit: Some(RateLimitSettings {
                    requests_per_minute: 60,
                    requests_per_hour: 1000,
                    burst_limit: 10,
                }),
                headers: HashMap::new(),
            },
            metadata: ContributionMetadata {
                contributor: "test_contributor".to_string(),
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                version: "1.0.0".to_string(),
                tags: vec!["test".to_string(), "experimental".to_string()],
                compatibility_notes: Some("Compatible with standard APIs".to_string()),
            },
            status: ContributionStatus::Pending,
            quality_metrics: None,
        };

        // Test validation
        let validator = ContributionValidator::new(ValidationRules::default());
        assert!(validator.validate(&contribution).is_ok());

        // Submit contribution
        let mut registry_guard = registry.write().await;
        let contribution_id = registry_guard.submit_contribution(contribution).unwrap();

        // Verify submission
        let pending = registry_guard.get_pending_contributions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, contribution_id);

        // Test review process
        let review = ContributionReview {
            id: "review_1".to_string(),
            contribution_id: contribution_id.clone(),
            reviewer: "reviewer_1".to_string(),
            reviewed_at: SystemTime::now(),
            decision: ContributionStatus::Approved,
            comments: "Good contribution with proper validation".to_string(),
            quality_score: Some(0.85),
            suggestions: vec![],
        };

        registry_guard.review_contribution(review).unwrap();

        // Verify approval
        let approved = registry_guard.get_all_approved_configs();
        assert_eq!(approved.len(), 1);
        assert_eq!(approved[0].provider_id, "test_provider");
    }

    #[tokio::test]
    async fn test_provider_analytics_monitoring_integration() {
        // Test provider analytics and monitoring integration
        let performance_monitor = Arc::new(ProviderPerformanceMonitor::default());
        let registry = Arc::new(RwLock::new(CommunityProviderRegistry::new()));

        // Record usage data
        let usage_data = vec![
            ProviderUsage {
                success: true,
                tokens_used: 100,
                cost: 0.002,
                response_time_ms: 500,
                model: "gpt-4".to_string(),
                error_type: None,
            },
            ProviderUsage {
                success: true,
                tokens_used: 150,
                cost: 0.003,
                response_time_ms: 600,
                model: "gpt-4".to_string(),
                error_type: None,
            },
            ProviderUsage {
                success: false,
                tokens_used: 50,
                cost: 0.001,
                response_time_ms: 300,
                model: "gpt-4".to_string(),
                error_type: Some("rate_limit".to_string()),
            },
        ];

        // Record analytics
        let mut registry_guard = registry.write().await;
        for usage in usage_data {
            registry_guard.record_usage("openai", usage);
        }

        // Record performance metrics
        performance_monitor.record_success("openai", 500, 100, 0.002);
        performance_monitor.record_success("openai", 600, 150, 0.003);
        performance_monitor.record_failure("openai", 300);

        // Test analytics retrieval
        let analytics = registry_guard.get_analytics("openai").unwrap();
        assert_eq!(analytics.total_requests, 3);
        assert_eq!(analytics.successful_requests, 2);
        assert_eq!(analytics.failed_requests, 1);
        assert_eq!(analytics.total_tokens, 300);
        assert!(analytics.avg_response_time_ms > 0.0);

        // Test performance monitoring
        let metrics = performance_monitor.get_metrics("openai").unwrap();
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 2);
        assert_eq!(metrics.failed_requests, 1);

        // Test performance summary
        let summary = performance_monitor.get_performance_summary();
        assert_eq!(summary.total_providers, 1);
        assert_eq!(summary.total_requests, 3);
        assert_eq!(summary.total_errors, 1);
        assert!(summary.avg_response_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_community_sync_validation_integration() {
        // Test community database synchronization and validation
        let registry = Arc::new(RwLock::new(CommunityProviderRegistry::new()));

        let config = CommunityDatabaseConfig {
            endpoints: vec!["https://test-endpoint.com".to_string()],
            sync_interval: Duration::from_secs(3600),
            trusted_sources: vec!["trusted_contributor".to_string()],
            auto_approve_trusted: true,
            validation_rules: ValidationRules {
                require_pricing: true,
                require_capabilities: true,
                max_models_per_provider: 10,
                min_quality_score: 0.7,
            },
        };

        let sync = CommunityDatabaseSync::new(config, registry.clone());

        // Test validation of configurations
        let valid_config = CommunityProviderConfig {
            id: "".to_string(),
            provider_id: "valid_provider".to_string(),
            name: "Valid Provider".to_string(),
            description: "A valid provider".to_string(),
            base_url: Some("https://api.valid.com".to_string()),
            models: vec![
                ModelInfo {
                    id: "model1".to_string(),
                    name: "Model 1".to_string(),
                    provider: "valid_provider".to_string(),
                    context_window: 4096,
                    capabilities: vec![Capability::Chat],
                    pricing: Some(Pricing {
                        input_per_1k_tokens: 0.01,
                        output_per_1k_tokens: 0.02,
                    }),
                    is_free: false,
                }
            ],
            default_config: ProviderSettings::default(),
            metadata: ContributionMetadata {
                contributor: "trusted_contributor".to_string(),
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                version: "1.0.0".to_string(),
                tags: vec![],
                compatibility_notes: None,
            },
            status: ContributionStatus::Pending,
            quality_metrics: None,
        };

        // Test validation
        let validator = ContributionValidator::new(sync.config.validation_rules.clone());
        assert!(validator.validate(&valid_config).is_ok());

        // Test invalid config (missing pricing)
        let invalid_config = CommunityProviderConfig {
            models: vec![
                ModelInfo {
                    id: "model1".to_string(),
                    name: "Model 1".to_string(),
                    provider: "invalid_provider".to_string(),
                    context_window: 4096,
                    capabilities: vec![Capability::Chat],
                    pricing: None, // Missing pricing
                    is_free: false,
                }
            ],
            ..valid_config
        };
        invalid_config.provider_id = "invalid_provider".to_string();

        assert!(validator.validate(&invalid_config).is_err());
    }

    #[tokio::test]
    async fn test_provider_security_compliance_integration() {
        // Test provider security headers and audit logging integration
        use tempfile::NamedTempFile;
        use std::io::Read;

        // Test security headers
        let mut builder = SecurityHeadersBuilder::new();

        // Add custom security headers
        builder
            .add_header("X-Custom-Security", "enabled")
            .add_header("X-API-Version", "v1");

        let headers = builder.build();

        // Validate security headers
        let validation_result = SecurityHeadersValidator::validate(&headers);
        assert!(validation_result.is_ok(), "Security headers validation failed: {:?}", validation_result);

        // Verify specific headers
        assert_eq!(headers.get("X-Frame-Options"), Some(&"DENY".to_string()));
        assert_eq!(headers.get("X-Content-Type-Options"), Some(&"nosniff".to_string()));
        assert_eq!(headers.get("Strict-Transport-Security"), Some(&"max-age=31536000; includeSubDomains".to_string()));

        // Test audit logging
        let temp_file = NamedTempFile::new().unwrap();
        let log_path = temp_file.path().to_path_buf();
        let logger = AuditLogger::new(log_path.clone());

        // Log various security events
        logger.log_api_key_access("openai", "user_123", "success").unwrap();
        logger.log_authentication_attempt("anthropic", "user_456", "success", "Valid credentials").unwrap();
        logger.log_rate_limit_exceeded("openai", "user_789", "Exceeded 60 requests per minute").unwrap();
        logger.log_security_error("providers", "system", "api_key", "Invalid key format").unwrap();

        // Verify logs were written
        let mut file_content = String::new();
        std::fs::File::open(&log_path).unwrap().read_to_string(&mut file_content).unwrap();

        let lines: Vec<&str> = file_content.lines().collect();
        assert_eq!(lines.len(), 4, "Expected 4 audit log entries");

        // Verify JSON structure
        for line in lines {
            let entry: AuditLogEntry = serde_json::from_str(line).unwrap();
            assert!(!entry.timestamp.is_empty());
            assert!(!entry.component.is_empty());
            assert!(!entry.actor.is_empty());
        }
    }

    #[tokio::test]
    async fn test_enterprise_compliance_monitoring_integration() {
        // Test enterprise compliance monitoring integration
        let performance_monitor = Arc::new(ProviderPerformanceMonitor::new(
            PerformanceThresholds {
                max_avg_response_time_ms: 2000,
                max_error_rate: 0.05,
                min_success_rate: 0.95,
                max_cost_per_request: 0.005,
            },
            Duration::from_secs(300),
        ));

        let registry = Arc::new(RwLock::new(CommunityProviderRegistry::new()));

        // Simulate enterprise usage patterns
        let mut registry_guard = registry.write().await;

        // Record high-volume usage
        for i in 0..100 {
            let success = i < 95; // 95% success rate
            let response_time = if success { 800 + (i % 400) } else { 1500 + (i % 500) };
            let tokens = 100 + (i % 200);
            let cost = tokens as f64 * 0.00002;

            let usage = ProviderUsage {
                success,
                tokens_used: tokens,
                cost,
                response_time_ms: response_time as u64,
                model: "enterprise-model".to_string(),
                error_type: if success { None } else { Some("timeout".to_string()) },
            };

            registry_guard.record_usage("enterprise_provider", usage);

            if success {
                performance_monitor.record_success("enterprise_provider", response_time as u64, tokens, cost);
            } else {
                performance_monitor.record_failure("enterprise_provider", response_time as u64);
            }
        }

        // Test compliance monitoring
        let analytics = registry_guard.get_analytics("enterprise_provider").unwrap();
        let metrics = performance_monitor.get_metrics("enterprise_provider").unwrap();

        // Verify enterprise compliance metrics
        assert_eq!(analytics.total_requests, 100);
        assert_eq!(analytics.successful_requests, 95);
        assert_eq!(analytics.failed_requests, 5);
        assert!(analytics.success_rate() >= 0.95, "Success rate below enterprise threshold");

        // Verify performance compliance
        assert!(metrics.is_performing_well(&performance_monitor.thresholds),
                "Provider not meeting enterprise performance thresholds");

        // Test cost monitoring
        let avg_cost = analytics.total_cost / analytics.total_requests as f64;
        assert!(avg_cost <= 0.005, "Average cost per request exceeds enterprise limit: {}", avg_cost);
    }

    #[tokio::test]
    async fn test_provider_update_synchronization_integration() {
        // Test provider update synchronization
        let registry = Arc::new(RwLock::new(CommunityProviderRegistry::new()));

        let mut registry_guard = registry.write().await;

        // Add provider updates
        let updates = vec![
            ProviderUpdate {
                id: "update_1".to_string(),
                provider_id: "openai".to_string(),
                update_type: UpdateType::NewModel,
                description: "Added GPT-4 Turbo model".to_string(),
                breaking_changes: false,
                required_actions: vec![],
                updated_at: SystemTime::now(),
                version: "4.0.0".to_string(),
            },
            ProviderUpdate {
                id: "update_2".to_string(),
                provider_id: "openai".to_string(),
                update_type: UpdateType::PricingChange,
                description: "Updated pricing for GPT-4 models".to_string(),
                breaking_changes: false,
                required_actions: vec!["Review API costs".to_string()],
                updated_at: SystemTime::now(),
                version: "4.1.0".to_string(),
            },
            ProviderUpdate {
                id: "update_3".to_string(),
                provider_id: "anthropic".to_string(),
                update_type: UpdateType::SecurityUpdate,
                description: "Security patch for API authentication".to_string(),
                breaking_changes: false,
                required_actions: vec!["Rotate API keys".to_string()],
                updated_at: SystemTime::now(),
                version: "1.2.0".to_string(),
            },
        ];

        for update in updates {
            registry_guard.add_update(update);
        }

        // Test update retrieval
        let openai_updates = registry_guard.get_updates("openai");
        assert_eq!(openai_updates.len(), 2);

        let anthropic_updates = registry_guard.get_updates("anthropic");
        assert_eq!(anthropic_updates.len(), 1);

        // Verify update types
        assert_eq!(openai_updates[0].update_type, UpdateType::NewModel);
        assert_eq!(openai_updates[1].update_type, UpdateType::PricingChange);
        assert_eq!(anthropic_updates[0].update_type, UpdateType::SecurityUpdate);

        // Test breaking changes filtering
        let breaking_updates: Vec<_> = openai_updates.iter()
            .filter(|u| u.breaking_changes)
            .collect();
        assert_eq!(breaking_updates.len(), 0, "No breaking changes expected in OpenAI updates");
    }

    #[tokio::test]
    async fn test_provider_quality_curation_integration() {
        // Test provider quality curation and optimization
        let performance_monitor = Arc::new(ProviderPerformanceMonitor::default());
        let mut curator = ProviderCurator::new(
            CurationConfig {
                min_quality_score: 0.7,
                min_reliability_score: 0.8,
                max_consecutive_failures: 3,
                reliability_window: Duration::from_secs(3600),
                auto_switch_enabled: true,
                cost_weight: 0.4,
                speed_weight: 0.3,
                reliability_weight: 0.3,
            },
            performance_monitor.clone(),
        );

        // Set up multiple providers with different characteristics
        let mut provider_models = HashMap::new();

        // High-quality provider
        provider_models.insert("premium_provider".to_string(), vec![
            ModelInfo {
                id: "premium-model".to_string(),
                name: "Premium Model".to_string(),
                provider: "premium_provider".to_string(),
                context_window: 128000,
                capabilities: vec![Capability::Chat, Capability::FunctionCalling, Capability::Vision, Capability::Streaming],
                    pricing: Some(Pricing {
                        input_per_1k_tokens: 0.01,
                        output_per_1k_tokens: 0.02,
                    }),
                is_free: false,
            }
        ]);

        // Budget provider
        provider_models.insert("budget_provider".to_string(), vec![
            ModelInfo {
                id: "budget-model".to_string(),
                name: "Budget Model".to_string(),
                provider: "budget_provider".to_string(),
                context_window: 4096,
                capabilities: vec![Capability::Chat],
                    pricing: Some(Pricing {
                        input_per_1k_tokens: 0.005,
                        output_per_1k_tokens: 0.01,
                    }),
                is_free: false,
            }
        ]);

        // Free provider
        provider_models.insert("free_provider".to_string(), vec![
            ModelInfo {
                id: "free-model".to_string(),
                name: "Free Model".to_string(),
                provider: "free_provider".to_string(),
                context_window: 2048,
                capabilities: vec![Capability::Chat],
                pricing: None,
                is_free: true,
            }
        ]);

        curator.update_quality_scores(&provider_models);

        // Record performance data
        performance_monitor.record_success("premium_provider", 300, 100, 0.004); // Fast, expensive
        performance_monitor.record_success("budget_provider", 800, 100, 0.0015); // Slower, cheaper
        performance_monitor.record_success("free_provider", 1500, 100, 0.0); // Slowest, free

        // Test quality scores
        let premium_score = curator.get_quality_score("premium_provider").unwrap();
        let budget_score = curator.get_quality_score("budget_provider").unwrap();
        let free_score = curator.get_quality_score("free_provider").unwrap();

        // Premium should have highest score due to features and performance
        assert!(premium_score.overall >= budget_score.overall);
        assert!(budget_score.overall >= free_score.overall);

        // Test provider selection with constraints
        let providers = vec![
            "premium_provider".to_string(),
            "budget_provider".to_string(),
            "free_provider".to_string(),
        ];

        // Select best overall
        let best = curator.select_best_provider(&providers, None).unwrap();
        assert_eq!(best, "premium_provider");

        // Select with cost constraint
        let cost_constraint = SelectionConstraints {
            max_cost_per_request: Some(0.002),
            ..SelectionConstraints::default()
        };
        let cost_effective = curator.select_best_provider(&providers, Some(&cost_constraint)).unwrap();
        assert_eq!(cost_effective, "budget_provider"); // Should select budget due to cost limit

        // Test reliability tracking
        curator.record_success("premium_provider");
        curator.record_success("premium_provider");
        curator.record_failure("budget_provider");
        curator.record_failure("budget_provider");

        let premium_reliability = curator.get_reliability_tracker("premium_provider").unwrap();
        let budget_reliability = curator.get_reliability_tracker("budget_provider").unwrap();

        assert_eq!(premium_reliability.reliability_score(), 1.0);
        assert_eq!(budget_reliability.reliability_score(), 0.0);
        assert!(!budget_reliability.should_avoid(&curator.config));
    }
}