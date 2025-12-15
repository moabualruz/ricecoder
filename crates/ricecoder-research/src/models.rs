//! Data models for the research system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Re-export DetectedPattern when patterns feature is enabled
#[cfg(feature = "patterns")]
pub use ricecoder_patterns::DetectedPattern;

// ============================================================================
// Project Analysis Models
// ============================================================================

/// Complete context about a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Type of project (library, application, service, etc.)
    pub project_type: ProjectType,
    /// Languages detected in the project
    pub languages: Vec<Language>,
    /// Frameworks detected in the project
    pub frameworks: Vec<Framework>,
    /// Project structure information
    pub structure: ProjectStructure,
    /// Detected patterns in the codebase
    pub patterns: Vec<DetectedPattern>,
    /// Project dependencies
    pub dependencies: Vec<Dependency>,
    /// Architectural intent and decisions
    pub architectural_intent: ArchitecturalIntent,
    /// Detected standards and conventions
    pub standards: StandardsProfile,
}

/// Type of project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    /// Library crate/package
    Library,
    /// Application/binary
    Application,
    /// Microservice
    Service,
    /// Monorepo with multiple packages
    Monorepo,
    /// Unknown project type
    Unknown,
}

// ============================================================================
// Pattern Detection Models (when patterns feature is disabled)
// ============================================================================

/// Category of detected pattern
#[cfg(not(feature = "patterns"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternCategory {
    /// Architectural pattern
    Architectural,
    /// Design pattern
    Design,
    /// Coding idiom
    Coding,
    /// Testing pattern
    Testing,
    /// Configuration pattern
    Configuration,
}

/// Detected pattern in the codebase (fallback when patterns feature is disabled)
#[cfg(not(feature = "patterns"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// Pattern name
    pub name: String,
    /// Pattern category
    pub category: PatternCategory,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Locations where pattern was detected
    pub locations: Vec<PathBuf>,
    /// Pattern description
    pub description: String,
}

/// Programming language
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// Rust
    Rust,
    /// TypeScript/JavaScript
    TypeScript,
    /// Python
    Python,
    /// Go
    Go,
    /// Java
    Java,
    /// Kotlin
    Kotlin,
    /// C#/.NET
    CSharp,
    /// PHP
    Php,
    /// Ruby
    Ruby,
    /// Swift
    Swift,
    /// Dart
    Dart,
    /// Other language
    Other(String),
}

/// Framework or library
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Framework {
    /// Framework name
    pub name: String,
    /// Framework version
    pub version: Option<String>,
}

/// Project structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    /// Root directory of the project
    pub root: PathBuf,
    /// Source directories
    pub source_dirs: Vec<PathBuf>,
    /// Test directories
    pub test_dirs: Vec<PathBuf>,
    /// Configuration files
    pub config_files: Vec<PathBuf>,
    /// Entry points (main.rs, index.js, etc.)
    pub entry_points: Vec<PathBuf>,
}



/// Project dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Dependency name
    pub name: String,
    /// Dependency version
    pub version: String,
    /// Version constraints (e.g., "^1.0", "~2.1")
    pub constraints: Option<String>,
    /// Whether this is a dev dependency
    pub is_dev: bool,
}

// ============================================================================
// Architectural Intent Models
// ============================================================================

/// Architectural intent and decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalIntent {
    /// Detected architectural style
    pub style: ArchitecturalStyle,
    /// Architectural principles
    pub principles: Vec<String>,
    /// Architectural constraints
    pub constraints: Vec<String>,
    /// Architectural decisions
    pub decisions: Vec<ArchitecturalDecision>,
}

/// Architectural style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchitecturalStyle {
    /// Layered architecture
    Layered,
    /// Microservices architecture
    Microservices,
    /// Event-driven architecture
    EventDriven,
    /// Monolithic architecture
    Monolithic,
    /// Serverless architecture
    Serverless,
    /// Unknown architecture
    Unknown,
}

/// Architectural decision record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalDecision {
    /// Decision ID
    pub id: String,
    /// Decision title
    pub title: String,
    /// Context for the decision
    pub context: String,
    /// The decision itself
    pub decision: String,
    /// Consequences of the decision
    pub consequences: Vec<String>,
    /// Date the decision was made
    pub date: DateTime<Utc>,
}

// ============================================================================
// Code Context Models
// ============================================================================

/// Code context for AI providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    /// Files included in the context
    pub files: Vec<FileContext>,
    /// Symbols in the context
    pub symbols: Vec<Symbol>,
    /// Symbol references
    pub references: Vec<SymbolReference>,
    /// Total tokens in the context
    pub total_tokens: usize,
}

/// File context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    /// File path
    pub path: PathBuf,
    /// Relevance score (0.0 to 1.0)
    pub relevance: f32,
    /// Summary of the file
    pub summary: Option<String>,
    /// File content (may be truncated)
    pub content: Option<String>,
}

/// Code symbol (function, class, type, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    /// Unique symbol ID
    pub id: String,
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// File containing the symbol
    pub file: PathBuf,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// References to this symbol
    pub references: Vec<SymbolReference>,
}

/// Kind of symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    /// Function or method
    Function,
    /// Class or struct
    Class,
    /// Type alias or type definition
    Type,
    /// Constant
    Constant,
    /// Variable
    Variable,
    /// Module or namespace
    Module,
    /// Trait or interface
    Trait,
    /// Enum
    Enum,
}

/// Reference to a symbol
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolReference {
    /// ID of the referenced symbol
    pub symbol_id: String,
    /// File containing the reference
    pub file: PathBuf,
    /// Line number of the reference
    pub line: usize,
    /// Kind of reference
    pub kind: ReferenceKind,
}

/// Kind of symbol reference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceKind {
    /// Symbol definition
    Definition,
    /// Symbol usage
    Usage,
    /// Import statement
    Import,
    /// Export statement
    Export,
}

/// Search result for semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Symbol found
    pub symbol: Symbol,
    /// Relevance score (0.0 to 1.0)
    pub relevance: f32,
    /// Context around the symbol
    pub context: Option<String>,
}

// ============================================================================
// Standards Detection Models
// ============================================================================

/// Detected standards and conventions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StandardsProfile {
    /// Naming conventions
    pub naming_conventions: NamingConventions,
    /// Formatting style
    pub formatting_style: FormattingStyle,
    /// Import organization
    pub import_organization: ImportOrganization,
    /// Documentation style
    pub documentation_style: DocumentationStyle,
}

impl Default for NamingConventions {
    fn default() -> Self {
        NamingConventions {
            function_case: CaseStyle::SnakeCase,
            variable_case: CaseStyle::SnakeCase,
            class_case: CaseStyle::PascalCase,
            constant_case: CaseStyle::UpperCase,
        }
    }
}

impl Default for FormattingStyle {
    fn default() -> Self {
        FormattingStyle {
            indent_size: 4,
            indent_type: IndentType::Spaces,
            line_length: 100,
        }
    }
}

impl Default for ImportOrganization {
    fn default() -> Self {
        ImportOrganization {
            order: vec![
                ImportGroup::Standard,
                ImportGroup::External,
                ImportGroup::Internal,
            ],
            sort_within_group: true,
        }
    }
}

impl Default for DocumentationStyle {
    fn default() -> Self {
        DocumentationStyle {
            format: DocFormat::RustDoc,
            required_for_public: true,
        }
    }
}

/// Naming conventions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamingConventions {
    /// Function naming case style
    pub function_case: CaseStyle,
    /// Variable naming case style
    pub variable_case: CaseStyle,
    /// Class/type naming case style
    pub class_case: CaseStyle,
    /// Constant naming case style
    pub constant_case: CaseStyle,
}

/// Case style for naming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseStyle {
    /// camelCase
    CamelCase,
    /// snake_case
    SnakeCase,
    /// PascalCase
    PascalCase,
    /// kebab-case
    KebabCase,
    /// UPPER_CASE
    UpperCase,
    /// Mixed case styles
    Mixed,
}

/// Formatting style
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormattingStyle {
    /// Indentation size in spaces
    pub indent_size: usize,
    /// Indentation type
    pub indent_type: IndentType,
    /// Preferred line length
    pub line_length: usize,
}

/// Indentation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndentType {
    /// Spaces for indentation
    Spaces,
    /// Tabs for indentation
    Tabs,
}

/// Import organization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportOrganization {
    /// Order of import groups
    pub order: Vec<ImportGroup>,
    /// Whether to sort within groups
    pub sort_within_group: bool,
}

/// Import group
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportGroup {
    /// Standard library imports
    Standard,
    /// External/third-party imports
    External,
    /// Internal project imports
    Internal,
    /// Relative imports
    Relative,
}

/// Documentation style
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentationStyle {
    /// Documentation format
    pub format: DocFormat,
    /// Whether documentation is required for public items
    pub required_for_public: bool,
}

/// Documentation format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocFormat {
    /// JavaDoc style (/** ... */)
    JavaDoc,
    /// RustDoc style (/// ...)
    RustDoc,
    /// JSDoc style (/** ... */)
    JSDoc,
    /// Python docstring style (""" ... """)
    PythonDoc,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_type_serialization() {
        let project_type = ProjectType::Library;
        let json = serde_json::to_string(&project_type).unwrap();
        let deserialized: ProjectType = serde_json::from_str(&json).unwrap();
        assert_eq!(project_type, deserialized);
    }

    #[test]
    fn test_language_serialization() {
        let language = Language::Rust;
        let json = serde_json::to_string(&language).unwrap();
        let deserialized: Language = serde_json::from_str(&json).unwrap();
        assert_eq!(language, deserialized);
    }

    #[test]
    fn test_symbol_kind_serialization() {
        let kind = SymbolKind::Function;
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: SymbolKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }

    #[test]
    fn test_case_style_serialization() {
        let style = CaseStyle::SnakeCase;
        let json = serde_json::to_string(&style).unwrap();
        let deserialized: CaseStyle = serde_json::from_str(&json).unwrap();
        assert_eq!(style, deserialized);
    }

    #[test]
    fn test_project_context_creation() {
        let context = ProjectContext {
            project_type: ProjectType::Library,
            languages: vec![Language::Rust],
            frameworks: vec![],
            structure: ProjectStructure {
                root: PathBuf::from("/project"),
                source_dirs: vec![PathBuf::from("/project/src")],
                test_dirs: vec![PathBuf::from("/project/tests")],
                config_files: vec![PathBuf::from("/project/Cargo.toml")],
                entry_points: vec![],
            },
            patterns: vec![],
            dependencies: vec![],
            architectural_intent: ArchitecturalIntent {
                style: ArchitecturalStyle::Layered,
                principles: vec![],
                constraints: vec![],
                decisions: vec![],
            },
            standards: StandardsProfile {
                naming_conventions: NamingConventions {
                    function_case: CaseStyle::SnakeCase,
                    variable_case: CaseStyle::SnakeCase,
                    class_case: CaseStyle::PascalCase,
                    constant_case: CaseStyle::UpperCase,
                },
                formatting_style: FormattingStyle {
                    indent_size: 4,
                    indent_type: IndentType::Spaces,
                    line_length: 100,
                },
                import_organization: ImportOrganization {
                    order: vec![
                        ImportGroup::Standard,
                        ImportGroup::External,
                        ImportGroup::Internal,
                    ],
                    sort_within_group: true,
                },
                documentation_style: DocumentationStyle {
                    format: DocFormat::RustDoc,
                    required_for_public: true,
                },
            },
        };

        assert_eq!(context.project_type, ProjectType::Library);
        assert_eq!(context.languages.len(), 1);
    }
}
