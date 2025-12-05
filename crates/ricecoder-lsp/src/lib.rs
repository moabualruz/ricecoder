//! Language Server Protocol (LSP) integration for RiceCoder
//!
//! This crate provides LSP server capabilities for semantic code analysis,
//! diagnostics, code actions, and hover information across multiple programming languages.

pub mod cache;
pub mod code_actions;
pub mod completion;
pub mod config;
pub mod diagnostics;
pub mod hover;
pub mod performance;
pub mod providers;
pub mod semantic;
pub mod server;
pub mod transport;
pub mod types;

// Re-export public API
pub use cache::{hash_input, AstCache, SemanticCache, SymbolIndexCache};
pub use code_actions::CodeActionsEngine;
pub use completion::CompletionHandler;
pub use config::{
    CodeActionTemplate, ConfigLoader, ConfigRegistry, ConfigurationManager, DiagnosticRule,
    LanguageConfig,
};
pub use diagnostics::DiagnosticsEngine;
pub use hover::HoverProvider;
pub use performance::{PerformanceAnalyzer, PerformanceTracker, Timer};
pub use providers::{
    CodeActionProvider, CodeActionRegistry, DiagnosticsProvider, DiagnosticsRegistry,
    SemanticAnalyzerProvider, SemanticAnalyzerRegistry,
};
pub use semantic::SemanticAnalyzer;
pub use server::LspServer;
pub use types::{CodeAction, Diagnostic, HoverInfo, Position, Range};
