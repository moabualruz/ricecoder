//! Session sidebar component
//!
//! Displays collapsible sections for MCP, LSP, todos, diffs, and context info.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

/// Sidebar section types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarSection {
    Mcp,
    Lsp,
    Todos,
    Diffs,
    Context,
    GettingStarted,
}

/// Sidebar item for display
#[derive(Debug, Clone)]
pub struct SidebarItem {
    /// Item label
    pub label: String,
    /// Item status (for MCP/LSP)
    pub status: Option<ItemStatus>,
    /// Secondary text
    pub secondary: Option<String>,
    /// Is item selected
    pub selected: bool,
}

/// Status for sidebar items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemStatus {
    Connected,
    Disconnected,
    Error,
    Pending,
}

impl ItemStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Connected => "●",
            Self::Disconnected => "○",
            Self::Error => "✗",
            Self::Pending => "◐",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Connected => Color::Green,
            Self::Disconnected => Color::Gray,
            Self::Error => Color::Red,
            Self::Pending => Color::Yellow,
        }
    }
}

/// Session sidebar state
#[derive(Debug, Clone)]
pub struct SessionSidebar {
    /// Width of sidebar
    pub width: u16,
    /// Collapsed sections
    pub collapsed: Vec<SidebarSection>,
    /// MCP servers
    pub mcp_servers: Vec<SidebarItem>,
    /// LSP servers
    pub lsp_servers: Vec<SidebarItem>,
    /// Todo items
    pub todos: Vec<SidebarItem>,
    /// Diff files
    pub diffs: Vec<SidebarItem>,
    /// Context files
    pub context_files: Vec<SidebarItem>,
    /// Show getting started
    pub show_getting_started: bool,
    /// Currently selected section
    pub selected_section: Option<SidebarSection>,
    /// List state for navigation
    list_state: ListState,
}

impl Default for SessionSidebar {
    fn default() -> Self {
        Self {
            width: 40,
            collapsed: Vec::new(),
            mcp_servers: Vec::new(),
            lsp_servers: Vec::new(),
            todos: Vec::new(),
            diffs: Vec::new(),
            context_files: Vec::new(),
            show_getting_started: true,
            selected_section: None,
            list_state: ListState::default(),
        }
    }
}

impl SessionSidebar {
    /// Create a new sidebar
    pub fn new() -> Self {
        Self::default()
    }

    /// Set sidebar width
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Toggle section collapse state
    pub fn toggle_section(&mut self, section: SidebarSection) {
        if let Some(pos) = self.collapsed.iter().position(|s| *s == section) {
            self.collapsed.remove(pos);
        } else {
            self.collapsed.push(section);
        }
    }

    /// Check if section is collapsed
    pub fn is_collapsed(&self, section: SidebarSection) -> bool {
        self.collapsed.contains(&section)
    }

    /// Set MCP servers
    pub fn with_mcp_servers(mut self, servers: Vec<SidebarItem>) -> Self {
        self.mcp_servers = servers;
        self
    }

    /// Set LSP servers
    pub fn with_lsp_servers(mut self, servers: Vec<SidebarItem>) -> Self {
        self.lsp_servers = servers;
        self
    }

    /// Set todo items
    pub fn with_todos(mut self, todos: Vec<SidebarItem>) -> Self {
        self.todos = todos;
        self
    }

    /// Set diff files
    pub fn with_diffs(mut self, diffs: Vec<SidebarItem>) -> Self {
        self.diffs = diffs;
        self
    }

    /// Dismiss getting started
    pub fn dismiss_getting_started(&mut self) {
        self.show_getting_started = false;
    }

    /// Render the sidebar
    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &SidebarTheme) {
        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(theme.border));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Calculate layout for sections
        let mut current_y = inner.y;
        let section_gap = 1;

        // MCP Section
        if !self.mcp_servers.is_empty() {
            let height = self.render_section(
                frame,
                Rect::new(inner.x, current_y, inner.width, inner.height.saturating_sub(current_y - inner.y)),
                "MCP Servers",
                SidebarSection::Mcp,
                &self.mcp_servers.clone(),
                theme,
            );
            current_y += height + section_gap;
        }

        // LSP Section
        if !self.lsp_servers.is_empty() && current_y < inner.y + inner.height {
            let height = self.render_section(
                frame,
                Rect::new(inner.x, current_y, inner.width, inner.height.saturating_sub(current_y - inner.y)),
                "LSP",
                SidebarSection::Lsp,
                &self.lsp_servers.clone(),
                theme,
            );
            current_y += height + section_gap;
        }

        // Todos Section
        if !self.todos.is_empty() && current_y < inner.y + inner.height {
            let height = self.render_section(
                frame,
                Rect::new(inner.x, current_y, inner.width, inner.height.saturating_sub(current_y - inner.y)),
                "Todos",
                SidebarSection::Todos,
                &self.todos.clone(),
                theme,
            );
            current_y += height + section_gap;
        }

        // Diffs Section
        if !self.diffs.is_empty() && current_y < inner.y + inner.height {
            let height = self.render_section(
                frame,
                Rect::new(inner.x, current_y, inner.width, inner.height.saturating_sub(current_y - inner.y)),
                "Changes",
                SidebarSection::Diffs,
                &self.diffs.clone(),
                theme,
            );
            current_y += height + section_gap;
        }

        // Getting Started (dismissible)
        if self.show_getting_started && current_y < inner.y + inner.height {
            self.render_getting_started(
                frame,
                Rect::new(inner.x, current_y, inner.width, inner.height.saturating_sub(current_y - inner.y).min(6)),
                theme,
            );
        }
    }

    /// Render a collapsible section
    fn render_section(
        &self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        section: SidebarSection,
        items: &[SidebarItem],
        theme: &SidebarTheme,
    ) -> u16 {
        let is_collapsed = self.is_collapsed(section);
        let collapse_icon = if is_collapsed { "▸" } else { "▾" };

        // Section header
        let header = Line::from(vec![
            Span::styled(collapse_icon, Style::default().fg(theme.section_icon)),
            Span::styled(" ", Style::default()),
            Span::styled(title, Style::default().fg(theme.section_title).bold()),
            Span::styled(
                format!(" ({})", items.len()),
                Style::default().fg(theme.section_count),
            ),
        ]);

        let header_para = Paragraph::new(header);
        frame.render_widget(header_para, Rect::new(area.x, area.y, area.width, 1));

        if is_collapsed {
            return 1;
        }

        // Render items
        let max_items = (area.height as usize).saturating_sub(1).min(items.len());
        let list_items: Vec<ListItem> = items
            .iter()
            .take(max_items)
            .map(|item| {
                let mut spans = Vec::new();
                
                // Status indicator
                if let Some(status) = &item.status {
                    spans.push(Span::styled(
                        format!("{} ", status.symbol()),
                        Style::default().fg(status.color()),
                    ));
                } else {
                    spans.push(Span::styled("  ", Style::default()));
                }

                // Label
                spans.push(Span::styled(
                    &item.label,
                    Style::default().fg(theme.item_label),
                ));

                // Secondary text
                if let Some(ref secondary) = item.secondary {
                    spans.push(Span::styled(
                        format!(" {}", secondary),
                        Style::default().fg(theme.item_secondary),
                    ));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(list_items);
        frame.render_widget(
            list,
            Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(1), max_items as u16),
        );

        1 + max_items as u16
    }

    /// Render getting started section
    fn render_getting_started(&self, frame: &mut Frame, area: Rect, theme: &SidebarTheme) {
        let tips = vec![
            "Type a message to get started",
            "Use ? for help",
            "Ctrl+P for command palette",
            "Press Esc to dismiss",
        ];

        let block = Block::default()
            .title(" Getting Started ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.getting_started_border));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        for (i, tip) in tips.iter().enumerate() {
            if i as u16 >= inner.height {
                break;
            }
            let para = Paragraph::new(*tip)
                .style(Style::default().fg(theme.getting_started_text));
            frame.render_widget(
                para,
                Rect::new(inner.x, inner.y + i as u16, inner.width, 1),
            );
        }
    }
}

/// Theme colors for sidebar
#[derive(Debug, Clone)]
pub struct SidebarTheme {
    pub border: Color,
    pub section_icon: Color,
    pub section_title: Color,
    pub section_count: Color,
    pub item_label: Color,
    pub item_secondary: Color,
    pub getting_started_border: Color,
    pub getting_started_text: Color,
}

impl Default for SidebarTheme {
    fn default() -> Self {
        Self {
            border: Color::DarkGray,
            section_icon: Color::Cyan,
            section_title: Color::White,
            section_count: Color::Gray,
            item_label: Color::White,
            item_secondary: Color::Gray,
            getting_started_border: Color::Cyan,
            getting_started_text: Color::Gray,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_creation() {
        let sidebar = SessionSidebar::new().with_width(50);
        assert_eq!(sidebar.width, 50);
        assert!(sidebar.show_getting_started);
    }

    #[test]
    fn test_section_collapse() {
        let mut sidebar = SessionSidebar::new();
        assert!(!sidebar.is_collapsed(SidebarSection::Mcp));

        sidebar.toggle_section(SidebarSection::Mcp);
        assert!(sidebar.is_collapsed(SidebarSection::Mcp));

        sidebar.toggle_section(SidebarSection::Mcp);
        assert!(!sidebar.is_collapsed(SidebarSection::Mcp));
    }

    #[test]
    fn test_item_status() {
        assert_eq!(ItemStatus::Connected.symbol(), "●");
        assert_eq!(ItemStatus::Error.symbol(), "✗");
    }

    #[test]
    fn test_dismiss_getting_started() {
        let mut sidebar = SessionSidebar::new();
        assert!(sidebar.show_getting_started);

        sidebar.dismiss_getting_started();
        assert!(!sidebar.show_getting_started);
    }
}
