/// Core data types for the completion engine
use serde::{Deserialize, Serialize};

/// Represents a position in a document (line and character)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// Zero-based line number
    pub line: u32,
    /// Zero-based character offset within the line
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// Represents a range in a document (start and end positions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

/// Represents a symbol (variable, function, type, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub scope: Scope,
    pub type_info: Option<String>,
    pub documentation: Option<String>,
}

/// Kind of symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Variable,
    Function,
    Type,
    Constant,
    Module,
    Class,
    Struct,
    Enum,
    Interface,
    Trait,
    Method,
    Property,
    Field,
    Parameter,
    Keyword,
}

/// Represents a scope in code (function, class, module, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scope {
    pub kind: ScopeKind,
    pub name: Option<String>,
    pub range: Range,
}

/// Kind of scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScopeKind {
    Global,
    Module,
    Function,
    Class,
    Struct,
    Impl,
    Block,
}

/// Type information for expected types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Type {
    pub name: String,
    pub is_optional: bool,
    pub is_array: bool,
    pub generic_params: Vec<Type>,
}

impl Type {
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_optional: false,
            is_array: false,
            generic_params: Vec::new(),
        }
    }

    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    pub fn array(mut self) -> Self {
        self.is_array = true;
        self
    }
}

/// Context information for completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionContext {
    pub scope: Scope,
    pub available_symbols: Vec<Symbol>,
    pub expected_type: Option<Type>,
    pub prefix: String,
    pub language: String,
    pub position: Position,
}

impl CompletionContext {
    pub fn new(language: String, position: Position, prefix: String) -> Self {
        Self {
            scope: Scope {
                kind: ScopeKind::Global,
                name: None,
                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            },
            available_symbols: Vec::new(),
            expected_type: None,
            prefix,
            language,
            position,
        }
    }
}

/// Kind of completion item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompletionItemKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    EventListener,
    Operator,
    TypeParameter,
    Trait,
}

/// Represents a single completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub sort_text: Option<String>,
    pub filter_text: Option<String>,
    pub insert_text: String,
    pub score: f32,
    pub additional_edits: Vec<TextEdit>,
}

impl CompletionItem {
    pub fn new(label: String, kind: CompletionItemKind, insert_text: String) -> Self {
        Self {
            label,
            kind,
            detail: None,
            documentation: None,
            sort_text: None,
            filter_text: None,
            insert_text,
            score: 0.0,
            additional_edits: Vec::new(),
        }
    }

    pub fn with_detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }

    pub fn with_documentation(mut self, documentation: String) -> Self {
        self.documentation = Some(documentation);
        self
    }

    pub fn with_score(mut self, score: f32) -> Self {
        self.score = score;
        self
    }
}

/// Represents a text edit (insertion, deletion, or replacement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

impl TextEdit {
    pub fn new(range: Range, new_text: String) -> Self {
        Self { range, new_text }
    }
}

/// Ghost text suggestion (inline suggestion shown in lighter color)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GhostText {
    pub text: String,
    pub range: Range,
}

impl GhostText {
    pub fn new(text: String, range: Range) -> Self {
        Self { text, range }
    }
}

/// Ranking weights for completion scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingWeights {
    pub relevance: f32,
    pub frequency: f32,
    pub recency: f32,
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            relevance: 0.5,
            frequency: 0.3,
            recency: 0.2,
        }
    }
}

/// Completion configuration for a language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionConfig {
    pub language: String,
    pub keywords: Vec<String>,
    pub snippets: Vec<CompletionSnippet>,
    pub ranking_weights: RankingWeights,
    pub provider: Option<String>,
}

impl CompletionConfig {
    pub fn new(language: String) -> Self {
        Self {
            language,
            keywords: Vec::new(),
            snippets: Vec::new(),
            ranking_weights: RankingWeights::default(),
            provider: None,
        }
    }
}

/// Snippet template for code completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionSnippet {
    pub label: String,
    pub template: String,
    pub description: Option<String>,
}

impl CompletionSnippet {
    pub fn new(label: String, template: String) -> Self {
        Self {
            label,
            template,
            description: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// Error type for completion operations
#[derive(Debug, thiserror::Error)]
pub enum CompletionError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Context analysis failed: {0}")]
    ContextAnalysisError(String),

    #[error("Completion generation failed: {0}")]
    GenerationError(String),

    #[error("Ranking failed: {0}")]
    RankingError(String),

    #[error("Invalid position: {0}")]
    InvalidPosition(String),

    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Storage error: {0}")]
    StorageError(#[from] ricecoder_storage::StorageError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type CompletionResult<T> = Result<T, CompletionError>;
