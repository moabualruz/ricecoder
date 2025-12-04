//! Ask Mode implementation for question answering without file modifications

use async_trait::async_trait;
use std::time::Instant;

use crate::error::Result;
use crate::mode::Mode;
use crate::models::{
    Capability, ModeAction, ModeConfig, ModeConstraints, ModeContext, ModeResponse, Operation,
};

/// Ask Mode for question answering and explanations
///
/// Ask Mode provides capabilities for:
/// - Question answering
/// - Explanations and guidance
/// - Code examples in responses
/// - No file modifications or command execution
#[derive(Debug, Clone)]
pub struct AskMode {
    config: ModeConfig,
}

impl AskMode {
    /// Create a new Ask Mode instance
    pub fn new() -> Self {
        Self {
            config: ModeConfig {
                temperature: 0.7,
                max_tokens: 2048,
                system_prompt: "You are a helpful assistant. Answer questions clearly and \
                    provide explanations when needed. Include code examples in responses."
                    .to_string(),
                capabilities: vec![Capability::QuestionAnswering, Capability::FreeformChat],
                constraints: ModeConstraints {
                    allow_file_operations: false,
                    allow_command_execution: false,
                    allow_code_generation: false,
                    require_specs: false,
                    auto_think_more_threshold: None,
                },
            },
        }
    }

    /// Create an Ask Mode with custom configuration
    pub fn with_config(config: ModeConfig) -> Self {
        Self { config }
    }

    /// Answer a question clearly
    ///
    /// This method generates a clear answer to the provided question.
    pub fn answer_question(&self, question: &str) -> Result<String> {
        // For now, return a placeholder response
        // In a real implementation, this would call an LLM
        Ok(format!(
            "Question: {}\n\nAnswer: This is a placeholder response. \
            In a real implementation, this would be answered by an LLM.",
            question
        ))
    }

    /// Provide an explanation for a concept
    ///
    /// This method generates an explanation for the given concept.
    pub fn explain_concept(&self, concept: &str) -> Result<String> {
        Ok(format!(
            "Explanation of '{}':\n\n\
            This is a placeholder explanation. \
            In a real implementation, this would provide a detailed explanation.",
            concept
        ))
    }

    /// Include code examples in a response
    ///
    /// This method adds code examples to a response.
    pub fn include_code_examples(&self, response: &str, language: &str) -> Result<String> {
        Ok(format!(
            "{}\n\nCode example ({}):\n```{}\n// Example code\n```",
            response, language, language
        ))
    }

    /// Suggest an approach without executing it
    ///
    /// This method provides guidance on how to approach a problem.
    pub fn suggest_approach(&self, problem: &str) -> Result<String> {
        Ok(format!(
            "Problem: {}\n\n\
            Suggested approach:\n\
            1. Analyze the problem\n\
            2. Break it down into smaller parts\n\
            3. Implement each part\n\
            4. Test and validate\n\n\
            Note: This is a general approach. Specific implementation details \
            would depend on your particular use case.",
            problem
        ))
    }

    /// Validate that an operation is allowed in Ask Mode
    ///
    /// This method checks if an operation can be executed in Ask Mode.
    /// Only question answering operations are allowed.
    pub fn validate_operation(&self, operation: &Operation) -> Result<()> {
        match operation {
            Operation::AnswerQuestion => Ok(()),
            Operation::GenerateCode => Err(crate::error::ModeError::OperationNotAllowed {
                mode: self.id().to_string(),
                operation: operation.to_string(),
            }),
            Operation::ModifyFile => Err(crate::error::ModeError::FileOperationBlocked),
            Operation::ExecuteCommand => Err(crate::error::ModeError::OperationNotAllowed {
                mode: self.id().to_string(),
                operation: operation.to_string(),
            }),
            Operation::RunTests => Err(crate::error::ModeError::OperationNotAllowed {
                mode: self.id().to_string(),
                operation: operation.to_string(),
            }),
            Operation::ValidateQuality => Err(crate::error::ModeError::OperationNotAllowed {
                mode: self.id().to_string(),
                operation: operation.to_string(),
            }),
        }
    }

    /// Provide clear error message for blocked operations
    ///
    /// This method generates a user-friendly error message for blocked operations.
    pub fn blocked_operation_message(&self, operation: &Operation) -> String {
        match operation {
            Operation::ModifyFile => {
                "File operations are not allowed in Ask Mode. \
                Switch to Code Mode if you need to modify files."
                    .to_string()
            }
            Operation::ExecuteCommand => {
                "Command execution is not allowed in Ask Mode. \
                Switch to Code Mode if you need to execute commands."
                    .to_string()
            }
            Operation::GenerateCode => {
                "Code generation is not allowed in Ask Mode. \
                Switch to Code Mode if you need to generate code."
                    .to_string()
            }
            Operation::RunTests => {
                "Test execution is not allowed in Ask Mode. \
                Switch to Code Mode if you need to run tests."
                    .to_string()
            }
            Operation::ValidateQuality => {
                "Quality validation is not allowed in Ask Mode. \
                Switch to Code Mode if you need to validate code quality."
                    .to_string()
            }
            Operation::AnswerQuestion => {
                "This operation should be allowed in Ask Mode.".to_string()
            }
        }
    }
}

impl Default for AskMode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Mode for AskMode {
    fn id(&self) -> &str {
        "ask"
    }

    fn name(&self) -> &str {
        "Ask Mode"
    }

    fn description(&self) -> &str {
        "Question answering and explanations without file modifications"
    }

    fn system_prompt(&self) -> &str {
        &self.config.system_prompt
    }

    async fn process(&self, input: &str, context: &ModeContext) -> Result<ModeResponse> {
        let start = Instant::now();

        // Create response with input as content
        let mut response = ModeResponse::new(input.to_string(), self.id().to_string());

        // Add question answering action
        response.add_action(ModeAction::AskQuestion {
            question: input.to_string(),
        });

        // Add suggestion to include code examples if appropriate
        if input.contains("code") || input.contains("example") {
            response.add_suggestion(
                "I can provide code examples to illustrate the concept.".to_string(),
            );
        }

        // Update metadata
        response.metadata.duration = start.elapsed();
        response.metadata.think_more_used = context.think_more_enabled;

        Ok(response)
    }

    fn capabilities(&self) -> Vec<Capability> {
        self.config.capabilities.clone()
    }

    fn config(&self) -> &ModeConfig {
        &self.config
    }

    fn can_execute(&self, operation: &Operation) -> bool {
        matches!(operation, Operation::AnswerQuestion)
    }

    fn constraints(&self) -> ModeConstraints {
        self.config.constraints.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ask_mode_creation() {
        let mode = AskMode::new();
        assert_eq!(mode.id(), "ask");
        assert_eq!(mode.name(), "Ask Mode");
    }

    #[test]
    fn test_ask_mode_capabilities() {
        let mode = AskMode::new();
        let capabilities = mode.capabilities();
        assert!(capabilities.contains(&Capability::QuestionAnswering));
        assert!(capabilities.contains(&Capability::FreeformChat));
        assert!(!capabilities.contains(&Capability::CodeGeneration));
        assert!(!capabilities.contains(&Capability::FileOperations));
    }

    #[test]
    fn test_ask_mode_can_execute() {
        let mode = AskMode::new();
        assert!(mode.can_execute(&Operation::AnswerQuestion));
        assert!(!mode.can_execute(&Operation::GenerateCode));
        assert!(!mode.can_execute(&Operation::ModifyFile));
        assert!(!mode.can_execute(&Operation::ExecuteCommand));
        assert!(!mode.can_execute(&Operation::RunTests));
        assert!(!mode.can_execute(&Operation::ValidateQuality));
    }

    #[test]
    fn test_ask_mode_constraints() {
        let mode = AskMode::new();
        let constraints = mode.constraints();
        assert!(!constraints.allow_file_operations);
        assert!(!constraints.allow_command_execution);
        assert!(!constraints.allow_code_generation);
        assert!(!constraints.require_specs);
        assert_eq!(constraints.auto_think_more_threshold, None);
    }

    #[test]
    fn test_ask_mode_system_prompt() {
        let mode = AskMode::new();
        let prompt = mode.system_prompt();
        assert!(prompt.contains("helpful assistant"));
        assert!(prompt.contains("Answer questions"));
        assert!(prompt.contains("code examples"));
    }

    #[tokio::test]
    async fn test_ask_mode_process() {
        let mode = AskMode::new();
        let context = ModeContext::new("test-session".to_string());
        let response = mode.process("What is Rust?", &context).await.unwrap();
        assert_eq!(response.content, "What is Rust?");
        assert_eq!(response.metadata.mode, "ask");
        assert!(!response.metadata.think_more_used);
        assert!(!response.actions.is_empty());
    }

    #[tokio::test]
    async fn test_ask_mode_process_with_think_more() {
        let mode = AskMode::new();
        let mut context = ModeContext::new("test-session".to_string());
        context.think_more_enabled = true;
        let response = mode.process("What is Rust?", &context).await.unwrap();
        assert!(response.metadata.think_more_used);
    }

    #[tokio::test]
    async fn test_ask_mode_process_with_code_keyword() {
        let mode = AskMode::new();
        let context = ModeContext::new("test-session".to_string());
        let response = mode.process("Show me a code example", &context).await.unwrap();
        assert!(!response.suggestions.is_empty());
    }

    #[test]
    fn test_ask_mode_default() {
        let mode = AskMode::default();
        assert_eq!(mode.id(), "ask");
    }

    #[test]
    fn test_ask_mode_with_custom_config() {
        let custom_config = ModeConfig {
            temperature: 0.5,
            max_tokens: 1024,
            system_prompt: "Custom prompt".to_string(),
            capabilities: vec![Capability::QuestionAnswering],
            constraints: ModeConstraints {
                allow_file_operations: false,
                allow_command_execution: false,
                allow_code_generation: false,
                require_specs: false,
                auto_think_more_threshold: None,
            },
        };
        let mode = AskMode::with_config(custom_config);
        assert_eq!(mode.config().temperature, 0.5);
        assert_eq!(mode.config().max_tokens, 1024);
    }

    #[test]
    fn test_answer_question() {
        let mode = AskMode::new();
        let question = "What is Rust?";
        let result = mode.answer_question(question);
        assert!(result.is_ok());
        let answer = result.unwrap();
        assert!(answer.contains("Question:"));
        assert!(answer.contains(question));
    }

    #[test]
    fn test_explain_concept() {
        let mode = AskMode::new();
        let concept = "ownership";
        let result = mode.explain_concept(concept);
        assert!(result.is_ok());
        let explanation = result.unwrap();
        assert!(explanation.contains("Explanation"));
        assert!(explanation.contains(concept));
    }

    #[test]
    fn test_include_code_examples() {
        let mode = AskMode::new();
        let response = "Here's how to do it";
        let result = mode.include_code_examples(response, "rust");
        assert!(result.is_ok());
        let with_examples = result.unwrap();
        assert!(with_examples.contains("Code example"));
        assert!(with_examples.contains("rust"));
        assert!(with_examples.contains("```"));
    }

    #[test]
    fn test_suggest_approach() {
        let mode = AskMode::new();
        let problem = "How to parse JSON?";
        let result = mode.suggest_approach(problem);
        assert!(result.is_ok());
        let approach = result.unwrap();
        assert!(approach.contains("Problem:"));
        assert!(approach.contains("Suggested approach"));
        assert!(approach.contains("Analyze"));
    }

    #[test]
    fn test_validate_operation_allowed() {
        let mode = AskMode::new();
        let result = mode.validate_operation(&Operation::AnswerQuestion);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_operation_blocked_generate_code() {
        let mode = AskMode::new();
        let result = mode.validate_operation(&Operation::GenerateCode);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_operation_blocked_modify_file() {
        let mode = AskMode::new();
        let result = mode.validate_operation(&Operation::ModifyFile);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_operation_blocked_execute_command() {
        let mode = AskMode::new();
        let result = mode.validate_operation(&Operation::ExecuteCommand);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_operation_blocked_run_tests() {
        let mode = AskMode::new();
        let result = mode.validate_operation(&Operation::RunTests);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_operation_blocked_validate_quality() {
        let mode = AskMode::new();
        let result = mode.validate_operation(&Operation::ValidateQuality);
        assert!(result.is_err());
    }

    #[test]
    fn test_blocked_operation_message_modify_file() {
        let mode = AskMode::new();
        let message = mode.blocked_operation_message(&Operation::ModifyFile);
        assert!(message.contains("File operations"));
        assert!(message.contains("Ask Mode"));
        assert!(message.contains("Code Mode"));
    }

    #[test]
    fn test_blocked_operation_message_execute_command() {
        let mode = AskMode::new();
        let message = mode.blocked_operation_message(&Operation::ExecuteCommand);
        assert!(message.contains("Command execution"));
        assert!(message.contains("Ask Mode"));
    }

    #[test]
    fn test_blocked_operation_message_generate_code() {
        let mode = AskMode::new();
        let message = mode.blocked_operation_message(&Operation::GenerateCode);
        assert!(message.contains("Code generation"));
        assert!(message.contains("Ask Mode"));
    }

    #[test]
    fn test_blocked_operation_message_run_tests() {
        let mode = AskMode::new();
        let message = mode.blocked_operation_message(&Operation::RunTests);
        assert!(message.contains("Test execution"));
        assert!(message.contains("Ask Mode"));
    }

    #[test]
    fn test_blocked_operation_message_validate_quality() {
        let mode = AskMode::new();
        let message = mode.blocked_operation_message(&Operation::ValidateQuality);
        assert!(message.contains("Quality validation"));
        assert!(message.contains("Ask Mode"));
    }
}
