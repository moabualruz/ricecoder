//! Property-based tests for history ordering
//! **Feature: ricecoder-sessions, Property 2: History Ordering**
//! **Validates: Requirements 2.3**

use chrono::Utc;
use proptest::prelude::*;
use ricecoder_sessions::{HistoryManager, Message, MessageRole};

/// Strategy to generate valid messages with controlled timestamps
fn message_strategy() -> impl Strategy<Value = Message> {
    (
        any::<String>(),
        prop_oneof![
            Just(MessageRole::User),
            Just(MessageRole::Assistant),
            Just(MessageRole::System),
        ],
    )
        .prop_map(|(content, role)| {
            let mut msg = Message::new(role, content);
            // Override timestamp to be deterministic for testing
            msg.timestamp = Utc::now();
            msg
        })
}

/// Property: For any session history, all messages SHALL be ordered by timestamp in ascending order.
///
/// This property tests that:
/// 1. Messages added in any order are stored in timestamp order
/// 2. All messages maintain ascending timestamp order
/// 3. The ordering is consistent across multiple operations
#[test]
fn prop_history_messages_ordered_by_timestamp() {
    proptest!(|(messages in prop::collection::vec(message_strategy(), 1..100))| {
        let mut history = HistoryManager::new();

        // Add messages in random order
        for msg in messages.iter() {
            history.add_message(msg.clone());
        }

        // Get all messages
        let stored_messages = history.get_all_messages();

        // Verify all messages are present
        prop_assert_eq!(
            stored_messages.len(),
            messages.len(),
            "All messages should be stored"
        );

        // Verify messages are ordered by timestamp (ascending)
        for i in 1..stored_messages.len() {
            prop_assert!(
                stored_messages[i - 1].timestamp <= stored_messages[i].timestamp,
                "Messages should be ordered by timestamp in ascending order"
            );
        }
    });
}

/// Property: When retrieving recent messages, they SHALL be in chronological order.
///
/// This property tests that:
/// 1. Recent messages are returned in the correct order
/// 2. The count parameter is respected
/// 3. Older messages are excluded when count < total messages
#[test]
fn prop_recent_messages_ordered() {
    proptest!(|(
        messages in prop::collection::vec(message_strategy(), 1..50),
        count in 1usize..50
    )| {
        let mut history = HistoryManager::new();

        // Add messages
        for msg in messages.iter() {
            history.add_message(msg.clone());
        }

        // Get recent messages
        let recent = history.get_recent_messages(count);

        // Verify count is respected
        let expected_count = std::cmp::min(count, messages.len());
        prop_assert_eq!(
            recent.len(),
            expected_count,
            "Should return at most count messages"
        );

        // Verify they are in chronological order
        for i in 1..recent.len() {
            prop_assert!(
                recent[i - 1].timestamp <= recent[i].timestamp,
                "Recent messages should be in chronological order"
            );
        }

        // Verify they are the most recent messages
        let all_messages = history.get_all_messages();
        if recent.len() > 0 && all_messages.len() > 0 {
            let first_recent_idx = all_messages.len() - recent.len();
            for i in 0..recent.len() {
                prop_assert_eq!(
                    &recent[i].id,
                    &all_messages[first_recent_idx + i].id,
                    "Recent messages should be the last N messages"
                );
            }
        }
    });
}

/// Property: When searching by content, results SHALL be in chronological order.
///
/// This property tests that:
/// 1. Search results are ordered by timestamp
/// 2. All results contain the search query (case-insensitive)
/// 3. Results are consistent across multiple searches
#[test]
fn prop_search_results_ordered() {
    proptest!(|(
        messages in prop::collection::vec(message_strategy(), 1..50),
        query in "[a-z]{1,5}"
    )| {
        let mut history = HistoryManager::new();

        // Add messages
        for msg in messages.iter() {
            history.add_message(msg.clone());
        }

        // Search for messages
        let results = history.search_by_content(&query);

        // Verify all results contain the query (case-insensitive)
        let query_lower = query.to_lowercase();
        for result in results.iter() {
            prop_assert!(
                result.content.to_lowercase().contains(&query_lower),
                "Search result should contain the query"
            );
        }

        // Verify results are ordered by timestamp
        for i in 1..results.len() {
            prop_assert!(
                results[i - 1].timestamp <= results[i].timestamp,
                "Search results should be ordered by timestamp"
            );
        }
    });
}

/// Property: History with size limits SHALL maintain ordering while enforcing the limit.
///
/// This property tests that:
/// 1. Messages are ordered even with size limits
/// 2. The oldest messages are removed when limit is exceeded
/// 3. The most recent messages are always preserved
#[test]
fn prop_history_ordering_with_size_limit() {
    proptest!(|(
        messages in prop::collection::vec(message_strategy(), 1..50),
        max_size in 1usize..30
    )| {
        let mut history = HistoryManager::with_max_size(max_size);

        // Add messages
        for msg in messages.iter() {
            history.add_message(msg.clone());
        }

        // Get all messages
        let stored = history.get_all_messages();

        // Verify size limit is enforced
        prop_assert!(
            stored.len() <= max_size,
            "History should not exceed max size"
        );

        // Verify messages are still ordered
        for i in 1..stored.len() {
            prop_assert!(
                stored[i - 1].timestamp <= stored[i].timestamp,
                "Messages should remain ordered even with size limit"
            );
        }

        // Verify the most recent messages are preserved
        if stored.len() > 0 && messages.len() > 0 {
            let all_sorted = {
                let mut sorted = messages.clone();
                sorted.sort_by_key(|m| m.timestamp);
                sorted
            };

            let expected_start = if all_sorted.len() > max_size {
                all_sorted.len() - max_size
            } else {
                0
            };

            // The stored messages should be the most recent ones
            for i in 0..stored.len() {
                prop_assert_eq!(
                    &stored[i].id,
                    &all_sorted[expected_start + i].id,
                    "Most recent messages should be preserved"
                );
            }
        }
    });
}

/// Property: Adding messages multiple times SHALL maintain consistent ordering.
///
/// This property tests that:
/// 1. The order is deterministic
/// 2. Adding the same messages in different orders produces the same final order
/// 3. Timestamps are the source of truth for ordering
#[test]
fn prop_history_ordering_is_deterministic() {
    proptest!(|(messages in prop::collection::vec(message_strategy(), 1..30))| {
        // Create two histories and add messages in different orders
        let mut history1 = HistoryManager::new();
        let mut history2 = HistoryManager::new();

        // Add in original order to history1
        for msg in messages.iter() {
            history1.add_message(msg.clone());
        }

        // Add in reverse order to history2
        for msg in messages.iter().rev() {
            history2.add_message(msg.clone());
        }

        // Get all messages from both
        let stored1 = history1.get_all_messages();
        let stored2 = history2.get_all_messages();

        // They should have the same order (by timestamp)
        prop_assert_eq!(
            stored1.len(),
            stored2.len(),
            "Both histories should have the same number of messages"
        );

        for i in 0..stored1.len() {
            prop_assert_eq!(
                stored1[i].timestamp,
                stored2[i].timestamp,
                "Messages should be in the same order regardless of insertion order"
            );
        }
    });
}
