use ricecoder_tui::widgets::{MessageAction, StreamingMessage};
use ricecoder_tui::{ChatWidget, Message, MessageAuthor};

#[test]
fn test_message_creation() {
    let user_msg = Message::user("Hello");
    assert_eq!(user_msg.content, "Hello");
    assert_eq!(user_msg.author, MessageAuthor::User);
    assert!(!user_msg.streaming);

    let ai_msg = Message::assistant("Hi there");
    assert_eq!(ai_msg.content, "Hi there");
    assert_eq!(ai_msg.author, MessageAuthor::Assistant);
}

#[test]
fn test_chat_widget_creation() {
    let widget = ChatWidget::new();
    assert!(widget.messages.is_empty());
    assert!(widget.input.is_empty());
    assert_eq!(widget.scroll, 0);
    assert!(widget.selected.is_none());
}

#[test]
fn test_add_message() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.add_message(Message::assistant("Hi"));

    assert_eq!(widget.messages.len(), 2);
    assert_eq!(widget.messages[0].author, MessageAuthor::User);
    assert_eq!(widget.messages[1].author, MessageAuthor::Assistant);
}

#[test]
fn test_clear() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.input = "test".to_string();
    widget.scroll = 5;

    widget.clear();
    assert!(widget.messages.is_empty());
    assert!(widget.input.is_empty());
    assert_eq!(widget.scroll, 0);
}

#[test]
fn test_scroll() {
    let mut widget = ChatWidget::new();
    for i in 0..10 {
        widget.add_message(Message::user(format!("Message {}", i)));
    }

    widget.scroll_down(5);
    assert_eq!(widget.scroll, 1);

    widget.scroll_up();
    assert_eq!(widget.scroll, 0);

    widget.scroll_up();
    assert_eq!(widget.scroll, 0);
}

#[test]
fn test_visible_messages() {
    let mut widget = ChatWidget::new();
    for i in 0..10 {
        widget.add_message(Message::user(format!("Message {}", i)));
    }

    let visible = widget.visible_messages(5);
    assert_eq!(visible.len(), 5);

    widget.scroll = 5;
    let visible = widget.visible_messages(5);
    assert_eq!(visible.len(), 5);
}

#[test]
fn test_selection() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Message 1"));
    widget.add_message(Message::user("Message 2"));
    widget.add_message(Message::user("Message 3"));

    assert!(widget.selected_message().is_none());

    widget.select_next();
    assert_eq!(widget.selected, Some(0));

    widget.select_next();
    assert_eq!(widget.selected, Some(1));

    widget.select_prev();
    assert_eq!(widget.selected, Some(0));

    widget.select_prev();
    assert!(widget.selected.is_none());
}

#[test]
fn test_streaming_message_creation() {
    let msg = StreamingMessage::new();
    assert!(msg.active);
    assert!(msg.content.is_empty());
    assert_eq!(msg.cursor_pos, 0);
}

#[test]
fn test_streaming_message_append() {
    let mut msg = StreamingMessage::new();
    msg.append("Hello");
    assert_eq!(msg.content, "Hello");
    assert_eq!(msg.cursor_pos, 5);

    msg.append(" world");
    assert_eq!(msg.content, "Hello world");
    assert_eq!(msg.cursor_pos, 11);
}

#[test]
fn test_streaming_message_display() {
    let mut msg = StreamingMessage::new();
    msg.append("Hello");
    assert_eq!(msg.display_text(), "Hello_");

    msg.finish();
    assert_eq!(msg.display_text(), "Hello");
    assert!(!msg.active);
}

#[test]
fn test_message_actions() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.add_message(Message::assistant("Hi there"));

    widget.select_next();
    widget.update_actions();

    assert!(!widget.available_actions.is_empty());
    assert!(widget.available_actions.contains(&MessageAction::Copy));
    assert!(widget.available_actions.contains(&MessageAction::Edit));
}

#[test]
fn test_execute_copy_action() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.select_next();

    let result = widget.execute_action(MessageAction::Copy);
    // Result depends on clipboard availability
    let _ = result;
}

#[test]
fn test_execute_delete_action() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Message 1"));
    widget.add_message(Message::user("Message 2"));

    widget.select_next();
    assert_eq!(widget.messages.len(), 2);

    let result = widget.execute_action(MessageAction::Delete);
    assert!(result.is_ok());
    assert_eq!(widget.messages.len(), 1);
}

#[test]
fn test_execute_edit_action() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Original message"));
    widget.select_next();

    let result = widget.execute_action(MessageAction::Edit);
    assert!(result.is_ok());
    assert_eq!(widget.input, "Original message");
}

#[test]
fn test_streaming_message_animation() {
    let mut msg = StreamingMessage::new();
    msg.append("Hello");

    // Test animation frames
    assert_eq!(msg.animation_frame, 0);
    msg.update_animation();
    assert_eq!(msg.animation_frame, 1);
    msg.update_animation();
    assert_eq!(msg.animation_frame, 2);
    msg.update_animation();
    assert_eq!(msg.animation_frame, 3);
    msg.update_animation();
    assert_eq!(msg.animation_frame, 0); // Wraps around
}

#[test]
fn test_streaming_message_cursor_animation() {
    let mut msg = StreamingMessage::new();
    msg.append("Test");

    // Frame 0-1: show cursor
    msg.animation_frame = 0;
    assert_eq!(msg.display_text(), "Test_");

    msg.animation_frame = 1;
    assert_eq!(msg.display_text(), "Test_");

    // Frame 2-3: hide cursor
    msg.animation_frame = 2;
    assert_eq!(msg.display_text(), "Test ");

    msg.animation_frame = 3;
    assert_eq!(msg.display_text(), "Test ");
}

#[test]
fn test_streaming_message_token_count() {
    let mut msg = StreamingMessage::new();
    assert_eq!(msg.token_count, 0);

    msg.append("Hello");
    assert_eq!(msg.token_count, 1);

    msg.append(" ");
    assert_eq!(msg.token_count, 2);

    msg.append("world");
    assert_eq!(msg.token_count, 3);
}

#[test]
fn test_streaming_message_custom_cursor() {
    let msg = StreamingMessage::new();
    let display = msg.display_text_with_cursor("▌");
    assert_eq!(display, "▌");

    let mut msg = StreamingMessage::new();
    msg.append("Loading");
    let display = msg.display_text_with_cursor("▌");
    assert_eq!(display, "Loading▌");

    msg.finish();
    let display = msg.display_text_with_cursor("▌");
    assert_eq!(display, "Loading");
}

#[test]
fn test_chat_widget_start_streaming() {
    let mut widget = ChatWidget::new();
    assert!(!widget.is_streaming);
    assert!(widget.streaming_message.is_none());

    widget.start_streaming();
    assert!(widget.is_streaming);
    assert!(widget.streaming_message.is_some());
}

#[test]
fn test_chat_widget_append_token() {
    let mut widget = ChatWidget::new();
    widget.start_streaming();

    widget.append_token("Hello");
    widget.append_token(" ");
    widget.append_token("world");

    let display = widget.get_streaming_display();
    assert!(display.is_some());
    assert!(display.unwrap().contains("Hello world"));
}

#[test]
fn test_chat_widget_finish_streaming() {
    let mut widget = ChatWidget::new();
    widget.start_streaming();
    widget.append_token("Test message");

    let message = widget.finish_streaming();
    assert!(message.is_some());
    assert!(!widget.is_streaming);
    assert!(widget.streaming_message.is_none());
    assert_eq!(widget.messages.len(), 1);
    assert_eq!(widget.messages[0].content, "Test message");
    assert_eq!(widget.messages[0].author, MessageAuthor::Assistant);
}

#[test]
fn test_chat_widget_cancel_streaming() {
    let mut widget = ChatWidget::new();
    widget.start_streaming();
    widget.append_token("Partial message");

    widget.cancel_streaming();
    assert!(!widget.is_streaming);
    assert!(widget.streaming_message.is_none());
    assert!(widget.messages.is_empty());
}

#[test]
fn test_chat_widget_update_streaming_animation() {
    let mut widget = ChatWidget::new();
    widget.start_streaming();
    widget.append_token("Animating");

    let initial_frame = widget.streaming_message.as_ref().unwrap().animation_frame;
    widget.update_streaming_animation();
    let new_frame = widget.streaming_message.as_ref().unwrap().animation_frame;

    assert_ne!(initial_frame, new_frame);
}

#[test]
fn test_chat_widget_clear_with_streaming() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.start_streaming();
    widget.append_token("Response");

    widget.clear();
    assert!(widget.messages.is_empty());
    assert!(!widget.is_streaming);
    assert!(widget.streaming_message.is_none());
}

#[test]
fn test_streaming_message_len_and_empty() {
    let mut msg = StreamingMessage::new();
    assert!(msg.is_empty());
    assert_eq!(msg.len(), 0);

    msg.append("Hello");
    assert!(!msg.is_empty());
    assert_eq!(msg.len(), 5);

    msg.append(" world");
    assert_eq!(msg.len(), 11);
}

#[test]
fn test_streaming_message_is_complete() {
    let mut msg = StreamingMessage::new();
    assert!(!msg.is_complete());

    msg.finish();
    assert!(msg.is_complete());
}

#[test]
fn test_message_extract_code_blocks() {
    let msg = Message::assistant("Here's some code:\n```rust\nfn main() {}\n```\nAnd more text");
    let blocks = msg.extract_code_blocks();
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].contains("fn main()"));
}

#[test]
fn test_message_extract_multiple_code_blocks() {
    let msg = Message::assistant(
        "First:\n```rust\nfn foo() {}\n```\nSecond:\n```python\ndef bar(): pass\n```",
    );
    let blocks = msg.extract_code_blocks();
    assert_eq!(blocks.len(), 2);
}

#[test]
fn test_message_get_first_code_block() {
    let msg = Message::assistant("Code:\n```rust\nfn main() {}\n```");
    let block = msg.get_first_code_block();
    assert!(block.is_some());
    assert!(block.unwrap().contains("fn main()"));
}

#[test]
fn test_message_has_code_blocks() {
    let msg_with_code = Message::assistant("```rust\ncode\n```");
    assert!(msg_with_code.has_code_blocks());

    let msg_without_code = Message::assistant("Just text");
    assert!(!msg_without_code.has_code_blocks());
}

#[test]
fn test_chat_widget_action_menu_toggle() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.select_next();
    widget.update_actions();

    assert!(!widget.show_action_menu);
    widget.toggle_action_menu();
    assert!(widget.show_action_menu);
    assert_eq!(widget.selected_action, Some(0));

    widget.toggle_action_menu();
    assert!(!widget.show_action_menu);
}

#[test]
fn test_chat_widget_action_menu_navigation() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.select_next();
    widget.update_actions();
    widget.toggle_action_menu();

    widget.action_menu_down();
    assert_eq!(widget.selected_action, Some(1));

    widget.action_menu_down();
    assert_eq!(widget.selected_action, Some(2));

    widget.action_menu_up();
    assert_eq!(widget.selected_action, Some(1));

    widget.action_menu_up();
    assert_eq!(widget.selected_action, Some(0));
}

#[test]
fn test_chat_widget_close_action_menu() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.select_next();
    widget.update_actions();
    widget.toggle_action_menu();

    assert!(widget.show_action_menu);
    widget.close_action_menu();
    assert!(!widget.show_action_menu);
    assert!(widget.selected_action.is_none());
}

#[test]
fn test_chat_widget_get_selected_action() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.select_next();
    widget.update_actions();
    widget.toggle_action_menu();

    let action = widget.get_selected_action();
    assert_eq!(action, Some(MessageAction::Copy));
}

#[test]
fn test_chat_widget_execute_copy_action_result() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.select_next();

    let result = widget.execute_action(MessageAction::Copy);
    // Result depends on clipboard availability, but should not panic
    let _ = result;
}

#[test]
fn test_chat_widget_execute_delete_action_result() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Message 1"));
    widget.add_message(Message::user("Message 2"));

    widget.select_next();
    assert_eq!(widget.messages.len(), 2);

    let result = widget.execute_action(MessageAction::Delete);
    assert!(result.is_ok());
    assert_eq!(widget.messages.len(), 1);
}

#[test]
fn test_chat_widget_execute_edit_action_result() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Original message"));
    widget.select_next();

    let result = widget.execute_action(MessageAction::Edit);
    assert!(result.is_ok());
    assert_eq!(widget.input, "Original message");
}

#[test]
fn test_chat_widget_execute_action_no_selection() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));

    let result = widget.execute_action(MessageAction::Copy);
    assert!(result.is_err());
}

#[test]
fn test_chat_widget_execute_action_by_shortcut() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.select_next();
    widget.update_actions();

    let result = widget.execute_action_by_shortcut('c');
    // Result depends on clipboard, but should not panic
    let _ = result;
}

#[test]
fn test_chat_widget_execute_action_by_invalid_shortcut() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.select_next();
    widget.update_actions();

    let result = widget.execute_action_by_shortcut('x');
    assert!(result.is_err());
}

#[test]
fn test_chat_widget_copy_feedback_visibility() {
    let mut widget = ChatWidget::new();
    assert!(!widget.is_copy_feedback_visible());

    widget.add_message(Message::user("Hello"));
    widget.select_next();
    let _ = widget.execute_action(MessageAction::Copy);

    // Feedback should be visible after copy
    assert!(widget.is_copy_feedback_visible());
}

#[test]
fn test_chat_widget_update_copy_feedback() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.select_next();
    let _ = widget.execute_action(MessageAction::Copy);

    assert!(widget.is_copy_feedback_visible());

    // Update feedback multiple times
    for _ in 0..100 {
        widget.update_copy_feedback();
    }

    // Feedback should eventually disappear
    assert!(!widget.is_copy_feedback_visible());
}

#[test]
fn test_chat_widget_execute_copy_code_action() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::assistant("Code:\n```rust\nfn main() {}\n```"));
    widget.select_next();
    widget.update_actions();

    assert!(widget.available_actions.contains(&MessageAction::CopyCode));
    let result = widget.execute_action(MessageAction::CopyCode);
    // Result depends on clipboard, but should not panic
    let _ = result;
}

#[test]
fn test_chat_widget_execute_copy_code_no_code() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::assistant("Just text, no code"));
    widget.select_next();

    let result = widget.execute_action(MessageAction::CopyCode);
    assert!(result.is_err());
}

#[test]
fn test_chat_widget_execute_regenerate_action() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::assistant("Response"));
    widget.select_next();
    widget.update_actions();

    assert!(widget
        .available_actions
        .contains(&MessageAction::Regenerate));
    let result = widget.execute_action(MessageAction::Regenerate);
    assert!(result.is_ok());
}

#[test]
fn test_chat_widget_execute_regenerate_user_message() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Question"));
    widget.select_next();

    let result = widget.execute_action(MessageAction::Regenerate);
    assert!(result.is_err());
}

#[test]
fn test_chat_widget_clear_with_copy_operation() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.select_next();
    let _ = widget.execute_action(MessageAction::Copy);

    assert!(widget.copy_operation.is_some());
    widget.clear();
    assert!(widget.copy_operation.is_none());
}

#[test]
fn test_chat_widget_execute_selected_action() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Message 1"));
    widget.add_message(Message::user("Message 2"));
    widget.select_next();
    widget.update_actions();
    widget.toggle_action_menu();

    // Navigate to delete action
    widget.action_menu_down();
    widget.action_menu_down();
    widget.action_menu_down();

    let result = widget.execute_selected_action();
    assert!(result.is_ok());
    assert_eq!(widget.messages.len(), 1);
}

#[test]
fn test_chat_widget_action_menu_no_selection() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));

    // Try to toggle menu without selecting message
    widget.toggle_action_menu();
    assert!(!widget.show_action_menu);
}

#[test]
fn test_chat_widget_action_menu_no_actions() {
    let mut widget = ChatWidget::new();
    widget.add_message(Message::user("Hello"));
    widget.select_next();
    // Don't call update_actions

    widget.toggle_action_menu();
    assert!(!widget.show_action_menu);
}
