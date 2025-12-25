//! Context analysis modules
//!
//! Provides context analysis for code completion, including:
//! - Generic text-based analysis (fallback)
//! - Tree-sitter based analysis (accurate parsing)

mod analyzer;
mod generic;
mod tree_sitter;
mod utils;

// Re-export the trait and implementations
pub use analyzer::ContextAnalyzer;
pub use generic::GenericContextAnalyzer;
pub use tree_sitter::TreeSitterContextAnalyzer;
