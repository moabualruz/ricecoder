//! View function for Elm Architecture (TEA) implementation
//!
//! This module contains the pure view function that renders the UI based on
//! the current application state. The view function is pure and only depends
//! on the model state.

use crate::model::*;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Main view function - pure function that renders UI based on model state
pub fn view(frame: &mut Frame, model: &AppModel) {
    let size = frame.size();

    // Create main layout with header, main content, and status
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header/Banner area
            Constraint::Min(10),   // Main content area (with sidebar)
            Constraint::Length(3), // Status bar
        ])
        .split(size);

    // Render header
    render_header(frame, main_chunks[0], model);

    // Render main content with sidebar
    render_main_area(frame, main_chunks[1], model);

    // Render status bar
    render_status_bar(frame, main_chunks[2], model);
}

/// Render the header area
fn render_header(frame: &mut Frame, area: Rect, model: &AppModel) {
    let banner_text = vec![
        Line::from(vec![
            Span::styled("🍚 RiceCoder", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - AI-Powered Terminal Interface"),
        ]),
        Line::from(vec![
            Span::styled("Mode: ", Style::default().fg(Color::Cyan)),
            Span::styled(model.mode.display_name(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled("Theme: ", Style::default().fg(Color::Cyan)),
            Span::styled(model.theme.name.clone(), Style::default().fg(Color::Magenta)),
        ]),
    ];

    let banner = Paragraph::new(banner_text)
        .block(Block::default().borders(Borders::ALL).title("RiceCoder"))
        .wrap(Wrap { trim: true });

    frame.render_widget(banner, area);
}

/// Render the main content area with sidebar
fn render_main_area(frame: &mut Frame, area: Rect, model: &AppModel) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30), // Sidebar
            Constraint::Min(10),    // Main content
        ])
        .split(area);

    // Render sidebar
    render_sidebar(frame, chunks[0], model);

    // Render main content based on current mode
    render_main_content(frame, chunks[1], model);
}

/// Render the sidebar
fn render_sidebar(frame: &mut Frame, area: Rect, model: &AppModel) {
    let sidebar_content = vec![
        Line::from(vec![
            Span::styled("📁 Sessions", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(format!("Active: {}", model.sessions.active_session_id.as_deref().unwrap_or("None"))),
        Line::from(format!("Total: {}", model.sessions.session_count)),
        Line::from(""),
        Line::from(vec![
            Span::styled("📊 Stats", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(format!("Tokens: {}", model.sessions.total_tokens.input_tokens + model.sessions.total_tokens.output_tokens)),
        Line::from(format!("Mode: {:?}", model.mode)),
        Line::from(""),
        Line::from(vec![
            Span::styled("🛠️ Tools", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("• File Picker"),
        Line::from("• Command Palette"),
        Line::from("• Help System"),
    ];

    let sidebar = Paragraph::new(sidebar_content)
        .block(Block::default().borders(Borders::ALL).title("Sidebar"))
        .wrap(Wrap { trim: true });

    frame.render_widget(sidebar, area);
}

/// Render the main content area based on current mode
fn render_main_content(frame: &mut Frame, area: Rect, model: &AppModel) {
    match model.mode {
        AppMode::Chat => render_chat_mode(frame, area, model),
        AppMode::Command => render_command_mode(frame, area, model),
        AppMode::Diff => render_diff_mode(frame, area, model),
        AppMode::Help => render_help_mode(frame, area, model),
    }
}

/// Render chat mode interface
fn render_chat_mode(frame: &mut Frame, area: Rect, model: &AppModel) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Chat messages area
            Constraint::Length(3), // Input area
        ])
        .split(area);

    // Render chat messages
    render_chat_messages(frame, chunks[0], model);

    // Render input area
    render_chat_input(frame, chunks[1], model);

    // Render command palette if visible
    if model.commands.command_palette_visible {
        render_command_palette_overlay(frame, area, model);
    }

    // Render file picker if visible
    if model.ui.file_picker_visible {
        render_file_picker_overlay(frame, area, model);
    }
}

/// Render command mode interface
fn render_command_mode(frame: &mut Frame, area: Rect, model: &AppModel) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Command history area
            Constraint::Length(3), // Command input area
        ])
        .split(area);

    // Render command history
    render_command_history(frame, chunks[0], model);

    // Render command input
    render_command_input(frame, chunks[1], model);

    // Render command palette if visible
    if model.commands.command_palette_visible {
        render_command_palette_overlay(frame, area, model);
    }
}

/// Render diff mode interface
fn render_diff_mode(frame: &mut Frame, area: Rect, _model: &AppModel) {
    let diff_text = vec![
        Line::from("Diff mode - Coming soon..."),
        Line::from(""),
        Line::from("This mode will show code differences and changes."),
    ];

    let diff_widget = Paragraph::new(diff_text)
        .block(Block::default().borders(Borders::ALL).title("Diff View"))
        .wrap(Wrap { trim: true });

    frame.render_widget(diff_widget, area);
}

/// Render help mode interface
fn render_help_mode(frame: &mut Frame, area: Rect, _model: &AppModel) {
    let help_content = "RiceCoder Help\n\n\
        Ctrl+C - Exit application\n\
        Ctrl+S - Save current file\n\
        Ctrl+O - Open file\n\
        Ctrl+N - New file\n\
        F1 - Show this help\n\
        \n\
        For more detailed help, please refer to the documentation.";

    let help_widget = Paragraph::new(help_content)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });

    frame.render_widget(help_widget, area);
}

/// Render chat messages area
fn render_chat_messages(frame: &mut Frame, area: Rect, _model: &AppModel) {
    // TODO: Implement actual chat message rendering
    // For now, show placeholder content
    let messages = vec![
        Line::from("Welcome to RiceCoder Chat!"),
        Line::from(""),
        Line::from("This is where chat messages will appear."),
        Line::from("Messages from AI assistants and users will be displayed here."),
        Line::from(""),
        Line::from("Type your message below and press Enter to send."),
    ];

    let messages_widget = Paragraph::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Chat Messages"))
        .wrap(Wrap { trim: true });

    frame.render_widget(messages_widget, area);
}

/// Render chat input area
fn render_chat_input(frame: &mut Frame, area: Rect, model: &AppModel) {
    let input_text = if model.ui.chat_widget.input_content().is_empty() {
        "Type your message here...".to_string()
    } else {
        model.ui.chat_widget.input_content()
    };

    let input_widget = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title("Chat Input"))
        .wrap(Wrap { trim: true });

    frame.render_widget(input_widget, area);
}

/// Render command history area
fn render_command_history(frame: &mut Frame, area: Rect, model: &AppModel) {
    let history_lines: Vec<Line> = model.commands.command_history
        .iter()
        .rev() // Show most recent first
        .take(20) // Limit to last 20 commands
        .map(|cmd| Line::from(format!("$ {}", cmd)))
        .collect();

    let history_widget = Paragraph::new(history_lines)
        .block(Block::default().borders(Borders::ALL).title("Command History"))
        .wrap(Wrap { trim: true });

    frame.render_widget(history_widget, area);
}

/// Render command input area
fn render_command_input(frame: &mut Frame, area: Rect, model: &AppModel) {
    let input_text = if model.commands.current_command.is_empty() {
        "$ ".to_string()
    } else {
        format!("$ {}", model.commands.current_command)
    };

    let input_widget = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title("Command Input"))
        .wrap(Wrap { trim: true });

    frame.render_widget(input_widget, area);
}

/// Render command palette overlay
fn render_command_palette_overlay(frame: &mut Frame, area: Rect, _model: &AppModel) {
    let palette_area = centered_rect(60, 20, area);

    // Clear the background
    frame.render_widget(Clear, palette_area);

    let palette_content = vec![
        Line::from("Command Palette"),
        Line::from(""),
        Line::from("Type to search commands..."),
        Line::from(""),
        Line::from("Available commands:"),
        Line::from("• /new - Create new session"),
        Line::from("• /sessions - List sessions"),
        Line::from("• /help - Show help"),
        Line::from("• /exit - Exit application"),
    ];

    let palette_widget = Paragraph::new(palette_content)
        .block(Block::default().borders(Borders::ALL).title("Commands"))
        .wrap(Wrap { trim: true });

    frame.render_widget(palette_widget, palette_area);
}

/// Render file picker overlay
fn render_file_picker_overlay(frame: &mut Frame, area: Rect, _model: &AppModel) {
    let picker_area = centered_rect(70, 25, area);

    // Clear the background
    frame.render_widget(Clear, picker_area);

    let picker_content = vec![
        Line::from("File Picker"),
        Line::from(""),
        Line::from("Select a file to include in your session..."),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("• ↑/↓ - Navigate files"),
        Line::from("• Enter - Select file"),
        Line::from("• Esc - Cancel"),
    ];

    let picker_widget = Paragraph::new(picker_content)
        .block(Block::default().borders(Borders::ALL).title("File Picker"))
        .wrap(Wrap { trim: true });

    frame.render_widget(picker_widget, picker_area);
}

/// Render status bar
fn render_status_bar(frame: &mut Frame, area: Rect, model: &AppModel) {
    let status_parts = vec![
        Span::styled(model.mode.display_name(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(
            format!("Session: {}", model.sessions.active_session_id.as_deref().unwrap_or("None")),
            Style::default().fg(Color::Blue)
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Tokens: {}", model.sessions.total_tokens.input_tokens + model.sessions.total_tokens.output_tokens),
            Style::default().fg(Color::Yellow)
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Size: {}x{}", model.terminal_caps.size.0, model.terminal_caps.size.1),
            Style::default().fg(Color::Cyan)
        ),
    ];

    let status_line = Line::from(status_parts);
    let status_widget = Paragraph::new(status_line)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(status_widget, area);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}