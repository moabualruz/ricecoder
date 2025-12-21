//! Main parser implementation with caching and optimization

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{ParserError, ParserResult, ParserWarning};
use crate::languages::{Language, LanguageRegistry, LanguageSupport};
use crate::types::{ASTNode, NodeType, Position, Range, SyntaxTree};
use ricecoder_cache::{Cache, CacheConfig};

/// Parser trait for parsing source code into syntax trees
pub trait CodeParser {
    /// Parse source code into a syntax tree
    fn parse<'a>(
        &'a self,
        source: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<SyntaxTree, ParserError>> + Send + 'a>>;
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Enable caching of parse results
    pub enable_caching: bool,
    /// Include comments in AST
    pub include_comments: bool,
    /// Include whitespace nodes
    pub include_whitespace: bool,
    /// Maximum parse time in seconds
    pub max_parse_time_seconds: u64,
    /// Language-specific options
    pub language_options: HashMap<String, serde_json::Value>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            include_comments: true,
            include_whitespace: false,
            max_parse_time_seconds: 30,
            language_options: HashMap::new(),
        }
    }
}

/// Parse result containing the syntax tree and metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParseResult {
    pub tree: SyntaxTree,
    pub parse_time_ms: u64,
    pub cache_hit: bool,
    pub warnings: Vec<ParserWarning>,
}

/// Parser statistics
#[derive(Debug, Clone)]
pub struct ParserStats {
    pub supported_languages: Vec<Language>,
    pub cache_enabled: bool,
    pub cache_stats: Option<ricecoder_cache::CacheStats>,
}

/// Main parser with caching and multi-language support
pub struct Parser {
    language_registry: Arc<RwLock<LanguageRegistry>>,
    cache: Option<Arc<Cache>>,
    config: ParserConfig,
}

impl Parser {
    /// Create a new parser with default configuration
    pub fn new() -> Self {
        Self::with_config(ParserConfig::default())
    }

    /// Create a new parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        let cache = if config.enable_caching {
            let cache_config = CacheConfig {
                default_ttl: Some(std::time::Duration::from_secs(3600)), // 1 hour
                max_entries: Some(1000),
                enable_metrics: true,
                ..Default::default()
            };
            let cache_storage = Arc::new(ricecoder_cache::storage::MemoryStorage::new());
            Some(Arc::new(Cache::with_config(cache_storage, cache_config)))
        } else {
            None
        };

        Self {
            language_registry: Arc::new(RwLock::new(LanguageRegistry::new())),
            cache,
            config,
        }
    }

    /// Register a language support
    pub async fn register_language(
        &self,
        support: Box<dyn LanguageSupport + 'static>,
    ) -> ParserResult<()> {
        let mut registry = self.language_registry.write().await;
        registry.register(support);
        Ok(())
    }

    /// Parse source code
    pub async fn parse(
        &self,
        source: &str,
        language: &Language,
        file_path: Option<&str>,
    ) -> ParserResult<ParseResult> {
        let start_time = std::time::Instant::now();

        // Check cache first
        let cache_key = self.create_cache_key(source, language, file_path);
        if let Some(cache) = &self.cache {
            if let Ok(Some(cached_result)) = cache.get::<ParseResult>(&cache_key).await {
                return Ok(ParseResult {
                    tree: cached_result.tree,
                    parse_time_ms: cached_result.parse_time_ms,
                    cache_hit: true,
                    warnings: cached_result.warnings,
                });
            }
        }

        // Get language support
        let registry = self.language_registry.read().await;
        let support = registry
            .get(language)
            .ok_or_else(|| ParserError::UnsupportedLanguage {
                language: language.to_string(),
            })?;

        // Parse with timeout (simplified - actual timeout would require async parsing)
        let tree = support.parse(source, &self.config)?;

        let mut tree = if let Some(path) = file_path {
            tree.with_file_path(path.to_string())
        } else {
            tree
        };

        // Add parse warnings if any
        let warnings = tree.warnings.clone();

        let parse_time_ms = start_time.elapsed().as_millis() as u64;

        let result = ParseResult {
            tree,
            parse_time_ms,
            cache_hit: false,
            warnings,
        };

        // Cache the result
        if let Some(cache) = &self.cache {
            let _ = cache.set(&cache_key, result.clone(), None).await;
        }

        Ok(result)
    }

    /// Parse source code from file
    pub async fn parse_file<P: AsRef<std::path::Path>>(
        &self,
        file_path: P,
    ) -> ParserResult<ParseResult> {
        let file_path = file_path.as_ref();
        let source = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| ParserError::IoError(e))?;

        let language =
            Language::from_path(file_path).ok_or_else(|| ParserError::UnsupportedLanguage {
                language: "unknown".to_string(),
            })?;

        let file_path_str = file_path.to_string_lossy().to_string();
        self.parse(&source, &language, Some(&file_path_str)).await
    }

    /// Get supported languages
    pub async fn supported_languages(&self) -> Vec<Language> {
        let registry = self.language_registry.read().await;
        registry.supported_languages()
    }

    /// Check if a language is supported
    pub async fn is_language_supported(&self, language: &Language) -> bool {
        let registry = self.language_registry.read().await;
        registry.is_supported(language)
    }

    /// Get language support for a file path
    pub async fn detect_language<P: AsRef<std::path::Path>>(
        &self,
        file_path: P,
    ) -> Option<Language> {
        let registry = self.language_registry.read().await;
        Language::from_path(file_path.as_ref()).filter(|lang| registry.is_supported(lang))
    }

    /// Get parser statistics
    pub async fn stats(&self) -> ParserStats {
        let supported_languages = self.supported_languages().await;

        let cache_stats = if let Some(cache) = &self.cache {
            Some(cache.stats())
        } else {
            None
        };

        ParserStats {
            supported_languages,
            cache_enabled: self.cache.is_some(),
            cache_stats,
        }
    }

    /// Clear parse cache
    pub async fn clear_cache(&self) -> ParserResult<()> {
        if let Some(cache) = &self.cache {
            cache
                .clear()
                .await
                .map_err(|e| ParserError::CacheError(e))?;
        }
        Ok(())
    }

    fn create_cache_key(
        &self,
        source: &str,
        language: &Language,
        file_path: Option<&str>,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        language.hash(&mut hasher);
        if let Some(path) = file_path {
            path.hash(&mut hasher);
        }
        format!("parse_{:x}", hasher.finish())
    }
}

impl CodeParser for Parser {
    fn parse<'a>(
        &'a self,
        source: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<SyntaxTree, ParserError>> + Send + 'a>> {
        Box::pin(async move {
            // Try to detect language from content (simple heuristic)
            let language = if source.contains("fn ") && source.contains("{") {
                Language::Rust
            } else if source.contains("def ") && source.contains(":") {
                Language::Python
            } else if source.contains("function")
                || source.contains("const")
                || source.contains("let")
            {
                Language::JavaScript
            } else {
                Language::Rust // default
            };

            let result = self.parse(source, &language, None).await?;
            Ok(result.tree)
        })
    }
}

impl std::fmt::Display for ParserStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Parser Statistics:")?;
        writeln!(
            f,
            "  Supported languages: {}",
            self.supported_languages.len()
        )?;
        for lang in &self.supported_languages {
            writeln!(f, "    - {}", lang)?;
        }
        writeln!(f, "  Cache enabled: {}", self.cache_enabled)?;
        if let Some(stats) = &self.cache_stats {
            writeln!(f, "  Cache hits: {}", stats.hits)?;
            writeln!(f, "  Cache misses: {}", stats.misses)?;
            writeln!(f, "  Hit rate: {:.2}%", stats.hit_rate())?;
        }
        Ok(())
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

/// Tree-sitter based language support implementation
pub mod tree_sitter_support {
    use super::*;
    use tree_sitter::{Language as TSLanguage, Parser as TSParser, Tree};

    /// Tree-sitter based language support
    pub struct TreeSitterSupport {
        language: Language,
        ts_language: TSLanguage,
        config: crate::languages::LanguageConfig,
    }

    impl TreeSitterSupport {
        /// Create a new tree-sitter support for a language
        pub fn new(language: Language, ts_language: TSLanguage) -> Self {
            let config = crate::languages::LanguageConfig::default_for_language(language.clone());
            Self {
                language,
                ts_language,
                config,
            }
        }

        fn convert_tree_sitter_node(
            &self,
            ts_node: tree_sitter::Node,
            source: &str,
        ) -> ParserResult<ASTNode> {
            let start_pos = ts_node.start_position();
            let end_pos = ts_node.end_position();

            let range = Range::new(
                Position::new(start_pos.row, start_pos.column),
                Position::new(end_pos.row, end_pos.column),
            );

            let text = ts_node
                .utf8_text(source.as_bytes())
                .map_err(|e| ParserError::ParseError {
                    message: format!("UTF-8 conversion error: {}", e),
                })?
                .to_string();

            let node_type = ts_node.kind();
            let our_node_type = self
                .config
                .node_mappings
                .get(node_type)
                .cloned()
                .unwrap_or_else(|| NodeType::Custom(node_type.to_string()));

            let mut ast_node = ASTNode::new(our_node_type, range, text);

            // Add children
            let mut cursor = ts_node.walk();
            for child in ts_node.children(&mut cursor) {
                if let Ok(child_ast) = self.convert_tree_sitter_node(child, source) {
                    ast_node.add_child(child_ast);
                }
            }

            Ok(ast_node)
        }
    }

    impl LanguageSupport for TreeSitterSupport {
        fn language(&self) -> Language {
            self.language.clone()
        }

        fn config(&self) -> &crate::languages::LanguageConfig {
            &self.config
        }

        fn parse(&self, source: &str, config: &ParserConfig) -> ParserResult<SyntaxTree> {
            let mut parser = TSParser::new();
            parser
                .set_language(self.ts_language)
                .map_err(|e| ParserError::ParseError {
                    message: format!("Failed to set language: {}", e),
                })?;

            let tree = parser
                .parse(source, None)
                .ok_or_else(|| ParserError::ParseError {
                    message: "Failed to parse source".to_string(),
                })?;

            let root_node = tree.root_node();
            let ast_root = self.convert_tree_sitter_node(root_node, source)?;

            let mut syntax_tree = SyntaxTree::new(ast_root, self.language.to_string());

            // Check for parse errors
            if root_node.has_error() {
                syntax_tree.add_warning(ParserWarning::new(
                    "Parse tree contains errors".to_string(),
                    crate::error::WarningSeverity::Error,
                ));
            }

            Ok(syntax_tree)
        }
    }
}

/// Create tree-sitter supports for supported languages
pub fn create_supports() -> Vec<Box<dyn LanguageSupport + 'static>> {
    let mut supports = Vec::new();

    // Rust support
    supports.push(Box::new(tree_sitter_support::TreeSitterSupport::new(
        Language::Rust,
        tree_sitter_rust::language(),
    )) as Box<dyn LanguageSupport + 'static>);

    // Python support
    supports.push(Box::new(tree_sitter_support::TreeSitterSupport::new(
        Language::Python,
        tree_sitter_python::language(),
    )) as Box<dyn LanguageSupport + 'static>);

    // TypeScript/JavaScript support
    supports.push(Box::new(tree_sitter_support::TreeSitterSupport::new(
        Language::TypeScript,
        tree_sitter_typescript::language_tsx(),
    )) as Box<dyn LanguageSupport + 'static>);
    supports.push(Box::new(tree_sitter_support::TreeSitterSupport::new(
        Language::JavaScript,
        tree_sitter_typescript::language_tsx(),
    )) as Box<dyn LanguageSupport + 'static>);

    // Go support
    supports.push(Box::new(tree_sitter_support::TreeSitterSupport::new(
        Language::Go,
        tree_sitter_go::language(),
    )) as Box<dyn LanguageSupport + 'static>);

    supports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parser_creation() {
        let parser = Parser::new();
        assert!(parser.supported_languages().await.is_empty());
    }

    #[tokio::test]
    async fn test_parser_with_config() {
        let config = ParserConfig {
            enable_caching: false,
            include_comments: false,
            ..Default::default()
        };
        let parser = Parser::with_config(config);
        let stats = parser.stats().await;
        assert!(!stats.cache_enabled);
    }

    #[tokio::test]
    async fn test_language_detection() {
        let parser = Parser::new();

        // Test extension detection
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("go"), Some(Language::Go));
        assert_eq!(Language::from_extension("unknown"), None);

        // Test path detection
        let rust_path = std::path::Path::new("src/main.rs");
        assert_eq!(Language::from_path(rust_path), Some(Language::Rust));
    }

    #[test]
    fn test_cache_key_generation() {
        let parser = Parser::new();
        let key1 = parser.create_cache_key("fn main() {}", &Language::Rust, Some("main.rs"));
        let key2 = parser.create_cache_key("fn main() {}", &Language::Rust, Some("main.rs"));
        let key3 = parser.create_cache_key("fn other() {}", &Language::Rust, Some("main.rs"));

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
