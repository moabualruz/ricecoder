//! Property-based tests for provider connection state consistency
//! **Feature: ricecoder-providers, Property 13: Provider Connection State Consistency**
//! **Validates: Requirements 27.1**

use proptest::prelude::*;
use ricecoder_providers::models::{FinishReason, Message};
use ricecoder_providers::{
    ChatRequest, ChatResponse, ConnectionState, HealthCheckCache, ModelInfo, Provider, ProviderError,
    ProviderManager, ProviderRegistry, ProviderStatus, TokenUsage,
};
use std::sync::Arc;
use std::time::Duration;

/// Mock provider that can change availability state
struct StatefulMockProvider {
    id: String,
    name: String,
    available: bool,
    models: Vec<ModelInfo>,
}

#[async_trait::async_trait]
impl Provider for StatefulMockProvider {
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
        if self.available {
            Ok(ChatResponse {
                content: "Response".to_string(),
                model: request.model,
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                finish_reason: FinishReason::Stop,
            })
        } else {
            Err(ProviderError::ProviderError(
                "Provider unavailable".to_string(),
            ))
        }
    }
}

proptest! {
    /// Property 13: Provider Connection State Consistency
    /// For any sequence of connect/disconnect operations, the connection state should accurately reflect the actual connection status.
    /// **Validates: Requirements 27.1**
    #[test]
    fn prop_provider_connection_state_consistency(
        operations in prop::collection::vec(
            prop_oneof![
                Just("connect".to_string()),
                Just("disconnect".to_string()),
                Just("error".to_string()),
            ],
            1..20
        )
    ) {
        // Create a mock provider
        let provider = Arc::new(StatefulMockProvider {
            id: "test_provider".to_string(),
            name: "Test Provider".to_string(),
            available: true,
            models: vec![ModelInfo {
                id: "test-model".to_string(),
                name: "Test Model".to_string(),
                provider: "test_provider".to_string(),
                context_window: 4096,
                capabilities: vec![],
                pricing: None,
                is_free: true,
            }],
        });

        // Create registry and manager
        let mut registry = ProviderRegistry::new();
        registry.register(provider.clone()).unwrap();

        let manager = ProviderManager::new(registry, "test_provider".to_string());

        // Track expected state
        let mut expected_connected = true; // Starts as available

        // Execute operations and verify state consistency
        for operation in operations {
            match operation.as_str() {
                "connect" => {
                    // Simulate provider becoming available
                    expected_connected = true;
                    // In a real scenario, this would be detected by auto-detection
                    // For this test, we manually update the state
                }
                "disconnect" => {
                    // Simulate provider becoming unavailable
                    expected_connected = false;
                }
                "error" => {
                    // Simulate provider encountering an error
                    expected_connected = false;
                }
                _ => {}
            }

            // Update the mock provider's availability
            unsafe {
                let provider_ptr = Arc::as_ptr(&provider) as *mut StatefulMockProvider;
                (*provider_ptr).available = expected_connected;
            }

            // Test a simple operation to verify state
            let request = ChatRequest {
                model: "test-model".to_string(),
                messages: vec![Message {
                    role: "user".to_string(),
                    content: "test".to_string(),
                }],
                temperature: None,
                max_tokens: None,
                stream: false,
            };

            let result = futures::executor::block_on(manager.chat_with_provider(&provider, request));

            // Verify that the result matches the expected state
            if expected_connected {
                prop_assert!(result.is_ok(), "Provider should succeed when connected");
            } else {
                prop_assert!(result.is_err(), "Provider should fail when disconnected");
            }
        }
    }

    /// Test provider status tracking consistency
    #[test]
    fn prop_provider_status_tracking(
        state_changes in prop::collection::vec(
            prop_oneof![
                Just(ConnectionState::Connected),
                Just(ConnectionState::Disconnected),
                Just(ConnectionState::Error),
                Just(ConnectionState::Disabled),
            ],
            1..10
        )
    ) {
        // Create a mock provider
        let provider = Arc::new(StatefulMockProvider {
            id: "status_test_provider".to_string(),
            name: "Status Test Provider".to_string(),
            available: true,
            models: vec![],
        });

        // Create registry and manager
        let mut registry = ProviderRegistry::new();
        registry.register(provider).unwrap();

        let mut manager = ProviderManager::new(registry, "status_test_provider".to_string());

        // Apply state changes and verify tracking
        for (i, new_state) in state_changes.into_iter().enumerate() {
            let error_msg = if new_state == ConnectionState::Error {
                Some(format!("Error {}", i))
            } else {
                None
            };

            manager.update_provider_state("status_test_provider", new_state, error_msg.clone());

            // Verify the status was updated correctly
            if let Some(status) = manager.get_provider_status("status_test_provider") {
                prop_assert_eq!(status.state, new_state, "State should be updated correctly");
                prop_assert_eq!(status.error_message, error_msg, "Error message should match");
                prop_assert!(status.last_checked.is_some(), "Last checked should be set");
            } else {
                prop_assert!(false, "Provider status should exist after update");
            }
        }

        // Verify all statuses can be retrieved
        let all_statuses = manager.get_all_provider_statuses();
        prop_assert_eq!(all_statuses.len(), 1, "Should have one provider status");
    }
}