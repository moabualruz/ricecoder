//! Core LSP types and data structures
//!
//! This module defines all the fundamental types used throughout the LSP integration,
//! including error types, message types, and domain models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Result type for LSP operations
pub type LspResult<T> = Result<T, LspError>;

/// LSP-specific error type
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum LspError {
    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Method not found
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Server error
    #[error("Server error: {0}")]
    ServerError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),
}

/// Server state tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Server is initializing
    Initializing,
    /// Server is initialized and ready
    Initialized,
    /// Server is shutting down
    ShuttingDown,
    /// Server is shut down
    ShutDown,
}

/// Position in a document (line and character)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Line number (0-based)
    pub line: u32,
    /// Character offset (0-based)
    pub character: u32,
}

impl Position {
    /// Create a new position
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// Range in a document (start and end positions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    /// Start position
    pub start: Position,
    /// End position
    pub end: Position,
}

impl Range {
    /// Create a new range
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

/// Symbol kind enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SymbolKind {
    /// Function symbol
    Function,
    /// Type symbol
    Type,
    /// Variable symbol
    Variable,
    /// Constant symbol
    Constant,
    /// Module symbol
    Module,
    /// Class symbol
    Class,
    /// Interface symbol
    Interface,
    /// Enum symbol
    Enum,
    /// Trait symbol
    Trait,
    /// Struct symbol
    Struct,
    /// Method symbol
    Method,
    /// Property symbol
    Property,
    /// Field symbol
    Field,
    /// Parameter symbol
    Parameter,
}

/// Symbol definition information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    /// File URI where symbol is defined
    pub uri: String,
    /// Range of the definition
    pub range: Range,
}

/// Symbol reference information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    /// File URI where symbol is referenced
    pub uri: String,
    /// Range of the reference
    pub range: Range,
}

/// Symbol information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Range of the symbol
    pub range: Range,
    /// Definition location
    pub definition: Option<Definition>,
    /// References to this symbol
    pub references: Vec<Reference>,
    /// Documentation for the symbol
    pub documentation: Option<String>,
}

impl Symbol {
    /// Create a new symbol
    pub fn new(name: String, kind: SymbolKind, range: Range) -> Self {
        Self {
            name,
            kind,
            range,
            definition: None,
            references: Vec::new(),
            documentation: None,
        }
    }
}

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Range of the diagnostic
    pub range: Range,
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Diagnostic message
    pub message: String,
    /// Diagnostic code
    pub code: Option<String>,
    /// Source of the diagnostic
    pub source: String,
    /// Related information
    pub related_information: Option<Vec<DiagnosticRelatedInformation>>,
}

impl Diagnostic {
    /// Create a new diagnostic
    pub fn new(range: Range, severity: DiagnosticSeverity, message: String) -> Self {
        Self {
            range,
            severity,
            message,
            code: None,
            source: "ricecoder-lsp".to_string(),
            related_information: None,
        }
    }
}

/// Related diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRelatedInformation {
    /// Location of the related information
    pub location: Location,
    /// Message for the related information
    pub message: String,
}

/// Location in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// File URI
    pub uri: String,
    /// Range in the file
    pub range: Range,
}

/// Code action kind
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CodeActionKind {
    /// Quick fix
    QuickFix,
    /// Refactor
    Refactor,
    /// Refactor extract
    RefactorExtract,
    /// Refactor inline
    RefactorInline,
    /// Refactor rewrite
    RefactorRewrite,
    /// Source
    Source,
    /// Source organize imports
    SourceOrganizeImports,
    /// Source fix all
    SourceFixAll,
}

/// Text edit for code modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    /// Range to replace
    pub range: Range,
    /// New text
    pub new_text: String,
}

/// Workspace edit for multi-file modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEdit {
    /// Changes by file URI
    pub changes: HashMap<String, Vec<TextEdit>>,
}

impl WorkspaceEdit {
    /// Create a new workspace edit
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
        }
    }

    /// Add a text edit for a file
    pub fn add_edit(&mut self, uri: String, edit: TextEdit) {
        self.changes.entry(uri).or_default().push(edit);
    }
}

impl Default for WorkspaceEdit {
    fn default() -> Self {
        Self::new()
    }
}

/// Code action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAction {
    /// Action title
    pub title: String,
    /// Action kind
    pub kind: CodeActionKind,
    /// Workspace edit
    pub edit: WorkspaceEdit,
    /// Associated diagnostics
    pub diagnostics: Option<Vec<Diagnostic>>,
}

impl CodeAction {
    /// Create a new code action
    pub fn new(title: String, kind: CodeActionKind, edit: WorkspaceEdit) -> Self {
        Self {
            title,
            kind,
            edit,
            diagnostics: None,
        }
    }
}

/// Markup content kind
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkupKind {
    /// Plain text
    PlainText,
    /// Markdown
    Markdown,
}

/// Markup content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkupContent {
    /// Content kind
    pub kind: MarkupKind,
    /// Content value
    pub value: String,
}

impl MarkupContent {
    /// Create plain text content
    pub fn plain_text(value: String) -> Self {
        Self {
            kind: MarkupKind::PlainText,
            value,
        }
    }

    /// Create markdown content
    pub fn markdown(value: String) -> Self {
        Self {
            kind: MarkupKind::Markdown,
            value,
        }
    }
}

/// Hover information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverInfo {
    /// Hover content
    pub contents: MarkupContent,
    /// Range of the hover
    pub range: Option<Range>,
}

impl HoverInfo {
    /// Create new hover information
    pub fn new(contents: MarkupContent) -> Self {
        Self {
            contents,
            range: None,
        }
    }

    /// Set the range
    pub fn with_range(mut self, range: Range) -> Self {
        self.range = Some(range);
        self
    }
}

/// Semantic information about code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticInfo {
    /// Symbols in the code
    pub symbols: Vec<Symbol>,
    /// Imports in the code
    pub imports: Vec<String>,
    /// Definitions in the code
    pub definitions: Vec<Definition>,
    /// References in the code
    pub references: Vec<Reference>,
}

impl SemanticInfo {
    /// Create new semantic information
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            imports: Vec::new(),
            definitions: Vec::new(),
            references: Vec::new(),
        }
    }
}

impl Default for SemanticInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Language enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Rust
    Rust,
    /// TypeScript
    TypeScript,
    /// Python
    Python,
    /// Unknown language
    Unknown,
}

impl Language {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "ts" | "tsx" | "js" | "jsx" => Language::TypeScript,
            "py" => Language::Python,
            _ => Language::Unknown,
        }
    }

    /// Get file extensions for this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &["rs"],
            Language::TypeScript => &["ts", "tsx", "js", "jsx"],
            Language::Python => &["py"],
            Language::Unknown => &[],
        }
    }

    /// Convert language to string identifier
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
            Language::Python => "python",
            Language::Unknown => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(10, 5);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn test_range_creation() {
        let start = Position::new(0, 0);
        let end = Position::new(1, 0);
        let range = Range::new(start, end);
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn test_symbol_creation() {
        let range = Range::new(Position::new(0, 0), Position::new(0, 5));
        let symbol = Symbol::new("test_fn".to_string(), SymbolKind::Function, range);
        assert_eq!(symbol.name, "test_fn");
        assert_eq!(symbol.kind, SymbolKind::Function);
    }

    #[test]
    fn test_diagnostic_creation() {
        let range = Range::new(Position::new(0, 0), Position::new(0, 5));
        let diag = Diagnostic::new(range, DiagnosticSeverity::Error, "Test error".to_string());
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.message, "Test error");
    }

    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_extension("rs"), Language::Rust);
        assert_eq!(Language::from_extension("ts"), Language::TypeScript);
        assert_eq!(Language::from_extension("py"), Language::Python);
        assert_eq!(Language::from_extension("unknown"), Language::Unknown);
    }

    #[test]
    fn test_markup_content_plain_text() {
        let content = MarkupContent::plain_text("Hello".to_string());
        assert_eq!(content.kind, MarkupKind::PlainText);
        assert_eq!(content.value, "Hello");
    }

    #[test]
    fn test_markup_content_markdown() {
        let content = MarkupContent::markdown("# Hello".to_string());
        assert_eq!(content.kind, MarkupKind::Markdown);
        assert_eq!(content.value, "# Hello");
    }

    #[test]
    fn test_workspace_edit() {
        let mut edit = WorkspaceEdit::new();
        let text_edit = TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(0, 5)),
            new_text: "new".to_string(),
        };
        edit.add_edit("file://test.rs".to_string(), text_edit);
        assert_eq!(edit.changes.len(), 1);
    }

    #[test]
    fn test_semantic_info() {
        let info = SemanticInfo::new();
        assert!(info.symbols.is_empty());
        assert!(info.imports.is_empty());
    }
}
