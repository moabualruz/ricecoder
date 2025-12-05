//! Unit tests for HistoryManager operations
//! **Feature: ricecoder-sessions, Unit Tests: HistoryManager**
//! **Validates: Requirements 2.3**

use ricecoder_sessions::{HistoryManager, Message, MessageRole};
use std::thread;
use std::time::Duration;

fn create_test_message(role: MessageRole, content: &str) -> Message {
    Message::new(role, content.to_string())
}

#[test]
fn test_history_manager_new() {
    let manager = HistoryManager::new();
    assert_eq!(manager.message_count(), 0);
    assert_eq!(manager.max_size(), None);
}

#[test]
fn test_history_manager_with_max_size() {
    let manager = HistoryManager::with_max_size(10);
    assert_eq!(manager.message_count(), 0);
    assert_eq!(manager.max_size(), Some(10));
}

#[test]
fn test_add_message() {
    let mut manager = HistoryManager::new();
    let message = create_test_message(MessageRole::User, "Hello");

    manager.add_message(message.clone());

    assert_eq!(manager.message_count(), 1);
    let messages = manager.get_all_messages();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Hello");
}

#[test]
fn test_add_multiple_messages() {
    let mut manager = HistoryManager::new();

    manager.add_message(create_test_message(MessageRole::User, "Message 1"));
    manager.add_message(create_test_message(MessageRole::Assistant, "Response 1"));
    manager.add_message(create_test_message(MessageRole::User, "Message 2"));

    assert_eq!(manager.message_count(), 3);
}

#[test]
fn test_message_ordering_by_timestamp() {
    let mut manager = HistoryManager::new();

    // Add messages with slight delays to ensure different timestamps
    let msg1 = create_test_message(MessageRole::User, "First");
    thread::sleep(Duration::from_millis(10));
    let msg2 = create_test_message(MessageRole::Assistant, "Second");
    thread::sleep(Duration::from_millis(10));
    let msg3 = create_test_message(MessageRole::User, "Third");

    // Add in different order
    manager.add_message(msg3.clone());
    manager.add_message(msg1.clone());
    manager.add_message(msg2.clone());

    // Messages should be ordered by timestamp
    let messages = manager.get_all_messages();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].content, "First");
    assert_eq!(messages[1].content, "Second");
    assert_eq!(messages[2].content, "Third");
}

#[test]
fn test_get_recent_messages() {
    let mut manager = HistoryManager::new();

    for i in 1..=5 {
        manager.add_message(create_test_message(
            MessageRole::User,
            &format!("Message {}", i),
        ));
    }

    let recent = manager.get_recent_messages(3);
    assert_eq!(recent.len(), 3);
    assert_eq!(recent[0].content, "Message 3");
    assert_eq!(recent[1].content, "Message 4");
    assert_eq!(recent[2].content, "Message 5");
}

#[test]
fn test_get_recent_messages_more_than_available() {
    let mut manager = HistoryManager::new();

    manager.add_message(create_test_message(MessageRole::User, "Message 1"));
    manager.add_message(create_test_message(MessageRole::User, "Message 2"));

    let recent = manager.get_recent_messages(10);
    assert_eq!(recent.len(), 2);
}

#[test]
fn test_get_recent_messages_zero() {
    let mut manager = HistoryManager::new();

    manager.add_message(create_test_message(MessageRole::User, "Message 1"));

    let recent = manager.get_recent_messages(0);
    assert_eq!(recent.len(), 0);
}

#[test]
fn test_search_by_content() {
    let mut manager = HistoryManager::new();

    manager.add_message(create_test_message(MessageRole::User, "Hello world"));
    manager.add_message(create_test_message(MessageRole::Assistant, "Hi there"));
    manager.add_message(create_test_message(MessageRole::User, "Hello again"));

    let results = manager.search_by_content("hello");
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|m| m.content == "Hello world"));
    assert!(results.iter().any(|m| m.content == "Hello again"));
}

#[test]
fn test_search_by_content_case_insensitive() {
    let mut manager = HistoryManager::new();

    manager.add_message(create_test_message(MessageRole::User, "HELLO WORLD"));
    manager.add_message(create_test_message(MessageRole::Assistant, "hello world"));
    manager.add_message(create_test_message(MessageRole::User, "HeLLo WoRLd"));

    let results = manager.search_by_content("hello");
    assert_eq!(results.len(), 3);
}

#[test]
fn test_search_by_content_no_matches() {
    let mut manager = HistoryManager::new();

    manager.add_message(create_test_message(MessageRole::User, "Hello world"));
    manager.add_message(create_test_message(MessageRole::Assistant, "Hi there"));

    let results = manager.search_by_content("goodbye");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_by_content_partial_match() {
    let mut manager = HistoryManager::new();

    manager.add_message(create_test_message(
        MessageRole::User,
        "The quick brown fox",
    ));
    manager.add_message(create_test_message(
        MessageRole::Assistant,
        "Jumps over the lazy dog",
    ));

    let results = manager.search_by_content("quick");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content, "The quick brown fox");
}

#[test]
fn test_max_size_enforcement() {
    let mut manager = HistoryManager::with_max_size(3);

    manager.add_message(create_test_message(MessageRole::User, "Message 1"));
    manager.add_message(create_test_message(MessageRole::User, "Message 2"));
    manager.add_message(create_test_message(MessageRole::User, "Message 3"));
    manager.add_message(create_test_message(MessageRole::User, "Message 4"));

    // Should only keep the last 3 messages
    assert_eq!(manager.message_count(), 3);
    let messages = manager.get_all_messages();
    assert_eq!(messages[0].content, "Message 2");
    assert_eq!(messages[1].content, "Message 3");
    assert_eq!(messages[2].content, "Message 4");
}

#[test]
fn test_max_size_enforcement_multiple_adds() {
    let mut manager = HistoryManager::with_max_size(2);

    for i in 1..=5 {
        manager.add_message(create_test_message(
            MessageRole::User,
            &format!("Message {}", i),
        ));
    }

    assert_eq!(manager.message_count(), 2);
    let messages = manager.get_all_messages();
    assert_eq!(messages[0].content, "Message 4");
    assert_eq!(messages[1].content, "Message 5");
}

#[test]
fn test_clear_history() {
    let mut manager = HistoryManager::new();

    manager.add_message(create_test_message(MessageRole::User, "Message 1"));
    manager.add_message(create_test_message(MessageRole::User, "Message 2"));

    assert_eq!(manager.message_count(), 2);

    manager.clear();

    assert_eq!(manager.message_count(), 0);
    assert_eq!(manager.get_all_messages().len(), 0);
}

#[test]
fn test_get_all_messages_ordering() {
    let mut manager = HistoryManager::new();

    let msg1 = create_test_message(MessageRole::User, "First");
    thread::sleep(Duration::from_millis(10));
    let msg2 = create_test_message(MessageRole::Assistant, "Second");
    thread::sleep(Duration::from_millis(10));
    let msg3 = create_test_message(MessageRole::User, "Third");

    // Add in reverse order
    manager.add_message(msg3);
    manager.add_message(msg2);
    manager.add_message(msg1);

    let all_messages = manager.get_all_messages();
    assert_eq!(all_messages.len(), 3);
    // Should be ordered by timestamp (oldest first)
    assert_eq!(all_messages[0].content, "First");
    assert_eq!(all_messages[1].content, "Second");
    assert_eq!(all_messages[2].content, "Third");
}

#[test]
fn test_message_roles_preserved() {
    let mut manager = HistoryManager::new();

    manager.add_message(create_test_message(MessageRole::User, "User message"));
    manager.add_message(create_test_message(
        MessageRole::Assistant,
        "Assistant message",
    ));
    manager.add_message(create_test_message(MessageRole::System, "System message"));

    let messages = manager.get_all_messages();
    assert_eq!(messages[0].role, MessageRole::User);
    assert_eq!(messages[1].role, MessageRole::Assistant);
    assert_eq!(messages[2].role, MessageRole::System);
}

#[test]
fn test_search_preserves_order() {
    let mut manager = HistoryManager::new();

    manager.add_message(create_test_message(MessageRole::User, "First hello"));
    manager.add_message(create_test_message(MessageRole::Assistant, "Response"));
    manager.add_message(create_test_message(MessageRole::User, "Second hello"));

    let results = manager.search_by_content("hello");
    assert_eq!(results.len(), 2);
    // Results should be in chronological order
    assert_eq!(results[0].content, "First hello");
    assert_eq!(results[1].content, "Second hello");
}

#[test]
fn test_default_history_manager() {
    let manager = HistoryManager::default();
    assert_eq!(manager.message_count(), 0);
    assert_eq!(manager.max_size(), None);
}
