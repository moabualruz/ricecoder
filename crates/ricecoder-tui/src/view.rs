//! View function for Elm Architecture (TEA) implementation
//!
//! This module contains the pure view function that renders the UI based on
//! the current application state. The view function is pure and only depends
//! on the model state.

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::model::*;

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
            Span::styled(
                "🍚 RiceCoder",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - AI-Powered Terminal Interface"),
        ]),
        Line::from(vec![
            Span::styled("Mode: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                model.mode.display_name(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled("Theme: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                model.theme.name.clone(),
                Style::default().fg(Color::Magenta),
            ),
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

    // Update sidebar with provider information
    render_sidebar_with_providers(frame, chunks[0], model);
}

/// Render the sidebar
fn render_sidebar(frame: &mut Frame, area: Rect, model: &AppModel) {
    render_sidebar_with_providers(frame, area, model);
}

/// Render the sidebar with provider information
fn render_sidebar_with_providers(frame: &mut Frame, area: Rect, model: &AppModel) {
    let mut sidebar_content = vec![
        Line::from(vec![Span::styled(
            "📁 Sessions",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(format!(
            "Active: {}",
            model
                .sessions
                .active_session_id
                .as_deref()
                .unwrap_or("None")
        )),
        Line::from(format!("Total: {}", model.sessions.session_count)),
        Line::from(""),
        Line::from(vec![Span::styled(
            "🤖 Providers",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

    // Add current provider info
    if let Some(current) = &model.providers.current_provider {
        sidebar_content.push(Line::from(format!("Current: {}", current)));
    } else {
        sidebar_content.push(Line::from("Current: None"));
    }

    sidebar_content.push(Line::from(format!(
        "Available: {}",
        model.providers.available_providers.len()
    )));
    sidebar_content.push(Line::from(""));

    // Add provider status summary
    let connected = model
        .providers
        .available_providers
        .iter()
        .filter(|p| matches!(p.state, crate::model::ProviderConnectionState::Connected))
        .count();
    let errors = model
        .providers
        .available_providers
        .iter()
        .filter(|p| matches!(p.state, crate::model::ProviderConnectionState::Error))
        .count();

    sidebar_content.push(Line::from(vec![Span::styled(
        "📊 Stats",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    sidebar_content.push(Line::from(""));
    sidebar_content.push(Line::from(format!(
        "Tokens: {}",
        model.sessions.total_tokens.input_tokens + model.sessions.total_tokens.output_tokens
    )));
    sidebar_content.push(Line::from(format!(
        "Connected: {}/{}",
        connected,
        model.providers.available_providers.len()
    )));
    if errors > 0 {
        sidebar_content.push(Line::from(vec![Span::styled(
            format!("Errors: {}", errors),
            Style::default().fg(Color::Red),
        )]));
    }
    sidebar_content.push(Line::from(format!("Mode: {:?}", model.mode)));
    sidebar_content.push(Line::from(""));

    sidebar_content.push(Line::from(vec![Span::styled(
        "🛠️ Tools",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    sidebar_content.push(Line::from(""));
    sidebar_content.push(Line::from("• File Picker"));
    sidebar_content.push(Line::from("• Command Palette"));
    sidebar_content.push(Line::from("• Provider Manager"));
    sidebar_content.push(Line::from("• Help System"));

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
        AppMode::Mcp => render_mcp_mode(frame, area, model),
        AppMode::Session => render_session_mode(frame, area, model),
        AppMode::Provider => render_provider_mode(frame, area, model),
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

/// Render MCP mode interface
fn render_mcp_mode(frame: &mut Frame, area: Rect, model: &AppModel) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Content
        ])
        .split(area);

    // Header with MCP status
    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "Model Context Protocol",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(format!("Servers: {}", model.mcp.servers.len())),
        Line::from(format!("Tools: {}", model.mcp.available_tools.len())),
    ])
    .block(Block::default().borders(Borders::ALL).title("MCP Status"))
    .wrap(Wrap { trim: true });

    frame.render_widget(header, chunks[0]);

    // MCP content area with server and tool lists
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Servers
            Constraint::Percentage(50), // Tools
        ])
        .split(chunks[1]);

    // Servers list
    let mut server_lines = Vec::new();
    for server in &model.mcp.servers {
        let status_icon = match server.health_status.as_str() {
            "healthy" => "🟢",
            "unhealthy" => "🔴",
            _ => "🟡",
        };
        server_lines.push(Line::from(format!(
            "{} {} - {}",
            status_icon,
            server.name,
            if server.enabled {
                "enabled"
            } else {
                "disabled"
            }
        )));
    }
    if server_lines.is_empty() {
        server_lines.push(Line::from("No MCP servers configured"));
    }

    let servers_list = Paragraph::new(server_lines)
        .block(Block::default().borders(Borders::ALL).title("MCP Servers"))
        .wrap(Wrap { trim: true });

    frame.render_widget(servers_list, content_chunks[0]);

    // Tools list
    let mut tool_lines = Vec::new();
    for tool in &model.mcp.available_tools {
        tool_lines.push(Line::from(format!(
            "{} - {} ({})",
            tool.tool_name, tool.description, tool.server_name
        )));
    }
    if tool_lines.is_empty() {
        tool_lines.push(Line::from("No tools available"));
    }

    let tools_list = Paragraph::new(tool_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Available Tools"),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(tools_list, content_chunks[1]);
}

/// Render session management mode interface
fn render_session_mode(frame: &mut Frame, area: Rect, model: &AppModel) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Content
        ])
        .split(area);

    // Header with session status
    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "Session Management",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(format!(
            "Active: {}",
            model
                .sessions
                .active_session_id
                .as_deref()
                .unwrap_or("None")
        )),
        Line::from(format!("Total: {}", model.sessions.session_count)),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Session Status"),
    )
    .wrap(Wrap { trim: true });

    frame.render_widget(header, chunks[0]);

    // Session content area
    if model.sessions.editor.is_editing {
        render_session_editor(frame, chunks[1], model);
    } else if model.sessions.sharing.is_sharing {
        render_session_sharing(frame, chunks[1], model);
    } else {
        render_session_browser(frame, chunks[1], model);
    }
}

/// Render session browser component
fn render_session_browser(frame: &mut Frame, area: Rect, model: &AppModel) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Session list
            Constraint::Percentage(30), // Session details
        ])
        .split(area);

    // Session list
    let mut session_lines = Vec::new();
    for (index, session) in model.sessions.browser.sessions.iter().enumerate() {
        let is_selected = index == model.sessions.browser.selected_index;
        let status_icon = match session.status {
            crate::model::SessionStatus::Active => "🟢",
            crate::model::SessionStatus::Paused => "🟡",
            crate::model::SessionStatus::Completed => "✅",
            crate::model::SessionStatus::Failed => "🔴",
        };

        let line = Line::from(format!(
            "{} {} - {} ({})",
            status_icon,
            session.name,
            session.provider,
            if session.is_shared {
                "shared"
            } else {
                "private"
            }
        ));

        if is_selected {
            session_lines.push(Line::from(Span::styled(
                line.spans[0].content.clone(),
                Style::default().bg(Color::Blue),
            )));
        } else {
            session_lines.push(line);
        }
    }

    if session_lines.is_empty() {
        session_lines.push(Line::from(
            "No sessions found. Press 'n' to create a new session.",
        ));
    }

    let session_list = Paragraph::new(session_lines)
        .block(Block::default().borders(Borders::ALL).title("Sessions"))
        .wrap(Wrap { trim: true });

    frame.render_widget(session_list, chunks[0]);

    // Session details
    let details_lines = if let Some(selected_session) = model
        .sessions
        .browser
        .sessions
        .get(model.sessions.browser.selected_index)
    {
        vec![
            Line::from(format!("Name: {}", selected_session.name)),
            Line::from(format!("ID: {}", selected_session.id)),
            Line::from(format!("Provider: {}", selected_session.provider)),
            Line::from(format!("Status: {:?}", selected_session.status)),
            Line::from(format!("Tokens: {}", selected_session.token_count)),
            Line::from(format!(
                "Shared: {}",
                if selected_session.is_shared {
                    "Yes"
                } else {
                    "No"
                }
            )),
            Line::from(format!(
                "Created: {}",
                format_timestamp(selected_session.created_at as i64)
            )),
            Line::from(format!(
                "Last Activity: {}",
                format_timestamp(selected_session.last_activity as i64)
            )),
        ]
    } else {
        vec![Line::from("No session selected")]
    };

    let details = Paragraph::new(details_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Session Details"),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(details, chunks[1]);
}

/// Render session editor component
fn render_session_editor(frame: &mut Frame, area: Rect, model: &AppModel) {
    let editor = &model.sessions.editor;

    let content = Paragraph::new(vec![
        Line::from(format!("Name: {}", editor.name)),
        Line::from(format!("Provider: {}", editor.provider)),
        Line::from(format!("Description: {}", editor.description)),
        Line::from(""),
        Line::from("Press Enter to save, Esc to cancel"),
    ])
    .block(Block::default().borders(Borders::ALL).title(
        if editor.is_editing && editor.session_id.is_some() {
            "Edit Session"
        } else {
            "Create Session"
        },
    ))
    .wrap(Wrap { trim: true });

    frame.render_widget(content, area);
}

/// Render session sharing component
fn render_session_sharing(frame: &mut Frame, area: Rect, model: &AppModel) {
    let sharing = &model.sessions.sharing;

    let mut content_lines = vec![
        Line::from(format!("Session: {}", sharing.session_id)),
        Line::from(format!(
            "Expires: {} seconds",
            sharing.expires_in.unwrap_or(0)
        )),
        Line::from(format!("Permissions: {:?}", sharing.permissions)),
    ];

    if let Some(url) = &sharing.share_url {
        content_lines.push(Line::from(""));
        content_lines.push(Line::from("Share URL:"));
        content_lines.push(Line::from(url.clone()));
    }

    let content = Paragraph::new(content_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Share Session"),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(content, area);
}

/// Format timestamp for display
fn format_timestamp(ts: i64) -> String {
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_else(|| DateTime::<Utc>::UNIX_EPOCH);
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Render provider mode interface
fn render_provider_mode(frame: &mut Frame, area: Rect, model: &AppModel) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with view mode
            Constraint::Min(5),    // Provider list/content
            Constraint::Length(3), // Footer with controls
        ])
        .split(area);

    // Header with current view mode
    let view_mode_indicator = match model.providers.view_mode {
        crate::model::ProviderViewMode::List => "📋 List",
        crate::model::ProviderViewMode::Status => "📊 Status",
        crate::model::ProviderViewMode::Performance => "⚡ Performance",
        crate::model::ProviderViewMode::Analytics => "📈 Analytics",
    };

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "AI Provider Management",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(view_mode_indicator, Style::default().fg(Color::Green)),
        ]),
        Line::from(format!(
            "Current: {}",
            model
                .providers
                .current_provider
                .as_deref()
                .unwrap_or("None")
        )),
        Line::from(format!(
            "Filter: {}",
            if model.providers.filter_text.is_empty() {
                "None"
            } else {
                &model.providers.filter_text
            }
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Provider Manager"),
    )
    .wrap(Wrap { trim: true });

    frame.render_widget(header, chunks[0]);

    // Main content based on view mode
    match model.providers.view_mode {
        crate::model::ProviderViewMode::List => render_provider_list(frame, chunks[1], model),
        crate::model::ProviderViewMode::Status => render_provider_status(frame, chunks[1], model),
        crate::model::ProviderViewMode::Performance => {
            render_provider_performance(frame, chunks[1], model)
        }
        crate::model::ProviderViewMode::Analytics => {
            render_provider_analytics(frame, chunks[1], model)
        }
    }

    // Footer with controls
    let footer = Paragraph::new(vec![
        Line::from("Controls:"),
        Line::from(vec![
            Span::styled(
                "l",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - List | "),
            Span::styled(
                "s",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Status | "),
            Span::styled(
                "p",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Performance | "),
            Span::styled(
                "a",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Analytics"),
        ]),
        Line::from(vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Navigate | "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Switch Provider | "),
            Span::styled(
                "/",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Command Mode"),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Controls"))
    .wrap(Wrap { trim: true });

    frame.render_widget(footer, chunks[2]);
}

/// Render provider list view
fn render_provider_list(frame: &mut Frame, area: Rect, model: &AppModel) {
    let mut content = Vec::new();

    for provider in &model.providers.available_providers {
        let is_selected = model.providers.selected_provider.as_ref() == Some(&provider.id);
        let is_current = model.providers.current_provider.as_ref() == Some(&provider.id);

        let status_icon = match provider.state {
            crate::model::ProviderConnectionState::Connected => "🟢",
            crate::model::ProviderConnectionState::Disconnected => "🟡",
            crate::model::ProviderConnectionState::Error => "🔴",
            crate::model::ProviderConnectionState::Disabled => "⚪",
        };

        let mut line_spans = vec![Span::raw(status_icon), Span::raw(" ")];

        if is_current {
            line_spans.push(Span::styled(
                &provider.name,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ));
            line_spans.push(Span::styled(
                " (current)",
                Style::default().fg(Color::Green),
            ));
        } else if is_selected {
            line_spans.push(Span::styled(
                &provider.name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
            line_spans.push(Span::styled(
                " (selected)",
                Style::default().fg(Color::Yellow),
            ));
        } else {
            line_spans.push(Span::raw(&provider.name));
        }

        line_spans.push(Span::raw(format!(" - {} models", provider.models.len())));

        content.push(Line::from(line_spans));

        if let Some(error) = &provider.error_message {
            content.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("Error: {}", error), Style::default().fg(Color::Red)),
            ]));
        }
    }

    if content.is_empty() {
        content.push(Line::from("No providers configured. Use 'ricecoder providers' CLI command to configure providers."));
    }

    let list = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Available Providers"),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(list, area);
}

/// Render provider status view
fn render_provider_status(frame: &mut Frame, area: Rect, model: &AppModel) {
    let mut content = Vec::new();

    for provider in &model.providers.available_providers {
        content.push(Line::from(vec![
            Span::styled(
                &provider.name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ("),
            Span::raw(&provider.id),
            Span::raw(")"),
        ]));

        content.push(Line::from(format!("  Status: {:?}", provider.state)));
        content.push(Line::from(format!("  Models: {}", provider.models.len())));

        if let Some(last_checked) = provider.last_checked {
            content.push(Line::from(format!(
                "  Last checked: {}",
                last_checked.format("%Y-%m-%d %H:%M:%S")
            )));
        }

        if let Some(error) = &provider.error_message {
            content.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("Error: {}", error), Style::default().fg(Color::Red)),
            ]));
        }

        content.push(Line::from(""));
    }

    if content.is_empty() {
        content.push(Line::from("No provider status information available."));
    }

    let status = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Provider Status"),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(status, area);
}

/// Render provider performance view
fn render_provider_performance(frame: &mut Frame, area: Rect, model: &AppModel) {
    let mut content = Vec::new();

    for (provider_id, metrics) in &model.providers.provider_metrics {
        content.push(Line::from(vec![Span::styled(
            provider_id,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));

        content.push(Line::from(format!(
            "  Requests: {} total ({} success, {} failed)",
            metrics.total_requests, metrics.successful_requests, metrics.failed_requests
        )));
        content.push(Line::from(format!(
            "  Response time: {:.2}ms avg",
            metrics.avg_response_time_ms
        )));
        content.push(Line::from(format!(
            "  Error rate: {:.2}%",
            metrics.error_rate * 100.0
        )));
        content.push(Line::from(format!(
            "  Tokens: {} total (${:.4} cost)",
            metrics.total_tokens, metrics.total_cost
        )));
        content.push(Line::from(format!(
            "  Throughput: {:.2} req/s, {:.2} tok/s",
            metrics.requests_per_second, metrics.tokens_per_second
        )));
        content.push(Line::from(""));
    }

    if content.is_empty() {
        content.push(Line::from(
            "No performance metrics available. Use providers to generate some activity.",
        ));
    }

    let performance = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Provider Performance"),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(performance, area);
}

/// Render provider analytics view
fn render_provider_analytics(frame: &mut Frame, area: Rect, model: &AppModel) {
    let mut content = Vec::new();

    // Summary statistics
    let total_providers = model.providers.available_providers.len();
    let connected_providers = model
        .providers
        .available_providers
        .iter()
        .filter(|p| matches!(p.state, crate::model::ProviderConnectionState::Connected))
        .count();
    let total_requests: u64 = model
        .providers
        .provider_metrics
        .values()
        .map(|m| m.total_requests)
        .sum();
    let total_errors: u64 = model
        .providers
        .provider_metrics
        .values()
        .map(|m| m.failed_requests)
        .sum();

    content.push(Line::from(vec![Span::styled(
        "Summary",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    content.push(Line::from(format!(
        "  Total providers: {}",
        total_providers
    )));
    content.push(Line::from(format!(
        "  Connected providers: {}",
        connected_providers
    )));
    content.push(Line::from(format!("  Total requests: {}", total_requests)));
    content.push(Line::from(format!("  Total errors: {}", total_errors)));

    if total_requests > 0 {
        let overall_error_rate = (total_errors as f64) / (total_requests as f64) * 100.0;
        content.push(Line::from(format!(
            "  Overall error rate: {:.2}%",
            overall_error_rate
        )));
    }

    content.push(Line::from(""));

    // Best performing providers
    content.push(Line::from(vec![Span::styled(
        "Best Performing (by response time)",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));

    let mut sorted_providers: Vec<_> = model.providers.provider_metrics.iter().collect();
    sorted_providers.sort_by(|a, b| {
        a.1.avg_response_time_ms
            .partial_cmp(&b.1.avg_response_time_ms)
            .unwrap()
    });

    for (provider_id, metrics) in sorted_providers.iter().take(3) {
        content.push(Line::from(format!(
            "  {}: {:.2}ms",
            provider_id, metrics.avg_response_time_ms
        )));
    }

    if content.len() <= 2 {
        content.push(Line::from("  No performance data available"));
    }

    let analytics = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Provider Analytics"),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(analytics, area);
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Chat Messages"),
        )
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
    let history_lines: Vec<Line> = model
        .commands
        .command_history
        .iter()
        .rev() // Show most recent first
        .take(20) // Limit to last 20 commands
        .map(|cmd| Line::from(format!("$ {}", cmd)))
        .collect();

    let history_widget = Paragraph::new(history_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command History"),
        )
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command Input"),
        )
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
        Span::styled(
            model.mode.display_name(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "Session: {}",
                model
                    .sessions
                    .active_session_id
                    .as_deref()
                    .unwrap_or("None")
            ),
            Style::default().fg(Color::Blue),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "Tokens: {}",
                model.sessions.total_tokens.input_tokens
                    + model.sessions.total_tokens.output_tokens
            ),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "Size: {}x{}",
                model.terminal_caps.size.0, model.terminal_caps.size.1
            ),
            Style::default().fg(Color::Cyan),
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
