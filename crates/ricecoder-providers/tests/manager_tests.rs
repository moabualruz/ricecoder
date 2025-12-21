use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChatResponse, FinishReason, TokenUsage};
    use crate::provider::ChatStream;
    use std::sync::Arc;

    struct MockProvider {
        id: String,
    }

    #[async_trait::async_trait]
    impl Provider for MockProvider {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            "Mock"
        }

        fn models(&self) -> Vec<crate::models::ModelInfo> {
            vec![]
        }

        async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
            Ok(ChatResponse {
                content: "test response".to_string(),
                model: "test-model".to_string(),
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                finish_reason: FinishReason::Stop,
            })
        }

        async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatStream, ProviderError> {
            Err(ProviderError::NotFound("Not implemented".to_string()))
        }

        fn count_tokens(&self, _content: &str, _model: &str) -> Result<usize, ProviderError> {
            Ok(0)
        }

        async fn health_check(&self) -> Result<bool, ProviderError> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
        });
        registry.register(provider).unwrap();

        let mut manager = ProviderManager::new(registry, "test".to_string());
        assert!(manager.default_provider().is_ok());
    }

    #[tokio::test]
    async fn test_chat_request() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
        });
        registry.register(provider).unwrap();

        let mut manager = ProviderManager::new(registry, "test".to_string());
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        let response = manager.chat(request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
        });
        registry.register(provider).unwrap();

        let mut manager = ProviderManager::new(registry, "test".to_string());
        let health = manager.health_check("test").await;
        assert!(health.is_ok());
        assert!(health.unwrap());
    }

    #[tokio::test]
    async fn test_performance_monitoring_integration() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
        });
        registry.register(provider).unwrap();

        let mut manager = ProviderManager::new(registry, "test".to_string());
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        // Make a successful request
        let response = manager.chat(request.clone()).await;
        assert!(response.is_ok());

        // Check that performance metrics were recorded
        let metrics = manager.performance_monitor().get_metrics("test");
        assert!(metrics.is_some());

        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.failed_requests, 0);
        assert_eq!(metrics.total_tokens, 15); // From MockProvider
        assert!(metrics.avg_response_time_ms >= 0.0);
        assert!(metrics.last_request_time.is_some());
    }

    #[tokio::test]
    async fn test_performance_monitoring_failure() {
        struct FailingProvider {
            id: String,
        }

        #[async_trait::async_trait]
        impl Provider for FailingProvider {
            fn id(&self) -> &str {
                &self.id
            }

            fn name(&self) -> &str {
                "Failing Mock"
            }

            fn models(&self) -> Vec<crate::models::ModelInfo> {
                vec![]
            }

            async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
                Err(ProviderError::ProviderError(
                    "Simulated failure".to_string(),
                ))
            }

            async fn chat_stream(
                &self,
                _request: ChatRequest,
            ) -> Result<ChatStream, ProviderError> {
                Err(ProviderError::NotFound("Not implemented".to_string()))
            }

            fn count_tokens(&self, _content: &str, _model: &str) -> Result<usize, ProviderError> {
                Ok(0)
            }

            async fn health_check(&self) -> Result<bool, ProviderError> {
                Ok(true)
            }
        }

        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(FailingProvider {
            id: "failing".to_string(),
        });
        registry.register(provider).unwrap();

        let mut manager = ProviderManager::new(registry, "failing".to_string());
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        // Make a failing request
        let response = manager.chat(request).await;
        assert!(response.is_err());

        // Check that failure was recorded
        let metrics = manager.performance_monitor().get_metrics("failing");
        assert!(metrics.is_some());

        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 0);
        assert_eq!(metrics.failed_requests, 1);
        assert!(metrics.error_rate > 0.0);
    }

    #[tokio::test]
    async fn test_performance_summary() {
        let mut registry = ProviderRegistry::new();

        // Register two providers
        let provider1 = Arc::new(MockProvider {
            id: "test1".to_string(),
        });
        let provider2 = Arc::new(MockProvider {
            id: "test2".to_string(),
        });
        registry.register(provider1).unwrap();
        registry.register(provider2).unwrap();

        let mut manager = ProviderManager::new(registry, "test1".to_string());

        // Make requests to both providers
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        manager.chat(request.clone()).await.unwrap();
        manager
            .chat_with_provider(&manager.get_provider("test2").unwrap(), request)
            .await
            .unwrap();

        // Get performance summary
        let summary = manager.performance_monitor().get_performance_summary();
        assert_eq!(summary.total_providers, 2);
        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.total_errors, 0);
        assert!(summary.avg_response_time_ms >= 0.0);
    }

    #[tokio::test]
    async fn test_provider_curation_integration() {
        let mut registry = ProviderRegistry::new();

        // Register two providers
        let provider1 = Arc::new(MockProvider {
            id: "provider1".to_string(),
        });
        let provider2 = Arc::new(MockProvider {
            id: "provider2".to_string(),
        });
        registry.register(provider1).unwrap();
        registry.register(provider2).unwrap();

        let mut manager = ProviderManager::new(registry, "provider1".to_string());

        // Manually set provider states to connected (since auto-detection requires env vars)
        manager.update_provider_state("provider1", ConnectionState::Connected, None);
        manager.update_provider_state("provider2", ConnectionState::Connected, None);

        // Simulate some usage
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        // Make requests to build performance data
        manager.chat(request.clone()).await.unwrap();
        manager
            .chat_with_provider(&manager.get_provider("provider2").unwrap(), request.clone())
            .await
            .unwrap();

        // Update quality scores
        manager.update_provider_quality_scores();

        // Test provider selection
        let best_provider = manager.select_best_provider(None);
        assert!(best_provider.is_some());

        // Test quality score retrieval
        let curator = manager.curator();
        let scores = curator.get_all_quality_scores();
        assert_eq!(scores.len(), 2);
        assert!(scores.contains_key("provider1"));
        assert!(scores.contains_key("provider2"));
    }

    #[tokio::test]
    async fn test_provider_failover() {
        let mut registry = ProviderRegistry::new();

        // Register providers
        let provider1 = Arc::new(MockProvider {
            id: "provider1".to_string(),
        });
        let provider2 = Arc::new(MockProvider {
            id: "provider2".to_string(),
        });
        registry.register(provider1).unwrap();
        registry.register(provider2).unwrap();

        let mut manager = ProviderManager::new(registry, "provider1".to_string());

        // Set provider states
        manager.update_provider_state("provider1", ConnectionState::Connected, None);
        manager.update_provider_state("provider2", ConnectionState::Connected, None);

        // Simulate provider1 having reliability issues
        for _ in 0..6 {
            manager.curator_mut().record_failure("provider1");
        }

        // Check if provider1 should be avoided
        assert!(manager.should_avoid_provider("provider1"));

        // Test failover selection
        let failover = manager.get_failover_provider("provider1");
        assert_eq!(failover, Some("provider2".to_string()));
    }

    #[tokio::test]
    async fn test_model_based_provider_selection() {
        use crate::models::Capability;

        let mut registry = ProviderRegistry::new();

        // Mock provider with vision capability
        struct VisionProvider {
            id: String,
        }

        #[async_trait::async_trait]
        impl Provider for VisionProvider {
            fn id(&self) -> &str {
                &self.id
            }

            fn name(&self) -> &str {
                "Vision Provider"
            }

            fn models(&self) -> Vec<crate::models::ModelInfo> {
                vec![ModelInfo {
                    id: "vision-model".to_string(),
                    name: "Vision Model".to_string(),
                    provider: self.id.clone(),
                    context_window: 4096,
                    capabilities: vec![Capability::Chat, Capability::Vision],
                    pricing: None,
                    is_free: false,
                }]
            }

            async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
                Ok(ChatResponse {
                    content: "vision response".to_string(),
                    model: "vision-model".to_string(),
                    usage: TokenUsage {
                        prompt_tokens: 10,
                        completion_tokens: 5,
                        total_tokens: 15,
                    },
                    finish_reason: FinishReason::Stop,
                })
            }

            async fn chat_stream(
                &self,
                _request: ChatRequest,
            ) -> Result<ChatStream, ProviderError> {
                Err(ProviderError::NotFound("Not implemented".to_string()))
            }

            fn count_tokens(&self, _content: &str, _model: &str) -> Result<usize, ProviderError> {
                Ok(0)
            }

            async fn health_check(&self) -> Result<bool, ProviderError> {
                Ok(true)
            }
        }

        let vision_provider = Arc::new(VisionProvider {
            id: "vision_provider".to_string(),
        });
        let regular_provider = Arc::new(MockProvider {
            id: "regular_provider".to_string(),
        });

        registry.register(vision_provider).unwrap();
        registry.register(regular_provider).unwrap();

        let mut manager = ProviderManager::new(registry, "regular_provider".to_string());

        // Set provider states
        manager.update_provider_state("vision_provider", ConnectionState::Connected, None);
        manager.update_provider_state("regular_provider", ConnectionState::Connected, None);

        // Select provider for vision capability
        let vision_provider_id =
            manager.select_best_provider_for_model(&[Capability::Vision], None);
        assert_eq!(vision_provider_id, Some("vision_provider".to_string()));

        // Select provider for regular chat (should work with any)
        let chat_provider_id = manager.select_best_provider_for_model(&[Capability::Chat], None);
        assert!(chat_provider_id.is_some());
    }

    #[tokio::test]
    async fn test_community_analytics_integration() {
        let mut registry = ProviderRegistry::new();

        let provider = Arc::new(MockProvider {
            id: "analytics_provider".to_string(),
        });
        registry.register(provider).unwrap();

        let mut manager = ProviderManager::new(registry, "analytics_provider".to_string());
        manager.update_provider_state("analytics_provider", ConnectionState::Connected, None);

        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        // Make a successful request
        manager.chat(request).await.unwrap();

        // Check that analytics were recorded
        let analytics = manager.get_provider_analytics("analytics_provider");
        assert!(analytics.is_some());

        let analytics = analytics.unwrap();
        assert_eq!(analytics.total_requests, 1);
        assert_eq!(analytics.successful_requests, 1);
        assert_eq!(analytics.total_tokens, 15);
        assert!(analytics.avg_response_time_ms >= 0.0);
    }

    #[tokio::test]
    async fn test_community_config_submission() {
        let registry = ProviderRegistry::new();
        let mut manager = ProviderManager::new(registry, "test".to_string());

        let config = CommunityProviderConfig {
            id: "".to_string(),
            provider_id: "community_provider".to_string(),
            name: "Community Provider".to_string(),
            description: "A community-contributed provider".to_string(),
            base_url: Some("https://community.api.com".to_string()),
            models: vec![],
            default_config: crate::community::ProviderSettings {
                timeout: Some(std::time::Duration::from_secs(30)),
                retry_count: Some(3),
                rate_limit: None,
                headers: std::collections::HashMap::new(),
            },
            metadata: crate::community::ContributionMetadata {
                contributor: "community_user".to_string(),
                created_at: std::time::SystemTime::now(),
                updated_at: std::time::SystemTime::now(),
                version: "1.0.0".to_string(),
                tags: vec!["community".to_string()],
                compatibility_notes: None,
            },
            status: crate::community::ContributionStatus::Pending,
            quality_metrics: None,
        };

        let contribution_id = manager.submit_community_config(config).unwrap();
        assert!(contribution_id.starts_with("contrib_community_provider_"));

        // Check that it was added to pending contributions
        let pending = manager.community_registry().get_pending_contributions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].provider_id, "community_provider");
    }

    #[tokio::test]
    async fn test_popular_providers_community() {
        let registry = ProviderRegistry::new();
        let mut manager = ProviderManager::new(registry, "test".to_string());

        // Simulate usage data by directly recording analytics
        manager.community_registry_mut().record_usage(
            "provider_x",
            ProviderUsage {
                success: true,
                tokens_used: 100,
                cost: 0.01,
                response_time_ms: 500,
                model: "model1".to_string(),
                error_type: None,
            },
        );
        manager.community_registry_mut().record_usage(
            "provider_x",
            ProviderUsage {
                success: true,
                tokens_used: 200,
                cost: 0.02,
                response_time_ms: 600,
                model: "model1".to_string(),
                error_type: None,
            },
        );
        manager.community_registry_mut().record_usage(
            "provider_y",
            ProviderUsage {
                success: true,
                tokens_used: 50,
                cost: 0.005,
                response_time_ms: 300,
                model: "model2".to_string(),
                error_type: None,
            },
        );

        let popular = manager.get_popular_providers(2);
        assert_eq!(popular.len(), 2);
        assert_eq!(popular[0], ("provider_x".to_string(), 2));
        assert_eq!(popular[1], ("provider_y".to_string(), 1));
    }
}
