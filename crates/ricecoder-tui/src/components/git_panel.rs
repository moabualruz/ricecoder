use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

/// Git panel state
#[derive(Debug, Clone, PartialEq)]
pub struct GitPanelState {
    pub staged_files: Vec<String>,
    pub unstaged_files: Vec<String>,
    pub commit_message: String,
    pub selected_section: GitSection,
    pub list_state: ListState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitSection {
    Staged,
    Unstaged,
    CommitMessage,
}

impl Default for GitPanelState {
    fn default() -> Self {
        Self {
            staged_files: Vec::new(),
            unstaged_files: Vec::new(),
            commit_message: String::new(),
            selected_section: GitSection::Unstaged,
            list_state: ListState::default(),
        }
    }
}

pub struct GitPanel;

impl StatefulWidget for GitPanel {
    type State = GitPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default()
            .title("Source Control")
            .borders(Borders::ALL);
        
        let inner_area = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Commit message input
                Constraint::Percentage(50), // Staged changes
                Constraint::Percentage(50), // Changes
            ])
            .split(inner_area);

        // 1. Commit Message Input
        let commit_block = Block::default()
            .title("Message (Ctrl+Enter to commit)")
            .borders(Borders::ALL)
            .style(if state.selected_section == GitSection::CommitMessage {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });
        
        Paragraph::new(state.commit_message.as_str())
            .block(commit_block)
            .render(chunks[0], buf);

        // 2. Staged Changes
        let staged_items: Vec<ListItem> = state.staged_files
            .iter()
            .map(|f| ListItem::new(Line::from(vec![Span::raw(f)])))
            .collect();
        
        let staged_block = Block::default()
            .title("Staged Changes")
            .borders(Borders::ALL)
            .style(if state.selected_section == GitSection::Staged {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        Widget::render(List::new(staged_items)
            .block(staged_block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD)), chunks[1], buf);

        // 3. Unstaged Changes
        let unstaged_items: Vec<ListItem> = state.unstaged_files
            .iter()
            .map(|f| ListItem::new(Line::from(vec![Span::raw(f)])))
            .collect();

        let unstaged_block = Block::default()
            .title("Changes")
            .borders(Borders::ALL)
            .style(if state.selected_section == GitSection::Unstaged {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        Widget::render(List::new(unstaged_items)
            .block(unstaged_block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD)), chunks[2], buf);
    }
}
