//! Property-based tests for message serialization round trip
//!
//! **Feature: ricecoder-sessions, Property 9: Message Part Serialization Round Trip**
//! **Validates: Requirements 12.2, 12.4**
//!
//! For any message with any combination of parts, serializing to JSON and deserializing
//! should produce an equivalent message.

use proptest::prelude::*;
use ricecoder_sessions::{Message, MessageRole, MessagePart, CodePart, ToolStatus};
use serde_json;
use chrono::{Duration, Utc};

// Strategy for generating simple message parts
fn arb_simple_message_part() -> impl Strategy<Value = MessagePart> {
    prop_oneof![
        // Text part
        "[a-zA-Z0-9 .,!?]{1,100}".prop_map(MessagePart::Text),
        // Reasoning part
        "[a-zA-Z0-9 .,!?]{1,100}".prop_map(MessagePart::Reasoning),
        // Code part
        (
            prop_oneof![Just("rust".to_string()), Just("python".to_string()), Just("javascript".to_string())],
            "[a-zA-Z0-9 .,!?]{1,200}".prop_map(|s| s),
            any::<bool>(),
        ).prop_map(|(language, content, line_numbers)| MessagePart::Code(CodePart {
            language,
            content,
            filename: None,
            line_numbers,
        })),
        // Error part
        "[a-zA-Z0-9 .,!?]{1,100}".prop_map(MessagePart::Error),
    ]
}

// Strategy for generating messages with parts
fn arb_message_with_parts() -> impl Strategy<Value = Message> {
    (
        prop_oneof![Just(MessageRole::User), Just(MessageRole::Assistant), Just(MessageRole::System)],
        prop::collection::vec(arb_simple_message_part(), 1..5), // parts
        0i64..86400i64, // seconds from now for timestamp
    )
        .prop_map(|(role, parts, seconds_offset)| {
            let mut message = Message::new_empty(role);
            message.parts = parts;
            message.timestamp = Utc::now() + Duration::seconds(seconds_offset);
            message
        })
}

proptest! {
    /// **Feature: ricecoder-sessions, Property 9: Message Part Serialization Round Trip**
    /// **Validates: Requirements 12.2, 12.4**
    ///
    /// For any message with any combination of parts, serializing to JSON and deserializing
    /// should produce an equivalent message.
    #[test]
    fn prop_message_serialization_round_trip(message in arb_message_with_parts()) {
        // Serialize to JSON
        let json = serde_json::to_string(&message);
        prop_assert!(json.is_ok(), "Failed to serialize message: {:?}", message);

        let json_str = json.unwrap();

        // Deserialize from JSON
        let deserialized: Result<Message, _> = serde_json::from_str(&json_str);
        prop_assert!(deserialized.is_ok(), "Failed to deserialize message from JSON: {}", json_str);

        let deserialized_msg = deserialized.unwrap();

        // Check that key fields are preserved
        prop_assert_eq!(message.id, deserialized_msg.id, "Message ID not preserved");
        prop_assert_eq!(message.role, deserialized_msg.role, "Message role not preserved");
        prop_assert_eq!(message.parts.len(), deserialized_msg.parts.len(), "Number of parts not preserved");

        // Check that timestamp is preserved (allowing for small differences)
        let time_diff = (message.timestamp - deserialized_msg.timestamp).num_milliseconds().abs();
        prop_assert!(time_diff < 1000, "Timestamp not preserved: original={:?}, deserialized={:?}",
            message.timestamp, deserialized_msg.timestamp);
    }

    /// **Feature: ricecoder-sessions, Property 9: Message Part Serialization - Content Preservation**
    /// **Validates: Requirements 12.2, 12.4**
    ///
    /// The full content representation should be preserved through serialization.
    #[test]
    fn prop_message_content_preservation(message in arb_message_with_parts()) {
        let original_content = message.full_content();

        // Serialize and deserialize
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();

        let deserialized_content = deserialized.full_content();

        prop_assert_eq!(original_content, deserialized_content,
            "Full content not preserved through serialization");
    }

    /// **Feature: ricecoder-sessions, Property 9: Message Part Serialization - Backwards Compatibility**
    /// **Validates: Requirements 12.2**
    ///
    /// Simple text messages should still work with the old content() method.
    #[test]
    fn prop_message_backwards_compatibility(role in prop_oneof![Just(MessageRole::User), Just(MessageRole::Assistant)],
                                           content in "[a-zA-Z0-9 .,!?]{1,100}") {
        // Create a message the old way
        let message = Message::new(role, content.clone());

        // Should have one text part
        prop_assert_eq!(message.parts.len(), 1, "Message should have exactly one part");
        prop_assert!(matches!(message.parts[0], MessagePart::Text(_)), "Part should be Text");

        // content() method should return the text
        prop_assert_eq!(message.content(), content.clone(), "content() should return the text");

        // Serialization should work
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(deserialized.content(), content, "Deserialized content() should match");
    }
}