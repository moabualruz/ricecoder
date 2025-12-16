//! Language support definitions and configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported programming languages
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
    // Add more languages as needed
}

impl Language {
    /// Get the file extensions for this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &["rs"],
            Language::Python => &["py", "pyw", "pyi"],
            Language::TypeScript => &["ts", "tsx"],
            Language::JavaScript => &["js", "jsx", "mjs"],
            Language::Go => &["go"],
        }
    }

    /// Get the language name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::TypeScript => "typescript",
            Language::JavaScript => "javascript",
            Language::Go => "go",
        }
    }

    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Language::Rust),
            "py" | "pyw" | "pyi" => Some(Language::Python),
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" | "mjs" => Some(Language::JavaScript),
            "go" => Some(Language::Go),
            _ => None,
        }
    }

    /// Detect language from file path
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| Self::from_extension(ext))
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" => Ok(Language::Rust),
            "python" => Ok(Language::Python),
            "typescript" => Ok(Language::TypeScript),
            "javascript" => Ok(Language::JavaScript),
            "go" => Ok(Language::Go),
            _ => Err(format!("Unsupported language: {}", s)),
        }
    }
}

/// Language-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Language identifier
    pub language: Language,
    /// Tree-sitter grammar name
    pub grammar_name: String,
    /// Parser-specific options
    pub options: HashMap<String, serde_json::Value>,
    /// Node type mappings (tree-sitter -> our NodeType)
    pub node_mappings: HashMap<String, crate::types::NodeType>,
}

impl LanguageConfig {
    /// Create default config for a language
    pub fn default_for_language(language: Language) -> Self {
        let (grammar_name, node_mappings) = match language {
            Language::Rust => ("rust", Self::rust_node_mappings()),
            Language::Python => ("python", Self::python_node_mappings()),
            Language::TypeScript | Language::JavaScript => ("typescript", Self::typescript_node_mappings()),
            Language::Go => ("go", Self::go_node_mappings()),
        };

        Self {
            language,
            grammar_name: grammar_name.to_string(),
            options: HashMap::new(),
            node_mappings,
        }
    }

    fn rust_node_mappings() -> HashMap<String, crate::types::NodeType> {
        let mut mappings = HashMap::new();
        mappings.insert("source_file".to_string(), crate::types::NodeType::Program);
        mappings.insert("function_item".to_string(), crate::types::NodeType::Function);
        mappings.insert("struct_item".to_string(), crate::types::NodeType::Class);
        mappings.insert("impl_item".to_string(), crate::types::NodeType::Class);
        mappings.insert("let_declaration".to_string(), crate::types::NodeType::Variable);
        mappings.insert("const_item".to_string(), crate::types::NodeType::Constant);
        mappings.insert("use_declaration".to_string(), crate::types::NodeType::Import);
        mappings.insert("if_expression".to_string(), crate::types::NodeType::IfStatement);
        mappings.insert("for_expression".to_string(), crate::types::NodeType::ForLoop);
        mappings.insert("while_expression".to_string(), crate::types::NodeType::WhileLoop);
        mappings.insert("call_expression".to_string(), crate::types::NodeType::CallExpression);
        mappings.insert("assignment_expression".to_string(), crate::types::NodeType::Assignment);
        mappings.insert("return_expression".to_string(), crate::types::NodeType::Return);
        mappings.insert("string_literal".to_string(), crate::types::NodeType::StringLiteral);
        mappings.insert("integer_literal".to_string(), crate::types::NodeType::NumberLiteral);
        mappings.insert("boolean_literal".to_string(), crate::types::NodeType::BooleanLiteral);
        mappings.insert("array_expression".to_string(), crate::types::NodeType::ArrayLiteral);
        mappings.insert("line_comment".to_string(), crate::types::NodeType::Comment);
        mappings.insert("block_comment".to_string(), crate::types::NodeType::Comment);
        mappings
    }

    fn python_node_mappings() -> HashMap<String, crate::types::NodeType> {
        let mut mappings = HashMap::new();
        mappings.insert("module".to_string(), crate::types::NodeType::Program);
        mappings.insert("function_definition".to_string(), crate::types::NodeType::Function);
        mappings.insert("class_definition".to_string(), crate::types::NodeType::Class);
        mappings.insert("assignment".to_string(), crate::types::NodeType::Variable);
        mappings.insert("import_statement".to_string(), crate::types::NodeType::Import);
        mappings.insert("import_from_statement".to_string(), crate::types::NodeType::Import);
        mappings.insert("if_statement".to_string(), crate::types::NodeType::IfStatement);
        mappings.insert("for_statement".to_string(), crate::types::NodeType::ForLoop);
        mappings.insert("while_statement".to_string(), crate::types::NodeType::WhileLoop);
        mappings.insert("try_statement".to_string(), crate::types::NodeType::TryCatch);
        mappings.insert("call".to_string(), crate::types::NodeType::CallExpression);
        mappings.insert("return_statement".to_string(), crate::types::NodeType::Return);
        mappings.insert("string".to_string(), crate::types::NodeType::StringLiteral);
        mappings.insert("integer".to_string(), crate::types::NodeType::NumberLiteral);
        mappings.insert("float".to_string(), crate::types::NodeType::NumberLiteral);
        mappings.insert("true".to_string(), crate::types::NodeType::BooleanLiteral);
        mappings.insert("false".to_string(), crate::types::NodeType::BooleanLiteral);
        mappings.insert("list".to_string(), crate::types::NodeType::ArrayLiteral);
        mappings.insert("dictionary".to_string(), crate::types::NodeType::ObjectLiteral);
        mappings.insert("comment".to_string(), crate::types::NodeType::Comment);
        mappings
    }

    fn typescript_node_mappings() -> HashMap<String, crate::types::NodeType> {
        let mut mappings = HashMap::new();
        mappings.insert("program".to_string(), crate::types::NodeType::Program);
        mappings.insert("function_declaration".to_string(), crate::types::NodeType::Function);
        mappings.insert("method_definition".to_string(), crate::types::NodeType::Method);
        mappings.insert("class_declaration".to_string(), crate::types::NodeType::Class);
        mappings.insert("interface_declaration".to_string(), crate::types::NodeType::Type);
        mappings.insert("variable_declaration".to_string(), crate::types::NodeType::Variable);
        mappings.insert("const_declaration".to_string(), crate::types::NodeType::Constant);
        mappings.insert("import_statement".to_string(), crate::types::NodeType::Import);
        mappings.insert("export_statement".to_string(), crate::types::NodeType::Export);
        mappings.insert("if_statement".to_string(), crate::types::NodeType::IfStatement);
        mappings.insert("for_statement".to_string(), crate::types::NodeType::ForLoop);
        mappings.insert("while_statement".to_string(), crate::types::NodeType::WhileLoop);
        mappings.insert("try_statement".to_string(), crate::types::NodeType::TryCatch);
        mappings.insert("call_expression".to_string(), crate::types::NodeType::CallExpression);
        mappings.insert("assignment_expression".to_string(), crate::types::NodeType::Assignment);
        mappings.insert("return_statement".to_string(), crate::types::NodeType::Return);
        mappings.insert("string".to_string(), crate::types::NodeType::StringLiteral);
        mappings.insert("number".to_string(), crate::types::NodeType::NumberLiteral);
        mappings.insert("true".to_string(), crate::types::NodeType::BooleanLiteral);
        mappings.insert("false".to_string(), crate::types::NodeType::BooleanLiteral);
        mappings.insert("array".to_string(), crate::types::NodeType::ArrayLiteral);
        mappings.insert("object".to_string(), crate::types::NodeType::ObjectLiteral);
        mappings.insert("comment".to_string(), crate::types::NodeType::Comment);
        mappings
    }

    fn go_node_mappings() -> HashMap<String, crate::types::NodeType> {
        let mut mappings = HashMap::new();
        mappings.insert("source_file".to_string(), crate::types::NodeType::Program);
        mappings.insert("function_declaration".to_string(), crate::types::NodeType::Function);
        mappings.insert("method_declaration".to_string(), crate::types::NodeType::Method);
        mappings.insert("type_declaration".to_string(), crate::types::NodeType::Type);
        mappings.insert("struct_type".to_string(), crate::types::NodeType::Class);
        mappings.insert("var_declaration".to_string(), crate::types::NodeType::Variable);
        mappings.insert("const_declaration".to_string(), crate::types::NodeType::Constant);
        mappings.insert("import_declaration".to_string(), crate::types::NodeType::Import);
        mappings.insert("if_statement".to_string(), crate::types::NodeType::IfStatement);
        mappings.insert("for_statement".to_string(), crate::types::NodeType::ForLoop);
        mappings.insert("call_expression".to_string(), crate::types::NodeType::CallExpression);
        mappings.insert("assignment_statement".to_string(), crate::types::NodeType::Assignment);
        mappings.insert("return_statement".to_string(), crate::types::NodeType::Return);
        mappings.insert("interpreted_string_literal".to_string(), crate::types::NodeType::StringLiteral);
        mappings.insert("int_literal".to_string(), crate::types::NodeType::NumberLiteral);
        mappings.insert("float_literal".to_string(), crate::types::NodeType::NumberLiteral);
        mappings.insert("true".to_string(), crate::types::NodeType::BooleanLiteral);
        mappings.insert("false".to_string(), crate::types::NodeType::BooleanLiteral);
        mappings.insert("slice_type".to_string(), crate::types::NodeType::ArrayLiteral);
        mappings.insert("map_type".to_string(), crate::types::NodeType::ObjectLiteral);
        mappings.insert("comment".to_string(), crate::types::NodeType::Comment);
        mappings
    }
}

/// Language support interface
pub trait LanguageSupport: Send + Sync {
    /// Get the language this support is for
    fn language(&self) -> Language;

    /// Check if this support can handle a file
    fn can_handle(&self, file_path: &std::path::Path) -> bool {
        file_path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.language().extensions().contains(&ext))
            .unwrap_or(false)
    }

    /// Get language-specific configuration
    fn config(&self) -> &LanguageConfig;

    /// Parse source code into AST
    fn parse(&self, source: &str, config: &crate::parser::ParserConfig) -> crate::Result<crate::types::SyntaxTree>;
}

/// Language registry for managing language supports
pub struct LanguageRegistry {
    supports: HashMap<Language, Box<dyn LanguageSupport + 'static>>,
}

impl LanguageRegistry {
    /// Create a new language registry
    pub fn new() -> Self {
        Self {
            supports: HashMap::new(),
        }
    }

    /// Register a language support
    pub fn register(&mut self, support: Box<dyn LanguageSupport + 'static>) {
        self.supports.insert(support.language(), support);
    }

    /// Get language support for a language
    pub fn get(&self, language: &Language) -> Option<&(dyn LanguageSupport + 'static)> {
        self.supports.get(language).map(|s| s.as_ref())
    }

    /// Detect language from file path and get support
    pub fn detect_and_get(&self, file_path: &std::path::Path) -> Option<&(dyn LanguageSupport + 'static)> {
        Language::from_path(file_path)
            .and_then(|lang| self.get(&lang))
    }

    /// Get all supported languages
    pub fn supported_languages(&self) -> Vec<Language> {
        self.supports.keys().cloned().collect()
    }

    /// Check if a language is supported
    pub fn is_supported(&self, language: &Language) -> bool {
        self.supports.contains_key(language)
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}