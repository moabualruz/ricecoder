use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Widget},
};

pub struct TopBar {
    pub version: String,
    pub active_agent: String,
}

impl TopBar {
    pub fn new(version: impl Into<String>, active_agent: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            active_agent: active_agent.into(),
        }
    }
}

impl Widget for TopBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().style(Style::default().bg(Color::Rgb(20, 20, 20))); // Dark background
        block.render(area, buf);

        // 1. Logo (Left)
        let logo_span = Span::styled(
            "r[", 
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        );
        let logo_end = Span::styled(
            "]", 
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        );
        
        // 2. Burger Menu (Left, after logo)
        let burger = Span::raw(" ☰ ");
        
        // 3. Navigation Arrows (Left, after burger)
        let arrows = Span::styled(" ← → ", Style::default().fg(Color::DarkGray));

        // 4. Search/Command Bar (Center-ish placeholder)
        // For now just empty space or a subtle hint

        // 5. Active Agent / Session Info (Right side)
        let agent_info = Span::styled(
            format!(" {} ", self.active_agent),
            Style::default().fg(Color::Cyan)
        );

        // 6. Textual Logo & Version (Far Right)
        let text_logo = Span::styled("riceCoder", Style::default().fg(Color::Green));
        let version = Span::styled(
            format!(" v{} ", self.version),
            Style::default().fg(Color::DarkGray)
        );

        // Render Left Side
        let left_content = Line::from(vec![
            logo_span,
            burger,
            arrows,
        ]);
        buf.set_line(area.x + 1, area.y, &left_content, area.width);

        // Render Right Side
        // Calculate width to align right
        let right_content = Line::from(vec![
            agent_info,
            Span::raw(" | "),
            text_logo,
            version,
        ]);
        
        let right_width = right_content.width() as u16;
        if area.width > right_width {
            buf.set_line(area.x + area.width - right_width - 1, area.y, &right_content, right_width);
        }
    }
}
