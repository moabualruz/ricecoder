//! Environment variable override support
//!
//! This module provides parsing and application of environment variables with
//! the RICECODER_ prefix to override configuration values.

use super::Config;
use std::collections::HashMap;

/// Environment variable overrides
pub struct EnvOverrides;

impl EnvOverrides {
    /// Parse environment variables with RICECODER_ prefix
    ///
    /// Returns a map of configuration paths to values.
    /// For example, RICECODER_PROVIDERS_DEFAULT=openai becomes
    /// {"providers.default_provider": "openai"}
    pub fn parse() -> HashMap<String, String> {
        let mut overrides = HashMap::new();

        for (key, value) in std::env::vars() {
            if let Some(config_key) = key.strip_prefix("RICECODER_") {
                let config_path = Self::env_key_to_config_path(config_key);
                overrides.insert(config_path, value);
            }
        }

        overrides
    }

    /// Apply environment variable overrides to configuration
    ///
    /// This function applies environment variable overrides to the configuration
    /// by parsing the environment and updating the config accordingly.
    pub fn apply(config: &mut Config) {
        let overrides = Self::parse();
        Self::apply_overrides(config, &overrides);
    }

    /// Apply specific overrides to configuration
    pub fn apply_overrides(config: &mut Config, overrides: &HashMap<String, String>) {
        for (path, value) in overrides {
            Self::set_config_value(config, path, value);
        }
    }

    /// Convert environment variable key to configuration path
    ///
    /// Examples:
    /// - PROVIDERS_DEFAULT -> providers.default_provider
    /// - PROVIDERS_API_KEYS_OPENAI -> providers.api_keys.openai
    /// - DEFAULTS_MODEL -> defaults.model
    fn env_key_to_config_path(key: &str) -> String {
        key.to_lowercase()
            .replace('_', ".")
    }

    /// Set a configuration value by path
    ///
    /// Supports nested paths like "providers.default_provider"
    fn set_config_value(config: &mut Config, path: &str, value: &str) {
        let parts: Vec<&str> = path.split('.').collect();

        match parts.as_slice() {
            ["providers", "default_provider"] => {
                config.providers.default_provider = Some(value.to_string());
            }
            ["providers", "api_keys", key] => {
                config.providers.api_keys.insert(key.to_string(), value.to_string());
            }
            ["providers", "endpoints", key] => {
                config.providers.endpoints.insert(key.to_string(), value.to_string());
            }
            ["defaults", "model"] => {
                config.defaults.model = Some(value.to_string());
            }
            ["defaults", "temperature"] => {
                if let Ok(temp) = value.parse::<f32>() {
                    config.defaults.temperature = Some(temp);
                }
            }
            ["defaults", "max_tokens"] => {
                if let Ok(tokens) = value.parse::<u32>() {
                    config.defaults.max_tokens = Some(tokens);
                }
            }
            _ => {
                // Store in custom map for unknown paths
                if let Ok(json_value) = serde_json::from_str(value) {
                    config.custom.insert(path.to_string(), json_value);
                } else {
                    config.custom.insert(
                        path.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_key_to_config_path() {
        assert_eq!(
            EnvOverrides::env_key_to_config_path("PROVIDERS_DEFAULT"),
            "providers.default"
        );
        assert_eq!(
            EnvOverrides::env_key_to_config_path("DEFAULTS_MODEL"),
            "defaults.model"
        );
    }

    #[test]
    fn test_apply_provider_default_override() {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        overrides.insert("providers.default_provider".to_string(), "openai".to_string());

        EnvOverrides::apply_overrides(&mut config, &overrides);

        assert_eq!(config.providers.default_provider, Some("openai".to_string()));
    }

    #[test]
    fn test_apply_api_key_override() {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        overrides.insert(
            "providers.api_keys.openai".to_string(),
            "test-key".to_string(),
        );

        EnvOverrides::apply_overrides(&mut config, &overrides);

        assert_eq!(
            config.providers.api_keys.get("openai"),
            Some(&"test-key".to_string())
        );
    }

    #[test]
    fn test_apply_defaults_override() {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        overrides.insert("defaults.model".to_string(), "gpt-4".to_string());
        overrides.insert("defaults.temperature".to_string(), "0.7".to_string());
        overrides.insert("defaults.max_tokens".to_string(), "2000".to_string());

        EnvOverrides::apply_overrides(&mut config, &overrides);

        assert_eq!(config.defaults.model, Some("gpt-4".to_string()));
        assert_eq!(config.defaults.temperature, Some(0.7));
        assert_eq!(config.defaults.max_tokens, Some(2000));
    }

    #[test]
    fn test_apply_multiple_overrides() {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        overrides.insert("providers.default_provider".to_string(), "openai".to_string());
        overrides.insert("defaults.model".to_string(), "gpt-4".to_string());
        overrides.insert(
            "providers.api_keys.openai".to_string(),
            "test-key".to_string(),
        );

        EnvOverrides::apply_overrides(&mut config, &overrides);

        assert_eq!(config.providers.default_provider, Some("openai".to_string()));
        assert_eq!(config.defaults.model, Some("gpt-4".to_string()));
        assert_eq!(
            config.providers.api_keys.get("openai"),
            Some(&"test-key".to_string())
        );
    }

    #[test]
    fn test_apply_custom_override() {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        overrides.insert("custom.setting".to_string(), "value".to_string());

        EnvOverrides::apply_overrides(&mut config, &overrides);

        assert_eq!(
            config.custom.get("custom.setting"),
            Some(&serde_json::Value::String("value".to_string()))
        );
    }
}
