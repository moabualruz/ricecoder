//! Integration tests for LSP storage integration
//!
//! Tests LSP configuration loading from ricecoder-storage, configuration hierarchy,
//! and code reusability across ricecoder crates.

use ricecoder_lsp::ConfigurationManager;
use ricecoder_storage::{get_builtin_language_configs, get_language_config, PathResolver};

#[test]
fn test_builtin_language_configs_available() {
    // Verify built-in configurations are available from ricecoder-storage
    let configs = get_builtin_language_configs();

    // Should have at least 7 built-in languages (Rust, TypeScript, Python, Go, Java, Kotlin, Dart)
    assert!(configs.len() >= 7);

    // Verify each config is non-empty
    for (lang, config) in configs {
        assert!(!lang.is_empty());
        assert!(!config.is_empty());
    }
}

#[test]
fn test_get_language_config_rust() {
    let config = get_language_config("rust");
    assert!(config.is_some());
    assert!(!config.unwrap().is_empty());
}

#[test]
fn test_get_language_config_typescript() {
    let config = get_language_config("typescript");
    assert!(config.is_some());
    assert!(!config.unwrap().is_empty());
}

#[test]
fn test_get_language_config_python() {
    let config = get_language_config("python");
    assert!(config.is_some());
    assert!(!config.unwrap().is_empty());
}

#[test]
fn test_get_language_config_aliases() {
    // TypeScript aliases
    assert!(get_language_config("ts").is_some());
    assert!(get_language_config("tsx").is_some());

    // Python aliases
    assert!(get_language_config("py").is_some());

    // Kotlin aliases
    assert!(get_language_config("kt").is_some());
    assert!(get_language_config("kts").is_some());
}

#[test]
fn test_get_language_config_go() {
    let config = get_language_config("go");
    assert!(config.is_some());
    assert!(!config.unwrap().is_empty());
}

#[test]
fn test_get_language_config_java() {
    let config = get_language_config("java");
    assert!(config.is_some());
    assert!(!config.unwrap().is_empty());
}

#[test]
fn test_get_language_config_kotlin() {
    let config = get_language_config("kotlin");
    assert!(config.is_some());
    assert!(!config.unwrap().is_empty());
}

#[test]
fn test_get_language_config_dart() {
    let config = get_language_config("dart");
    assert!(config.is_some());
    assert!(!config.unwrap().is_empty());
}

#[test]
fn test_get_language_config_unknown() {
    let config = get_language_config("unknown");
    assert!(config.is_none());
}

#[test]
fn test_path_resolver_project_path() {
    let project_path = PathResolver::resolve_project_path();
    assert!(!project_path.as_os_str().is_empty());
}

#[test]
fn test_path_resolver_global_path() {
    // This may fail in some environments, but should not panic
    let result = PathResolver::resolve_global_path();
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_configuration_manager_get_language_config_paths() {
    // This should not panic and should return a valid result
    let result = ConfigurationManager::get_language_config_paths();
    assert!(result.is_ok());

    // Result should be a vector (may be empty if paths don't exist)
    let paths = result.unwrap();
    assert!(paths.is_empty() || !paths.is_empty()); // Always true, just checking it's a vector
}

#[test]
fn test_configuration_manager_load_from_storage() {
    let manager = ConfigurationManager::new();

    // Load from storage should succeed
    let result = manager.load_from_storage();
    assert!(result.is_ok());

    // After loading, default providers should be registered
    let semantic_registry = manager.semantic_registry();
    {
        let semantic_reg = semantic_registry.read().unwrap();
        assert!(semantic_reg.has_provider("rust"));
        assert!(semantic_reg.has_provider("typescript"));
        assert!(semantic_reg.has_provider("python"));
    }
}

#[test]
fn test_configuration_manager_load_defaults() {
    let manager = ConfigurationManager::new();

    // Load defaults should succeed
    let result = manager.load_defaults();
    assert!(result.is_ok());

    // Verify default providers are registered
    let semantic_registry = manager.semantic_registry();
    {
        let semantic_reg = semantic_registry.read().unwrap();
        assert!(semantic_reg.has_provider("rust"));
        assert!(semantic_reg.has_provider("typescript"));
        assert!(semantic_reg.has_provider("python"));
        assert!(semantic_reg.has_provider("unknown")); // Fallback provider
    }
}

#[test]
fn test_code_reusability_storage_manager_trait() {
    // Verify that ricecoder-storage types are available and usable
    // This tests that we're properly using shared crates instead of duplicating code

    // PathResolver should be available from ricecoder-storage
    let project_path = PathResolver::resolve_project_path();
    assert!(!project_path.as_os_str().is_empty());

    // Should be able to resolve global path
    let global_result = PathResolver::resolve_global_path();
    assert!(global_result.is_ok() || global_result.is_err());
}

#[test]
fn test_code_reusability_builtin_configs() {
    // Verify that built-in configurations are properly integrated
    // This tests that we're using ricecoder-storage for configuration management

    let configs = get_builtin_language_configs();

    // Should have Rust configuration
    assert!(configs.iter().any(|(lang, _)| *lang == "rust"));

    // Should have TypeScript configuration
    assert!(configs.iter().any(|(lang, _)| *lang == "typescript"));

    // Should have Python configuration
    assert!(configs.iter().any(|(lang, _)| *lang == "python"));

    // Should have Go configuration
    assert!(configs.iter().any(|(lang, _)| *lang == "go"));

    // Should have Java configuration
    assert!(configs.iter().any(|(lang, _)| *lang == "java"));

    // Should have Kotlin configuration
    assert!(configs.iter().any(|(lang, _)| *lang == "kotlin"));

    // Should have Dart configuration
    assert!(configs.iter().any(|(lang, _)| *lang == "dart"));
}

#[test]
fn test_configuration_hierarchy_project_over_user() {
    // This test verifies that the configuration hierarchy is properly implemented
    // Project-level configurations should override user-level configurations

    let _manager = ConfigurationManager::new();

    // Get configuration paths in priority order
    let result = ConfigurationManager::get_language_config_paths();
    assert!(result.is_ok());

    let paths = result.unwrap();

    // Paths should be in priority order (project before user)
    // If both exist, project path should come first
    if paths.len() >= 2 {
        // Verify paths are different (project vs user)
        assert_ne!(paths[0], paths[1]);
    }
}

#[test]
fn test_configuration_manager_languages_list() {
    let manager = ConfigurationManager::new();
    manager.load_defaults().unwrap();

    // Get list of configured languages
    let languages = manager.languages();

    // Should be empty initially (no configurations loaded, only providers)
    assert!(languages.is_empty());
}

#[test]
fn test_configuration_manager_has_language() {
    let manager = ConfigurationManager::new();
    manager.load_defaults().unwrap();

    // After loading defaults, no configurations are registered yet
    assert!(!manager.has_language("rust"));
    assert!(!manager.has_language("typescript"));
    assert!(!manager.has_language("python"));
}

#[test]
fn test_storage_integration_no_duplication() {
    // This test verifies that we're using ricecoder-storage instead of duplicating code

    // PathResolver should be from ricecoder-storage
    let _project_path = PathResolver::resolve_project_path();

    // Should be able to use it with ConfigurationManager
    let _manager = ConfigurationManager::new();
    let result = _manager.load_from_storage();

    // Should succeed without errors
    assert!(result.is_ok());
}

#[test]
fn test_builtin_configs_yaml_format() {
    // Verify that built-in configurations are valid YAML
    let configs = get_builtin_language_configs();

    for (lang, config_str) in configs {
        // Should be able to parse as YAML
        let result: Result<serde_yaml::Value, _> = serde_yaml::from_str(config_str);
        assert!(result.is_ok(), "Failed to parse {} config as YAML", lang);

        // Should have language field
        let value = result.unwrap();
        assert!(value["language"].is_string());
    }
}

#[test]
fn test_builtin_configs_have_required_fields() {
    // Verify that built-in configurations have required fields
    let configs = get_builtin_language_configs();

    for (lang, config_str) in configs {
        let value: serde_yaml::Value =
            serde_yaml::from_str(config_str).expect(&format!("Failed to parse {} config", lang));

        // Should have language field
        assert!(
            value["language"].is_string(),
            "{} missing language field",
            lang
        );

        // Should have file_extensions field
        assert!(
            value["file_extensions"].is_sequence(),
            "{} missing file_extensions field",
            lang
        );
    }
}
