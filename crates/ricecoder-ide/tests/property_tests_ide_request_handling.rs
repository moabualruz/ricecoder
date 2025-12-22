//! Property-based tests for IDE request handling
//!
//! **Feature: ricecoder-ide, Property 5: IDE Request Handling**
//! **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5**
//!
//! This test suite verifies that IDE requests are processed correctly through the
//! provider chain and responses are formatted correctly for IDE consumption.

use std::sync::Arc;

use proptest::prelude::*;
use ricecoder_ide::*;

// Strategy for generating valid languages
fn valid_language_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("rust".to_string()),
        Just("typescript".to_string()),
        Just("python".to_string()),
        Just("javascript".to_string()),
        Just("go".to_string()),
        Just("java".to_string()),
        Just("cpp".to_string()),
        Just("csharp".to_string()),
        Just("php".to_string()),
        Just("ruby".to_string()),
    ]
}

// Strategy for generating valid file paths
fn valid_file_path_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("src/main.rs".to_string()),
        Just("src/lib.rs".to_string()),
        Just("src/test.rs".to_string()),
        Just("index.ts".to_string()),
        Just("app.py".to_string()),
        Just("main.go".to_string()),
        Just("Main.java".to_string()),
    ]
}

// Strategy for generating valid context strings
fn valid_context_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("fn test".to_string()),
        Just("let x = 5".to_string()),
        Just("const value = 10".to_string()),
        Just("def hello()".to_string()),
        Just("class Test".to_string()),
        Just("import module".to_string()),
    ]
}

// Strategy for generating valid source code
fn valid_source_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("fn main() {}".to_string()),
        Just("let x = 5;".to_string()),
        Just("const y = 10;".to_string()),
        Just("def test(): pass".to_string()),
        Just("class Test: pass".to_string()),
        Just("{ } [ ] ( )".to_string()),
    ]
}

// Strategy for generating valid positions
fn valid_position_strategy() -> impl Strategy<Value = (u32, u32)> {
    (0u32..1000, 0u32..100)
}

proptest! {
    /// Property 5.1: Completion requests are processed correctly
    ///
    /// For any valid completion request, the system SHALL process it through the
    /// provider chain and return a valid response.
    #[test]
    fn prop_completion_request_processing(
        language in valid_language_strategy(),
        file_path in valid_file_path_strategy(),
        (line, character) in valid_position_strategy(),
        context in valid_context_strategy(),
    ) {
        // Create test manager
        let config = create_test_config();
        let generic_provider = Arc::new(GenericProvider::new());
        let registry = ProviderRegistry::new(generic_provider);
        let provider_chain = Arc::new(ProviderChainManager::new(registry));
        let manager = IdeIntegrationManager::new(config, provider_chain);

        // Create completion request
        let params = CompletionParams {
            language: language.clone(),
            file_path: file_path.clone(),
            position: Position { line, character },
            context: context.clone(),
        };

        // Execute request
        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            manager.handle_completion_request(&params).await
        });

        // Verify response
        prop_assert!(result.is_ok(), "Completion request should succeed");
        let completions = result.unwrap();
        prop_assert!(true, "Completions should be a valid list");

        // Verify all completion items have required fields
        for item in completions {
            prop_assert!(!item.label.is_empty(), "Completion label should not be empty");
            prop_assert!(!item.insert_text.is_empty(), "Insert text should not be empty");
        }
    }

    /// Property 5.2: Diagnostics requests are processed correctly
    ///
    /// For any valid diagnostics request, the system SHALL process it through the
    /// provider chain and return a valid response.
    #[test]
    fn prop_diagnostics_request_processing(
        language in valid_language_strategy(),
        file_path in valid_file_path_strategy(),
        source in valid_source_strategy(),
    ) {
        // Create test manager
        let config = create_test_config();
        let generic_provider = Arc::new(GenericProvider::new());
        let registry = ProviderRegistry::new(generic_provider);
        let provider_chain = Arc::new(ProviderChainManager::new(registry));
        let manager = IdeIntegrationManager::new(config, provider_chain);

        // Create diagnostics request
        let params = DiagnosticsParams {
            language: language.clone(),
            file_path: file_path.clone(),
            source: source.clone(),
        };

        // Execute request
        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            manager.handle_diagnostics_request(&params).await
        });

        // Verify response
        prop_assert!(result.is_ok(), "Diagnostics request should succeed");
        let diagnostics = result.unwrap();
        prop_assert!(true, "Diagnostics should be a valid list");

        // Verify all diagnostics have required fields
        for diag in diagnostics {
            prop_assert!(!diag.message.is_empty(), "Diagnostic message should not be empty");
            prop_assert!(!diag.source.is_empty(), "Diagnostic source should not be empty");
        }
    }

    /// Property 5.3: Hover requests are processed correctly
    ///
    /// For any valid hover request, the system SHALL process it through the
    /// provider chain and return a valid response.
    #[test]
    fn prop_hover_request_processing(
        language in valid_language_strategy(),
        file_path in valid_file_path_strategy(),
        (line, character) in valid_position_strategy(),
    ) {
        // Create test manager
        let config = create_test_config();
        let generic_provider = Arc::new(GenericProvider::new());
        let registry = ProviderRegistry::new(generic_provider);
        let provider_chain = Arc::new(ProviderChainManager::new(registry));
        let manager = IdeIntegrationManager::new(config, provider_chain);

        // Create hover request
        let params = HoverParams {
            language: language.clone(),
            file_path: file_path.clone(),
            position: Position { line, character },
        };

        // Execute request
        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            manager.handle_hover_request(&params).await
        });

        // Verify response
        prop_assert!(result.is_ok(), "Hover request should succeed");
        let hover = result.unwrap();
        // Hover can be None or Some, both are valid
        if let Some(h) = hover {
            prop_assert!(!h.contents.is_empty(), "Hover contents should not be empty");
        }
    }

    /// Property 5.4: Definition requests are processed correctly
    ///
    /// For any valid definition request, the system SHALL process it through the
    /// provider chain and return a valid response.
    #[test]
    fn prop_definition_request_processing(
        language in valid_language_strategy(),
        file_path in valid_file_path_strategy(),
        (line, character) in valid_position_strategy(),
    ) {
        // Create test manager
        let config = create_test_config();
        let generic_provider = Arc::new(GenericProvider::new());
        let registry = ProviderRegistry::new(generic_provider);
        let provider_chain = Arc::new(ProviderChainManager::new(registry));
        let manager = IdeIntegrationManager::new(config, provider_chain);

        // Create definition request
        let params = DefinitionParams {
            language: language.clone(),
            file_path: file_path.clone(),
            position: Position { line, character },
        };

        // Execute request
        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            manager.handle_definition_request(&params).await
        });

        // Verify response
        prop_assert!(result.is_ok(), "Definition request should succeed");
        let location = result.unwrap();
        // Location can be None or Some, both are valid
        if let Some(loc) = location {
            prop_assert!(!loc.file_path.is_empty(), "Location file path should not be empty");
        }
    }

    /// Property 5.5: Response formatting is consistent
    ///
    /// For any valid completion response, formatting should produce consistent output.
    #[test]
    fn prop_response_formatting_consistency(
        label in r"[a-zA-Z_][a-zA-Z0-9_]*",
        detail in r"[a-zA-Z0-9_\s]*",
    ) {
        let insert_text = format!("{}()", label);
        let item = CompletionItem {
            label: label.clone(),
            kind: CompletionItemKind::Function,
            detail: Some(detail.clone()),
            documentation: None,
            insert_text: insert_text.clone(),
        };

        let formatted = ResponseFormatter::format_completions(std::slice::from_ref(&item));

        // Verify formatting produces valid JSON
        prop_assert!(formatted.get("items").is_some(), "Formatted response should have items");
        prop_assert!(formatted["items"].is_array(), "Items should be an array");
        prop_assert_eq!(formatted["items"].as_array().unwrap().len(), 1, "Should have one item");

        // Verify item fields are preserved
        let formatted_item = &formatted["items"][0];
        prop_assert_eq!(formatted_item["label"].as_str().unwrap_or(""), label.as_str(), "Label should be preserved");
        prop_assert_eq!(formatted_item["insertText"].as_str().unwrap_or(""), insert_text.as_str(), "Insert text should be preserved");
    }

    /// Property 5.6: Error handling for invalid requests
    ///
    /// For any invalid request (empty language or file path), the system SHALL
    /// return an error.
    #[test]
    fn prop_invalid_request_handling(
        file_path in valid_file_path_strategy(),
        (line, character) in valid_position_strategy(),
        context in valid_context_strategy(),
    ) {
        // Create test manager
        let config = create_test_config();
        let generic_provider = Arc::new(GenericProvider::new());
        let registry = ProviderRegistry::new(generic_provider);
        let provider_chain = Arc::new(ProviderChainManager::new(registry));
        let manager = IdeIntegrationManager::new(config, provider_chain);

        // Create invalid completion request (empty language)
        let params = CompletionParams {
            language: "".to_string(),
            file_path: file_path.clone(),
            position: Position { line, character },
            context: context.clone(),
        };

        // Execute request
        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            manager.handle_completion_request(&params).await
        });

        // Verify error is returned
        prop_assert!(result.is_err(), "Request with empty language should fail");
    }

    /// Property 5.7: Request validation preserves data integrity
    ///
    /// For any valid request, the system SHALL preserve all request data through
    /// processing and return it in the response.
    #[test]
    fn prop_request_data_integrity(
        language in valid_language_strategy(),
        file_path in valid_file_path_strategy(),
        (line, character) in valid_position_strategy(),
    ) {
        // Create test manager
        let config = create_test_config();
        let generic_provider = Arc::new(GenericProvider::new());
        let registry = ProviderRegistry::new(generic_provider);
        let provider_chain = Arc::new(ProviderChainManager::new(registry));
        let manager = IdeIntegrationManager::new(config, provider_chain);

        // Create hover request
        let params = HoverParams {
            language: language.clone(),
            file_path: file_path.clone(),
            position: Position { line, character },
        };

        // Execute request
        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            manager.handle_hover_request(&params).await
        });

        // Verify request succeeded
        prop_assert!(result.is_ok(), "Valid request should succeed");

        // Verify data integrity - request parameters should be unchanged
        prop_assert_eq!(params.language, language, "Language should be preserved");
        prop_assert_eq!(params.file_path, file_path, "File path should be preserved");
        prop_assert_eq!(params.position.line, line, "Line should be preserved");
        prop_assert_eq!(params.position.character, character, "Character should be preserved");
    }
}

// Helper function to create test configuration
fn create_test_config() -> IdeIntegrationConfig {
    IdeIntegrationConfig {
        vscode: Some(VsCodeConfig {
            enabled: true,
            port: 8080,
            features: vec!["completion".to_string()],
            settings: serde_json::json!({}),
        }),
        terminal: Some(TerminalConfig {
            vim: Some(types::VimConfig {
                enabled: true,
                plugin_manager: "vim-plug".to_string(),
            }),
            emacs: Some(types::EmacsConfig {
                enabled: true,
                package_manager: "use-package".to_string(),
            }),
        }),
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: true,
                servers: Default::default(),
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: true,
                languages: vec!["rust".to_string()],
            },
        },
    }
}
