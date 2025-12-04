//! Language Server Protocol (LSP) integration for RiceCoder
//!
//! This crate provides LSP server capabilities for semantic code analysis,
//! diagnostics, code actions, and hover information across multiple programming languages.

pub mod types;
pub mod transport;
pub mod server;
pub mod semantic;
pub mod diagnostics;
pub mod code_actions;
pub mod hover;
pub mod cache;
pub mod performance;
pub mod config;
pub mod providers;
pub mod completion;

// Re-export public API
pub use server::LspServer;
pub use types::{Position, Range, Diagnostic, CodeAction, HoverInfo};
pub use semantic::SemanticAnalyzer;
pub use diagnostics::DiagnosticsEngine;
pub use code_actions::CodeActionsEngine;
pub use hover::HoverProvider;
pub use cache::{SemanticCache, AstCache, SymbolIndexCache, hash_input};
pub use performance::{PerformanceTracker, Timer, PerformanceAnalyzer};
pub use config::{ConfigRegistry, ConfigLoader, LanguageConfig, DiagnosticRule, CodeActionTemplate, ConfigurationManager};
pub use providers::{
    SemanticAnalyzerProvider, DiagnosticsProvider, CodeActionProvider,
    SemanticAnalyzerRegistry, DiagnosticsRegistry, CodeActionRegistry,
};
pub use completion::CompletionHandler;
