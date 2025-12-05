//! Property-based tests for chat widget
//!
//! These tests verify correctness properties that should hold across all inputs.
//! Using proptest for property-based testing with 100+ iterations per property.

use proptest::prelude::*;
use ricecoder_tui::widgets::{ChatWidget, Message, MessageAuthor, StreamingMessage};

/// Strategy for generating valid token strings
fn token_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 \\-_.,!?;:'\"]+".prop_map(|s| s.to_string())
}

/// Strategy for generating token streams (sequences of tokens)
fn token_stream_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(token_strategy(), 1..100)
}

/// Strategy for generating realistic chat messages
fn message_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 \\-_.,!?;:'\"]+".prop_map(|s| s.to_string())
}

proptest! {
    /// Property 5: Chat Message Streaming Completeness
    ///
    /// For any token stream, all tokens should be displayed in order without loss or duplication.
    /// This validates Requirements 7.3 and 7.4.
    ///
    /// **Feature: ricecoder-tui, Property 5: Chat Message Streaming Completeness**
    /// **Validates: Requirements 7.3, 7.4**
    #[test]
    fn prop_streaming_message_completeness(tokens in token_stream_strategy()) {
        let mut streaming_msg = StreamingMessage::new();

        // Append all tokens
        for token in &tokens {
            streaming_msg.append(token);
        }

        // Verify all tokens are present in order
        let expected_content: String = tokens.join("");
        let expected_len = expected_content.len();
        prop_assert_eq!(&streaming_msg.content, &expected_content);

        // Verify token count matches
        prop_assert_eq!(streaming_msg.token_count, tokens.len());

        // Verify cursor position is at the end
        prop_assert_eq!(streaming_msg.cursor_pos, expected_len);

        // Verify no tokens were lost or duplicated
        prop_assert_eq!(streaming_msg.content.len(), expected_len);
    }

    /// Property: Streaming message display preserves content
    ///
    /// When streaming is active, the display text should contain all accumulated content.
    /// When streaming is finished, the display text should equal the content exactly.
    #[test]
    fn prop_streaming_display_preserves_content(tokens in token_stream_strategy()) {
        let mut streaming_msg = StreamingMessage::new();

        for token in &tokens {
            streaming_msg.append(token);
        }

        // While streaming, display should contain content
        let display_active = streaming_msg.display_text();
        prop_assert!(display_active.contains(&streaming_msg.content));

        // After finishing, display should equal content
        streaming_msg.finish();
        let display_finished = streaming_msg.display_text();
        prop_assert_eq!(display_finished, streaming_msg.content);
    }

    /// Property: Chat widget streaming accumulates all tokens
    ///
    /// When appending tokens to a chat widget's streaming message,
    /// all tokens should be accumulated without loss.
    #[test]
    fn prop_chat_widget_streaming_accumulation(tokens in token_stream_strategy()) {
        let mut widget = ChatWidget::new();
        widget.start_streaming();

        // Append all tokens
        for token in &tokens {
            widget.append_token(token);
        }

        // Verify all tokens are accumulated
        let expected_content: String = tokens.join("");
        let display = widget.get_streaming_display();
        prop_assert!(display.is_some());

        let display_text = display.unwrap();
        prop_assert!(display_text.contains(&expected_content));
    }

    /// Property: Finishing streaming creates message with all content
    ///
    /// When finishing a streaming message, the created message should contain
    /// all accumulated tokens.
    #[test]
    fn prop_finish_streaming_preserves_content(tokens in token_stream_strategy()) {
        let mut widget = ChatWidget::new();
        widget.start_streaming();

        for token in &tokens {
            widget.append_token(token);
        }

        let message = widget.finish_streaming();
        prop_assert!(message.is_some());

        let msg = message.unwrap();
        let expected_content: String = tokens.join("");
        prop_assert_eq!(msg.content, expected_content);
        prop_assert_eq!(msg.author, MessageAuthor::Assistant);
        prop_assert!(!msg.streaming);
    }

    /// Property: Message content extraction is lossless
    ///
    /// When extracting code blocks from a message, all code blocks should be
    /// recovered exactly as they were in the original content.
    #[test]
    fn prop_code_block_extraction_lossless(code_blocks in prop::collection::vec(message_strategy(), 1..5)) {
        // Build a message with code blocks
        let mut content = String::new();
        for (i, block) in code_blocks.iter().enumerate() {
            if i > 0 {
                content.push_str("\n\n");
            }
            content.push_str("```rust\n");
            content.push_str(block);
            content.push_str("\n```");
        }

        let msg = Message::assistant(content);
        let extracted = msg.extract_code_blocks();

        // Verify all code blocks were extracted
        prop_assert_eq!(extracted.len(), code_blocks.len());

        // Verify each extracted block matches the original
        for (i, extracted_block) in extracted.iter().enumerate() {
            prop_assert_eq!(extracted_block.trim(), code_blocks[i].trim());
        }
    }

    /// Property: Message selection and action menu consistency
    ///
    /// When selecting a message and opening the action menu,
    /// the selected action should always be valid and within bounds.
    #[test]
    fn prop_action_menu_consistency(messages in prop::collection::vec(message_strategy(), 1..10)) {
        let mut widget = ChatWidget::new();

        // Add messages
        for (i, msg_content) in messages.iter().enumerate() {
            if i % 2 == 0 {
                widget.add_message(Message::user(msg_content.clone()));
            } else {
                widget.add_message(Message::assistant(msg_content.clone()));
            }
        }

        // Select each message and verify action menu
        for i in 0..widget.messages.len() {
            widget.selected = Some(i);
            widget.update_actions();

            if !widget.available_actions.is_empty() {
                widget.toggle_action_menu();
                prop_assert!(widget.show_action_menu);

                // Verify selected action is valid
                if let Some(action_idx) = widget.selected_action {
                    prop_assert!(action_idx < widget.available_actions.len());
                }

                widget.close_action_menu();
                prop_assert!(!widget.show_action_menu);
            }
        }
    }

    /// Property: Streaming message animation frames cycle correctly
    ///
    /// Animation frames should cycle from 0 to animation_frame-1 and back to 0.
    #[test]
    fn prop_streaming_animation_cycles(iterations in 1usize..1000) {
        let mut msg = StreamingMessage::new();
        msg.append("test");

        // Update animation multiple times
        for _ in 0..iterations {
            msg.update_animation();
        }

        // Frame should be within valid range
        prop_assert!(msg.animation_frame < 4);

        // Verify cycling behavior
        let expected_frame = (iterations % 4) as u32;
        prop_assert_eq!(msg.animation_frame, expected_frame);
    }

    /// Property: Message scrolling maintains bounds
    ///
    /// Scrolling should never go below 0 or above the maximum scroll position.
    #[test]
    fn prop_scroll_bounds_maintained(
        num_messages in 1usize..100,
        scroll_operations in prop::collection::vec(0u8..2, 1..100)
    ) {
        let mut widget = ChatWidget::new();

        // Add messages
        for i in 0..num_messages {
            widget.add_message(Message::user(format!("Message {}", i)));
        }

        let height = 10;

        // Perform scroll operations
        for op in scroll_operations {
            if op == 0 {
                widget.scroll_up();
            } else {
                widget.scroll_down(height);
            }
        }

        // Verify scroll is within bounds
        prop_assert!(widget.scroll <= widget.messages.len());

        // Verify visible messages are within bounds
        let visible = widget.visible_messages(height);
        prop_assert!(visible.len() <= height);
    }

    /// Property: Message selection navigation is consistent
    ///
    /// Selecting next and previous should navigate through messages consistently.
    #[test]
    fn prop_message_selection_navigation(num_messages in 1usize..50) {
        let mut widget = ChatWidget::new();

        for i in 0..num_messages {
            widget.add_message(Message::user(format!("Message {}", i)));
        }

        // Start with no selection
        prop_assert!(widget.selected.is_none());

        // Select next should select first message
        widget.select_next();
        prop_assert_eq!(widget.selected, Some(0));

        // Select next should increment
        for i in 1..num_messages {
            widget.select_next();
            prop_assert_eq!(widget.selected, Some(i));
        }

        // Select next at end should stay at end
        widget.select_next();
        prop_assert_eq!(widget.selected, Some(num_messages - 1));

        // Select previous should decrement
        for i in (1..num_messages).rev() {
            widget.select_prev();
            prop_assert_eq!(widget.selected, Some(i - 1));
        }

        // Select previous at start should deselect
        widget.select_prev();
        prop_assert!(widget.selected.is_none());
    }

    /// Property: Clear operation resets all state
    ///
    /// After clearing, all widget state should be reset to initial values.
    #[test]
    fn prop_clear_resets_state(messages in prop::collection::vec(message_strategy(), 1..10)) {
        let mut widget = ChatWidget::new();

        // Add messages and set state
        for msg_content in messages {
            widget.add_message(Message::user(msg_content));
        }
        widget.input = "some input".to_string();
        widget.scroll = 5;
        widget.selected = Some(2);
        widget.show_action_menu = true;

        // Clear
        widget.clear();

        // Verify all state is reset
        prop_assert!(widget.messages.is_empty());
        prop_assert!(widget.input.is_empty());
        prop_assert_eq!(widget.scroll, 0);
        prop_assert!(widget.selected.is_none());
        prop_assert!(!widget.show_action_menu);
        prop_assert!(widget.streaming_message.is_none());
        prop_assert!(!widget.is_streaming);
        prop_assert!(widget.copy_operation.is_none());
    }

    /// Property: Token count matches number of append operations
    ///
    /// The token count should equal the number of times append was called.
    #[test]
    fn prop_token_count_accuracy(tokens in token_stream_strategy()) {
        let mut msg = StreamingMessage::new();

        for (i, token) in tokens.iter().enumerate() {
            msg.append(token);
            prop_assert_eq!(msg.token_count, i + 1);
        }
    }

    /// Property: Message content length matches accumulated tokens
    ///
    /// The message content length should equal the sum of all token lengths.
    #[test]
    fn prop_content_length_accuracy(tokens in token_stream_strategy()) {
        let mut msg = StreamingMessage::new();

        let expected_length: usize = tokens.iter().map(|t| t.len()).sum();

        for token in &tokens {
            msg.append(token);
        }

        prop_assert_eq!(msg.content.len(), expected_length);
        prop_assert_eq!(msg.len(), expected_length);
    }
}
