//! Tool output component for displaying MCP tool execution results

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

use crate::{
    code_editor_widget::Language,
    components::{Component, ComponentId, FocusDirection, FocusResult},
    model::{AppMessage, AppModel},
};

/// Tool output display component
///
/// Displays MCP tool execution results with:
/// - Collapsible sections
/// - Syntax highlighting for code
/// - Error display
/// - Tool metadata (server, tool name)
#[derive(Clone, Debug)]
pub struct ToolOutput {
    /// Component ID
    id: ComponentId,
    /// Server name
    server: String,
    /// Tool name
    tool: String,
    /// Result (JSON value) or error message
    result: ToolResult,
    /// Collapsed/expanded state
    collapsed: bool,
    /// Bounds
    bounds: Rect,
    /// Focused state
    focused: bool,
    /// Z-index
    z_index: i32,
}

/// Tool execution result
#[derive(Clone, Debug)]
pub enum ToolResult {
    /// Successful execution
    Success(serde_json::Value),
    /// Failed execution
    Error(String),
}

impl ToolOutput {
    /// Create new ToolOutput for successful execution
    pub fn new_success(
        server: impl Into<String>,
        tool: impl Into<String>,
        result: serde_json::Value,
    ) -> Self {
        let server = server.into();
        let tool = tool.into();
        Self {
            id: format!("tool-output-{}-{}", server, tool),
            server,
            tool,
            result: ToolResult::Success(result),
            collapsed: false,
            bounds: Rect::default(),
            focused: false,
            z_index: 0,
        }
    }

    /// Create new ToolOutput for failed execution
    pub fn new_error(
        server: impl Into<String>,
        tool: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        let server = server.into();
        let tool = tool.into();
        Self {
            id: format!("tool-output-{}-{}", server, tool),
            server,
            tool,
            result: ToolResult::Error(error.into()),
            collapsed: false,
            bounds: Rect::default(),
            focused: false,
            z_index: 0,
        }
    }

    /// Toggle collapsed/expanded state
    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Set collapsed state
    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.collapsed = collapsed;
    }

    /// Check if collapsed
    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    /// Get tool name
    pub fn tool_name(&self) -> &str {
        &self.tool
    }

    /// Get server name
    pub fn server_name(&self) -> &str {
        &self.server
    }

    /// Format result as pretty JSON or error message
    fn format_output(&self) -> Vec<Line<'static>> {
        match &self.result {
            ToolResult::Success(value) => {
                // Pretty-print JSON with syntax highlighting
                let json_str = serde_json::to_string_pretty(value)
                    .unwrap_or_else(|_| format!("{:?}", value));
                
                // Simple syntax highlighting for JSON
                json_str
                    .lines()
                    .map(|line| {
                        if line.trim_start().starts_with('"') && line.contains(':') {
                            // Key-value pair
                            Line::from(vec![
                                Span::styled(line.to_string(), Style::default().fg(Color::Cyan)),
                            ])
                        } else if line.trim().starts_with('{') || line.trim().starts_with('}') {
                            // Braces
                            Line::from(vec![
                                Span::styled(line.to_string(), Style::default().fg(Color::Yellow)),
                            ])
                        } else {
                            // Default
                            Line::from(line.to_string())
                        }
                    })
                    .collect()
            }
            ToolResult::Error(err) => {
                // Error message in red
                vec![Line::from(vec![
                    Span::styled(
                        format!("Error: {}", err),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                ])]
            }
        }
    }
}

#[allow(deprecated)]
impl Component for ToolOutput {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn render(&self, frame: &mut Frame, area: Rect, _model: &AppModel) {
        // Title with server/tool info
        let title = format!(
            "{} [{}] {}",
            self.server,
            self.tool,
            if self.collapsed { "▶" } else { "▼" }
        );

        // Block with borders
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if self.focused {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            });

        if self.collapsed {
            // Render collapsed (just the block)
            frame.render_widget(block, area);
        } else {
            // Render expanded with output
            let lines = self.format_output();
            let paragraph = Paragraph::new(lines).block(block);
            frame.render_widget(paragraph, area);
        }
    }

    fn update(&mut self, message: &AppMessage, _model: &AppModel) -> bool {
        match message {
            AppMessage::KeyPress(key) => {
                // Handle collapse/expand on Space or Enter
                if self.focused
                    && (key.code == crossterm::event::KeyCode::Char(' ')
                        || key.code == crossterm::event::KeyCode::Enter)
                {
                    self.toggle_collapsed();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn is_visible(&self) -> bool {
        true
    }

    fn set_visible(&mut self, _visible: bool) {}

    fn is_enabled(&self) -> bool {
        true
    }

    fn set_enabled(&mut self, _enabled: bool) {}

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn handle_focus(&mut self, direction: FocusDirection) -> FocusResult {
        match direction {
            FocusDirection::Next | FocusDirection::Previous => FocusResult::Boundary,
            _ => FocusResult::NotFocusable,
        }
    }

    fn children(&self) -> Vec<&dyn Component> {
        Vec::new()
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Component> {
        Vec::new()
    }

    fn find_child(&self, _id: &ComponentId) -> Option<&dyn Component> {
        None
    }

    fn find_child_mut(&mut self, _id: &ComponentId) -> Option<&mut dyn Component> {
        None
    }

    fn add_child(&mut self, _child: Box<dyn Component>) {}

    fn remove_child(&mut self, _id: &ComponentId) -> Option<Box<dyn Component>> {
        None
    }

    fn z_index(&self) -> i32 {
        self.z_index
    }

    fn set_z_index(&mut self, z_index: i32) {
        self.z_index = z_index;
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn tab_order(&self) -> Option<usize> {
        None
    }

    fn set_tab_order(&mut self, _order: Option<usize>) {}

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_success() {
        let output = ToolOutput::new_success(
            "test-server",
            "test-tool",
            serde_json::json!({"status": "ok"}),
        );
        assert_eq!(output.server_name(), "test-server");
        assert_eq!(output.tool_name(), "test-tool");
        assert!(!output.is_collapsed());
    }

    #[test]
    fn test_create_error() {
        let output = ToolOutput::new_error("test-server", "test-tool", "Test error");
        assert_eq!(output.server_name(), "test-server");
        assert_eq!(output.tool_name(), "test-tool");
        assert!(!output.is_collapsed());
    }

    #[test]
    fn test_toggle_collapsed() {
        let mut output = ToolOutput::new_success(
            "test-server",
            "test-tool",
            serde_json::json!({"status": "ok"}),
        );
        assert!(!output.is_collapsed());
        output.toggle_collapsed();
        assert!(output.is_collapsed());
        output.toggle_collapsed();
        assert!(!output.is_collapsed());
    }

    #[test]
    fn test_component_id() {
        let output = ToolOutput::new_success(
            "test-server",
            "test-tool",
            serde_json::json!({"status": "ok"}),
        );
        assert_eq!(output.id(), "tool-output-test-server-test-tool");
    }

    #[test]
    fn test_focus() {
        let mut output = ToolOutput::new_success(
            "test-server",
            "test-tool",
            serde_json::json!({"status": "ok"}),
        );
        assert!(!output.is_focused());
        output.set_focused(true);
        assert!(output.is_focused());
    }
}
