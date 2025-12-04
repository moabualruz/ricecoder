//! Adapter implementations for existing code actions to implement CodeActionProvider trait
//!
//! This module provides adapter wrappers that allow existing language-specific code actions
//! to be used as pluggable providers in the configuration-driven architecture.

use crate::providers::{CodeActionProvider, ProviderResult};
use crate::types::Diagnostic;
use crate::config::LanguageConfig;

/// Adapter for Rust code actions provider
pub struct RustCodeActionAdapter {
    config: Option<LanguageConfig>,
}

impl RustCodeActionAdapter {
    /// Create a new Rust code action adapter
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Create with configuration
    pub fn with_config(config: LanguageConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

impl Default for RustCodeActionAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeActionProvider for RustCodeActionAdapter {
    fn language(&self) -> &str {
        "rust"
    }

    fn suggest_actions(&self, _diagnostic: &Diagnostic, _code: &str) -> ProviderResult<Vec<String>> {
        // Rust-specific code action suggestions
        Ok(vec![])
    }

    fn apply_action(&self, code: &str, _action: &str) -> ProviderResult<String> {
        // Apply Rust-specific code action
        Ok(code.to_string())
    }

    fn config(&self) -> Option<&LanguageConfig> {
        self.config.as_ref()
    }
}

/// Adapter for TypeScript code actions provider
pub struct TypeScriptCodeActionAdapter {
    config: Option<LanguageConfig>,
}

impl TypeScriptCodeActionAdapter {
    /// Create a new TypeScript code action adapter
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Create with configuration
    pub fn with_config(config: LanguageConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

impl Default for TypeScriptCodeActionAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeActionProvider for TypeScriptCodeActionAdapter {
    fn language(&self) -> &str {
        "typescript"
    }

    fn suggest_actions(&self, _diagnostic: &Diagnostic, _code: &str) -> ProviderResult<Vec<String>> {
        // TypeScript-specific code action suggestions
        Ok(vec![])
    }

    fn apply_action(&self, code: &str, _action: &str) -> ProviderResult<String> {
        // Apply TypeScript-specific code action
        Ok(code.to_string())
    }

    fn config(&self) -> Option<&LanguageConfig> {
        self.config.as_ref()
    }
}

/// Adapter for Python code actions provider
pub struct PythonCodeActionAdapter {
    config: Option<LanguageConfig>,
}

impl PythonCodeActionAdapter {
    /// Create a new Python code action adapter
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Create with configuration
    pub fn with_config(config: LanguageConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

impl Default for PythonCodeActionAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeActionProvider for PythonCodeActionAdapter {
    fn language(&self) -> &str {
        "python"
    }

    fn suggest_actions(&self, _diagnostic: &Diagnostic, _code: &str) -> ProviderResult<Vec<String>> {
        // Python-specific code action suggestions
        Ok(vec![])
    }

    fn apply_action(&self, code: &str, _action: &str) -> ProviderResult<String> {
        // Apply Python-specific code action
        Ok(code.to_string())
    }

    fn config(&self) -> Option<&LanguageConfig> {
        self.config.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_code_action_adapter_language() {
        let adapter = RustCodeActionAdapter::new();
        assert_eq!(adapter.language(), "rust");
    }

    #[test]
    fn test_typescript_code_action_adapter_language() {
        let adapter = TypeScriptCodeActionAdapter::new();
        assert_eq!(adapter.language(), "typescript");
    }

    #[test]
    fn test_python_code_action_adapter_language() {
        let adapter = PythonCodeActionAdapter::new();
        assert_eq!(adapter.language(), "python");
    }

    #[test]
    fn test_rust_code_action_adapter_no_config() {
        let adapter = RustCodeActionAdapter::new();
        assert!(adapter.config().is_none());
    }
}
