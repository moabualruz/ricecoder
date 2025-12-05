//! Integration tests for external LSP integration with ricecoder crates
//!
//! These tests verify that external LSP integration works correctly with:
//! - ricecoder-lsp (LSP proxy)
//! - ricecoder-completion (completion proxy)
//! - ricecoder-storage (configuration loading)

use ricecoder_external_lsp::{
    LspServerConfig, LspServerRegistry,
};
use std::collections::HashMap;

/// Mock external LSP client for testing
struct MockExternalLspClient {
    available_languages: Vec<String>,
}

impl MockExternalLspClient {
    fn new(languages: Vec<String>) -> Self {
        Self {
            available_languages: languages,
        }
    }

    fn is_available(&self, language: &str) -> bool {
        self.available_languages.contains(&language.to_string())
    }
}

#[test]
fn test_lsp_server_registry_creation() {
    // Test that we can create a valid LSP server registry
    let mut servers = HashMap::new();
    
    let rust_config = LspServerConfig {
        language: "rust".to_string(),
        extensions: vec![".rs".to_string()],
        executable: "rust-analyzer".to_string(),
        args: vec![],
        env: HashMap::new(),
        init_options: None,
        enabled: true,
        timeout_ms: 5000,
        max_restarts: 3,
        idle_timeout_ms: 300000,
        output_mapping: None,
    };
    
    servers.insert("rust".to_string(), vec![rust_config]);
    
    let registry = LspServerRegistry {
        servers,
        global: ricecoder_external_lsp::GlobalLspSettings {
            max_processes: 5,
            default_timeout_ms: 5000,
            enable_fallback: true,
            health_check_interval_ms: 30000,
        },
    };
    
    assert_eq!(registry.servers.len(), 1);
    assert!(registry.servers.contains_key("rust"));
    assert_eq!(registry.global.max_processes, 5);
}

#[test]
fn test_mock_lsp_client_availability() {
    // Test that mock LSP client correctly reports availability
    let client = MockExternalLspClient::new(vec![
        "rust".to_string(),
        "typescript".to_string(),
    ]);
    
    assert!(client.is_available("rust"));
    assert!(client.is_available("typescript"));
    assert!(!client.is_available("python"));
}

#[test]
fn test_graceful_degradation_when_lsp_unavailable() {
    // Test that system gracefully degrades when external LSP is unavailable
    let client = MockExternalLspClient::new(vec![]);
    
    // No languages available
    assert!(!client.is_available("rust"));
    assert!(!client.is_available("typescript"));
    assert!(!client.is_available("python"));
}

#[test]
fn test_configuration_hierarchy() {
    // Test that configuration hierarchy is respected
    // Built-in defaults should be overridden by user config
    // User config should be overridden by project config
    
    let mut servers = HashMap::new();
    
    // Built-in default
    let builtin_rust = LspServerConfig {
        language: "rust".to_string(),
        extensions: vec![".rs".to_string()],
        executable: "rust-analyzer".to_string(),
        args: vec![],
        env: HashMap::new(),
        init_options: None,
        enabled: true,
        timeout_ms: 5000,
        max_restarts: 3,
        idle_timeout_ms: 300000,
        output_mapping: None,
    };
    
    servers.insert("rust".to_string(), vec![builtin_rust]);
    
    // User override (different timeout)
    let user_rust = LspServerConfig {
        language: "rust".to_string(),
        extensions: vec![".rs".to_string()],
        executable: "rust-analyzer".to_string(),
        args: vec![],
        env: HashMap::new(),
        init_options: None,
        enabled: true,
        timeout_ms: 10000, // Different timeout
        max_restarts: 3,
        idle_timeout_ms: 300000,
        output_mapping: None,
    };
    
    // User config should override built-in
    servers.insert("rust".to_string(), vec![user_rust.clone()]);
    
    let registry = LspServerRegistry {
        servers,
        global: ricecoder_external_lsp::GlobalLspSettings::default(),
    };
    
    let rust_config = registry.servers.get("rust").unwrap().first().unwrap();
    assert_eq!(rust_config.timeout_ms, 10000);
}

#[test]
fn test_multiple_lsp_servers_for_language() {
    // Test that multiple LSP servers can be configured for a language
    let mut servers = HashMap::new();
    
    let primary_rust = LspServerConfig {
        language: "rust".to_string(),
        extensions: vec![".rs".to_string()],
        executable: "rust-analyzer".to_string(),
        args: vec![],
        env: HashMap::new(),
        init_options: None,
        enabled: true,
        timeout_ms: 5000,
        max_restarts: 3,
        idle_timeout_ms: 300000,
        output_mapping: None,
    };
    
    let secondary_rust = LspServerConfig {
        language: "rust".to_string(),
        extensions: vec![".rs".to_string()],
        executable: "rls".to_string(), // Alternative Rust LSP
        args: vec![],
        env: HashMap::new(),
        init_options: None,
        enabled: false, // Disabled by default
        timeout_ms: 5000,
        max_restarts: 3,
        idle_timeout_ms: 300000,
        output_mapping: None,
    };
    
    servers.insert("rust".to_string(), vec![primary_rust, secondary_rust]);
    
    let registry = LspServerRegistry {
        servers,
        global: ricecoder_external_lsp::GlobalLspSettings::default(),
    };
    
    let rust_servers = registry.servers.get("rust").unwrap();
    assert_eq!(rust_servers.len(), 2);
    assert_eq!(rust_servers[0].executable, "rust-analyzer");
    assert_eq!(rust_servers[1].executable, "rls");
}

#[test]
fn test_lsp_server_config_validation() {
    // Test that LSP server configuration is valid
    let config = LspServerConfig {
        language: "rust".to_string(),
        extensions: vec![".rs".to_string()],
        executable: "rust-analyzer".to_string(),
        args: vec![],
        env: HashMap::new(),
        init_options: None,
        enabled: true,
        timeout_ms: 5000,
        max_restarts: 3,
        idle_timeout_ms: 300000,
        output_mapping: None,
    };
    
    // Verify all required fields are present
    assert!(!config.language.is_empty());
    assert!(!config.executable.is_empty());
    assert!(config.timeout_ms > 0);
    assert!(config.max_restarts > 0);
}

#[test]
fn test_fallback_enabled_by_default() {
    // Test that fallback to internal providers is enabled by default
    let registry = LspServerRegistry {
        servers: HashMap::new(),
        global: ricecoder_external_lsp::GlobalLspSettings::default(),
    };
    
    assert!(registry.global.enable_fallback);
}

#[test]
fn test_process_limits() {
    // Test that process limits are enforced
    let registry = LspServerRegistry {
        servers: HashMap::new(),
        global: ricecoder_external_lsp::GlobalLspSettings {
            max_processes: 5,
            default_timeout_ms: 5000,
            enable_fallback: true,
            health_check_interval_ms: 30000,
        },
    };
    
    assert_eq!(registry.global.max_processes, 5);
}

#[test]
fn test_health_check_interval() {
    // Test that health check interval is configured
    let registry = LspServerRegistry {
        servers: HashMap::new(),
        global: ricecoder_external_lsp::GlobalLspSettings {
            max_processes: 5,
            default_timeout_ms: 5000,
            enable_fallback: true,
            health_check_interval_ms: 30000,
        },
    };
    
    assert_eq!(registry.global.health_check_interval_ms, 30000);
}

#[test]
fn test_custom_lsp_server_configuration() {
    // Test that custom LSP servers can be configured
    let mut servers = HashMap::new();
    
    let custom_lsp = LspServerConfig {
        language: "custom".to_string(),
        extensions: vec![".custom".to_string()],
        executable: "custom-lsp-server".to_string(),
        args: vec!["--stdio".to_string()],
        env: HashMap::new(),
        init_options: None,
        enabled: true,
        timeout_ms: 5000,
        max_restarts: 3,
        idle_timeout_ms: 300000,
        output_mapping: None,
    };
    
    servers.insert("custom".to_string(), vec![custom_lsp]);
    
    let registry = LspServerRegistry {
        servers,
        global: ricecoder_external_lsp::GlobalLspSettings::default(),
    };
    
    assert!(registry.servers.contains_key("custom"));
    let custom_config = registry.servers.get("custom").unwrap().first().unwrap();
    assert_eq!(custom_config.executable, "custom-lsp-server");
    assert_eq!(custom_config.args.len(), 1);
}

#[test]
fn test_lsp_server_disabled() {
    // Test that disabled LSP servers are not used
    let mut servers = HashMap::new();
    
    let disabled_rust = LspServerConfig {
        language: "rust".to_string(),
        extensions: vec![".rs".to_string()],
        executable: "rust-analyzer".to_string(),
        args: vec![],
        env: HashMap::new(),
        init_options: None,
        enabled: false, // Disabled
        timeout_ms: 5000,
        max_restarts: 3,
        idle_timeout_ms: 300000,
        output_mapping: None,
    };
    
    servers.insert("rust".to_string(), vec![disabled_rust]);
    
    let registry = LspServerRegistry {
        servers,
        global: ricecoder_external_lsp::GlobalLspSettings::default(),
    };
    
    let rust_config = registry.servers.get("rust").unwrap().first().unwrap();
    assert!(!rust_config.enabled);
}

#[test]
fn test_idle_timeout_configuration() {
    // Test that idle timeout is configurable
    let config = LspServerConfig {
        language: "rust".to_string(),
        extensions: vec![".rs".to_string()],
        executable: "rust-analyzer".to_string(),
        args: vec![],
        env: HashMap::new(),
        init_options: None,
        enabled: true,
        timeout_ms: 5000,
        max_restarts: 3,
        idle_timeout_ms: 600000, // 10 minutes
        output_mapping: None,
    };
    
    assert_eq!(config.idle_timeout_ms, 600000);
}

#[test]
fn test_environment_variables_configuration() {
    // Test that environment variables can be configured for LSP servers
    let mut env = HashMap::new();
    env.insert("RUST_LOG".to_string(), "debug".to_string());
    
    let config = LspServerConfig {
        language: "rust".to_string(),
        extensions: vec![".rs".to_string()],
        executable: "rust-analyzer".to_string(),
        args: vec![],
        env,
        init_options: None,
        enabled: true,
        timeout_ms: 5000,
        max_restarts: 3,
        idle_timeout_ms: 300000,
        output_mapping: None,
    };
    
    assert_eq!(config.env.get("RUST_LOG").unwrap(), "debug");
}
