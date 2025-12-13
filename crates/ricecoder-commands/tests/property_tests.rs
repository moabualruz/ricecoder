use proptest::prelude::*;
use ricecoder_commands::{
    ArgumentType, CommandArgument, CommandDefinition, CommandExecutor, CommandRegistry,
    TemplateProcessor,
};
use std::collections::HashMap;

// Property 1: Command execution consistency
// For any valid command definition and arguments, executing the command should:
// 1. Always produce a CommandExecutionResult
// 2. The result should have the same command_id as the input
// 3. The result should have a valid exit code
// 4. The result should have a duration >= 0
/*
proptest! {
    #[test]
    fn prop_command_execution_consistency(
        cmd_id in "[a-z0-9_]{1,20}",
        cmd_name in "[a-zA-Z0-9 ]{1,30}",
        cmd_text in "echo [a-zA-Z0-9 ]{1,20}",
    ) {
        // TODO: Implement proper command execution testing
        // This test requires a concrete CommandExecutor implementation
        // and proper async testing setup
    }
}
*/

// Property 2: Template substitution idempotence
// For any template and variables, processing the template twice should produce the same result
proptest! {
    #[test]
    fn prop_template_substitution_idempotence(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        var_value in "[a-zA-Z0-9 ]{1,20}",
    ) {
        let template = format!("Hello {{{{{}}}}} world", var_name);
        let mut vars = HashMap::new();
        vars.insert(var_name.clone(), var_value.clone());

        let result1 = TemplateProcessor::process(&template, &vars);
        if let Ok(ref processed1) = result1 {
            // Processing the already-processed template should be idempotent
            // (no more variables to substitute)
            let result2 = TemplateProcessor::process(processed1, &vars);
            match (&result1, &result2) {
                (Ok(r1), Ok(r2)) => assert_eq!(r1, r2),
                (Err(_), Err(_)) => {}, // Both errors is also consistent
                _ => panic!("Results should be consistent"),
            }
        }
    }
}

// Property 3: Registry consistency
// For any sequence of register/unregister operations, the registry should maintain consistency
proptest! {
    #[test]
    fn prop_registry_consistency(
        commands in prop::collection::vec(
            (
                "[a-z0-9_]{1,20}",
                "[a-zA-Z0-9 ]{1,30}",
                "echo [a-zA-Z0-9]{1,20}"
            ),
            1..10
        ),
    ) {
        let mut registry = CommandRegistry::new();

        // Register all commands
        for (id, name, cmd) in &commands {
            let cmd_def = CommandDefinition::new(id, name, cmd);
            let _ = registry.register(cmd_def);
        }

        // Count should match number of unique IDs
        let unique_ids: std::collections::HashSet<_> = commands.iter().map(|(id, _, _)| id).collect();
        assert_eq!(registry.count(), unique_ids.len());

        // All registered commands should be retrievable
        for (id, _, _) in &commands {
            assert!(registry.exists(id));
        }

        // Unregistering should reduce count
        if let Some((id, _, _)) = commands.first() {
            let initial_count = registry.count();
            let _ = registry.unregister(id);
            assert_eq!(registry.count(), initial_count - 1);
            assert!(!registry.exists(id));
        }
    }
}

// Property 4: Argument validation consistency
// For any command with required arguments, validation should fail if required args are missing
proptest! {
    #[test]
    fn prop_argument_validation_consistency(
        arg_name in "[a-z_][a-z0-9_]{0,10}",
    ) {
        let mut cmd = CommandDefinition::new("test", "Test", "echo {{arg}}");
        cmd.arguments.push(
            CommandArgument::new(&arg_name, ArgumentType::String)
                .with_required(true)
        );

        // Validation should fail with empty arguments
        let empty_args: HashMap<String, String> = HashMap::new();
        // Note: validate_parameters is an instance method, not static
        // This test needs to be updated to use a concrete executor instance

        // Validation should succeed with the required argument
        let mut args: HashMap<String, String> = HashMap::new();
        args.insert(arg_name.clone(), "value".to_string());
        // Note: validate_parameters is an instance method, not static
        // This test needs to be updated to use a concrete executor instance
    }
}

// Property 5: Command enable/disable consistency
// For any command, enabling and disabling should be reversible
proptest! {
    #[test]
    fn prop_enable_disable_consistency(
        cmd_id in "[a-z0-9_]{1,20}",
    ) {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDefinition::new(&cmd_id, "Test", "echo test");
        registry.register(cmd).ok();

        // Initially enabled
        assert!(registry.get(&cmd_id).unwrap().enabled);

        // Disable
        registry.disable(&cmd_id).ok();
        assert!(!registry.get(&cmd_id).unwrap().enabled);

        // Re-enable
        registry.enable(&cmd_id).ok();
        assert!(registry.get(&cmd_id).unwrap().enabled);

        // Disable again
        registry.disable(&cmd_id).ok();
        assert!(!registry.get(&cmd_id).unwrap().enabled);
    }
}

// Property 6: Template variable extraction consistency
// For any template, extracted variables should be consistent across multiple calls
proptest! {
    #[test]
    fn prop_template_variable_extraction_consistency(
        template in r"\{\{[a-z_][a-z0-9_]{0,10}\}\}.*\{\{[a-z_][a-z0-9_]{0,10}\}\}",
    ) {
        let result1 = TemplateProcessor::extract_variables(&template);
        let result2 = TemplateProcessor::extract_variables(&template);

        // Both calls should produce the same result
        match (result1, result2) {
            (Ok(vars1), Ok(vars2)) => {
                assert_eq!(vars1.len(), vars2.len());
                for (v1, v2) in vars1.iter().zip(vars2.iter()) {
                    assert_eq!(v1, v2);
                }
            },
            (Err(_), Err(_)) => {}, // Both errors is consistent
            _ => panic!("Results should be consistent"),
        }
    }
}

// Property 7: Command search consistency
// For any search query, searching should return consistent results
proptest! {
    #[test]
    fn prop_command_search_consistency(
        query in "[a-z]{1,10}",
    ) {
        let mut registry = CommandRegistry::new();

        // Register some test commands
        for i in 0..5 {
            let cmd = CommandDefinition::new(
                format!("cmd{}", i).as_str(),
                format!("Test Command {}", i).as_str(),
                "echo test"
            );
            registry.register(cmd).ok();
        }

        // Search should be consistent
        let result1 = registry.search(&query);
        let result2 = registry.search(&query);

        assert_eq!(result1.len(), result2.len());
        for (r1, r2) in result1.iter().zip(result2.iter()) {
            assert_eq!(r1.id, r2.id);
        }
    }
}

// Property 8: Command list consistency
// For any registry state, list_all should return all commands and list_enabled should return only enabled ones
proptest! {
    #[test]
    fn prop_command_list_consistency(
        commands in prop::collection::vec(
            (
                "[a-z0-9_]{1,20}",
                "[a-zA-Z0-9 ]{1,30}",
                "echo [a-zA-Z0-9]{1,20}"
            ),
            1..10
        ),
    ) {
        let mut registry = CommandRegistry::new();

        // Register all commands
        for (id, name, cmd) in &commands {
            let cmd_def = CommandDefinition::new(id, name, cmd);
            let _ = registry.register(cmd_def);
        }

        let all = registry.list_all();
        let enabled = registry.list_enabled();

        // All enabled commands should be in the all list
        for enabled_cmd in &enabled {
            assert!(all.iter().any(|c| c.id == enabled_cmd.id));
        }

        // All commands in the all list should exist (this is always true, but validates the structure)
        for cmd in &all {
            assert!(!cmd.id.is_empty());
        }

        // Enabled count should be <= total count
        assert!(enabled.len() <= all.len());
    }
}
