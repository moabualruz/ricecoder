//! TUI integration for LSP functionality
//!
//! This module provides TUI-specific widgets and integration code for LSP features,
//! including diagnostics display and LSP response conversion.

pub mod diagnostics_widget;
pub mod lsp_integration;

// Re-export public API
pub use diagnostics_widget::{
    DiagnosticDetailWidget, DiagnosticItem, DiagnosticLocation, DiagnosticRelatedInformation,
    DiagnosticSeverity, DiagnosticsWidget, HoverWidget,
};
pub use lsp_integration::{language_from_file_path, lsp_diagnostics_to_tui, lsp_hover_to_text};
