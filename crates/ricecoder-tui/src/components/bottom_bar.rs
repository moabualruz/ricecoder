use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Widget},
};

pub struct BottomBar {
    pub context_info: String,
    pub status_message: String,
    pub current_path: String,
}

impl BottomBar {
    pub fn new(context_info: impl Into<String>, status_message: impl Into<String>, current_path: impl Into<String>) -> Self {
        Self {
            context_info: context_info.into(),
            status_message: status_message.into(),
            current_path: current_path.into(),
        }
    }
}

impl Widget for BottomBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().style(Style::default().bg(Color::Rgb(30, 30, 30)));
        block.render(area, buf);

        // 1. Current Path (Left)
        let path_span = Span::styled(
            format!(" {} ", self.current_path),
            Style::default().fg(Color::Blue)
        );

        // 2. Status Message (Left, after path)
        let status_span = Span::raw(format!(" {} ", self.status_message));

        // 3. Context Info (Center/Right)
        let context_span = Span::styled(
            self.context_info,
            Style::default().fg(Color::DarkGray)
        );

        let left_content = Line::from(vec![path_span, status_span]);
        buf.set_line(area.x, area.y, &left_content, area.width);

        // Render context info on the right
        let context_width = context_span.width() as u16;
        if area.width > context_width {
             buf.set_span(area.x + area.width - context_width - 1, area.y, &context_span, context_width);
        }
    }
}
