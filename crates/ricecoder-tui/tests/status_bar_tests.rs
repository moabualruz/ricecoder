//! Unit tests for StatusBarWidget
//!
//! These tests verify the functionality of the StatusBarWidget,
//! including status display, configuration, and rendering.

use ricecoder_tui::status_bar::{ConnectionStatus, InputMode, StatusBarWidget};

#[test]
fn test_status_bar_creation() {
    let status_bar = StatusBarWidget::new();
    assert_eq!(status_bar.provider, "None");
    assert_eq!(status_bar.model, "None");
    assert_eq!(status_bar.connection_status, ConnectionStatus::Disconnected);
    assert_eq!(status_bar.session_name, "Untitled");
    assert_eq!(status_bar.message_count, 0);
}

#[test]
fn test_status_bar_with_methods() {
    let status_bar = StatusBarWidget::new()
        .with_provider("Anthropic")
        .with_model("claude-3")
        .with_connection_status(ConnectionStatus::Connected)
        .with_session_name("My Session")
        .with_message_count(42)
        .with_project_name(Some("my-project".to_string()))
        .with_git_branch(Some("main".to_string()))
        .with_input_mode(InputMode::Normal);

    assert_eq!(status_bar.provider, "Anthropic");
    assert_eq!(status_bar.model, "claude-3");
    assert_eq!(status_bar.connection_status, ConnectionStatus::Connected);
    assert_eq!(status_bar.session_name, "My Session");
    assert_eq!(status_bar.message_count, 42);
    assert_eq!(status_bar.project_name, Some("my-project".to_string()));
    assert_eq!(status_bar.git_branch, Some("main".to_string()));
    assert_eq!(status_bar.input_mode, InputMode::Normal);
}

#[test]
fn test_connection_status_display() {
    assert_eq!(ConnectionStatus::Connected.display_text(), "✓");
    assert_eq!(ConnectionStatus::Disconnected.display_text(), "✗");
    assert_eq!(ConnectionStatus::Error.display_text(), "⚠");
    assert_eq!(ConnectionStatus::Connecting.display_text(), "⟳");
}

#[test]
fn test_input_mode_display() {
    assert_eq!(InputMode::Insert.display_text(), "INSERT");
    assert_eq!(InputMode::Normal.display_text(), "NORMAL");
    assert_eq!(InputMode::Visual.display_text(), "VISUAL");
    assert_eq!(InputMode::Command.display_text(), "COMMAND");
}
