//! Layout tests for TUI components
//!
//! Tests for layout constraints and area calculations.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[test]
fn test_horizontal_split_equal() {
    let area = Rect::new(0, 0, 100, 50);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].width, 50);
    assert_eq!(chunks[1].width, 50);
    assert_eq!(chunks[0].height, 50);
    assert_eq!(chunks[1].height, 50);
}

#[test]
fn test_vertical_split_equal() {
    let area = Rect::new(0, 0, 100, 50);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].height, 25);
    assert_eq!(chunks[1].height, 25);
    assert_eq!(chunks[0].width, 100);
    assert_eq!(chunks[1].width, 100);
}

#[test]
fn test_three_column_layout() {
    let area = Rect::new(0, 0, 120, 40);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),  // Left sidebar
            Constraint::Percentage(50),  // Main content
            Constraint::Percentage(25),  // Right sidebar
        ])
        .split(area);
    
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].width, 30);
    assert_eq!(chunks[1].width, 60);
    assert_eq!(chunks[2].width, 30);
}

#[test]
fn test_fixed_and_flexible_layout() {
    let area = Rect::new(0, 0, 100, 50);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),    // Fixed header
            Constraint::Min(10),      // Flexible content
            Constraint::Length(1),    // Fixed footer
        ])
        .split(area);
    
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].height, 3);
    assert_eq!(chunks[2].height, 1);
    // Content takes remaining space
    assert_eq!(chunks[1].height, 46);
}

#[test]
fn test_min_constraint() {
    let area = Rect::new(0, 0, 100, 50);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),
            Constraint::Min(20),
        ])
        .split(area);
    
    assert_eq!(chunks.len(), 2);
    // Both get at least 20, remainder distributed
    assert!(chunks[0].width >= 20);
    assert!(chunks[1].width >= 20);
}

#[test]
fn test_max_constraint() {
    let area = Rect::new(0, 0, 100, 50);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Max(30),
            Constraint::Min(10),
        ])
        .split(area);
    
    assert_eq!(chunks.len(), 2);
    // First is capped at 30
    assert!(chunks[0].width <= 30);
}

#[test]
fn test_nested_layout() {
    let area = Rect::new(0, 0, 100, 50);
    
    // First split: horizontal
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);
    
    // Nested split: vertical in second chunk
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(horizontal[1]);
    
    assert_eq!(horizontal[0].width, 30);
    assert_eq!(vertical[0].width, 70);
    assert_eq!(vertical[1].width, 70);
    assert_eq!(vertical[0].height, 25);
    assert_eq!(vertical[1].height, 25);
}

#[test]
fn test_ratio_constraint() {
    let area = Rect::new(0, 0, 100, 50);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(2, 3),
        ])
        .split(area);
    
    assert_eq!(chunks.len(), 2);
    // 1/3 and 2/3 of 100
    assert_eq!(chunks[0].width, 33);
    assert_eq!(chunks[1].width, 67);
}

#[test]
fn test_empty_area_handling() {
    let area = Rect::new(0, 0, 0, 0);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].width, 0);
    assert_eq!(chunks[1].width, 0);
}

#[test]
fn test_single_constraint() {
    let area = Rect::new(0, 0, 100, 50);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(area);
    
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].width, 100);
    assert_eq!(chunks[0].height, 50);
}

#[test]
fn test_offset_area() {
    let area = Rect::new(10, 5, 80, 40);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].x, 10);
    assert_eq!(chunks[0].y, 5);
    assert_eq!(chunks[1].x, 50);
    assert_eq!(chunks[1].y, 5);
}
