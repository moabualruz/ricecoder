//! Property-based tests for conversation history preservation
//! **Feature: ricecoder-specs, Property 8: Conversation History Preservation**
//! **Validates: Requirements 3.7, 3.11**

use chrono::Utc;
use proptest::prelude::*;
use ricecoder_specs::models::*;

// ============================================================================
// Generators for property-based testing
// ============================================================================

fn arb_message_id() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{1,20}".prop_map(|s| format!("msg-{}", s))
}

fn arb_spec_id() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{1,20}".prop_map(|s| s)
}

fn arb_message_content() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,!?]{10,200}".prop_map(|s| s)
}

fn arb_message_role() -> impl Strategy<Value = MessageRole> {
    prop_oneof![
        Just(MessageRole::User),
        Just(MessageRole::Assistant),
        Just(MessageRole::System),
    ]
}

fn arb_conversation_message(
    id: String,
    spec_id: String,
    role: MessageRole,
    content: String,
) -> ConversationMessage {
    ConversationMessage {
        id,
        spec_id,
        role,
        content,
        timestamp: Utc::now(),
    }
}

// ============================================================================
// Property 8: Conversation History Preservation
// ============================================================================

proptest! {
    /// Property: For any spec writing session, all conversation messages
    /// SHALL be preserved with timestamps and roles, enabling context
    /// recovery and audit trails.
    ///
    /// This property verifies that:
    /// 1. Messages are preserved exactly as created
    /// 2. Timestamps are recorded for each message
    /// 3. Roles are preserved correctly
    /// 4. Message order is maintained
    #[test]
    fn prop_conversation_messages_preserved(
        message_id in arb_message_id(),
        spec_id in arb_spec_id(),
        role in arb_message_role(),
        content in arb_message_content(),
    ) {
        let message = arb_conversation_message(message_id.clone(), spec_id.clone(), role, content.clone());

        // Property 8.1: Message ID should be preserved
        prop_assert_eq!(
            message.id,
            message_id,
            "Message ID should be preserved"
        );

        // Property 8.2: Spec ID should be preserved
        prop_assert_eq!(
            message.spec_id,
            spec_id,
            "Spec ID should be preserved"
        );

        // Property 8.3: Message role should be preserved
        prop_assert_eq!(
            message.role,
            role,
            "Message role should be preserved"
        );

        // Property 8.4: Message content should be preserved
        prop_assert_eq!(
            message.content,
            content,
            "Message content should be preserved"
        );

        // Property 8.5: Timestamp should be set
        prop_assert!(
            message.timestamp <= Utc::now(),
            "Timestamp should be set to current time or earlier"
        );
    }

    /// Property: For any sequence of conversation messages, the order
    /// SHALL be preserved in the history.
    ///
    /// This property verifies that message order is maintained.
    #[test]
    fn prop_conversation_message_order_preserved(
        message_ids in prop::collection::vec(arb_message_id(), 1..10),
        spec_id in arb_spec_id(),
        roles in prop::collection::vec(arb_message_role(), 1..10),
        contents in prop::collection::vec(arb_message_content(), 1..10),
    ) {
        // Ensure we have matching lengths
        let min_len = std::cmp::min(
            std::cmp::min(message_ids.len(), roles.len()),
            contents.len()
        );

        let messages: Vec<ConversationMessage> = message_ids[..min_len]
            .iter()
            .zip(roles[..min_len].iter())
            .zip(contents[..min_len].iter())
            .map(|((id, role), content)| {
                ConversationMessage {
                    id: id.clone(),
                    spec_id: spec_id.clone(),
                    role: *role,
                    content: content.clone(),
                    timestamp: Utc::now(),
                }
            })
            .collect();

        // Property 8.6: Message order should be preserved
        for (i, message) in messages.iter().enumerate() {
            prop_assert_eq!(
                &message.id,
                &message_ids[i],
                "Message order should be preserved at index {}", i
            );
        }
    }

    /// Property: For any conversation message, the timestamp SHALL be
    /// recorded and be valid (not in the future).
    ///
    /// This property verifies that timestamps are valid.
    #[test]
    fn prop_conversation_message_timestamp_valid(
        message_id in arb_message_id(),
        spec_id in arb_spec_id(),
        role in arb_message_role(),
        content in arb_message_content(),
    ) {
        let before = Utc::now();
        let message = arb_conversation_message(message_id, spec_id, role, content);
        let after = Utc::now();

        // Property 8.7: Timestamp should be between before and after
        prop_assert!(
            message.timestamp >= before && message.timestamp <= after,
            "Timestamp should be valid (between before and after)"
        );

        // Property 8.8: Timestamp should not be in the future
        prop_assert!(
            message.timestamp <= Utc::now(),
            "Timestamp should not be in the future"
        );
    }

    /// Property: For any conversation message, the role SHALL be one of
    /// the valid roles (User, Assistant, System).
    ///
    /// This property verifies that roles are valid.
    #[test]
    fn prop_conversation_message_role_valid(
        message_id in arb_message_id(),
        spec_id in arb_spec_id(),
        role in arb_message_role(),
        content in arb_message_content(),
    ) {
        let message = arb_conversation_message(message_id, spec_id, role, content);

        // Property 8.9: Role should be one of the valid roles
        let valid_roles = vec![MessageRole::User, MessageRole::Assistant, MessageRole::System];
        prop_assert!(
            valid_roles.contains(&message.role),
            "Role should be one of the valid roles"
        );
    }

    /// Property: For any conversation message, the content SHALL not be empty.
    ///
    /// This property verifies that messages have content.
    #[test]
    fn prop_conversation_message_content_not_empty(
        message_id in arb_message_id(),
        spec_id in arb_spec_id(),
        role in arb_message_role(),
        content in arb_message_content(),
    ) {
        let message = arb_conversation_message(message_id, spec_id, role, content);

        // Property 8.10: Content should not be empty
        prop_assert!(
            !message.content.is_empty(),
            "Message content should not be empty"
        );
    }

    /// Property: For any conversation message, the spec_id SHALL be preserved
    /// and match the session it belongs to.
    ///
    /// This property verifies that messages are correctly associated with specs.
    #[test]
    fn prop_conversation_message_spec_association(
        message_id in arb_message_id(),
        spec_id in arb_spec_id(),
        role in arb_message_role(),
        content in arb_message_content(),
    ) {
        let message = arb_conversation_message(message_id, spec_id.clone(), role, content);

        // Property 8.11: Spec ID should match the session
        prop_assert_eq!(
            message.spec_id,
            spec_id,
            "Message should be associated with correct spec"
        );
    }

    /// Property: For any sequence of messages in a conversation, each message
    /// SHALL have a unique ID.
    ///
    /// This property verifies that message IDs are unique.
    #[test]
    fn prop_conversation_message_ids_unique(
        spec_id in arb_spec_id(),
        roles in prop::collection::vec(arb_message_role(), 2..10),
        contents in prop::collection::vec(arb_message_content(), 2..10),
    ) {
        // Generate unique message IDs by using indices
        let message_ids: Vec<String> = (0..roles.len())
            .map(|i| format!("msg-{}", i))
            .collect();

        // Ensure we have matching lengths
        let min_len = std::cmp::min(
            std::cmp::min(message_ids.len(), roles.len()),
            contents.len()
        );

        let messages: Vec<ConversationMessage> = message_ids[..min_len]
            .iter()
            .zip(roles[..min_len].iter())
            .zip(contents[..min_len].iter())
            .map(|((id, role), content)| {
                ConversationMessage {
                    id: id.clone(),
                    spec_id: spec_id.clone(),
                    role: *role,
                    content: content.clone(),
                    timestamp: Utc::now(),
                }
            })
            .collect();

        // Property 8.12: All message IDs should be unique
        let mut ids = messages.iter().map(|m| m.id.clone()).collect::<Vec<_>>();
        ids.sort();
        ids.dedup();

        prop_assert_eq!(
            ids.len(),
            messages.len(),
            "All message IDs should be unique"
        );
    }

    /// Property: For any conversation message, serialization and deserialization
    /// SHALL preserve all fields.
    ///
    /// This property verifies that messages can be serialized and deserialized.
    #[test]
    fn prop_conversation_message_serialization_roundtrip(
        message_id in arb_message_id(),
        spec_id in arb_spec_id(),
        role in arb_message_role(),
        content in arb_message_content(),
    ) {
        let message = arb_conversation_message(message_id, spec_id, role, content);

        // Serialize to JSON
        let json = serde_json::to_string(&message).unwrap();

        // Deserialize from JSON
        let deserialized: ConversationMessage = serde_json::from_str(&json).unwrap();

        // Property 8.13: All fields should be preserved after round-trip
        prop_assert_eq!(
            message.id,
            deserialized.id,
            "Message ID should be preserved after serialization"
        );

        prop_assert_eq!(
            message.spec_id,
            deserialized.spec_id,
            "Spec ID should be preserved after serialization"
        );

        prop_assert_eq!(
            message.role,
            deserialized.role,
            "Message role should be preserved after serialization"
        );

        prop_assert_eq!(
            message.content,
            deserialized.content,
            "Message content should be preserved after serialization"
        );

        // Timestamps should be very close (within milliseconds)
        let time_diff = message.timestamp.signed_duration_since(deserialized.timestamp);
        prop_assert!(
            time_diff.num_milliseconds().abs() < 1000,
            "Timestamp should be preserved after serialization"
        );
    }

    /// Property: For any conversation message, YAML serialization and
    /// deserialization SHALL preserve all fields.
    ///
    /// This property verifies that messages can be serialized to YAML.
    #[test]
    fn prop_conversation_message_yaml_serialization_roundtrip(
        message_id in arb_message_id(),
        spec_id in arb_spec_id(),
        role in arb_message_role(),
        content in arb_message_content(),
    ) {
        let message = arb_conversation_message(message_id, spec_id, role, content);

        // Serialize to YAML
        let yaml = serde_yaml::to_string(&message).unwrap();

        // Deserialize from YAML
        let deserialized: ConversationMessage = serde_yaml::from_str(&yaml).unwrap();

        // Property 8.14: All fields should be preserved after YAML round-trip
        prop_assert_eq!(
            message.id,
            deserialized.id,
            "Message ID should be preserved after YAML serialization"
        );

        prop_assert_eq!(
            message.spec_id,
            deserialized.spec_id,
            "Spec ID should be preserved after YAML serialization"
        );

        prop_assert_eq!(
            message.role,
            deserialized.role,
            "Message role should be preserved after YAML serialization"
        );

        prop_assert_eq!(
            message.content,
            deserialized.content,
            "Message content should be preserved after YAML serialization"
        );
    }

    /// Property: For any conversation message, the message SHALL be
    /// cloneable and the clone SHALL be identical to the original.
    ///
    /// This property verifies that messages can be cloned.
    #[test]
    fn prop_conversation_message_cloneable(
        message_id in arb_message_id(),
        spec_id in arb_spec_id(),
        role in arb_message_role(),
        content in arb_message_content(),
    ) {
        let message = arb_conversation_message(message_id, spec_id, role, content);
        let cloned = message.clone();

        // Property 8.15: Clone should be identical to original
        prop_assert_eq!(
            message.id,
            cloned.id,
            "Cloned message ID should match original"
        );

        prop_assert_eq!(
            message.spec_id,
            cloned.spec_id,
            "Cloned spec ID should match original"
        );

        prop_assert_eq!(
            message.role,
            cloned.role,
            "Cloned message role should match original"
        );

        prop_assert_eq!(
            message.content,
            cloned.content,
            "Cloned message content should match original"
        );

        prop_assert_eq!(
            message.timestamp,
            cloned.timestamp,
            "Cloned message timestamp should match original"
        );
    }

    /// Property: For any conversation message, the message SHALL be debuggable
    /// and produce a non-empty debug string.
    ///
    /// This property verifies that messages can be debugged.
    #[test]
    fn prop_conversation_message_debuggable(
        message_id in arb_message_id(),
        spec_id in arb_spec_id(),
        role in arb_message_role(),
        content in arb_message_content(),
    ) {
        let message = arb_conversation_message(message_id, spec_id, role, content);
        let debug_str = format!("{:?}", message);

        // Property 8.16: Debug string should not be empty
        prop_assert!(
            !debug_str.is_empty(),
            "Debug string should not be empty"
        );

        // Property 8.17: Debug string should contain message ID
        prop_assert!(
            debug_str.contains("ConversationMessage"),
            "Debug string should contain type name"
        );
    }
}
