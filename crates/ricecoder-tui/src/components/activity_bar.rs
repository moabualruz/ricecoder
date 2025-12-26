use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, StatefulWidget, Widget},
};

/// Activity bar item
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityItem {
    Files,
    Git,
    Search,
    Chat,
    Mcp,
    Extensions,
    Settings,
}

impl ActivityItem {
    /// Get the icon for the activity item
    pub fn icon(&self) -> &'static str {
        match self {
            ActivityItem::Files => "📁",
            ActivityItem::Git => "code_branch", // Using text for now, nerd fonts later
            ActivityItem::Search => "🔍",
            ActivityItem::Chat => "💬",
            ActivityItem::Mcp => "🔌",
            ActivityItem::Extensions => "🧩",
            ActivityItem::Settings => "⚙️",
        }
    }

    /// Get the label for the activity item
    pub fn label(&self) -> &'static str {
        match self {
            ActivityItem::Files => "Files",
            ActivityItem::Git => "Source Control",
            ActivityItem::Search => "Search",
            ActivityItem::Chat => "Chat",
            ActivityItem::Mcp => "MCP",
            ActivityItem::Extensions => "Extensions",
            ActivityItem::Settings => "Settings",
        }
    }
}

/// Activity bar state
#[derive(Debug, Clone, PartialEq)]
pub struct ActivityBarState {
    pub selected: Option<ActivityItem>,
    pub items: Vec<ActivityItem>,
}

impl Default for ActivityBarState {
    fn default() -> Self {
        Self {
            selected: Some(ActivityItem::Files),
            items: vec![
                ActivityItem::Files,
                ActivityItem::Search,
                ActivityItem::Git,
                ActivityItem::Chat,
                ActivityItem::Mcp,
                ActivityItem::Extensions,
                ActivityItem::Settings,
            ],
        }
    }
}

impl ActivityBarState {
    pub fn select(&mut self, item: ActivityItem) {
        if self.selected == Some(item.clone()) {
            // Toggle off if already selected (optional behavior)
            // self.selected = None; 
        } else {
            self.selected = Some(item);
        }
    }
    
    pub fn select_next(&mut self) {
        if let Some(current) = &self.selected {
            if let Some(pos) = self.items.iter().position(|i| i == current) {
                let next = (pos + 1) % self.items.len();
                self.selected = Some(self.items[next].clone());
            }
        } else if !self.items.is_empty() {
            self.selected = Some(self.items[0].clone());
        }
    }

    pub fn select_prev(&mut self) {
        if let Some(current) = &self.selected {
            if let Some(pos) = self.items.iter().position(|i| i == current) {
                let prev = if pos == 0 { self.items.len() - 1 } else { pos - 1 };
                self.selected = Some(self.items[prev].clone());
            }
        } else if !self.items.is_empty() {
            self.selected = Some(self.items[self.items.len() - 1].clone());
        }
    }
}

/// Activity bar widget
pub struct ActivityBar;

impl StatefulWidget for ActivityBar {
    type State = ActivityBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default()
            .borders(Borders::RIGHT)
            .style(Style::default().fg(Color::DarkGray));
        
        block.render(area, buf);

        let inner_area = area; // Simplified for now, borders handled by block

        for (i, item) in state.items.iter().enumerate() {
            if i >= inner_area.height as usize {
                break;
            }

            let y = inner_area.y + i as u16;
            let is_selected = state.selected.as_ref() == Some(item);

            let icon = item.icon();
            // Fallback for git icon if nerd fonts aren't rendering well in simple terminals
            let display_icon = if item == &ActivityItem::Git { "G" } else { icon };

            let style = if is_selected {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            // Draw selection indicator
            if is_selected {
                buf.set_string(inner_area.x, y, "│", Style::default().fg(Color::Cyan));
            }

            // Draw icon centered-ish
            buf.set_string(inner_area.x + 1, y, display_icon, style);
        }
        
        // Draw settings at the bottom
        // Logic omitted for brevity, can add later
    }
}
