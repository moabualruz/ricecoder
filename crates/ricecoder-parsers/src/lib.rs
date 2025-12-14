//! # RiceCoder Parsers
//!
//! AST parsing and syntax tree analysis for multiple programming languages.
//! Provides unified interfaces for parsing, traversing, and analyzing code
//! across Rust, Python, TypeScript, and Go.
//!
//! ## Features
//!
//! - **Multi-language Support**: AST parsing for Rust, Python, TypeScript, and Go
//! - **Unified API**: Consistent interfaces across all supported languages
//! - **Syntax Tree Traversal**: Powerful utilities for code analysis
//! - **Performance Optimized**: Caching and incremental parsing
//! - **Extensible Architecture**: Easy to add support for new languages
//! - **Comprehensive Testing**: Property-based tests and benchmarks

pub mod error;
pub mod languages;
pub mod parser;
pub mod traversal;
pub mod types;

pub use error::{ParserError, ParserResult};
pub use languages::{Language, LanguageSupport};
pub use parser::{Parser, ParserConfig, ParseResult};
pub use traversal::{NodeVisitor, TreeWalker, VisitorResult};
pub use types::{ASTNode, SyntaxTree, Position, Range, NodeType};

/// Re-export commonly used types
pub type Result<T> = std::result::Result<T, ParserError>;