//! Integration tests for Ollama provider configuration loading
//! Tests configuration loading from environment variables, files, and validation
//! **Feature: ricecoder-local-models, Integration Tests: Provider Configuration**
//! **Validates: Requirements 1.1, 1.2**

use std::path::PathBuf;

use ricecoder_providers::{providers::OllamaConfig, OllamaProvider, Provider};

/// Note: Environment variable loading tests are covered in unit tests in ollama_config.rs
/// These integration tests focus on provider registration and configuration validation

/// Test: Configuration validation - default values
/// For any OllamaProvider with default configuration, validation SHALL succeed
#[test]
fn test_ollama_config_validation_default_values() {
    let config = OllamaConfig::default();

    let result = config.validate();
    assert!(result.is_ok());
}

/// Test: Configuration validation - custom values
/// For any OllamaProvider with custom configuration, validation SHALL succeed if values are valid
#[test]
fn test_ollama_config_validation_custom_values() {
    let config = OllamaConfig {
        base_url: "http://custom:11434".to_string(),
        default_model: "custom-model".to_string(),
        timeout_secs: 60,
        cache_ttl_secs: 600,
    };

    let result = config.validate();
    assert!(result.is_ok());
}

/// Test: Configuration validation - empty base URL
/// For any OllamaProvider with empty base URL, validation SHALL fail
#[test]
fn test_ollama_config_validation_empty_base_url() {
    let mut config = OllamaConfig::default();
    config.base_url = String::new();

    let result = config.validate();
    assert!(result.is_err());
}

/// Test: Configuration validation - invalid base URL scheme
/// For any OllamaProvider with invalid URL scheme, validation SHALL fail
#[test]
fn test_ollama_config_validation_invalid_url_scheme() {
    let mut config = OllamaConfig::default();
    config.base_url = "ftp://localhost:11434".to_string();

    let result = config.validate();
    assert!(result.is_err());
}

/// Test: Configuration validation - empty default model
/// For any OllamaProvider with empty default model, validation SHALL fail
#[test]
fn test_ollama_config_validation_empty_default_model() {
    let mut config = OllamaConfig::default();
    config.default_model = String::new();

    let result = config.validate();
    assert!(result.is_err());
}

/// Test: Configuration validation - zero timeout
/// For any OllamaProvider with zero timeout, validation SHALL fail
#[test]
fn test_ollama_config_validation_zero_timeout() {
    let mut config = OllamaConfig::default();
    config.timeout_secs = 0;

    let result = config.validate();
    assert!(result.is_err());
}

/// Test: Configuration validation - zero cache TTL
/// For any OllamaProvider with zero cache TTL, validation SHALL fail
#[test]
fn test_ollama_config_validation_zero_cache_ttl() {
    let mut config = OllamaConfig::default();
    config.cache_ttl_secs = 0;

    let result = config.validate();
    assert!(result.is_err());
}

/// Test: Configuration validation - HTTPS URL
/// For any OllamaProvider with HTTPS URL, validation SHALL succeed
#[test]
fn test_ollama_config_validation_https_url() {
    let config = OllamaConfig {
        base_url: "https://secure-ollama:11434".to_string(),
        default_model: "mistral".to_string(),
        timeout_secs: 30,
        cache_ttl_secs: 300,
    };

    let result = config.validate();
    assert!(result.is_ok());
}

/// Test: Configuration validation - HTTP URL
/// For any OllamaProvider with HTTP URL, validation SHALL succeed
#[test]
fn test_ollama_config_validation_http_url() {
    let config = OllamaConfig {
        base_url: "http://localhost:11434".to_string(),
        default_model: "mistral".to_string(),
        timeout_secs: 30,
        cache_ttl_secs: 300,
    };

    let result = config.validate();
    assert!(result.is_ok());
}

/// Test: Configuration default values
/// For any OllamaProvider with default configuration, values SHALL match expected defaults
#[test]
fn test_ollama_config_default_values() {
    let config = OllamaConfig::default();

    assert_eq!(config.base_url, "http://localhost:11434");
    assert_eq!(config.default_model, "mistral");
    assert_eq!(config.timeout_secs, 30);
    assert_eq!(config.cache_ttl_secs, 300);
}

/// Test: Configuration timeout as Duration
/// For any OllamaProvider configuration, timeout() SHALL return correct Duration
#[test]
fn test_ollama_config_timeout_as_duration() {
    let config = OllamaConfig {
        base_url: "http://localhost:11434".to_string(),
        default_model: "mistral".to_string(),
        timeout_secs: 45,
        cache_ttl_secs: 300,
    };

    let duration = config.timeout();
    assert_eq!(duration.as_secs(), 45);
}

/// Test: Configuration cache TTL as Duration
/// For any OllamaProvider configuration, cache_ttl() SHALL return correct Duration
#[test]
fn test_ollama_config_cache_ttl_as_duration() {
    let config = OllamaConfig {
        base_url: "http://localhost:11434".to_string(),
        default_model: "mistral".to_string(),
        timeout_secs: 30,
        cache_ttl_secs: 600,
    };

    let duration = config.cache_ttl();
    assert_eq!(duration.as_secs(), 600);
}

/// Test: Configuration global config path
/// For any OllamaProvider, get_global_config_path() SHALL return valid path
#[test]
fn test_ollama_config_global_config_path() {
    let path = OllamaConfig::get_global_config_path();

    assert!(path.to_string_lossy().contains(".ricecoder"));
    assert!(path.to_string_lossy().contains("config.yaml"));
}

/// Test: Configuration project config path
/// For any OllamaProvider, get_project_config_path() SHALL return valid path
#[test]
fn test_ollama_config_project_config_path() {
    let path = OllamaConfig::get_project_config_path();

    assert_eq!(path, PathBuf::from(".ricecoder/config.yaml"));
}

/// Test: OllamaProvider creation from configuration
/// For any OllamaProvider created from configuration, creation SHALL succeed
#[test]
fn test_ollama_provider_creation_from_config() {
    let provider = OllamaProvider::from_config();
    assert!(provider.is_ok());

    let prov = provider.unwrap();
    assert_eq!(prov.id(), "ollama");
}

/// Test: OllamaProvider config retrieval
/// For any OllamaProvider, config() SHALL return current configuration
#[test]
fn test_ollama_provider_config_retrieval() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();

    let config = provider.config();
    assert!(config.is_ok());

    let cfg = config.unwrap();
    assert_eq!(cfg.base_url, "http://localhost:11434");
}

/// Test: Configuration with large timeout
/// For any OllamaProvider with large timeout value, validation SHALL succeed
#[test]
fn test_ollama_config_validation_large_timeout() {
    let config = OllamaConfig {
        base_url: "http://localhost:11434".to_string(),
        default_model: "mistral".to_string(),
        timeout_secs: 3600, // 1 hour
        cache_ttl_secs: 300,
    };

    let result = config.validate();
    assert!(result.is_ok());
}

/// Test: Configuration with large cache TTL
/// For any OllamaProvider with large cache TTL value, validation SHALL succeed
#[test]
fn test_ollama_config_validation_large_cache_ttl() {
    let config = OllamaConfig {
        base_url: "http://localhost:11434".to_string(),
        default_model: "mistral".to_string(),
        timeout_secs: 30,
        cache_ttl_secs: 86400, // 1 day
    };

    let result = config.validate();
    assert!(result.is_ok());
}

/// Test: Configuration with custom model name
/// For any OllamaProvider with custom model name, configuration SHALL succeed
#[test]
fn test_ollama_config_custom_model_name() {
    let config = OllamaConfig {
        base_url: "http://localhost:11434".to_string(),
        default_model: "my-custom-model:latest".to_string(),
        timeout_secs: 30,
        cache_ttl_secs: 300,
    };

    let result = config.validate();
    assert!(result.is_ok());
}

/// Test: Configuration with localhost variations
/// For any OllamaProvider with localhost variations, configuration SHALL succeed
#[test]
fn test_ollama_config_localhost_variations() {
    let configs = vec![
        "http://localhost:11434",
        "http://127.0.0.1:11434",
        "http://0.0.0.0:11434",
    ];

    for base_url in configs {
        let config = OllamaConfig {
            base_url: base_url.to_string(),
            default_model: "mistral".to_string(),
            timeout_secs: 30,
            cache_ttl_secs: 300,
        };

        assert!(config.validate().is_ok(), "Failed for {}", base_url);
    }
}

/// Test: Configuration with port variations
/// For any OllamaProvider with different ports, configuration SHALL succeed
#[test]
fn test_ollama_config_port_variations() {
    let configs = vec![
        "http://localhost:11434",
        "http://localhost:8080",
        "http://localhost:3000",
    ];

    for base_url in configs {
        let config = OllamaConfig {
            base_url: base_url.to_string(),
            default_model: "mistral".to_string(),
            timeout_secs: 30,
            cache_ttl_secs: 300,
        };

        assert!(config.validate().is_ok(), "Failed for {}", base_url);
    }
}

/// Test: Configuration with numeric model names
/// For any OllamaProvider with numeric model names, configuration SHALL succeed
#[test]
fn test_ollama_config_numeric_model_names() {
    let config = OllamaConfig {
        base_url: "http://localhost:11434".to_string(),
        default_model: "model-7b".to_string(),
        timeout_secs: 30,
        cache_ttl_secs: 300,
    };

    let result = config.validate();
    assert!(result.is_ok());
}
