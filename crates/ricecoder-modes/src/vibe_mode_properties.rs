//! Property-based tests for Vibe Mode

#[cfg(test)]
mod tests {
    use crate::{Mode, ModeContext, Operation, VibeMode};
    use proptest::prelude::*;

    // Property 5: Vibe Mode Spec Bypass
    // **Feature: ricecoder-modes, Property 5: Vibe Mode Spec Bypass**
    // **Validates: Requirements 3.1, 3.2**
    proptest! {
        #[test]
        fn prop_vibe_mode_accepts_natural_language_without_specs(
            input in ".*[a-zA-Z0-9 ]+.*"
        ) {
            let mode = VibeMode::new();

            // Verify that Vibe Mode does not require specs
            assert!(!mode.constraints().require_specs);

            // Verify that code generation is allowed
            assert!(mode.can_execute(&Operation::GenerateCode));

            // Verify that natural language input is accepted
            let result = mode.accept_natural_language(&input);
            assert!(result.is_ok());

            let code = result.unwrap();
            assert!(code.contains("Natural language input"));
            assert!(code.contains("without formal specs"));
        }
    }

    // Property 6: Vibe Mode Warnings
    // **Feature: ricecoder-modes, Property 6: Vibe Mode Warnings**
    // **Validates: Requirements 3.5**
    proptest! {
        #[test]
        fn prop_vibe_mode_includes_warnings_in_all_responses(
            input in ".*[a-zA-Z0-9 ]+.*"
        ) {
            let mode = VibeMode::new();
            let context = ModeContext::new("test-session".to_string());

            // Process input and get response
            let response = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(mode.process(&input, &context))
                .unwrap();

            // Verify that warnings are included in suggestions
            assert!(!response.suggestions.is_empty());

            // Verify that at least one suggestion contains a warning indicator
            let has_warning = response.suggestions.iter().any(|s| {
                s.contains("⚠️") || s.contains("💡") || s.contains("Best Practice")
            });
            assert!(has_warning);
        }
    }

    // Additional property: Vibe Mode capabilities are correct
    proptest! {
        #[test]
        fn prop_vibe_mode_has_correct_capabilities(
            _dummy in Just(())
        ) {
            let mode = VibeMode::new();
            let capabilities = mode.capabilities();

            // Verify that Vibe Mode has the expected capabilities
            assert!(capabilities.iter().any(|c| c.to_string() == "CodeGeneration"));
            assert!(capabilities.iter().any(|c| c.to_string() == "CodeModification"));
            assert!(capabilities.iter().any(|c| c.to_string() == "FileOperations"));
            assert!(capabilities.iter().any(|c| c.to_string() == "FreeformChat"));
            assert!(capabilities.iter().any(|c| c.to_string() == "QuestionAnswering"));
            assert!(capabilities.iter().any(|c| c.to_string() == "SpecConversion"));
        }
    }

    // Additional property: Vibe Mode blocks disallowed operations
    proptest! {
        #[test]
        fn prop_vibe_mode_blocks_disallowed_operations(
            _dummy in Just(())
        ) {
            let mode = VibeMode::new();

            // Verify that disallowed operations are blocked
            assert!(!mode.can_execute(&Operation::ExecuteCommand));
            assert!(!mode.can_execute(&Operation::RunTests));
            assert!(!mode.can_execute(&Operation::ValidateQuality));
        }
    }

    // Additional property: Vibe Mode warnings are consistent
    proptest! {
        #[test]
        fn prop_vibe_mode_warnings_are_consistent(
            _dummy in Just(())
        ) {
            let mode = VibeMode::new();

            // Get warnings multiple times
            let warnings1 = mode.generate_warnings();
            let warnings2 = mode.generate_warnings();

            // Verify that warnings are consistent
            assert_eq!(warnings1.len(), warnings2.len());
            for (w1, w2) in warnings1.iter().zip(warnings2.iter()) {
                assert_eq!(w1, w2);
            }
        }
    }

    // Additional property: Vibe Mode spec conversion preserves code
    proptest! {
        #[test]
        fn prop_vibe_mode_spec_conversion_preserves_code(
            code in ".*[a-zA-Z0-9 ]+.*"
        ) {
            let mode = VibeMode::new();

            // Convert code to specs
            let result = mode.convert_to_specs(&code);
            assert!(result.is_ok());

            let spec = result.unwrap();

            // Verify that the original code is preserved in the spec
            assert!(spec.contains(&code));
        }
    }
}
