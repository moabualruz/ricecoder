//! Prompt module for RiceCoder TUI
//!
//! A comprehensive prompt system migrated from OpenCode, providing:
//! - Rich text input with multi-line support
//! - Extmarks (inline annotations for files, agents, pasted text)
//! - Clipboard operations (text and images)
//! - External editor integration
//! - Command palette integration
//! - History and stash management
//! - Autocomplete support
//! - Provider/model management
//! - Agent cycling
//! - MCP integration
//! - Mouse selection
//! - Animated spinners
//! - Terminal detection
//!
//! # Module Structure (DDD/SOLID)
//!
//! ## Domain Layer
//! - `parts` - Domain entities (TextPart, FilePart, AgentPart, PromptPart)
//!
//! ## Application Layer
//! - `state` - Application state (PromptState, PromptMode, PromptInfo)
//! - `commands` - Command palette commands
//! - `handler` - Event handling and orchestration
//! - `provider` - Provider/model management
//! - `cycling` - Agent/model cycling logic
//! - `mcp` - MCP server integration
//!
//! ## Infrastructure Layer
//! - `input` - TextInput wrapper for tui-textarea
//! - `extmarks` - Inline annotation management
//! - `clipboard` - Paste handling for text/images
//! - `editor` - External editor integration
//! - `selection` - Mouse selection handling
//! - `terminal` - Terminal capability detection
//!
//! ## Presentation Layer
//! - `widget` - Main prompt widget
//! - `autocomplete_widget` - Autocomplete popup widget
//! - `spinner` - Animated spinner widgets

// Domain layer - core entities
pub mod parts;

// Application layer - state, commands, and orchestration
pub mod state;
pub mod commands;
pub mod handler;
pub mod provider;
pub mod cycling;
pub mod mcp;

// Infrastructure layer - I/O and external integrations
pub mod input;
pub mod extmarks;
pub mod clipboard;
pub mod editor;
pub mod selection;
pub mod terminal;

// Presentation layer - UI widgets
pub mod widget;
pub mod autocomplete_widget;
pub mod spinner;

// Re-exports for convenience
pub use parts::{AgentPart, AgentSource, FilePart, FileSource, PromptPart, TextPart, TextSource};
pub use state::{PromptInfo, PromptMode, PromptState, PLACEHOLDERS};
pub use input::{CursorPosition, InputAction, PromptInput};
pub use extmarks::{Extmark, ExtmarkManager, ExtmarkStyle};
pub use clipboard::{Clipboard, ClipboardContent, Osc52Clipboard, PasteConfig, PastedContent};
pub use editor::{EditorConfig, EditorError, EditorResult, ExternalEditor};
pub use commands::{CommandContext, CommandResult, PromptCommand, PromptCommandId};
pub use handler::{DialogRequest, KeybindConfig, PromptEvent, PromptHandler, PromptRef, ToastVariant};
pub use widget::{PromptBorderStyle, PromptWidget, PromptWidgetConfig, SessionStatus};
pub use provider::{Model, ParsedModel, Provider, ProviderManager, ProviderStatus};
pub use cycling::{Agent, AgentManager, CycleDirection, CyclingEvent, CyclingMode, CyclingState};
pub use mcp::{McpEvent, McpManager, McpServer, McpServerStatus, McpStatusSummary, McpTool};
pub use selection::{Selection, SelectionHandler, SelectionMode, SelectionPoint};
pub use spinner::{SimpleSpinner, Spinner, SpinnerConfig, SpinnerState, SpinnerStyle};
pub use autocomplete_widget::{AutocompleteState, AutocompleteWidget, AutocompleteWidgetConfig, Suggestion, SuggestionCategory};
pub use terminal::{ColorScheme, TerminalCapabilities, TerminalInfo, ThemedColors};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all exports are accessible
        let _part = PromptPart::Text(TextPart::new("test"));
        let _state = PromptState::new();
        let _mode = PromptMode::Normal;
        let _info = PromptInfo::new();
        let _style = ExtmarkStyle::File;
        let _cmd = PromptCommandId::Submit;
    }

    #[test]
    fn test_placeholder_availability() {
        assert!(!PLACEHOLDERS.is_empty());
        assert!(PLACEHOLDERS.len() >= 3);
    }

    #[test]
    fn test_input_action_count() {
        let actions = InputAction::all();
        assert!(actions.len() >= 30, "Should have all textarea actions");
    }

    #[test]
    fn test_command_ids() {
        let all = PromptCommandId::all();
        assert!(all.len() >= 10, "Should have all prompt commands");
    }
}
