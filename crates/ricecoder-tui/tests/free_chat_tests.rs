//! Tests for free chat mode

use ricecoder_tui::{App, AppMode, ChatInputWidget, InputAnalyzer, Intent};

#[test]
fn test_intent_detection_generate() {
    assert_eq!(
        InputAnalyzer::detect_intent("generate a function"),
        Intent::Generate
    );
    assert_eq!(
        InputAnalyzer::detect_intent("create a class"),
        Intent::Generate
    );
    assert_eq!(
        InputAnalyzer::detect_intent("write a module"),
        Intent::Generate
    );
}

#[test]
fn test_intent_detection_explain() {
    assert_eq!(
        InputAnalyzer::detect_intent("explain this code"),
        Intent::Explain
    );
    assert_eq!(
        InputAnalyzer::detect_intent("what is a closure"),
        Intent::Explain
    );
    assert_eq!(
        InputAnalyzer::detect_intent("how does async work"),
        Intent::Explain
    );
}

#[test]
fn test_intent_detection_fix() {
    assert_eq!(InputAnalyzer::detect_intent("fix this bug"), Intent::Fix);
    assert_eq!(
        InputAnalyzer::detect_intent("there's an error"),
        Intent::Fix
    );
}

#[test]
fn test_intent_detection_refactor() {
    assert_eq!(
        InputAnalyzer::detect_intent("refactor this code"),
        Intent::Refactor
    );
    assert_eq!(
        InputAnalyzer::detect_intent("improve performance"),
        Intent::Refactor
    );
    assert_eq!(
        InputAnalyzer::detect_intent("optimize this"),
        Intent::Refactor
    );
}

#[test]
fn test_intent_detection_test() {
    assert_eq!(InputAnalyzer::detect_intent("unit test this"), Intent::Test);
    assert_eq!(InputAnalyzer::detect_intent("test this code"), Intent::Test);
}

#[test]
fn test_intent_detection_document() {
    assert_eq!(
        InputAnalyzer::detect_intent("document this"),
        Intent::Document
    );
    assert_eq!(
        InputAnalyzer::detect_intent("add comments"),
        Intent::Document
    );
}

#[test]
fn test_intent_detection_execute() {
    assert_eq!(
        InputAnalyzer::detect_intent("execute this"),
        Intent::Execute
    );
    assert_eq!(
        InputAnalyzer::detect_intent("run the command"),
        Intent::Execute
    );
}

#[test]
fn test_intent_detection_help() {
    assert_eq!(InputAnalyzer::detect_intent("help me"), Intent::Help);
    assert_eq!(InputAnalyzer::detect_intent("?"), Intent::Help);
}

#[test]
fn test_intent_detection_chat() {
    assert_eq!(InputAnalyzer::detect_intent("hello"), Intent::Chat);
    assert_eq!(InputAnalyzer::detect_intent("how are you"), Intent::Chat);
}

#[test]
fn test_chat_input_widget_basic() {
    let widget = ChatInputWidget::new();
    assert!(widget.text.is_empty());
    assert_eq!(widget.cursor, 0);
    assert!(widget.history.is_empty());
}

#[test]
fn test_chat_input_widget_typing() {
    let mut widget = ChatInputWidget::new();
    widget.insert_char('h');
    widget.insert_char('e');
    widget.insert_char('l');
    widget.insert_char('l');
    widget.insert_char('o');

    assert_eq!(widget.text, "hello");
    assert_eq!(widget.cursor, 5);
}

#[test]
fn test_chat_input_widget_backspace() {
    let mut widget = ChatInputWidget::new();
    widget.text = "hello".to_string();
    widget.cursor = 5;

    widget.backspace();
    assert_eq!(widget.text, "hell");
    assert_eq!(widget.cursor, 4);
}

#[test]
fn test_chat_input_widget_delete() {
    let mut widget = ChatInputWidget::new();
    widget.text = "hello".to_string();
    widget.cursor = 0;

    widget.delete();
    assert_eq!(widget.text, "ello");
}

#[test]
fn test_chat_input_widget_cursor_movement() {
    let mut widget = ChatInputWidget::new();
    widget.text = "hello".to_string();
    widget.cursor = 2;

    widget.move_left();
    assert_eq!(widget.cursor, 1);

    widget.move_right();
    assert_eq!(widget.cursor, 2);

    widget.move_start();
    assert_eq!(widget.cursor, 0);

    widget.move_end();
    assert_eq!(widget.cursor, 5);
}

#[test]
fn test_chat_input_widget_submit() {
    let mut widget = ChatInputWidget::new();
    widget.text = "generate a function".to_string();

    let submitted = widget.submit();
    assert_eq!(submitted, "generate a function");
    assert!(widget.text.is_empty());
    assert_eq!(widget.history.len(), 1);
}

#[test]
fn test_chat_input_widget_history() {
    let mut widget = ChatInputWidget::new();

    widget.text = "first".to_string();
    widget.submit();

    widget.text = "second".to_string();
    widget.submit();

    widget.text = "third".to_string();
    widget.submit();

    assert_eq!(widget.history.len(), 3);

    widget.history_up();
    assert_eq!(widget.text, "third");

    widget.history_up();
    assert_eq!(widget.text, "second");

    widget.history_down();
    assert_eq!(widget.text, "third");

    widget.history_down();
    assert!(widget.text.is_empty());
}

#[test]
fn test_app_mode_switching() {
    let mut app = App::new().expect("Failed to create app");
    assert_eq!(app.mode, AppMode::Chat);

    app.switch_mode(AppMode::Command);
    assert_eq!(app.mode, AppMode::Command);

    app.switch_mode(AppMode::Diff);
    assert_eq!(app.mode, AppMode::Diff);

    app.switch_mode(AppMode::Help);
    assert_eq!(app.mode, AppMode::Help);

    app.switch_mode(AppMode::Chat);
    assert_eq!(app.mode, AppMode::Chat);
}

#[test]
fn test_input_suggestions() {
    let mut widget = ChatInputWidget::new();
    widget.text = "generate code".to_string();
    widget.update_intent();

    let suggestions = widget.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions.contains(&"generate"));
}

#[test]
fn test_input_validation() {
    assert!(InputAnalyzer::validate_input("hello").is_ok());
    assert!(InputAnalyzer::validate_input("").is_err());
    assert!(InputAnalyzer::validate_input("   ").is_err());
}

#[test]
fn test_intent_updates_on_input() {
    let mut widget = ChatInputWidget::new();
    assert_eq!(widget.intent, Intent::Chat);

    widget.text = "generate code".to_string();
    widget.update_intent();
    assert_eq!(widget.intent, Intent::Generate);

    widget.text = "explain this".to_string();
    widget.update_intent();
    assert_eq!(widget.intent, Intent::Explain);
}

// Mode switching tests
#[test]
fn test_app_mode_display_names() {
    assert_eq!(AppMode::Chat.display_name(), "Chat");
    assert_eq!(AppMode::Command.display_name(), "Command");
    assert_eq!(AppMode::Diff.display_name(), "Diff");
    assert_eq!(AppMode::Help.display_name(), "Help");
}

#[test]
fn test_app_mode_shortcuts() {
    assert_eq!(AppMode::Chat.shortcut(), "Ctrl+1");
    assert_eq!(AppMode::Command.shortcut(), "Ctrl+2");
    assert_eq!(AppMode::Diff.shortcut(), "Ctrl+3");
    assert_eq!(AppMode::Help.shortcut(), "Ctrl+4");
}

#[test]
fn test_app_mode_next() {
    assert_eq!(AppMode::Chat.next(), AppMode::Command);
    assert_eq!(AppMode::Command.next(), AppMode::Diff);
    assert_eq!(AppMode::Diff.next(), AppMode::Help);
    assert_eq!(AppMode::Help.next(), AppMode::Chat);
}

#[test]
fn test_app_mode_previous() {
    assert_eq!(AppMode::Chat.previous(), AppMode::Help);
    assert_eq!(AppMode::Command.previous(), AppMode::Chat);
    assert_eq!(AppMode::Diff.previous(), AppMode::Command);
    assert_eq!(AppMode::Help.previous(), AppMode::Diff);
}

#[test]
fn test_app_switch_mode() {
    let mut app = App::new().expect("Failed to create app");
    assert_eq!(app.mode, AppMode::Chat);
    assert_eq!(app.previous_mode, AppMode::Chat);

    app.switch_mode(AppMode::Command);
    assert_eq!(app.mode, AppMode::Command);
    assert_eq!(app.previous_mode, AppMode::Chat);

    app.switch_mode(AppMode::Diff);
    assert_eq!(app.mode, AppMode::Diff);
    assert_eq!(app.previous_mode, AppMode::Command);
}

#[test]
fn test_app_next_mode() {
    let mut app = App::new().expect("Failed to create app");
    assert_eq!(app.mode, AppMode::Chat);

    app.next_mode();
    assert_eq!(app.mode, AppMode::Command);

    app.next_mode();
    assert_eq!(app.mode, AppMode::Diff);

    app.next_mode();
    assert_eq!(app.mode, AppMode::Help);

    app.next_mode();
    assert_eq!(app.mode, AppMode::Chat);
}

#[test]
fn test_app_previous_mode_switch() {
    let mut app = App::new().expect("Failed to create app");
    app.switch_mode(AppMode::Help);
    assert_eq!(app.mode, AppMode::Help);

    app.previous_mode_switch();
    assert_eq!(app.mode, AppMode::Diff);

    app.previous_mode_switch();
    assert_eq!(app.mode, AppMode::Command);

    app.previous_mode_switch();
    assert_eq!(app.mode, AppMode::Chat);

    app.previous_mode_switch();
    assert_eq!(app.mode, AppMode::Help);
}

#[test]
fn test_app_toggle_mode() {
    let mut app = App::new().expect("Failed to create app");
    assert_eq!(app.mode, AppMode::Chat);
    assert_eq!(app.previous_mode, AppMode::Chat);

    app.switch_mode(AppMode::Command);
    assert_eq!(app.mode, AppMode::Command);
    assert_eq!(app.previous_mode, AppMode::Chat);

    app.toggle_mode();
    assert_eq!(app.mode, AppMode::Chat);
    assert_eq!(app.previous_mode, AppMode::Command);

    app.toggle_mode();
    assert_eq!(app.mode, AppMode::Command);
    assert_eq!(app.previous_mode, AppMode::Chat);
}

#[test]
fn test_app_current_mode_name() {
    let mut app = App::new().expect("Failed to create app");
    assert_eq!(app.current_mode_name(), "Chat");

    app.switch_mode(AppMode::Command);
    assert_eq!(app.current_mode_name(), "Command");

    app.switch_mode(AppMode::Diff);
    assert_eq!(app.current_mode_name(), "Diff");

    app.switch_mode(AppMode::Help);
    assert_eq!(app.current_mode_name(), "Help");
}

#[test]
fn test_app_current_mode_shortcut() {
    let mut app = App::new().expect("Failed to create app");
    assert_eq!(app.current_mode_shortcut(), "Ctrl+1");

    app.switch_mode(AppMode::Command);
    assert_eq!(app.current_mode_shortcut(), "Ctrl+2");

    app.switch_mode(AppMode::Diff);
    assert_eq!(app.current_mode_shortcut(), "Ctrl+3");

    app.switch_mode(AppMode::Help);
    assert_eq!(app.current_mode_shortcut(), "Ctrl+4");
}

#[test]
fn test_app_keybindings_enabled() {
    let app = App::new().expect("Failed to create app");
    assert!(app.keybindings_enabled);
}

#[test]
fn test_mode_switching_preserves_previous_mode() {
    let mut app = App::new().expect("Failed to create app");

    app.switch_mode(AppMode::Command);
    app.switch_mode(AppMode::Diff);
    app.switch_mode(AppMode::Help);

    assert_eq!(app.mode, AppMode::Help);
    assert_eq!(app.previous_mode, AppMode::Diff);
}

#[test]
fn test_mode_switching_same_mode_no_change() {
    let mut app = App::new().expect("Failed to create app");
    app.switch_mode(AppMode::Command);
    let prev = app.previous_mode;

    app.switch_mode(AppMode::Command);
    assert_eq!(app.mode, AppMode::Command);
    assert_eq!(app.previous_mode, prev);
}
