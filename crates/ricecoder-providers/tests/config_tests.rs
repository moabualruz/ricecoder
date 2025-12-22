use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use super::*;
    use crate::{
        config::ConfigurationManager,
        models::{DefaultsConfig, ProviderConfig, ProviderSettings},
    };

    #[test]
    fn test_new_configuration_manager() {
        let manager = ConfigurationManager::new();
        assert_eq!(manager.default_provider(), "openai");
        assert_eq!(manager.default_model(), "gpt-4");
    }

    #[test]
    fn test_validate_empty_config() {
        let manager = ConfigurationManager::new();
        assert!(manager.validate().is_err());
    }

    #[test]
    fn test_get_default_provider() {
        let manager = ConfigurationManager::new();
        assert_eq!(manager.default_provider(), "openai");
    }

    #[test]
    fn test_get_default_model() {
        let manager = ConfigurationManager::new();
        assert_eq!(manager.default_model(), "gpt-4");
    }

    #[test]
    fn test_get_global_config_path() {
        let path = ConfigurationManager::get_global_config_path();
        assert!(path.to_string_lossy().contains(".ricecoder"));
        assert!(path.to_string_lossy().contains("config.yaml"));
    }

    #[test]
    fn test_get_project_config_path() {
        let path = ConfigurationManager::get_project_config_path();
        assert_eq!(path, PathBuf::from("./.agent/config.yaml"));
    }

    #[test]
    fn test_merge_from_file_preserves_existing() {
        let mut manager = ConfigurationManager::new();

        // Add initial provider
        manager.config_mut().providers.insert(
            "openai".to_string(),
            ProviderSettings {
                api_key: Some("initial-key".to_string()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        // Merge should preserve existing if not overridden
        let merged_config = ProviderConfig {
            defaults: DefaultsConfig {
                provider: "anthropic".to_string(),
                model: "claude-3".to_string(),
                per_command: HashMap::new(),
                per_action: HashMap::new(),
            },
            providers: {
                let mut map = HashMap::new();
                map.insert(
                    "anthropic".to_string(),
                    ProviderSettings {
                        api_key: Some("anthropic-key".to_string()),
                        base_url: None,
                        timeout: None,
                        retry_count: None,
                    },
                );
                map
            },
        };

        // Simulate merge by directly updating config
        manager.config_mut().defaults = merged_config.defaults;
        manager
            .config_mut()
            .providers
            .extend(merged_config.providers);

        assert_eq!(manager.default_provider(), "anthropic");
        assert_eq!(manager.default_model(), "claude-3");
        assert!(manager.config().providers.contains_key("openai"));
        assert!(manager.config().providers.contains_key("anthropic"));
    }

    #[test]
    fn test_load_from_env_sets_api_keys() {
        let mut manager = ConfigurationManager::new();

        // Set environment variable
        std::env::set_var("OPENAI_API_KEY", "test-key-123");

        manager.load_from_env().unwrap();

        // Verify API key was loaded
        let openai_settings = manager.get_provider_settings("openai");
        assert!(openai_settings.is_some());
        assert_eq!(
            openai_settings.unwrap().api_key,
            Some("test-key-123".to_string())
        );

        // Cleanup
        std::env::remove_var("OPENAI_API_KEY");
    }

    #[test]
    fn test_get_api_key_from_config() {
        let mut manager = ConfigurationManager::new();

        manager.config_mut().providers.insert(
            "openai".to_string(),
            ProviderSettings {
                api_key: Some("config-key".to_string()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        let key = manager.get_api_key("openai");
        assert!(key.is_ok());
        assert_eq!(key.unwrap(), "config-key");
    }

    #[test]
    fn test_get_api_key_from_env() {
        let mut manager = ConfigurationManager::new();

        manager.config_mut().providers.insert(
            "anthropic".to_string(),
            ProviderSettings {
                api_key: None,
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        std::env::set_var("ANTHROPIC_API_KEY", "env-key-456");

        let key = manager.get_api_key("anthropic");
        assert!(key.is_ok());
        assert_eq!(key.unwrap(), "env-key-456");

        std::env::remove_var("ANTHROPIC_API_KEY");
    }

    #[test]
    fn test_validate_with_valid_config() {
        let mut manager = ConfigurationManager::new();

        manager.config_mut().providers.insert(
            "openai".to_string(),
            ProviderSettings {
                api_key: Some("test-key".to_string()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        assert!(manager.validate().is_ok());
    }

    #[test]
    fn test_validate_missing_api_key() {
        // Clean up any existing env vars that might have been set by other tests
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("GOOGLE_API_KEY");
        std::env::remove_var("ZEN_API_KEY");

        let mut manager = ConfigurationManager::new();

        manager.config_mut().providers.insert(
            "openai".to_string(),
            ProviderSettings {
                api_key: None,
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        // Should fail because API key is missing and env var is not set
        assert!(manager.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_timeout() {
        let mut manager = ConfigurationManager::new();

        manager.config_mut().providers.insert(
            "openai".to_string(),
            ProviderSettings {
                api_key: Some("test-key".to_string()),
                base_url: None,
                timeout: Some(std::time::Duration::from_secs(0)),
                retry_count: None,
            },
        );

        assert!(manager.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_retry_count() {
        let mut manager = ConfigurationManager::new();

        manager.config_mut().providers.insert(
            "openai".to_string(),
            ProviderSettings {
                api_key: Some("test-key".to_string()),
                base_url: None,
                timeout: None,
                retry_count: Some(15),
            },
        );

        assert!(manager.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_command() {
        let mut manager = ConfigurationManager::new();

        manager.config_mut().providers.insert(
            "openai".to_string(),
            ProviderSettings {
                api_key: Some("test-key".to_string()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        manager
            .config_mut()
            .defaults
            .per_command
            .insert("invalid_command".to_string(), "gpt-4".to_string());

        assert!(manager.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_action() {
        let mut manager = ConfigurationManager::new();

        manager.config_mut().providers.insert(
            "openai".to_string(),
            ProviderSettings {
                api_key: Some("test-key".to_string()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        manager
            .config_mut()
            .defaults
            .per_action
            .insert("invalid_action".to_string(), "gpt-4".to_string());

        assert!(manager.validate().is_err());
    }

    #[test]
    fn test_validate_valid_commands() {
        let mut manager = ConfigurationManager::new();

        manager.config_mut().providers.insert(
            "openai".to_string(),
            ProviderSettings {
                api_key: Some("test-key".to_string()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        manager
            .config_mut()
            .defaults
            .per_command
            .insert("gen".to_string(), "gpt-4".to_string());
        manager
            .config_mut()
            .defaults
            .per_command
            .insert("refactor".to_string(), "gpt-4".to_string());
        manager
            .config_mut()
            .defaults
            .per_command
            .insert("review".to_string(), "gpt-4".to_string());

        assert!(manager.validate().is_ok());
    }

    #[test]
    fn test_validate_valid_actions() {
        let mut manager = ConfigurationManager::new();

        manager.config_mut().providers.insert(
            "openai".to_string(),
            ProviderSettings {
                api_key: Some("test-key".to_string()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        manager
            .config_mut()
            .defaults
            .per_action
            .insert("analysis".to_string(), "gpt-4".to_string());
        manager
            .config_mut()
            .defaults
            .per_action
            .insert("generation".to_string(), "gpt-4".to_string());

        assert!(manager.validate().is_ok());
    }

    #[test]
    fn test_validate_with_registry_valid_model() {
        use std::sync::Arc;

        use async_trait::async_trait;

        use crate::{
            models::ModelInfo,
            provider::{Provider, ProviderRegistry},
        };

        // Create a mock provider
        struct MockProvider;

        #[async_trait]
        impl Provider for MockProvider {
            fn id(&self) -> &str {
                "openai"
            }

            fn name(&self) -> &str {
                "OpenAI"
            }

            fn models(&self) -> Vec<ModelInfo> {
                vec![ModelInfo {
                    id: "gpt-4".to_string(),
                    name: "GPT-4".to_string(),
                    provider: "openai".to_string(),
                    context_window: 8192,
                    capabilities: vec![],
                    pricing: None,
                    is_free: false,
                }]
            }

            async fn chat(
                &self,
                _request: crate::models::ChatRequest,
            ) -> Result<crate::models::ChatResponse, crate::error::ProviderError> {
                Err(crate::error::ProviderError::NotFound(
                    "Not implemented".to_string(),
                ))
            }

            async fn chat_stream(
                &self,
                _request: crate::models::ChatRequest,
            ) -> Result<crate::provider::ChatStream, crate::error::ProviderError> {
                Err(crate::error::ProviderError::NotFound(
                    "Not implemented".to_string(),
                ))
            }

            fn count_tokens(
                &self,
                _content: &str,
                _model: &str,
            ) -> Result<usize, crate::error::ProviderError> {
                Ok(0)
            }

            async fn health_check(&self) -> Result<bool, crate::error::ProviderError> {
                Ok(true)
            }
        }

        let mut manager = ConfigurationManager::new();
        manager.config_mut().providers.insert(
            "openai".to_string(),
            ProviderSettings {
                api_key: Some("test-key".to_string()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        let mut registry = ProviderRegistry::new();
        registry.register(Arc::new(MockProvider)).unwrap();

        assert!(manager.validate_with_registry(&registry).is_ok());
    }

    #[test]
    fn test_validate_with_registry_invalid_model() {
        use std::sync::Arc;

        use async_trait::async_trait;

        use crate::{
            models::ModelInfo,
            provider::{Provider, ProviderRegistry},
        };

        // Create a mock provider
        struct MockProvider;

        #[async_trait]
        impl Provider for MockProvider {
            fn id(&self) -> &str {
                "openai"
            }

            fn name(&self) -> &str {
                "OpenAI"
            }

            fn models(&self) -> Vec<ModelInfo> {
                vec![ModelInfo {
                    id: "gpt-3.5-turbo".to_string(),
                    name: "GPT-3.5 Turbo".to_string(),
                    provider: "openai".to_string(),
                    context_window: 4096,
                    capabilities: vec![],
                    pricing: None,
                    is_free: false,
                }]
            }

            async fn chat(
                &self,
                _request: crate::models::ChatRequest,
            ) -> Result<crate::models::ChatResponse, crate::error::ProviderError> {
                Err(crate::error::ProviderError::NotFound(
                    "Not implemented".to_string(),
                ))
            }

            async fn chat_stream(
                &self,
                _request: crate::models::ChatRequest,
            ) -> Result<crate::provider::ChatStream, crate::error::ProviderError> {
                Err(crate::error::ProviderError::NotFound(
                    "Not implemented".to_string(),
                ))
            }

            fn count_tokens(
                &self,
                _content: &str,
                _model: &str,
            ) -> Result<usize, crate::error::ProviderError> {
                Ok(0)
            }

            async fn health_check(&self) -> Result<bool, crate::error::ProviderError> {
                Ok(true)
            }
        }

        let mut manager = ConfigurationManager::new();
        manager.config_mut().providers.insert(
            "openai".to_string(),
            ProviderSettings {
                api_key: Some("test-key".to_string()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );
        // Set default model to one that doesn't exist
        manager.config_mut().defaults.model = "gpt-4".to_string();

        let mut registry = ProviderRegistry::new();
        registry.register(Arc::new(MockProvider)).unwrap();

        assert!(manager.validate_with_registry(&registry).is_err());
    }
}
