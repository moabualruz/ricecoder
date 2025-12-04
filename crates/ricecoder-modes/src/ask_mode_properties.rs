//! Property-based tests for Ask Mode
//!
//! **Feature: ricecoder-modes, Property 2: Ask Mode File Protection**
//! **Validates: Requirements 2.2**

#[cfg(test)]
mod tests {
    use crate::ask_mode::AskMode;
    use crate::mode::Mode;
    use crate::models::Operation;

    /// Property 2: Ask Mode File Protection
    ///
    /// *For any* operation in Ask Mode, file modification operations SHALL be rejected
    /// and no files SHALL be modified.
    ///
    /// **Validates: Requirements 2.2**
    #[test]
    fn prop_ask_mode_blocks_file_operations() {
        let mode = AskMode::new();

        // Test that ModifyFile operation is blocked
        let result = mode.validate_operation(&Operation::ModifyFile);
        assert!(result.is_err(), "ModifyFile should be blocked in Ask Mode");

        // Test that GenerateCode operation is blocked
        let result = mode.validate_operation(&Operation::GenerateCode);
        assert!(result.is_err(), "GenerateCode should be blocked in Ask Mode");

        // Test that ExecuteCommand operation is blocked
        let result = mode.validate_operation(&Operation::ExecuteCommand);
        assert!(result.is_err(), "ExecuteCommand should be blocked in Ask Mode");

        // Test that RunTests operation is blocked
        let result = mode.validate_operation(&Operation::RunTests);
        assert!(result.is_err(), "RunTests should be blocked in Ask Mode");

        // Test that ValidateQuality operation is blocked
        let result = mode.validate_operation(&Operation::ValidateQuality);
        assert!(result.is_err(), "ValidateQuality should be blocked in Ask Mode");

        // Test that AnswerQuestion operation is allowed
        let result = mode.validate_operation(&Operation::AnswerQuestion);
        assert!(result.is_ok(), "AnswerQuestion should be allowed in Ask Mode");
    }

    /// Property: Ask Mode never allows file operations
    ///
    /// *For any* file operation attempt in Ask Mode, the operation SHALL be rejected.
    #[test]
    fn prop_ask_mode_file_operations_always_blocked() {
        let mode = AskMode::new();

        // Test all file-related operations
        let file_operations = vec![
            Operation::ModifyFile,
            Operation::GenerateCode,
            Operation::ExecuteCommand,
            Operation::RunTests,
            Operation::ValidateQuality,
        ];

        for operation in file_operations {
            let result = mode.validate_operation(&operation);
            assert!(
                result.is_err(),
                "Operation {:?} should be blocked in Ask Mode",
                operation
            );
        }
    }

    /// Property: Ask Mode capabilities exclude file operations
    ///
    /// *For any* Ask Mode instance, the capabilities SHALL NOT include
    /// CodeGeneration, CodeModification, FileOperations, CommandExecution,
    /// TestExecution, or QualityValidation.
    #[test]
    fn prop_ask_mode_capabilities_exclude_file_operations() {
        let mode = AskMode::new();
        let capabilities = mode.capabilities();

        use crate::models::Capability;

        // These capabilities should NOT be present
        let forbidden_capabilities = vec![
            Capability::CodeGeneration,
            Capability::CodeModification,
            Capability::FileOperations,
            Capability::CommandExecution,
            Capability::TestExecution,
            Capability::QualityValidation,
        ];

        for capability in forbidden_capabilities {
            assert!(
                !capabilities.contains(&capability),
                "Ask Mode should not have {:?} capability",
                capability
            );
        }

        // These capabilities SHOULD be present
        assert!(
            capabilities.contains(&Capability::QuestionAnswering),
            "Ask Mode should have QuestionAnswering capability"
        );
        assert!(
            capabilities.contains(&Capability::FreeformChat),
            "Ask Mode should have FreeformChat capability"
        );
    }

    /// Property: Ask Mode constraints block file operations
    ///
    /// *For any* Ask Mode instance, the constraints SHALL have:
    /// - allow_file_operations = false
    /// - allow_command_execution = false
    /// - allow_code_generation = false
    #[test]
    fn prop_ask_mode_constraints_block_operations() {
        let mode = AskMode::new();
        let constraints = mode.constraints();

        assert!(
            !constraints.allow_file_operations,
            "Ask Mode should not allow file operations"
        );
        assert!(
            !constraints.allow_command_execution,
            "Ask Mode should not allow command execution"
        );
        assert!(
            !constraints.allow_code_generation,
            "Ask Mode should not allow code generation"
        );
    }

    /// Property: Ask Mode can_execute only allows AnswerQuestion
    ///
    /// *For any* operation, Ask Mode can_execute SHALL return true only for
    /// AnswerQuestion operations.
    #[test]
    fn prop_ask_mode_can_execute_only_answer_question() {
        let mode = AskMode::new();

        // AnswerQuestion should be allowed
        assert!(
            mode.can_execute(&Operation::AnswerQuestion),
            "Ask Mode should be able to execute AnswerQuestion"
        );

        // All other operations should be blocked
        let other_operations = vec![
            Operation::GenerateCode,
            Operation::ModifyFile,
            Operation::ExecuteCommand,
            Operation::RunTests,
            Operation::ValidateQuality,
        ];

        for operation in other_operations {
            assert!(
                !mode.can_execute(&operation),
                "Ask Mode should not be able to execute {:?}",
                operation
            );
        }
    }

    /// Property: Blocked operation messages are informative
    ///
    /// *For any* blocked operation, the error message SHALL contain:
    /// - The operation name
    /// - "Ask Mode"
    /// - Guidance about switching to Code Mode (where applicable)
    #[test]
    fn prop_ask_mode_blocked_messages_are_informative() {
        let mode = AskMode::new();

        let blocked_operations = vec![
            (Operation::ModifyFile, "File operations"),
            (Operation::GenerateCode, "Code generation"),
            (Operation::ExecuteCommand, "Command execution"),
            (Operation::RunTests, "Test execution"),
            (Operation::ValidateQuality, "Quality validation"),
        ];

        for (operation, expected_text) in blocked_operations {
            let message = mode.blocked_operation_message(&operation);
            assert!(
                message.contains("Ask Mode"),
                "Message for {:?} should mention Ask Mode",
                operation
            );
            assert!(
                message.contains(expected_text),
                "Message for {:?} should mention '{}'",
                operation,
                expected_text
            );
        }
    }
}
