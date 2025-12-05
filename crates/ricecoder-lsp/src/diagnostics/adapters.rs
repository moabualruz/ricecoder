//! Adapter implementations for existing diagnostics engines to implement DiagnosticsProvider trait
//!
//! This module provides adapter wrappers that allow existing language-specific diagnostics
//! to be used as pluggable providers in the configuration-driven architecture.

use crate::config::LanguageConfig;
use crate::diagnostics::{python_rules, rust_rules, typescript_rules};
use crate::providers::{DiagnosticsProvider, ProviderResult};
use crate::types::Diagnostic;

/// Adapter for Rust diagnostics provider
pub struct RustDiagnosticsAdapter {
    config: Option<LanguageConfig>,
}

impl RustDiagnosticsAdapter {
    /// Create a new Rust diagnostics adapter
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

impl Default for RustDiagnosticsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsProvider for RustDiagnosticsAdapter {
    fn language(&self) -> &str {
        "rust"
    }

    fn generate_diagnostics(&self, code: &str) -> ProviderResult<Vec<Diagnostic>> {
        rust_rules::generate_rust_diagnostics(code)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }

    fn config(&self) -> Option<&LanguageConfig> {
        self.config.as_ref()
    }
}

/// Adapter for TypeScript diagnostics provider
pub struct TypeScriptDiagnosticsAdapter {
    config: Option<LanguageConfig>,
}

impl TypeScriptDiagnosticsAdapter {
    /// Create a new TypeScript diagnostics adapter
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

impl Default for TypeScriptDiagnosticsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsProvider for TypeScriptDiagnosticsAdapter {
    fn language(&self) -> &str {
        "typescript"
    }

    fn generate_diagnostics(&self, code: &str) -> ProviderResult<Vec<Diagnostic>> {
        typescript_rules::generate_typescript_diagnostics(code)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }

    fn config(&self) -> Option<&LanguageConfig> {
        self.config.as_ref()
    }
}

/// Adapter for Python diagnostics provider
pub struct PythonDiagnosticsAdapter {
    config: Option<LanguageConfig>,
}

impl PythonDiagnosticsAdapter {
    /// Create a new Python diagnostics adapter
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

impl Default for PythonDiagnosticsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsProvider for PythonDiagnosticsAdapter {
    fn language(&self) -> &str {
        "python"
    }

    fn generate_diagnostics(&self, code: &str) -> ProviderResult<Vec<Diagnostic>> {
        python_rules::generate_python_diagnostics(code)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }

    fn config(&self) -> Option<&LanguageConfig> {
        self.config.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_diagnostics_adapter_language() {
        let adapter = RustDiagnosticsAdapter::new();
        assert_eq!(adapter.language(), "rust");
    }

    #[test]
    fn test_typescript_diagnostics_adapter_language() {
        let adapter = TypeScriptDiagnosticsAdapter::new();
        assert_eq!(adapter.language(), "typescript");
    }

    #[test]
    fn test_python_diagnostics_adapter_language() {
        let adapter = PythonDiagnosticsAdapter::new();
        assert_eq!(adapter.language(), "python");
    }

    #[test]
    fn test_rust_diagnostics_adapter_no_config() {
        let adapter = RustDiagnosticsAdapter::new();
        assert!(adapter.config().is_none());
    }
}
