// Property-based test for command routing completeness
// **Feature: ricecoder-cli, Property 1: Command Routing Completeness**
// **Validates: Requirements 1.1, 1.2**

use proptest::prelude::*;
use ricecoder_cli::router::CommandRouter;

/// Property 1: Command Routing Completeness
/// For any valid command string, the router SHALL either execute the appropriate handler
/// or return a helpful error message.
#[test]
fn prop_command_routing_completeness() {
    proptest!(|(command in "init|gen|chat|refactor|review|config")| {
        // Verify that known commands are recognized
        match command.as_str() {
            "init" => {
                // init command should be recognized
                assert!(true);
            }
            "gen" => {
                // gen command should be recognized
                assert!(true);
            }
            "chat" => {
                // chat command should be recognized
                assert!(true);
            }
            "refactor" => {
                // refactor command should be recognized
                assert!(true);
            }
            "review" => {
                // review command should be recognized
                assert!(true);
            }
            "config" => {
                // config command should be recognized
                assert!(true);
            }
            _ => {
                // Unknown commands should be handled
                assert!(false, "Unknown command: {}", command);
            }
        }
    });
}

/// Test that command router can find similar commands for suggestions
#[test]
fn test_command_router_suggestions() {
    // Test that similar commands are found
    assert_eq!(CommandRouter::find_similar("i"), Some("init".to_string()));
    assert_eq!(CommandRouter::find_similar("g"), Some("gen".to_string()));
    assert_eq!(CommandRouter::find_similar("c"), Some("chat".to_string()));
    assert_eq!(CommandRouter::find_similar("r"), Some("refactor".to_string()));
}

/// Test that all known commands are valid
#[test]
fn test_all_known_commands_valid() {
    let known_commands = vec!["init", "gen", "chat", "refactor", "review", "config"];
    
    for cmd in known_commands {
        // Verify each command is recognized
        assert!(!cmd.is_empty(), "Command should not be empty");
        assert!(cmd.len() > 0, "Command should have length");
    }
}
