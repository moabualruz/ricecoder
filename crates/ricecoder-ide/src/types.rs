//! Core data types for IDE integration

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// IDE Integration Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeIntegrationConfig {
    /// VS Code configuration
    pub vscode: Option<VsCodeConfig>,
    /// Terminal editor configuration
    pub terminal: Option<TerminalConfig>,
    /// Provider chain configuration
    pub providers: ProviderChainConfig,
}

/// VS Code specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsCodeConfig {
    /// Whether VS Code integration is enabled
    pub enabled: bool,
    /// Port for communication
    pub port: u16,
    /// Enabled features
    pub features: Vec<String>,
    /// VS Code settings
    #[serde(default)]
    pub settings: serde_json::Value,
}

/// Terminal editor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Vim/Neovim configuration
    pub vim: Option<VimConfig>,
    /// Emacs configuration
    pub emacs: Option<EmacsConfig>,
}

/// Vim/Neovim configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimConfig {
    /// Whether vim integration is enabled
    pub enabled: bool,
    /// Plugin manager (vim-plug, packer, etc.)
    pub plugin_manager: String,
}

/// Emacs configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmacsConfig {
    /// Whether emacs integration is enabled
    pub enabled: bool,
    /// Package manager (use-package, straight.el, etc.)
    pub package_manager: String,
}

/// Provider chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderChainConfig {
    /// External LSP configuration
    pub external_lsp: ExternalLspConfig,
    /// Configured rules configuration
    pub configured_rules: Option<ConfiguredRulesConfig>,
    /// Built-in providers configuration
    pub builtin_providers: BuiltinProvidersConfig,
}

/// External LSP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalLspConfig {
    /// Whether external LSP is enabled
    pub enabled: bool,
    /// LSP server configurations by language
    pub servers: HashMap<String, LspServerConfig>,
    /// Health check interval in milliseconds
    pub health_check_interval_ms: u64,
}

/// LSP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspServerConfig {
    /// Language this server supports
    pub language: String,
    /// Command to start the server
    pub command: String,
    /// Arguments for the command
    pub args: Vec<String>,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

/// Configured rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguredRulesConfig {
    /// Whether configured rules are enabled
    pub enabled: bool,
    /// Path to rules file (resolved via ricecoder_storage::PathResolver)
    pub rules_path: String,
}

/// Built-in providers configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinProvidersConfig {
    /// Whether built-in providers are enabled
    pub enabled: bool,
    /// Languages supported by built-in providers
    pub languages: Vec<String>,
}

/// Completion request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionParams {
    /// Programming language
    pub language: String,
    /// File path (resolved via ricecoder_storage::PathResolver)
    pub file_path: String,
    /// Cursor position
    pub position: Position,
    /// Context around cursor
    pub context: String,
}

/// Diagnostics request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsParams {
    /// Programming language
    pub language: String,
    /// File path (resolved via ricecoder_storage::PathResolver)
    pub file_path: String,
    /// Source code
    pub source: String,
}

/// Hover request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverParams {
    /// Programming language
    pub language: String,
    /// File path (resolved via ricecoder_storage::PathResolver)
    pub file_path: String,
    /// Cursor position
    pub position: Position,
}

/// Definition request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionParams {
    /// Programming language
    pub language: String,
    /// File path (resolved via ricecoder_storage::PathResolver)
    pub file_path: String,
    /// Cursor position
    pub position: Position,
}

/// Diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Range of the diagnostic
    pub range: Range,
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Diagnostic message
    pub message: String,
    /// Source of the diagnostic
    pub source: String,
}

/// Diagnostic severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    /// Error severity
    Error,
    /// Warning severity
    Warning,
    /// Information severity
    Information,
    /// Hint severity
    Hint,
}

/// Completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Display label
    pub label: String,
    /// Completion kind
    pub kind: CompletionItemKind,
    /// Additional details
    pub detail: Option<String>,
    /// Documentation
    pub documentation: Option<String>,
    /// Text to insert
    pub insert_text: String,
}

/// Completion item kind
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CompletionItemKind {
    /// Text completion
    Text,
    /// Method completion
    Method,
    /// Function completion
    Function,
    /// Constructor completion
    Constructor,
    /// Field completion
    Field,
    /// Variable completion
    Variable,
    /// Class completion
    Class,
    /// Interface completion
    Interface,
    /// Module completion
    Module,
    /// Property completion
    Property,
    /// Unit completion
    Unit,
    /// Value completion
    Value,
    /// Enum completion
    Enum,
    /// Keyword completion
    Keyword,
    /// Snippet completion
    Snippet,
    /// Color completion
    Color,
    /// File completion
    File,
    /// Reference completion
    Reference,
    /// Folder completion
    Folder,
    /// Enum member completion
    EnumMember,
    /// Constant completion
    Constant,
    /// Struct completion
    Struct,
    /// Event completion
    Event,
    /// Operator completion
    Operator,
    /// Type parameter completion
    TypeParameter,
}

/// Hover information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hover {
    /// Hover contents
    pub contents: String,
    /// Range of the hover
    pub range: Option<Range>,
}

/// Location in a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// File path (resolved via ricecoder_storage::PathResolver)
    pub file_path: String,
    /// Range in the file
    pub range: Range,
}

/// Position in a file
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    /// Line number (0-based)
    pub line: u32,
    /// Character position (0-based)
    pub character: u32,
}

/// Range in a file
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Range {
    /// Start position
    pub start: Position,
    /// End position
    pub end: Position,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position {
            line: 10,
            character: 5,
        };
        assert_eq!(pos.line, 10);
        assert_eq!(pos.character, 5);
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
        assert_eq!(range.end.character, 10);
    }

    #[test]
    fn test_completion_item_serialization() {
        let item = CompletionItem {
            label: "test".to_string(),
            kind: CompletionItemKind::Function,
            detail: Some("test function".to_string()),
            documentation: None,
            insert_text: "test()".to_string(),
        };

        let json = serde_json::to_string(&item).unwrap();
        let deserialized: CompletionItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.label, "test");
        assert_eq!(deserialized.kind, CompletionItemKind::Function);
    }

    #[test]
    fn test_diagnostic_severity_serialization() {
        let severity = DiagnosticSeverity::Error;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"error\"");

        let deserialized: DiagnosticSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, DiagnosticSeverity::Error);
    }
}
