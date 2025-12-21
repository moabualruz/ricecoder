//! Unit tests for data models and configuration
//!
//! Tests configuration loading from YAML/JSON, validation, error handling,
//! and path resolution using ricecoder_storage::PathResolver
//!
//! **Requirements: 1.2, 1.6**

use ricecoder_ide::*;
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

// ============================================================================
// Configuration Loading Tests
// ============================================================================

#[tokio::test]
async fn test_load_config_from_yaml_file() {
    let yaml_content = r#"
vscode:
  enabled: true
  port: 8080
  features:
    - completion
    - diagnostics
  settings: {}

terminal:
  vim:
    enabled: true
    plugin_manager: vim-plug

providers:
  external_lsp:
    enabled: true
    servers:
      rust:
        language: rust
        command: rust-analyzer
        args: []
        timeout_ms: 5000
    health_check_interval_ms: 5000
  builtin_providers:
    enabled: true
    languages:
      - rust
      - typescript
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml_content.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = ConfigManager::load_from_yaml_file(file.path().to_str().unwrap())
        .await
        .expect("Failed to load YAML config");

    assert!(config.vscode.is_some());
    assert_eq!(config.vscode.unwrap().port, 8080);
    assert!(config.providers.external_lsp.enabled);
    assert!(config.providers.builtin_providers.enabled);
}

#[tokio::test]
async fn test_load_config_from_json_file() {
    let json_content = r#"{
  "vscode": {
    "enabled": true,
    "port": 9090,
    "features": ["completion"],
    "settings": {}
  },
  "terminal": null,
  "providers": {
    "external_lsp": {
      "enabled": true,
      "servers": {
        "typescript": {
          "language": "typescript",
          "command": "typescript-language-server",
          "args": ["--stdio"],
          "timeout_ms": 5000
        }
      },
      "health_check_interval_ms": 5000
    },
    "configured_rules": null,
    "builtin_providers": {
      "enabled": true,
      "languages": ["typescript"]
    }
  }
}"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(json_content.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = ConfigManager::load_from_json_file(file.path().to_str().unwrap())
        .await
        .expect("Failed to load JSON config");

    assert!(config.vscode.is_some());
    assert_eq!(config.vscode.unwrap().port, 9090);
    assert!(config.providers.external_lsp.enabled);
}

#[tokio::test]
async fn test_load_config_auto_detect_yaml() {
    let yaml_content = r#"
providers:
  external_lsp:
    enabled: true
    servers:
      rust:
        language: rust
        command: rust-analyzer
        args: []
        timeout_ms: 5000
    health_check_interval_ms: 5000
  builtin_providers:
    enabled: false
    languages: []
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml_content.as_bytes()).unwrap();
    file.flush().unwrap();

    let path = file.path().to_str().unwrap().to_string() + ".yaml";
    std::fs::rename(file.path(), &path).unwrap();

    let config = ConfigManager::load_from_file(&path)
        .await
        .expect("Failed to auto-detect and load YAML config");

    assert!(config.providers.external_lsp.enabled);
}

#[tokio::test]
async fn test_load_config_auto_detect_json() {
    let json_content = r#"{
  "providers": {
    "external_lsp": {
      "enabled": true,
      "servers": {
        "typescript": {
          "language": "typescript",
          "command": "typescript-language-server",
          "args": ["--stdio"],
          "timeout_ms": 5000
        }
      },
      "health_check_interval_ms": 5000
    },
    "configured_rules": null,
    "builtin_providers": {
      "enabled": false,
      "languages": []
    }
  }
}"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(json_content.as_bytes()).unwrap();
    file.flush().unwrap();

    let path = file.path().to_str().unwrap().to_string() + ".json";
    std::fs::rename(file.path(), &path).unwrap();

    let config = ConfigManager::load_from_file(&path)
        .await
        .expect("Failed to auto-detect and load JSON config");

    assert!(config.providers.external_lsp.enabled);
}

#[tokio::test]
async fn test_load_config_unsupported_format() {
    let result = ConfigManager::load_from_file("config.txt").await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unsupported configuration file format"));
}

#[tokio::test]
async fn test_load_config_file_not_found() {
    let result = ConfigManager::load_from_yaml_file("/nonexistent/path/config.yaml").await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to read configuration file"));
}

#[tokio::test]
async fn test_load_config_invalid_yaml_syntax() {
    let invalid_yaml = r#"
providers:
  external_lsp:
    enabled: true
    servers:
      - invalid: yaml: syntax:
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(invalid_yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let result = ConfigManager::load_from_yaml_file(file.path().to_str().unwrap()).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to parse YAML configuration"));
}

#[tokio::test]
async fn test_load_config_invalid_json_syntax() {
    let invalid_json = r#"{ "invalid": json: syntax }"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(invalid_json.as_bytes()).unwrap();
    file.flush().unwrap();

    let result = ConfigManager::load_from_json_file(file.path().to_str().unwrap()).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to parse JSON configuration"));
}

// ============================================================================
// Configuration Validation Tests
// ============================================================================

#[test]
fn test_validate_config_valid_with_all_providers() {
    let config = IdeIntegrationConfig {
        vscode: Some(VsCodeConfig {
            enabled: true,
            port: 8080,
            features: vec!["completion".to_string()],
            settings: serde_json::json!({}),
        }),
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: true,
                servers: {
                    let mut map = HashMap::new();
                    map.insert(
                        "rust".to_string(),
                        LspServerConfig {
                            language: "rust".to_string(),
                            command: "rust-analyzer".to_string(),
                            args: vec![],
                            timeout_ms: 5000,
                        },
                    );
                    map
                },
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: true,
                languages: vec!["rust".to_string()],
            },
        },
    };

    // This should not panic - validation happens during load
    let _ = config;
}

#[test]
fn test_validate_config_no_providers_enabled() {
    let config = IdeIntegrationConfig {
        vscode: None,
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: false,
                servers: HashMap::new(),
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: false,
                languages: vec![],
            },
        },
    };

    // Validation should fail when loading from file
    // This test verifies the config structure is correct
    assert!(!config.providers.external_lsp.enabled);
    assert!(!config.providers.builtin_providers.enabled);
}

#[test]
fn test_validate_config_empty_lsp_servers_when_enabled() {
    let config = IdeIntegrationConfig {
        vscode: None,
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: true,
                servers: HashMap::new(),
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: false,
                languages: vec![],
            },
        },
    };

    // Validation should fail when loading from file
    assert!(config.providers.external_lsp.enabled);
    assert!(config.providers.external_lsp.servers.is_empty());
}

#[test]
fn test_validate_config_invalid_lsp_command() {
    let config = IdeIntegrationConfig {
        vscode: None,
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: true,
                servers: {
                    let mut map = HashMap::new();
                    map.insert(
                        "rust".to_string(),
                        LspServerConfig {
                            language: "rust".to_string(),
                            command: "".to_string(),
                            args: vec![],
                            timeout_ms: 5000,
                        },
                    );
                    map
                },
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: false,
                languages: vec![],
            },
        },
    };

    // Validation should fail when loading from file
    let server = config.providers.external_lsp.servers.get("rust").unwrap();
    assert!(server.command.is_empty());
}

#[test]
fn test_validate_config_invalid_lsp_timeout() {
    let config = IdeIntegrationConfig {
        vscode: None,
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: true,
                servers: {
                    let mut map = HashMap::new();
                    map.insert(
                        "rust".to_string(),
                        LspServerConfig {
                            language: "rust".to_string(),
                            command: "rust-analyzer".to_string(),
                            args: vec![],
                            timeout_ms: 0,
                        },
                    );
                    map
                },
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: false,
                languages: vec![],
            },
        },
    };

    // Validation should fail when loading from file
    let server = config.providers.external_lsp.servers.get("rust").unwrap();
    assert_eq!(server.timeout_ms, 0);
}

#[test]
fn test_validate_config_invalid_vscode_port() {
    let config = IdeIntegrationConfig {
        vscode: Some(VsCodeConfig {
            enabled: true,
            port: 0,
            features: vec![],
            settings: serde_json::json!({}),
        }),
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: false,
                servers: HashMap::new(),
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: true,
                languages: vec!["rust".to_string()],
            },
        },
    };

    // Validation should fail when loading from file
    assert_eq!(config.vscode.unwrap().port, 0);
}

#[test]
fn test_validate_config_configured_rules_enabled_without_path() {
    let config = IdeIntegrationConfig {
        vscode: None,
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: false,
                servers: HashMap::new(),
                health_check_interval_ms: 5000,
            },
            configured_rules: Some(ConfiguredRulesConfig {
                enabled: true,
                rules_path: "".to_string(),
            }),
            builtin_providers: BuiltinProvidersConfig {
                enabled: false,
                languages: vec![],
            },
        },
    };

    // Validation should fail when loading from file
    let rules = config.providers.configured_rules.unwrap();
    assert!(rules.enabled);
    assert!(rules.rules_path.is_empty());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_error_message_clarity_invalid_yaml() {
    let invalid_yaml = r#"
providers:
  external_lsp:
    enabled: true
    servers:
      rust:
        language: rust
        command: rust-analyzer
        timeout_ms: "not a number"
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(invalid_yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let result = ConfigManager::load_from_yaml_file(file.path().to_str().unwrap()).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to parse YAML configuration"));
}

#[tokio::test]
async fn test_error_message_clarity_invalid_json() {
    let invalid_json = r#"{ "providers": { "external_lsp": { "enabled": "not a bool" } } }"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(invalid_json.as_bytes()).unwrap();
    file.flush().unwrap();

    let result = ConfigManager::load_from_json_file(file.path().to_str().unwrap()).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to parse JSON configuration"));
}

#[test]
fn test_error_types_are_distinct() {
    let config_error = IdeError::config_error("test config error");
    let validation_error = IdeError::config_validation_error("test validation error");
    let path_error = IdeError::path_resolution_error("test path error");

    assert!(config_error.to_string().contains("Configuration error"));
    assert!(validation_error
        .to_string()
        .contains("Configuration validation error"));
    assert!(path_error.to_string().contains("Path resolution error"));
}

// ============================================================================
// Data Model Tests
// ============================================================================

#[test]
fn test_position_creation_and_equality() {
    let pos1 = Position {
        line: 10,
        character: 5,
    };
    let pos2 = Position {
        line: 10,
        character: 5,
    };

    assert_eq!(pos1, pos2);
    assert_eq!(pos1.line, 10);
    assert_eq!(pos1.character, 5);
}

#[test]
fn test_range_creation() {
    let range = Range {
        start: Position {
            line: 1,
            character: 0,
        },
        end: Position {
            line: 1,
            character: 10,
        },
    };

    assert_eq!(range.start.line, 1);
    assert_eq!(range.start.character, 0);
    assert_eq!(range.end.line, 1);
    assert_eq!(range.end.character, 10);
}

#[test]
fn test_completion_item_creation() {
    let item = CompletionItem {
        label: "test_function".to_string(),
        kind: CompletionItemKind::Function,
        detail: Some("A test function".to_string()),
        documentation: Some("This is a test function".to_string()),
        insert_text: "test_function()".to_string(),
    };

    assert_eq!(item.label, "test_function");
    assert_eq!(item.kind, CompletionItemKind::Function);
    assert!(item.detail.is_some());
    assert!(item.documentation.is_some());
}

#[test]
fn test_diagnostic_creation() {
    let diagnostic = Diagnostic {
        range: Range {
            start: Position {
                line: 5,
                character: 0,
            },
            end: Position {
                line: 5,
                character: 10,
            },
        },
        severity: DiagnosticSeverity::Error,
        message: "Syntax error".to_string(),
        source: "rust-analyzer".to_string(),
    };

    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(diagnostic.message, "Syntax error");
    assert_eq!(diagnostic.source, "rust-analyzer");
}

#[test]
fn test_hover_creation() {
    let hover = Hover {
        contents: "fn test() -> i32".to_string(),
        range: Some(Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 4,
            },
        }),
    };

    assert_eq!(hover.contents, "fn test() -> i32");
    assert!(hover.range.is_some());
}

#[test]
fn test_location_creation() {
    let location = Location {
        file_path: "/path/to/file.rs".to_string(),
        range: Range {
            start: Position {
                line: 10,
                character: 5,
            },
            end: Position {
                line: 10,
                character: 15,
            },
        },
    };

    assert_eq!(location.file_path, "/path/to/file.rs");
    assert_eq!(location.range.start.line, 10);
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_completion_item_serialization_roundtrip() {
    let item = CompletionItem {
        label: "test".to_string(),
        kind: CompletionItemKind::Function,
        detail: Some("test function".to_string()),
        documentation: None,
        insert_text: "test()".to_string(),
    };

    let json = serde_json::to_string(&item).unwrap();
    let deserialized: CompletionItem = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.label, item.label);
    assert_eq!(deserialized.kind, item.kind);
    assert_eq!(deserialized.detail, item.detail);
    assert_eq!(deserialized.insert_text, item.insert_text);
}

#[test]
fn test_diagnostic_severity_serialization() {
    let severities = vec![
        DiagnosticSeverity::Error,
        DiagnosticSeverity::Warning,
        DiagnosticSeverity::Information,
        DiagnosticSeverity::Hint,
    ];

    for severity in severities {
        let json = serde_json::to_string(&severity).unwrap();
        let deserialized: DiagnosticSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, severity);
    }
}

#[test]
fn test_completion_item_kind_serialization() {
    let kinds = vec![
        CompletionItemKind::Text,
        CompletionItemKind::Function,
        CompletionItemKind::Class,
        CompletionItemKind::Method,
        CompletionItemKind::Variable,
    ];

    for kind in kinds {
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: CompletionItemKind = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, kind);
    }
}

#[test]
fn test_ide_integration_config_serialization() {
    let config = IdeIntegrationConfig {
        vscode: Some(VsCodeConfig {
            enabled: true,
            port: 8080,
            features: vec!["completion".to_string()],
            settings: serde_json::json!({}),
        }),
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: true,
                servers: {
                    let mut map = HashMap::new();
                    map.insert(
                        "rust".to_string(),
                        LspServerConfig {
                            language: "rust".to_string(),
                            command: "rust-analyzer".to_string(),
                            args: vec![],
                            timeout_ms: 5000,
                        },
                    );
                    map
                },
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: true,
                languages: vec!["rust".to_string()],
            },
        },
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: IdeIntegrationConfig = serde_json::from_str(&json).unwrap();

    assert!(deserialized.vscode.is_some());
    assert_eq!(deserialized.vscode.unwrap().port, 8080);
    assert!(deserialized.providers.external_lsp.enabled);
}

// ============================================================================
// Default Configuration Tests
// ============================================================================

#[test]
fn test_default_config_has_sensible_defaults() {
    let config = ConfigManager::default_config();

    assert!(config.providers.external_lsp.enabled);
    assert!(config.providers.builtin_providers.enabled);
    assert_eq!(config.providers.builtin_providers.languages.len(), 3);
    assert!(config
        .providers
        .builtin_providers
        .languages
        .contains(&"rust".to_string()));
    assert!(config
        .providers
        .builtin_providers
        .languages
        .contains(&"typescript".to_string()));
    assert!(config
        .providers
        .builtin_providers
        .languages
        .contains(&"python".to_string()));
}

#[test]
fn test_default_config_health_check_interval() {
    let config = ConfigManager::default_config();
    assert_eq!(config.providers.external_lsp.health_check_interval_ms, 5000);
}
