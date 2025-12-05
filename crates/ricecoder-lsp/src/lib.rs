//! Language Server Protocol (LSP) integration for RiceCoder
//!
//! This crate provides LSP server capabilities for semantic code analysis,
//! diagnostics, code actions, and hover information across multiple programming languages.
//!
//! # Architecture
//!
//! The LSP integration follows a layered architecture with external LSP proxy support:
//!
//! 1. **External LSP Proxy Layer**: Routes requests to external LSP servers (rust-analyzer, tsserver, pylsp, etc.)
//! 2. **Internal Semantic Analysis Layer**: Provides fallback semantic analysis when external LSP is unavailable
//! 3. **Diagnostics Layer**: Collects and merges diagnostics from external and internal sources
//! 4. **Hover Layer**: Provides hover information from external and internal sources
//! 5. **Code Actions Layer**: Provides code actions from external and internal sources
//!
//! # External LSP Integration
//!
//! The LSP module integrates with external LSP servers through the `ExternalLspClient` and `LspProxy`.
//! When a request is made:
//!
//! 1. If an external LSP server is configured for the language, the request is forwarded to it
//! 2. The external LSP response is transformed to ricecoder's internal model
//! 3. External results are merged with internal results (external takes priority)
//! 4. If the external LSP is unavailable, the system falls back to internal providers
//!
//! # Fallback Behavior
//!
//! When external LSP servers are unavailable:
//!
//! - **Completions**: Fall back to internal completion providers (keyword and pattern-based)
//! - **Diagnostics**: Fall back to internal diagnostics engine
//! - **Hover**: Fall back to internal hover provider
//! - **Navigation**: Fall back to internal definition/reference providers
//!
//! This ensures users always get some results, even if not semantic.

pub mod cache;
pub mod code_actions;
pub mod completion;
pub mod config;
pub mod diagnostics;
pub mod hover;
pub mod performance;
pub mod providers;
pub mod proxy;
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
pub use proxy::{ExternalLspClient, LspProxy};
pub use semantic::SemanticAnalyzer;
pub use server::LspServer;
pub use types::{CodeAction, Diagnostic, HoverInfo, Position, Range};
