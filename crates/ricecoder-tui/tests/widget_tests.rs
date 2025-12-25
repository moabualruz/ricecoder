//! Widget tests for TUI components
//!
//! Tests for various widget behaviors and state management.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget},
};

#[test]
fn test_paragraph_renders_text() {
    let area = Rect::new(0, 0, 20, 3);
    let mut buffer = Buffer::empty(area);
    
    let paragraph = Paragraph::new("Hello, World!");
    paragraph.render(area, &mut buffer);
    
    // Check that text is rendered
    let content = buffer.content.iter()
        .map(|c| c.symbol())
        .collect::<String>();
    assert!(content.contains("Hello"));
    assert!(content.contains("World"));
}

#[test]
fn test_paragraph_with_block() {
    let area = Rect::new(0, 0, 20, 5);
    let mut buffer = Buffer::empty(area);
    
    let paragraph = Paragraph::new("Content")
        .block(Block::default().borders(Borders::ALL).title("Title"));
    paragraph.render(area, &mut buffer);
    
    // Check that title is rendered
    let content = buffer.content.iter()
        .map(|c| c.symbol())
        .collect::<String>();
    assert!(content.contains("Title"));
    assert!(content.contains("Content"));
}

#[test]
fn test_list_with_items() {
    let area = Rect::new(0, 0, 20, 5);
    let mut buffer = Buffer::empty(area);
    
    let items = vec![
        ListItem::new("Item 1"),
        ListItem::new("Item 2"),
        ListItem::new("Item 3"),
    ];
    let list = List::new(items);
    list.render(area, &mut buffer);
    
    let content = buffer.content.iter()
        .map(|c| c.symbol())
        .collect::<String>();
    assert!(content.contains("Item 1"));
    assert!(content.contains("Item 2"));
    assert!(content.contains("Item 3"));
}

#[test]
fn test_list_state_selection() {
    let mut state = ListState::default();
    state.select(Some(1));
    
    assert_eq!(state.selected(), Some(1));
    
    state.select(Some(0));
    assert_eq!(state.selected(), Some(0));
    
    state.select(None);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_block_with_all_borders() {
    let area = Rect::new(0, 0, 10, 5);
    let mut buffer = Buffer::empty(area);
    
    let block = Block::default().borders(Borders::ALL);
    block.render(area, &mut buffer);
    
    // Check corners are rendered (they use special characters)
    assert_ne!(buffer.get(0, 0).symbol(), " ");
    assert_ne!(buffer.get(9, 0).symbol(), " ");
    assert_ne!(buffer.get(0, 4).symbol(), " ");
    assert_ne!(buffer.get(9, 4).symbol(), " ");
}

#[test]
fn test_styled_text() {
    let span = Span::styled("Bold Text", Style::default().fg(Color::Red));
    assert_eq!(span.content, "Bold Text");
    assert_eq!(span.style.fg, Some(Color::Red));
}

#[test]
fn test_line_from_spans() {
    let line = Line::from(vec![
        Span::raw("Normal "),
        Span::styled("Styled", Style::default().fg(Color::Blue)),
    ]);
    
    assert_eq!(line.spans.len(), 2);
    assert_eq!(line.spans[0].content, "Normal ");
    assert_eq!(line.spans[1].content, "Styled");
}

#[test]
fn test_buffer_empty() {
    let area = Rect::new(0, 0, 10, 5);
    let buffer = Buffer::empty(area);
    
    assert_eq!(buffer.area, area);
    // All cells should be empty spaces
    for cell in buffer.content.iter() {
        assert_eq!(cell.symbol(), " ");
    }
}

#[test]
fn test_buffer_set_string() {
    let area = Rect::new(0, 0, 20, 3);
    let mut buffer = Buffer::empty(area);
    
    buffer.set_string(0, 0, "Test", Style::default());
    
    assert_eq!(buffer.get(0, 0).symbol(), "T");
    assert_eq!(buffer.get(1, 0).symbol(), "e");
    assert_eq!(buffer.get(2, 0).symbol(), "s");
    assert_eq!(buffer.get(3, 0).symbol(), "t");
}

#[test]
fn test_rect_intersection() {
    let r1 = Rect::new(0, 0, 10, 10);
    let r2 = Rect::new(5, 5, 10, 10);
    
    let intersection = r1.intersection(r2);
    assert_eq!(intersection.x, 5);
    assert_eq!(intersection.y, 5);
    assert_eq!(intersection.width, 5);
    assert_eq!(intersection.height, 5);
}

#[test]
fn test_rect_no_intersection() {
    let r1 = Rect::new(0, 0, 5, 5);
    let r2 = Rect::new(10, 10, 5, 5);
    
    let intersection = r1.intersection(r2);
    assert_eq!(intersection.width, 0);
    assert_eq!(intersection.height, 0);
}

#[test]
fn test_rect_union() {
    let r1 = Rect::new(0, 0, 5, 5);
    let r2 = Rect::new(3, 3, 5, 5);
    
    let union = r1.union(r2);
    assert_eq!(union.x, 0);
    assert_eq!(union.y, 0);
    assert_eq!(union.width, 8);
    assert_eq!(union.height, 8);
}

#[test]
fn test_rect_contains_point() {
    let rect = Rect::new(5, 5, 10, 10);
    
    // Point inside
    assert!(rect.x <= 7 && 7 < rect.x + rect.width);
    assert!(rect.y <= 7 && 7 < rect.y + rect.height);
    
    // Point outside
    assert!(!(rect.x <= 0 && 0 < rect.x + rect.width));
}

#[test]
fn test_style_combinations() {
    let base = Style::default().fg(Color::White);
    let overlay = Style::default().bg(Color::Black);
    
    let combined = base.patch(overlay);
    assert_eq!(combined.fg, Some(Color::White));
    assert_eq!(combined.bg, Some(Color::Black));
}

#[test]
fn test_list_item_height() {
    let single_line = ListItem::new("Single line");
    assert_eq!(single_line.height(), 1);
    
    let multi_line = ListItem::new("Line 1\nLine 2\nLine 3");
    assert_eq!(multi_line.height(), 3);
}
