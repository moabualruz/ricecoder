//! Integration tests for configuration-driven LSP approach
//!
//! Tests LSP server with multiple language configurations, configuration loading,
//! hot-reload, and fallback behavior for unconfigured languages.

use ricecoder_lsp::{
    code_actions::{
        adapters::{PythonCodeActionAdapter, RustCodeActionAdapter, TypeScriptCodeActionAdapter},
        GenericCodeActionsEngine,
    },
    diagnostics::{
        adapters::{
            PythonDiagnosticsAdapter, RustDiagnosticsAdapter, TypeScriptDiagnosticsAdapter,
        },
        GenericDiagnosticsEngine,
    },
    semantic::{
        adapters::{PythonAnalyzerAdapter, RustAnalyzerAdapter, TypeScriptAnalyzerAdapter},
        GenericSemanticAnalyzer,
    },
    CodeActionTemplate, ConfigRegistry, ConfigurationManager, DiagnosticRule, LanguageConfig,
};

#[test]
fn test_lsp_server_with_multiple_language_configurations() {
    let mut registry = ConfigRegistry::new();

    // Create configurations for multiple languages
    let rust_config = LanguageConfig {
        language: "rust".to_string(),
        extensions: vec!["rs".to_string()],
        parser_plugin: Some("tree-sitter-rust".to_string()),
        diagnostic_rules: vec![],
        code_actions: vec![],
    };

    let typescript_config = LanguageConfig {
        language: "typescript".to_string(),
        extensions: vec!["ts".to_string(), "tsx".to_string()],
        parser_plugin: Some("tree-sitter-typescript".to_string()),
        diagnostic_rules: vec![],
        code_actions: vec![],
    };

    let python_config = LanguageConfig {
        language: "python".to_string(),
        extensions: vec!["py".to_string()],
        parser_plugin: Some("tree-sitter-python".to_string()),
        diagnostic_rules: vec![],
        code_actions: vec![],
    };

    // Register all configurations
    assert!(registry.register(rust_config).is_ok());
    assert!(registry.register(typescript_config).is_ok());
    assert!(registry.register(python_config).is_ok());

    // Verify all languages are registered
    assert!(registry.has_language("rust"));
    assert!(registry.has_language("typescript"));
    assert!(registry.has_language("python"));

    // Verify we can retrieve each configuration
    assert!(registry.get("rust").is_some());
    assert!(registry.get("typescript").is_some());
    assert!(registry.get("python").is_some());

    // Verify we can find by extension
    assert!(registry.get_by_extension("rs").is_some());
    assert!(registry.get_by_extension("ts").is_some());
    assert!(registry.get_by_extension("py").is_some());
}

#[test]
fn test_configuration_loading_and_validation() {
    let mut registry = ConfigRegistry::new();

    // Create a valid configuration
    let config = LanguageConfig {
        language: "go".to_string(),
        extensions: vec!["go".to_string()],
        parser_plugin: Some("tree-sitter-go".to_string()),
        diagnostic_rules: vec![DiagnosticRule {
            name: "unused-import".to_string(),
            pattern: "import".to_string(),
            severity: "warning".to_string(),
            message: "Unused import".to_string(),
            fix_template: None,
            code: Some("unused-import".to_string()),
        }],
        code_actions: vec![CodeActionTemplate {
            name: "remove-import".to_string(),
            title: "Remove import".to_string(),
            kind: "quickfix".to_string(),
            transformation: "delete_line".to_string(),
        }],
    };

    // Registration should succeed
    assert!(registry.register(config.clone()).is_ok());

    // Verify the configuration is stored correctly
    let retrieved = registry.get("go").unwrap();
    assert_eq!(retrieved.language, "go");
    assert_eq!(retrieved.extensions.len(), 1);
    assert_eq!(retrieved.diagnostic_rules.len(), 1);
    assert_eq!(retrieved.code_actions.len(), 1);
}

#[test]
fn test_configuration_hot_reload() {
    let mut registry = ConfigRegistry::new();

    // Create initial configuration
    let config1 = LanguageConfig {
        language: "rust".to_string(),
        extensions: vec!["rs".to_string()],
        parser_plugin: Some("tree-sitter-rust".to_string()),
        diagnostic_rules: vec![],
        code_actions: vec![],
    };

    // Register initial configuration
    assert!(registry.register(config1).is_ok());
    let retrieved1_len = registry.get("rust").unwrap().extensions.len();
    assert_eq!(retrieved1_len, 1);

    // Create updated configuration with more extensions
    let config2 = LanguageConfig {
        language: "rust".to_string(),
        extensions: vec!["rs".to_string(), "rlib".to_string()],
        parser_plugin: Some("tree-sitter-rust".to_string()),
        diagnostic_rules: vec![],
        code_actions: vec![],
    };

    // Register updated configuration (hot-reload)
    assert!(registry.register(config2).is_ok());
    let retrieved2_len = registry.get("rust").unwrap().extensions.len();
    assert_eq!(retrieved2_len, 2);

    // Verify the configuration was updated
    assert_ne!(retrieved1_len, retrieved2_len);
}

#[test]
fn test_fallback_behavior_for_unconfigured_languages() {
    let registry = ConfigRegistry::new();

    // Unconfigured language should return None
    assert!(registry.get("unknown").is_none());
    assert!(!registry.has_language("unknown"));

    // But we should be able to check without panicking
    let languages = registry.languages();
    assert!(languages.is_empty());
}

#[test]
fn test_configuration_manager_initialization() {
    let manager = ConfigurationManager::new();

    // Load defaults
    assert!(manager.load_defaults().is_ok());

    // Verify default providers are registered
    let semantic_registry = manager.semantic_registry();
    {
        let semantic_reg = semantic_registry.read().unwrap();
        assert!(semantic_reg.has_provider("rust"));
        assert!(semantic_reg.has_provider("typescript"));
        assert!(semantic_reg.has_provider("python"));
        assert!(semantic_reg.has_provider("unknown"));
    }

    let diag_registry = manager.diagnostics_registry();
    {
        let diag_reg = diag_registry.read().unwrap();
        assert!(diag_reg.has_provider("rust"));
        assert!(diag_reg.has_provider("typescript"));
        assert!(diag_reg.has_provider("python"));
    }

    let action_registry = manager.code_action_registry();
    {
        let action_reg = action_registry.read().unwrap();
        assert!(action_reg.has_provider("rust"));
        assert!(action_reg.has_provider("typescript"));
        assert!(action_reg.has_provider("python"));
    }
}

#[test]
fn test_generic_semantic_analyzer_with_providers() {
    let mut analyzer = GenericSemanticAnalyzer::new();

    // Register providers
    analyzer.register_provider(Box::new(RustAnalyzerAdapter::new()));
    analyzer.register_provider(Box::new(TypeScriptAnalyzerAdapter::new()));
    analyzer.register_provider(Box::new(PythonAnalyzerAdapter::new()));

    // Verify providers are registered
    assert!(analyzer.has_provider("rust"));
    assert!(analyzer.has_provider("typescript"));
    assert!(analyzer.has_provider("python"));

    // Verify we can analyze with each provider
    assert!(analyzer.analyze("fn main() {}", "rust").is_ok());
    assert!(analyzer.analyze("const x = 1;", "typescript").is_ok());
    assert!(analyzer.analyze("def foo(): pass", "python").is_ok());

    // Verify fallback for unknown language
    assert!(analyzer.analyze("unknown code", "unknown").is_ok());
}

#[test]
fn test_generic_diagnostics_engine_with_providers() {
    let mut engine = GenericDiagnosticsEngine::new();

    // Register providers
    engine.register_provider(Box::new(RustDiagnosticsAdapter::new()));
    engine.register_provider(Box::new(TypeScriptDiagnosticsAdapter::new()));
    engine.register_provider(Box::new(PythonDiagnosticsAdapter::new()));

    // Verify providers are registered
    assert!(engine.has_provider("rust"));
    assert!(engine.has_provider("typescript"));
    assert!(engine.has_provider("python"));

    // Verify we can generate diagnostics with each provider
    assert!(engine.generate_diagnostics("fn main() {}", "rust").is_ok());
    assert!(engine
        .generate_diagnostics("const x = 1;", "typescript")
        .is_ok());
    assert!(engine
        .generate_diagnostics("def foo(): pass", "python")
        .is_ok());

    // Verify fallback for unknown language (should return empty)
    let result = engine.generate_diagnostics("unknown code", "unknown");
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_generic_code_actions_engine_with_providers() {
    let mut engine = GenericCodeActionsEngine::new();

    // Register providers
    engine.register_provider(Box::new(RustCodeActionAdapter::new()));
    engine.register_provider(Box::new(TypeScriptCodeActionAdapter::new()));
    engine.register_provider(Box::new(PythonCodeActionAdapter::new()));

    // Verify providers are registered
    assert!(engine.has_provider("rust"));
    assert!(engine.has_provider("typescript"));
    assert!(engine.has_provider("python"));

    // Create a test diagnostic
    use ricecoder_lsp::types::{Diagnostic, DiagnosticSeverity, Position, Range};
    let test_diagnostic = Diagnostic::new(
        Range::new(Position::new(0, 0), Position::new(0, 5)),
        DiagnosticSeverity::Error,
        "test".to_string(),
    );

    // Verify we can suggest actions with each provider
    assert!(engine
        .suggest_actions(&test_diagnostic, "fn main() {}", "rust")
        .is_ok());
    assert!(engine
        .suggest_actions(&test_diagnostic, "const x = 1;", "typescript")
        .is_ok());
    assert!(engine
        .suggest_actions(&test_diagnostic, "def foo(): pass", "python")
        .is_ok());

    // Verify fallback for unknown language (should return empty)
    let result = engine.suggest_actions(&test_diagnostic, "unknown code", "unknown");
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_configuration_with_diagnostic_rules() {
    let mut registry = ConfigRegistry::new();

    let config = LanguageConfig {
        language: "rust".to_string(),
        extensions: vec!["rs".to_string()],
        parser_plugin: Some("tree-sitter-rust".to_string()),
        diagnostic_rules: vec![
            DiagnosticRule {
                name: "unused-import".to_string(),
                pattern: "use .*".to_string(),
                severity: "warning".to_string(),
                message: "Unused import".to_string(),
                fix_template: Some("Remove import".to_string()),
                code: Some("unused-import".to_string()),
            },
            DiagnosticRule {
                name: "missing-semicolon".to_string(),
                pattern: ".*;".to_string(),
                severity: "error".to_string(),
                message: "Missing semicolon".to_string(),
                fix_template: Some("Add semicolon".to_string()),
                code: Some("missing-semicolon".to_string()),
            },
        ],
        code_actions: vec![],
    };

    assert!(registry.register(config).is_ok());

    let retrieved = registry.get("rust").unwrap();
    assert_eq!(retrieved.diagnostic_rules.len(), 2);
    assert_eq!(retrieved.diagnostic_rules[0].name, "unused-import");
    assert_eq!(retrieved.diagnostic_rules[1].name, "missing-semicolon");
}

#[test]
fn test_configuration_with_code_actions() {
    let mut registry = ConfigRegistry::new();

    let config = LanguageConfig {
        language: "typescript".to_string(),
        extensions: vec!["ts".to_string()],
        parser_plugin: Some("tree-sitter-typescript".to_string()),
        diagnostic_rules: vec![],
        code_actions: vec![
            CodeActionTemplate {
                name: "remove-unused".to_string(),
                title: "Remove unused variable".to_string(),
                kind: "quickfix".to_string(),
                transformation: "delete_line".to_string(),
            },
            CodeActionTemplate {
                name: "add-type".to_string(),
                title: "Add type annotation".to_string(),
                kind: "refactor".to_string(),
                transformation: "add_type_annotation".to_string(),
            },
        ],
    };

    assert!(registry.register(config).is_ok());

    let retrieved = registry.get("typescript").unwrap();
    assert_eq!(retrieved.code_actions.len(), 2);
    assert_eq!(retrieved.code_actions[0].name, "remove-unused");
    assert_eq!(retrieved.code_actions[1].name, "add-type");
}

#[test]
fn test_multiple_languages_no_interference() {
    let mut registry = ConfigRegistry::new();

    // Register multiple languages
    for lang in &["rust", "typescript", "python", "go", "java"] {
        let config = LanguageConfig {
            language: lang.to_string(),
            extensions: vec![format!(".{}", lang)],
            parser_plugin: Some(format!("tree-sitter-{}", lang)),
            diagnostic_rules: vec![],
            code_actions: vec![],
        };
        assert!(registry.register(config).is_ok());
    }

    // Verify all are registered
    for lang in &["rust", "typescript", "python", "go", "java"] {
        assert!(registry.has_language(lang));
        assert!(registry.get(lang).is_some());
    }

    // Verify count
    assert_eq!(registry.languages().len(), 5);

    // Verify no interference between languages
    let rust_config = registry.get("rust").unwrap();
    let ts_config = registry.get("typescript").unwrap();
    assert_ne!(rust_config.language, ts_config.language);
}
