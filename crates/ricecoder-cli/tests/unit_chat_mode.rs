// Unit tests for chat mode
// **Feature: ricecoder-cli, Tests for Requirements 4.1-4.6**

use ricecoder_cli::chat::{ChatSession, ChatMessage};
use ricecoder_cli::commands::{ChatCommand, Command};

// ============================================================================
// ChatSession Tests
// ============================================================================

#[test]
fn test_chat_session_creation() {
    let session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    assert_eq!(session.provider, "openai");
    assert_eq!(session.model, "gpt-4");
    assert!(session.history.is_empty());
}

#[test]
fn test_chat_session_with_different_providers() {
    let providers = vec!["openai", "anthropic", "local"];
    
    for provider in providers {
        let session = ChatSession::new(provider.to_string(), "model".to_string());
        assert_eq!(session.provider, provider);
    }
}

#[test]
fn test_chat_session_with_different_models() {
    let models = vec!["gpt-4", "gpt-3.5-turbo", "claude-3", "llama-2"];
    
    for model in models {
        let session = ChatSession::new("openai".to_string(), model.to_string());
        assert_eq!(session.model, model);
    }
}

// ============================================================================
// ChatMessage Tests
// ============================================================================

#[test]
fn test_chat_message_creation() {
    let msg = ChatMessage {
        role: "user".to_string(),
        content: "Hello".to_string(),
    };
    
    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, "Hello");
}

#[test]
fn test_chat_message_clone() {
    let msg = ChatMessage {
        role: "assistant".to_string(),
        content: "Hi there!".to_string(),
    };
    
    let cloned = msg.clone();
    assert_eq!(msg.role, cloned.role);
    assert_eq!(msg.content, cloned.content);
}

#[test]
fn test_chat_message_debug_format() {
    let msg = ChatMessage {
        role: "user".to_string(),
        content: "Test".to_string(),
    };
    
    let debug_str = format!("{:?}", msg);
    assert!(debug_str.contains("user"));
    assert!(debug_str.contains("Test"));
}

// ============================================================================
// ChatSession Message Management Tests
// ============================================================================

#[test]
fn test_add_single_message() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    session.add_message("user".to_string(), "Hello".to_string());
    
    assert_eq!(session.history.len(), 1);
    assert_eq!(session.history[0].role, "user");
    assert_eq!(session.history[0].content, "Hello");
}

#[test]
fn test_add_multiple_messages() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    session.add_message("user".to_string(), "Hello".to_string());
    session.add_message("assistant".to_string(), "Hi there!".to_string());
    session.add_message("user".to_string(), "How are you?".to_string());
    
    assert_eq!(session.history.len(), 3);
}

#[test]
fn test_add_message_preserves_order() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    session.add_message("user".to_string(), "First".to_string());
    session.add_message("assistant".to_string(), "Second".to_string());
    session.add_message("user".to_string(), "Third".to_string());
    
    assert_eq!(session.history[0].content, "First");
    assert_eq!(session.history[1].content, "Second");
    assert_eq!(session.history[2].content, "Third");
}

#[test]
fn test_add_message_with_empty_content() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    session.add_message("user".to_string(), "".to_string());
    
    assert_eq!(session.history.len(), 1);
    assert_eq!(session.history[0].content, "");
}

#[test]
fn test_add_message_with_multiline_content() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    let multiline = "Line 1\nLine 2\nLine 3";
    session.add_message("user".to_string(), multiline.to_string());
    
    assert_eq!(session.history[0].content, multiline);
}

#[test]
fn test_add_message_with_special_characters() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    let special = "Hello! @#$%^&*() 世界 🌍";
    session.add_message("user".to_string(), special.to_string());
    
    assert_eq!(session.history[0].content, special);
}

// ============================================================================
// ChatSession History Tests
// ============================================================================

#[test]
fn test_get_history_empty() {
    let session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    let history = session.get_history();
    assert!(history.is_empty());
}

#[test]
fn test_get_history_with_messages() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    session.add_message("user".to_string(), "Hello".to_string());
    session.add_message("assistant".to_string(), "Hi!".to_string());
    
    let history = session.get_history();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].role, "user");
    assert_eq!(history[1].role, "assistant");
}

#[test]
fn test_get_history_returns_slice() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    session.add_message("user".to_string(), "Test".to_string());
    
    let history = session.get_history();
    assert_eq!(history.len(), 1);
    
    // Verify it's a slice
    let _: &[ChatMessage] = history;
}

// ============================================================================
// ChatCommand Tests
// ============================================================================

#[test]
fn test_chat_command_creation_with_defaults() {
    let cmd = ChatCommand::new(None, None, None);
    
    assert!(cmd.message.is_none());
    assert!(cmd.provider.is_none());
    assert!(cmd.model.is_none());
}

#[test]
fn test_chat_command_creation_with_message() {
    let cmd = ChatCommand::new(Some("Hello".to_string()), None, None);
    
    assert_eq!(cmd.message, Some("Hello".to_string()));
}

#[test]
fn test_chat_command_creation_with_provider() {
    let cmd = ChatCommand::new(None, Some("openai".to_string()), None);
    
    assert_eq!(cmd.provider, Some("openai".to_string()));
}

#[test]
fn test_chat_command_creation_with_model() {
    let cmd = ChatCommand::new(None, None, Some("gpt-4".to_string()));
    
    assert_eq!(cmd.model, Some("gpt-4".to_string()));
}

#[test]
fn test_chat_command_creation_with_all_options() {
    let cmd = ChatCommand::new(
        Some("Hello".to_string()),
        Some("openai".to_string()),
        Some("gpt-4".to_string()),
    );
    
    assert_eq!(cmd.message, Some("Hello".to_string()));
    assert_eq!(cmd.provider, Some("openai".to_string()));
    assert_eq!(cmd.model, Some("gpt-4".to_string()));
}

#[test]
fn test_chat_command_implements_command_trait() {
    let cmd = ChatCommand::new(None, None, None);
    let _: &dyn Command = &cmd;
}

// ============================================================================
// ChatSession Conversation Flow Tests
// ============================================================================

#[test]
fn test_conversation_flow_user_assistant() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    // User message
    session.add_message("user".to_string(), "What is 2+2?".to_string());
    assert_eq!(session.history.len(), 1);
    
    // Assistant response
    session.add_message("assistant".to_string(), "2+2 equals 4".to_string());
    assert_eq!(session.history.len(), 2);
    
    // Verify conversation order
    let history = session.get_history();
    assert_eq!(history[0].role, "user");
    assert_eq!(history[1].role, "assistant");
}

#[test]
fn test_conversation_flow_multiple_turns() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    // Turn 1
    session.add_message("user".to_string(), "Hello".to_string());
    session.add_message("assistant".to_string(), "Hi!".to_string());
    
    // Turn 2
    session.add_message("user".to_string(), "How are you?".to_string());
    session.add_message("assistant".to_string(), "I'm doing well!".to_string());
    
    // Turn 3
    session.add_message("user".to_string(), "Goodbye".to_string());
    session.add_message("assistant".to_string(), "Goodbye!".to_string());
    
    assert_eq!(session.history.len(), 6);
}

// ============================================================================
// ChatSession State Tests
// ============================================================================

#[test]
fn test_chat_session_state_persistence() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    session.add_message("user".to_string(), "Message 1".to_string());
    let history1 = session.get_history().len();
    
    session.add_message("user".to_string(), "Message 2".to_string());
    let history2 = session.get_history().len();
    
    assert_eq!(history1, 1);
    assert_eq!(history2, 2);
}

#[test]
fn test_chat_session_provider_model_immutable() {
    let session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    let provider = session.provider.clone();
    let model = session.model.clone();
    
    assert_eq!(provider, "openai");
    assert_eq!(model, "gpt-4");
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_chat_session_with_very_long_message() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    let long_msg = "a".repeat(10000);
    session.add_message("user".to_string(), long_msg.clone());
    
    assert_eq!(session.history[0].content, long_msg);
}

#[test]
fn test_chat_session_with_many_messages() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    for i in 0..1000 {
        session.add_message("user".to_string(), format!("Message {}", i));
    }
    
    assert_eq!(session.history.len(), 1000);
}

#[test]
fn test_chat_message_with_unicode_roles() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    session.add_message("用户".to_string(), "你好".to_string());
    
    assert_eq!(session.history[0].role, "用户");
    assert_eq!(session.history[0].content, "你好");
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn test_chat_session_idempotent_creation() {
    let session1 = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    let session2 = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    assert_eq!(session1.provider, session2.provider);
    assert_eq!(session1.model, session2.model);
    assert_eq!(session1.history.len(), session2.history.len());
}

#[test]
fn test_chat_session_add_message_consistency() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    let msg = "Test message";
    session.add_message("user".to_string(), msg.to_string());
    session.add_message("user".to_string(), msg.to_string());
    
    // Both messages should be identical
    assert_eq!(session.history[0].content, session.history[1].content);
}

#[test]
fn test_chat_session_history_monotonic() {
    let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
    
    let len1 = session.get_history().len();
    session.add_message("user".to_string(), "Message".to_string());
    let len2 = session.get_history().len();
    
    assert!(len2 >= len1);
    assert_eq!(len2, len1 + 1);
}

#[test]
fn test_chat_command_provider_validation() {
    // Valid providers should be accepted
    let cmd1 = ChatCommand::new(None, Some("openai".to_string()), None);
    let cmd2 = ChatCommand::new(None, Some("anthropic".to_string()), None);
    let cmd3 = ChatCommand::new(None, Some("local".to_string()), None);
    
    assert_eq!(cmd1.provider, Some("openai".to_string()));
    assert_eq!(cmd2.provider, Some("anthropic".to_string()));
    assert_eq!(cmd3.provider, Some("local".to_string()));
}
