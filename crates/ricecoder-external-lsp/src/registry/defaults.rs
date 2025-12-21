//! Default LSP server configurations for Tier 1 servers

use crate::types::{GlobalLspSettings, LspServerConfig, LspServerRegistry};
use std::collections::HashMap;

/// Provides default configurations for built-in LSP servers
pub struct DefaultServerConfigs;

impl DefaultServerConfigs {
    /// Get default registry with Tier 1 servers pre-configured
    pub fn tier1_registry() -> LspServerRegistry {
        LspServerRegistry {
            servers: HashMap::from([
                ("rust".to_string(), vec![Self::rust_analyzer()]),
                (
                    "typescript".to_string(),
                    vec![Self::typescript_language_server()],
                ),
                ("python".to_string(), vec![Self::pylsp()]),
            ]),
            global: GlobalLspSettings::default(),
        }
    }

    /// Get default registry with Tier 1 and Tier 2 servers
    pub fn tier1_and_tier2_registry() -> LspServerRegistry {
        let mut registry = Self::tier1_registry();

        registry
            .servers
            .insert("go".to_string(), vec![Self::gopls()]);
        registry
            .servers
            .insert("java".to_string(), vec![Self::jdtls()]);
        registry
            .servers
            .insert("kotlin".to_string(), vec![Self::kotlin_language_server()]);
        registry
            .servers
            .insert("dart".to_string(), vec![Self::dart_language_server()]);

        registry
    }

    /// Get default registry with all Tier 1, 2, and 3 servers
    pub fn all_tiers_registry() -> LspServerRegistry {
        let mut registry = Self::tier1_and_tier2_registry();

        registry
            .servers
            .insert("c".to_string(), vec![Self::clangd()]);
        registry
            .servers
            .insert("cpp".to_string(), vec![Self::clangd()]);
        registry
            .servers
            .insert("csharp".to_string(), vec![Self::omnisharp()]);
        registry
            .servers
            .insert("ruby".to_string(), vec![Self::solargraph()]);
        registry
            .servers
            .insert("php".to_string(), vec![Self::intelephense()]);

        registry
    }

    // Tier 1 Servers

    /// Rust-analyzer configuration
    pub fn rust_analyzer() -> LspServerConfig {
        LspServerConfig {
            language: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            executable: "rust-analyzer".to_string(),
            args: vec![],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 10000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }
    }

    /// TypeScript Language Server configuration
    pub fn typescript_language_server() -> LspServerConfig {
        LspServerConfig {
            language: "typescript".to_string(),
            extensions: vec![
                ".ts".to_string(),
                ".tsx".to_string(),
                ".js".to_string(),
                ".jsx".to_string(),
            ],
            executable: "typescript-language-server".to_string(),
            args: vec!["--stdio".to_string()],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }
    }

    /// Python Language Server (pylsp) configuration
    pub fn pylsp() -> LspServerConfig {
        LspServerConfig {
            language: "python".to_string(),
            extensions: vec![".py".to_string()],
            executable: "pylsp".to_string(),
            args: vec![],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }
    }

    // Tier 2 Servers

    /// Go Language Server (gopls) configuration
    fn gopls() -> LspServerConfig {
        LspServerConfig {
            language: "go".to_string(),
            extensions: vec![".go".to_string()],
            executable: "gopls".to_string(),
            args: vec!["serve".to_string()],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }
    }

    /// Java Development Tools Language Server configuration
    fn jdtls() -> LspServerConfig {
        LspServerConfig {
            language: "java".to_string(),
            extensions: vec![".java".to_string()],
            executable: "jdtls".to_string(),
            args: vec![],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 10000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }
    }

    /// Kotlin Language Server configuration
    fn kotlin_language_server() -> LspServerConfig {
        LspServerConfig {
            language: "kotlin".to_string(),
            extensions: vec![".kt".to_string(), ".kts".to_string()],
            executable: "kotlin-language-server".to_string(),
            args: vec![],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 10000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }
    }

    /// Dart Language Server configuration
    fn dart_language_server() -> LspServerConfig {
        LspServerConfig {
            language: "dart".to_string(),
            extensions: vec![".dart".to_string()],
            executable: "dart".to_string(),
            args: vec!["language-server".to_string()],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }
    }

    // Tier 3 Servers

    /// Clang Language Server configuration
    fn clangd() -> LspServerConfig {
        LspServerConfig {
            language: "c".to_string(),
            extensions: vec![".c".to_string(), ".h".to_string()],
            executable: "clangd".to_string(),
            args: vec![],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 10000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }
    }

    /// OmniSharp (.NET) Language Server configuration
    fn omnisharp() -> LspServerConfig {
        LspServerConfig {
            language: "csharp".to_string(),
            extensions: vec![".cs".to_string()],
            executable: "OmniSharp".to_string(),
            args: vec!["-lsp".to_string()],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 10000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }
    }

    /// Solargraph (Ruby) Language Server configuration
    fn solargraph() -> LspServerConfig {
        LspServerConfig {
            language: "ruby".to_string(),
            extensions: vec![".rb".to_string()],
            executable: "solargraph".to_string(),
            args: vec!["stdio".to_string()],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }
    }

    /// Intelephense (PHP) Language Server configuration
    fn intelephense() -> LspServerConfig {
        LspServerConfig {
            language: "php".to_string(),
            extensions: vec![".php".to_string()],
            executable: "intelephense".to_string(),
            args: vec!["--stdio".to_string()],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier1_registry() {
        let registry = DefaultServerConfigs::tier1_registry();
        assert_eq!(registry.servers.len(), 3);
        assert!(registry.servers.contains_key("rust"));
        assert!(registry.servers.contains_key("typescript"));
        assert!(registry.servers.contains_key("python"));
    }

    #[test]
    fn test_tier1_and_tier2_registry() {
        let registry = DefaultServerConfigs::tier1_and_tier2_registry();
        assert_eq!(registry.servers.len(), 7);
        assert!(registry.servers.contains_key("go"));
        assert!(registry.servers.contains_key("java"));
        assert!(registry.servers.contains_key("kotlin"));
        assert!(registry.servers.contains_key("dart"));
    }

    #[test]
    fn test_all_tiers_registry() {
        let registry = DefaultServerConfigs::all_tiers_registry();
        assert_eq!(registry.servers.len(), 12);
        assert!(registry.servers.contains_key("c"));
        assert!(registry.servers.contains_key("cpp"));
        assert!(registry.servers.contains_key("csharp"));
        assert!(registry.servers.contains_key("ruby"));
        assert!(registry.servers.contains_key("php"));
    }

    #[test]
    fn test_rust_analyzer_config() {
        let config = DefaultServerConfigs::rust_analyzer();
        assert_eq!(config.language, "rust");
        assert_eq!(config.executable, "rust-analyzer");
        assert!(config.extensions.contains(&".rs".to_string()));
        assert_eq!(config.timeout_ms, 10000);
    }

    #[test]
    fn test_typescript_config() {
        let config = DefaultServerConfigs::typescript_language_server();
        assert_eq!(config.language, "typescript");
        assert_eq!(config.executable, "typescript-language-server");
        assert!(config.extensions.contains(&".ts".to_string()));
        assert!(config.extensions.contains(&".js".to_string()));
    }

    #[test]
    fn test_python_config() {
        let config = DefaultServerConfigs::pylsp();
        assert_eq!(config.language, "python");
        assert_eq!(config.executable, "pylsp");
        assert!(config.extensions.contains(&".py".to_string()));
    }
}
