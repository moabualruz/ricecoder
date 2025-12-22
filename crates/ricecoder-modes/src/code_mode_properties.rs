//! Property-based tests for Code Mode

#[cfg(test)]
mod properties {
    use crate::{code_mode::CodeMode, mode::Mode, models::Capability};

    /// **Feature: ricecoder-modes, Property 1: Mode Capability Activation**
    /// **Validates: Requirements 1.1**
    ///
    /// For any mode selection, the activated capabilities SHALL exactly match
    /// the declared capabilities for that mode.
    #[test]
    fn prop_code_mode_capabilities_match_declared() {
        // Create a Code Mode instance
        let mode = CodeMode::new();

        // Get the declared capabilities
        let declared_capabilities = mode.capabilities();

        // Get the capabilities from the config
        let config_capabilities = mode.config().capabilities.clone();

        // Property: The capabilities returned by capabilities() should match
        // the capabilities in the config
        assert_eq!(declared_capabilities, config_capabilities);

        // Property: Code Mode should have exactly 6 capabilities
        assert_eq!(declared_capabilities.len(), 6);

        // Property: Code Mode should have all required capabilities
        assert!(declared_capabilities.contains(&Capability::CodeGeneration));
        assert!(declared_capabilities.contains(&Capability::CodeModification));
        assert!(declared_capabilities.contains(&Capability::FileOperations));
        assert!(declared_capabilities.contains(&Capability::CommandExecution));
        assert!(declared_capabilities.contains(&Capability::TestExecution));
        assert!(declared_capabilities.contains(&Capability::QualityValidation));

        // Property: Code Mode should NOT have capabilities it doesn't declare
        assert!(!declared_capabilities.contains(&Capability::QuestionAnswering));
        assert!(!declared_capabilities.contains(&Capability::FreeformChat));
        assert!(!declared_capabilities.contains(&Capability::SpecConversion));
    }

    /// **Feature: ricecoder-modes, Property 1: Mode Capability Activation**
    /// **Validates: Requirements 1.1**
    ///
    /// For any mode, the capabilities should be consistent across multiple calls.
    #[test]
    fn prop_code_mode_capabilities_consistent() {
        let mode = CodeMode::new();

        // Get capabilities multiple times
        let caps1 = mode.capabilities();
        let caps2 = mode.capabilities();
        let caps3 = mode.capabilities();

        // Property: Capabilities should be identical across calls
        assert_eq!(caps1, caps2);
        assert_eq!(caps2, caps3);
    }

    /// **Feature: ricecoder-modes, Property 1: Mode Capability Activation**
    /// **Validates: Requirements 1.1**
    ///
    /// For any mode, the can_execute method should only return true for
    /// operations that correspond to declared capabilities.
    #[test]
    fn prop_code_mode_can_execute_matches_capabilities() {
        use crate::models::Operation;

        let mode = CodeMode::new();
        let capabilities = mode.capabilities();

        // Property: If a capability is declared, the corresponding operation
        // should be executable
        if capabilities.contains(&Capability::CodeGeneration) {
            assert!(mode.can_execute(&Operation::GenerateCode));
        }
        if capabilities.contains(&Capability::CodeModification) {
            assert!(mode.can_execute(&Operation::ModifyFile));
        }
        if capabilities.contains(&Capability::CommandExecution) {
            assert!(mode.can_execute(&Operation::ExecuteCommand));
        }
        if capabilities.contains(&Capability::TestExecution) {
            assert!(mode.can_execute(&Operation::RunTests));
        }
        if capabilities.contains(&Capability::QualityValidation) {
            assert!(mode.can_execute(&Operation::ValidateQuality));
        }

        // Property: If a capability is NOT declared, the corresponding operation
        // should NOT be executable
        if !capabilities.contains(&Capability::QuestionAnswering) {
            assert!(!mode.can_execute(&Operation::AnswerQuestion));
        }
    }

    /// **Feature: ricecoder-modes, Property 1: Mode Capability Activation**
    /// **Validates: Requirements 1.1**
    ///
    /// For any mode, the constraints should reflect the capabilities.
    #[test]
    fn prop_code_mode_constraints_match_capabilities() {
        let mode = CodeMode::new();
        let capabilities = mode.capabilities();
        let constraints = mode.constraints();

        // Property: If file operations capability is declared, constraints should allow it
        if capabilities.contains(&Capability::FileOperations) {
            assert!(constraints.allow_file_operations);
        }

        // Property: If command execution capability is declared, constraints should allow it
        if capabilities.contains(&Capability::CommandExecution) {
            assert!(constraints.allow_command_execution);
        }

        // Property: If code generation capability is declared, constraints should allow it
        if capabilities.contains(&Capability::CodeGeneration) {
            assert!(constraints.allow_code_generation);
        }
    }

    /// **Feature: ricecoder-modes, Property 1: Mode Capability Activation**
    /// **Validates: Requirements 1.1**
    ///
    /// For any mode, the mode ID should be consistent with the mode name.
    #[test]
    fn prop_code_mode_identity_consistent() {
        let mode = CodeMode::new();

        // Property: Mode ID should be "code"
        assert_eq!(mode.id(), "code");

        // Property: Mode name should contain "Code"
        assert!(mode.name().contains("Code"));

        // Property: Mode description should be non-empty
        assert!(!mode.description().is_empty());

        // Property: System prompt should be non-empty
        assert!(!mode.system_prompt().is_empty());
    }

    /// **Feature: ricecoder-modes, Property 4: Code Mode Execution Pipeline**
    /// **Validates: Requirements 1.2, 1.3, 1.4, 1.5**
    ///
    /// For any code generation in Code Mode, the system SHALL execute the following
    /// pipeline: generate code → create/modify files → run tests → validate quality → provide summary.
    #[tokio::test]
    async fn prop_code_mode_execution_pipeline() {
        let mode = CodeMode::new();

        // Step 1: Generate code from specification
        let spec = "Create a function that adds two numbers";
        let generated_code = mode.generate_code(spec);
        assert!(generated_code.is_ok());
        let code = generated_code.unwrap();

        // Property: Generated code should contain the spec reference
        assert!(code.contains("Generated from spec"));

        // Step 2: Create/modify files
        let temp_dir = std::env::temp_dir().join("ricecoder_pipeline_test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("generated.rs");

        let create_result = mode.create_file(&file_path, &code).await;
        assert!(create_result.is_ok());

        // Property: File should exist after creation
        assert!(file_path.exists());

        // Step 3: Run tests
        let test_result = mode.run_tests(&[file_path.clone()]).await;
        assert!(test_result.is_ok());
        let (tests_run, tests_passed, _failures) = test_result.unwrap();

        // Property: Tests should be counted
        assert!(tests_run > 0);

        // Step 4: Validate quality
        let quality_result = mode.validate_quality(&[file_path.clone()]).await;
        assert!(quality_result.is_ok());
        let quality_issues = quality_result.unwrap();

        // Property: Quality validation should complete without error
        // (may or may not have issues, but should not error)

        // Step 5: Provide summary
        let summary = mode.generate_change_summary(1, 0, tests_run, tests_passed, quality_issues);

        // Property: Summary should reflect the pipeline execution
        assert_eq!(summary.files_created, 1);
        assert_eq!(summary.tests_run, tests_run);
        assert_eq!(summary.tests_passed, tests_passed);

        // Property: Summary should be formattable
        let formatted = mode.format_change_summary(&summary);
        assert!(formatted.contains("Change Summary"));

        // Property: Feedback should be provided
        let feedback = mode.provide_feedback(&summary);
        assert!(!feedback.is_empty());

        // Cleanup
        let _ = std::fs::remove_file(&file_path);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    /// **Feature: ricecoder-modes, Property 4: Code Mode Execution Pipeline**
    /// **Validates: Requirements 1.2, 1.3, 1.4, 1.5**
    ///
    /// For any code generation, the pipeline should be repeatable and consistent.
    #[tokio::test]
    async fn prop_code_mode_pipeline_consistency() {
        let mode = CodeMode::new();
        let spec = "Create a simple function";

        // Run the pipeline twice
        let code1 = mode.generate_code(spec).unwrap();
        let code2 = mode.generate_code(spec).unwrap();

        // Property: Generated code should be identical for the same spec
        assert_eq!(code1, code2);

        // Property: Both should be processable
        let temp_dir = std::env::temp_dir().join("ricecoder_consistency_test");
        let _ = std::fs::create_dir_all(&temp_dir);

        let file1 = temp_dir.join("test1.rs");
        let file2 = temp_dir.join("test2.rs");

        let result1 = mode.create_file(&file1, &code1).await;
        let result2 = mode.create_file(&file2, &code2).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Property: Both files should exist
        assert!(file1.exists());
        assert!(file2.exists());

        // Property: File contents should be identical
        let content1 = std::fs::read_to_string(&file1).unwrap();
        let content2 = std::fs::read_to_string(&file2).unwrap();
        assert_eq!(content1, content2);

        // Cleanup
        let _ = std::fs::remove_file(&file1);
        let _ = std::fs::remove_file(&file2);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    /// **Feature: ricecoder-modes, Property 4: Code Mode Execution Pipeline**
    /// **Validates: Requirements 1.2, 1.3, 1.4, 1.5**
    ///
    /// For any code generation, all pipeline steps should be executable.
    #[tokio::test]
    async fn prop_code_mode_all_operations_allowed() {
        use crate::models::Operation;

        let mode = CodeMode::new();

        // Property: All operations in the pipeline should be allowed
        assert!(mode.can_execute(&Operation::GenerateCode));
        assert!(mode.can_execute(&Operation::ModifyFile));
        assert!(mode.can_execute(&Operation::RunTests));
        assert!(mode.can_execute(&Operation::ValidateQuality));

        // Property: Constraints should allow all pipeline operations
        let constraints = mode.constraints();
        assert!(constraints.allow_file_operations);
        assert!(constraints.allow_code_generation);
    }
}
