//! Property-based tests for markdown configuration module
//!
//! Tests the correctness properties of the markdown configuration system:
//! - Parsing consistency: parsing produces consistent results
//! - YAML validation correctness: validation correctly accepts/rejects
//! - Configuration completeness: valid configs can be registered and retrieved
//! - Error reporting completeness: invalid configs report all errors

use proptest::prelude::*;
use ricecoder_storage::markdown_config::{
    parser::MarkdownParser, registry::ConfigRegistry, types::*,
    validation::*, yaml_parser::YamlParser,
};
use std::sync::Arc;

// ============ Generators ============

/// Generate valid agent names
fn agent_name_strategy() -> impl Strategy<Value = String> {
    r"[a-z][a-z0-9_-]{0,30}".prop_map(|s| s.to_string())
}

/// Generate valid agent prompts
fn agent_prompt_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 .,!?]{10,100}".prop_map(|s| s.to_string())
}

/// Generate valid agent configurations
fn agent_config_strategy() -> impl Strategy<Value = AgentConfig> {
    (
        agent_name_strategy(),
        agent_prompt_strategy(),
        prop::option::of(r"[a-z0-9-]{3,20}"),
        prop::option::of(0.0f32..=2.0f32),
        prop::option::of(100u32..=4000u32),
    )
        .prop_map(|(name, prompt, model, temp, tokens)| AgentConfig {
            name,
            description: Some("Test agent".to_string()),
            prompt,
            model,
            temperature: temp,
            max_tokens: tokens,
            tools: vec![],
        })
}

/// Generate valid mode names
fn mode_name_strategy() -> impl Strategy<Value = String> {
    r"[a-z][a-z0-9_-]{0,30}".prop_map(|s| s.to_string())
}

/// Generate valid mode prompts
fn mode_prompt_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 .,!?]{10,100}".prop_map(|s| s.to_string())
}

/// Generate valid mode configurations
fn mode_config_strategy() -> impl Strategy<Value = ModeConfig> {
    (mode_name_strategy(), mode_prompt_strategy())
        .prop_map(|(name, prompt)| ModeConfig {
            name,
            description: Some("Test mode".to_string()),
            prompt,
            keybinding: None,
            enabled: true,
        })
}

/// Generate valid command names
fn command_name_strategy() -> impl Strategy<Value = String> {
    r"[a-z][a-z0-9_-]{0,30}".prop_map(|s| s.to_string())
}

/// Generate valid command templates
fn command_template_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 .,!?{}]{10,100}".prop_map(|s| s.to_string())
}

/// Generate valid command configurations
fn command_config_strategy() -> impl Strategy<Value = CommandConfig> {
    (command_name_strategy(), command_template_strategy())
        .prop_map(|(name, template)| CommandConfig {
            name,
            description: Some("Test command".to_string()),
            template,
            parameters: vec![],
            keybinding: None,
        })
}

/// Generate valid markdown with frontmatter
fn markdown_with_frontmatter_strategy() -> impl Strategy<Value = String> {
    (
        r"[a-z][a-z0-9_-]{0,20}",
        r"[a-zA-Z0-9 .,!?]{10,50}",
    )
        .prop_map(|(name, desc)| {
            format!(
                "---\nname: {}\ndescription: {}\n---\n# Content\nBody text",
                name, desc
            )
        })
}

/// Generate valid YAML frontmatter
fn yaml_frontmatter_strategy() -> impl Strategy<Value = String> {
    (
        r"[a-z][a-z0-9_-]{0,20}",
        r"[a-zA-Z0-9 ]{10,50}",
    )
        .prop_map(|(name, desc)| format!("name: {}\ndescription: \"{}\"", name, desc))
}

// ============ Property 1: Parsing Consistency ============
// **Feature: ricecoder-markdown-config, Property 1: Parsing consistency**
// For any valid markdown with frontmatter, parsing produces consistent results

proptest! {
    #[test]
    fn prop_parsing_consistency(content in markdown_with_frontmatter_strategy()) {
        let parser = MarkdownParser::new();

        // Parse the same content twice
        let result1 = parser.parse(&content).expect("First parse should succeed");
        let result2 = parser.parse(&content).expect("Second parse should succeed");

        // Results should be identical
        prop_assert_eq!(result1, result2, "Parsing should be consistent");
    }
}

// ============ Property 2: YAML Validation Correctness ============
// **Feature: ricecoder-markdown-config, Property 2: YAML validation correctness**
// Validation correctly accepts/rejects based on schema

proptest! {
    #[test]
    fn prop_yaml_validation_accepts_valid(yaml in yaml_frontmatter_strategy()) {
        let parser = YamlParser::new();

        // Valid YAML should pass validation
        let result = parser.validate_structure(&yaml);
        prop_assert!(result.is_ok(), "Valid YAML should pass validation");
    }
}

proptest! {
    #[test]
    fn prop_yaml_validation_rejects_invalid(
        yaml in r"[a-z]+: \[unclosed"
    ) {
        let parser = YamlParser::new();

        // Invalid YAML should fail validation
        let result = parser.validate_structure(&yaml);
        prop_assert!(result.is_err(), "Invalid YAML should fail validation");
    }
}

// ============ Property 3: Configuration Completeness ============
// **Feature: ricecoder-markdown-config, Property 3: Configuration completeness**
// Valid configs can be registered and retrieved

proptest! {
    #[test]
    fn prop_agent_registration_and_retrieval(agent in agent_config_strategy()) {
        let registry = Arc::new(ConfigRegistry::new());

        // Register the agent
        let register_result = registry.register_agent(agent.clone());
        prop_assert!(register_result.is_ok(), "Valid agent should register successfully");

        // Retrieve the agent
        let retrieved = registry.get_agent(&agent.name)
            .expect("Should be able to retrieve agent")
            .expect("Agent should exist");

        // Retrieved agent should match original
        prop_assert_eq!(retrieved, agent, "Retrieved agent should match registered agent");
    }
}

proptest! {
    #[test]
    fn prop_mode_registration_and_retrieval(mode in mode_config_strategy()) {
        let registry = Arc::new(ConfigRegistry::new());

        // Register the mode
        let register_result = registry.register_mode(mode.clone());
        prop_assert!(register_result.is_ok(), "Valid mode should register successfully");

        // Retrieve the mode
        let retrieved = registry.get_mode(&mode.name)
            .expect("Should be able to retrieve mode")
            .expect("Mode should exist");

        // Retrieved mode should match original
        prop_assert_eq!(retrieved, mode, "Retrieved mode should match registered mode");
    }
}

proptest! {
    #[test]
    fn prop_command_registration_and_retrieval(command in command_config_strategy()) {
        let registry = Arc::new(ConfigRegistry::new());

        // Register the command
        let register_result = registry.register_command(command.clone());
        prop_assert!(register_result.is_ok(), "Valid command should register successfully");

        // Retrieve the command
        let retrieved = registry.get_command(&command.name)
            .expect("Should be able to retrieve command")
            .expect("Command should exist");

        // Retrieved command should match original
        prop_assert_eq!(retrieved, command, "Retrieved command should match registered command");
    }
}

// ============ Property 4: Error Reporting Completeness ============
// **Feature: ricecoder-markdown-config, Property 4: Error reporting completeness**
// Invalid configs report all errors without crashing

proptest! {
    #[test]
    fn prop_invalid_agent_validation_reports_errors(
        name in r"[a-z0-9_-]{0,30}",  // May be empty
        prompt in r"[a-zA-Z0-9 .,!?]{0,50}"  // May be empty
    ) {
        let agent = AgentConfig {
            name,
            description: None,
            prompt,
            model: None,
            temperature: None,
            max_tokens: None,
            tools: vec![],
        };

        // Validate the agent
        let result = validate_agent_config(&agent);

        // If name or prompt is empty, validation should fail
        if agent.name.is_empty() || agent.prompt.is_empty() {
            prop_assert!(!result.is_valid(), "Invalid agent should fail validation");
            prop_assert!(!result.errors.is_empty(), "Should report errors");
        } else {
            prop_assert!(result.is_valid(), "Valid agent should pass validation");
        }
    }
}

proptest! {
    #[test]
    fn prop_invalid_mode_validation_reports_errors(
        name in r"[a-z0-9_-]{0,30}",  // May be empty
        prompt in r"[a-zA-Z0-9 .,!?]{0,50}"  // May be empty
    ) {
        let mode = ModeConfig {
            name,
            description: None,
            prompt,
            keybinding: None,
            enabled: true,
        };

        // Validate the mode
        let result = validate_mode_config(&mode);

        // If name or prompt is empty, validation should fail
        if mode.name.is_empty() || mode.prompt.is_empty() {
            prop_assert!(!result.is_valid(), "Invalid mode should fail validation");
            prop_assert!(!result.errors.is_empty(), "Should report errors");
        } else {
            prop_assert!(result.is_valid(), "Valid mode should pass validation");
        }
    }
}

proptest! {
    #[test]
    fn prop_invalid_command_validation_reports_errors(
        name in r"[a-z0-9_-]{0,30}",  // May be empty
        template in r"[a-zA-Z0-9 .,!?{}]{0,50}"  // May be empty
    ) {
        let command = CommandConfig {
            name,
            description: None,
            template,
            parameters: vec![],
            keybinding: None,
        };

        // Validate the command
        let result = validate_command_config(&command);

        // If name or template is empty, validation should fail
        if command.name.is_empty() || command.template.is_empty() {
            prop_assert!(!result.is_valid(), "Invalid command should fail validation");
            prop_assert!(!result.errors.is_empty(), "Should report errors");
        } else {
            prop_assert!(result.is_valid(), "Valid command should pass validation");
        }
    }
}

// ============ Additional Properties ============

proptest! {
    #[test]
    fn prop_duplicate_registration_prevented(agent in agent_config_strategy()) {
        let registry = Arc::new(ConfigRegistry::new());

        // Register the agent once
        let first_result = registry.register_agent(agent.clone());
        prop_assert!(first_result.is_ok(), "First registration should succeed");

        // Try to register the same agent again
        let second_result = registry.register_agent(agent);
        prop_assert!(second_result.is_err(), "Duplicate registration should fail");
    }
}

proptest! {
    #[test]
    fn prop_registry_isolation(
        agent1 in agent_config_strategy(),
        agent2 in agent_config_strategy()
    ) {
        // Ensure agents have different names for this test
        prop_assume!(agent1.name != agent2.name);

        let registry = Arc::new(ConfigRegistry::new());

        // Register both agents
        registry.register_agent(agent1.clone()).ok();
        registry.register_agent(agent2.clone()).ok();

        // Retrieving one should not affect the other
        let retrieved1 = registry.get_agent(&agent1.name)
            .expect("Should retrieve agent1")
            .expect("Agent1 should exist");
        let retrieved2 = registry.get_agent(&agent2.name)
            .expect("Should retrieve agent2")
            .expect("Agent2 should exist");

        prop_assert_eq!(retrieved1, agent1, "Agent1 should be unchanged");
        prop_assert_eq!(retrieved2, agent2, "Agent2 should be unchanged");
    }
}

proptest! {
    #[test]
    fn prop_parsing_handles_edge_cases(
        content in r"[\x20-\x7E\n\r\t]{0,1000}"
    ) {
        let parser = MarkdownParser::new();

        // Parsing should not panic on any input
        let _result = parser.parse(&content);
        // We don't assert on the result, just that it doesn't panic
    }
}

proptest! {
    #[test]
    fn prop_yaml_parsing_handles_edge_cases(
        yaml in r"[\x20-\x7E\n\r\t]{0,500}"
    ) {
        let parser = YamlParser::new();

        // YAML parsing should not panic on any input
        let _result = parser.validate_structure(&yaml);
        // We don't assert on the result, just that it doesn't panic
    }
}
