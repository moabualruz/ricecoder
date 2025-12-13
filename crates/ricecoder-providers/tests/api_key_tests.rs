use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ApiKeyConfig;

    #[test]
    fn test_new_api_key_manager() {
        let manager = ApiKeyManager::new();
        assert_eq!(manager.cached_key_count(), 0);
    }

    #[test]
    fn test_store_and_get_key() {
        let mut manager = ApiKeyManager::new();
        manager.store_key("openai".to_string(), "sk-test-123".to_string());

        let key = manager.get_key("openai");
        assert!(key.is_ok());
        assert_eq!(key.unwrap(), "sk-test-123");
    }

    #[test]
    fn test_get_nonexistent_key() {
        let manager = ApiKeyManager::new();
        let key = manager.get_key("nonexistent");
        assert!(key.is_err());
    }

    #[test]
    fn test_has_key() {
        let mut manager = ApiKeyManager::new();
        assert!(!manager.has_key("openai"));

        manager.store_key("openai".to_string(), "sk-test-123".to_string());
        assert!(manager.has_key("openai"));
    }

    #[test]
    fn test_rotate_key() {
        let mut manager = ApiKeyManager::new();
        manager.store_key("openai".to_string(), "sk-old-key".to_string());

        let result = manager.rotate_key("openai".to_string(), "sk-new-key".to_string());
        assert!(result.is_ok());

        let key = manager.get_key("openai");
        assert_eq!(key.unwrap(), "sk-new-key");
    }

    #[test]
    fn test_rotate_key_empty() {
        let mut manager = ApiKeyManager::new();
        let result = manager.rotate_key("openai".to_string(), "".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_cached_key() {
        let mut manager = ApiKeyManager::new();
        manager.store_key("openai".to_string(), "sk-test-123".to_string());
        assert!(manager.has_key("openai"));

        manager.clear_cached_key("openai");
        assert!(!manager.has_key("openai"));
    }

    #[test]
    fn test_clear_all_cached_keys() {
        let mut manager = ApiKeyManager::new();
        manager.store_key("openai".to_string(), "sk-test-123".to_string());
        manager.store_key("anthropic".to_string(), "sk-test-456".to_string());
        assert_eq!(manager.cached_key_count(), 2);

        manager.clear_all_cached_keys();
        assert_eq!(manager.cached_key_count(), 0);
    }

    #[test]
    fn test_register_config() {
        let mut manager = ApiKeyManager::new();
        let config = ApiKeyConfig {
            env_var: "OPENAI_API_KEY".to_string(),
            secure_storage: false,
        };
        manager.register_config("openai".to_string(), config);

        assert!(manager.configured_providers().contains(&"openai".to_string()));
    }

    #[test]
    fn test_get_key_from_env() {
        let mut manager = ApiKeyManager::new();
        let config = ApiKeyConfig {
            env_var: "TEST_API_KEY_ENV".to_string(),
            secure_storage: false,
        };
        manager.register_config("test".to_string(), config);

        // Set environment variable
        std::env::set_var("TEST_API_KEY_ENV", "env-key-123");

        let key = manager.get_key("test");
        assert!(key.is_ok());
        assert_eq!(key.unwrap(), "env-key-123");

        // Cleanup
        std::env::remove_var("TEST_API_KEY_ENV");
    }

    #[test]
    fn test_load_from_env() {
        let mut manager = ApiKeyManager::new();

        let config1 = ApiKeyConfig {
            env_var: "TEST_KEY_1".to_string(),
            secure_storage: false,
        };
        let config2 = ApiKeyConfig {
            env_var: "TEST_KEY_2".to_string(),
            secure_storage: false,
        };

        manager.register_config("provider1".to_string(), config1);
        manager.register_config("provider2".to_string(), config2);

        std::env::set_var("TEST_KEY_1", "key-1");
        std::env::set_var("TEST_KEY_2", "key-2");

        manager.load_from_env().unwrap();

        assert_eq!(manager.cached_key_count(), 2);
        assert_eq!(manager.get_key("provider1").unwrap(), "key-1");
        assert_eq!(manager.get_key("provider2").unwrap(), "key-2");

        std::env::remove_var("TEST_KEY_1");
        std::env::remove_var("TEST_KEY_2");
    }

    #[test]
    fn test_configured_providers() {
        let mut manager = ApiKeyManager::new();

        let config1 = ApiKeyConfig {
            env_var: "KEY_1".to_string(),
            secure_storage: false,
        };
        let config2 = ApiKeyConfig {
            env_var: "KEY_2".to_string(),
            secure_storage: false,
        };

        manager.register_config("openai".to_string(), config1);
        manager.register_config("anthropic".to_string(), config2);

        let providers = manager.configured_providers();
        assert_eq!(providers.len(), 2);
        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"anthropic".to_string()));
    }

    #[test]
    fn test_cached_key_takes_precedence_over_env() {
        let mut manager = ApiKeyManager::new();

        let config = ApiKeyConfig {
            env_var: "TEST_PRECEDENCE_KEY".to_string(),
            secure_storage: false,
        };
        manager.register_config("test".to_string(), config);

        // Set environment variable
        std::env::set_var("TEST_PRECEDENCE_KEY", "env-key");

        // Store a different key in cache
        manager.store_key("test".to_string(), "cached-key".to_string());

        // Cached key should take precedence
        let key = manager.get_key("test");
        assert_eq!(key.unwrap(), "cached-key");

        std::env::remove_var("TEST_PRECEDENCE_KEY");
    }
}