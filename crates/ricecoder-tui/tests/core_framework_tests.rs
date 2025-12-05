//! Integration tests for core TUI framework

use ricecoder_tui::{App, AppMode, Constraint, Layout, Rect, Theme};

#[test]
fn test_app_creation() {
    let app = App::new().expect("Failed to create app");
    assert_eq!(app.mode, AppMode::Chat);
    assert!(!app.should_exit);
}

#[test]
fn test_app_mode_switching() {
    let mut app = App::new().expect("Failed to create app");
    assert_eq!(app.mode, AppMode::Chat);

    app.mode = AppMode::Command;
    assert_eq!(app.mode, AppMode::Command);

    app.mode = AppMode::Diff;
    assert_eq!(app.mode, AppMode::Diff);

    app.mode = AppMode::Help;
    assert_eq!(app.mode, AppMode::Help);
}

#[test]
fn test_layout_creation() {
    let layout = Layout::new(80, 24);
    assert_eq!(layout.width, 80);
    assert_eq!(layout.height, 24);
    assert!(layout.is_valid());
}

#[test]
fn test_layout_invalid_size() {
    let layout = Layout::new(79, 24);
    assert!(!layout.is_valid());

    let layout = Layout::new(80, 23);
    assert!(!layout.is_valid());
}

#[test]
fn test_layout_content_and_input_areas() {
    let layout = Layout::new(80, 24);
    let content = layout.content_area();
    let input = layout.input_area();

    assert_eq!(content.width, 80);
    assert_eq!(content.height, 21);
    assert_eq!(input.width, 80);
    assert_eq!(input.height, 3);
    assert_eq!(input.y, 21);
}

#[test]
fn test_rect_operations() {
    let rect = Rect::new(10, 5, 20, 15);
    assert_eq!(rect.right(), 30);
    assert_eq!(rect.bottom(), 20);
    assert!(!rect.is_empty());

    let empty_rect = Rect::new(0, 0, 0, 0);
    assert!(empty_rect.is_empty());
}

#[test]
fn test_theme_creation() {
    let dark = Theme::default();
    assert_eq!(dark.name, "dark");

    let light = Theme::light();
    assert_eq!(light.name, "light");

    let monokai = Theme::monokai();
    assert_eq!(monokai.name, "monokai");

    let dracula = Theme::dracula();
    assert_eq!(dracula.name, "dracula");

    let nord = Theme::nord();
    assert_eq!(nord.name, "nord");
}

#[test]
fn test_layout_split_vertical() {
    let layout = Layout::new(80, 24);
    let rect = Rect::new(0, 0, 80, 20);
    let constraints = vec![Constraint::percentage(50), Constraint::percentage(50)];
    let rects = layout.split_vertical(rect, &constraints);

    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].y, 0);
    assert_eq!(rects[0].height, 10);
    assert_eq!(rects[1].y, 10);
    assert_eq!(rects[1].height, 10);
}

#[test]
fn test_layout_split_horizontal() {
    let layout = Layout::new(80, 24);
    let rect = Rect::new(0, 0, 80, 20);
    let constraints = vec![Constraint::percentage(25), Constraint::percentage(75)];
    let rects = layout.split_horizontal(rect, &constraints);

    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].x, 0);
    assert_eq!(rects[0].width, 20);
    assert_eq!(rects[1].x, 20);
    assert_eq!(rects[1].width, 60);
}

#[test]
fn test_chat_state() {
    let app = App::new().expect("Failed to create app");
    assert!(app.chat.messages.is_empty());
    assert!(app.chat.input.is_empty());
    assert!(!app.chat.streaming);
}

#[test]
fn test_app_config() {
    let app = App::new().expect("Failed to create app");
    assert_eq!(app.config.theme, "dark");
    assert!(app.config.animations);
    assert!(app.config.mouse);
}
