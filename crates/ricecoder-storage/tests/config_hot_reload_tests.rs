use ricecoder_storage::*;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hot_reload_manager_creation() {
        let config = Config::default();
        let manager = HotReloadManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[test]
    fn test_config_conflict_resolution() {
        let config1 = Config {
            providers: ProvidersConfig {
                api_keys: [("key1".to_string(), "value1".to_string())].into(),
                endpoints: HashMap::new(),
                default_provider: Some("provider1".to_string()),
            },
            defaults: DefaultsConfig {
                model: Some("model1".to_string()),
                temperature: Some(0.5),
                max_tokens: Some(100),
            },
            steering: vec![],
            tui: Default::default(),
            custom: HashMap::new(),
        };

        let config2 = Config {
            providers: ProvidersConfig {
                api_keys: [("key2".to_string(), "value2".to_string())].into(),
                endpoints: HashMap::new(),
                default_provider: Some("provider2".to_string()),
            },
            defaults: DefaultsConfig {
                model: Some("model2".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(200),
            },
            steering: vec![],
            tui: Default::default(),
            custom: HashMap::new(),
        };

        let configs = vec![&config1, &config2];
        let resolved = ConfigConflictResolver::resolve_conflicts(&configs);

        // config1 should take priority for conflicts
        assert_eq!(resolved.providers.default_provider, Some("provider1".to_string()));
        assert_eq!(resolved.defaults.model, Some("model1".to_string()));
        assert_eq!(resolved.defaults.temperature, Some(0.5));

        // But config2 values should be included where config1 doesn't have them
        assert_eq!(resolved.providers.api_keys.len(), 2);
        assert!(resolved.providers.api_keys.contains_key("key1"));
        assert!(resolved.providers.api_keys.contains_key("key2"));
    }
}