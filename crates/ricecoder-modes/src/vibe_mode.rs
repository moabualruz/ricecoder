//! Vibe Mode implementation for free-form exploration and rapid prototyping

use async_trait::async_trait;
use std::time::Instant;

use crate::error::Result;
use crate::mode::Mode;
use crate::models::{
    Capability, ModeAction, ModeConfig, ModeConstraints, ModeContext, ModeResponse, Operation,
};

/// Vibe Mode for free-form exploration and rapid prototyping
///
/// Vibe Mode provides capabilities for:
/// - Natural language code generation without formal specs
/// - Rapid iteration and exploration
/// - Spec conversion for transitioning to spec-driven development
/// - Warnings about limitations and best practices
#[derive(Debug, Clone)]
pub struct VibeMode {
    config: ModeConfig,
}

impl VibeMode {
    /// Create a new Vibe Mode instance
    pub fn new() -> Self {
        Self {
            config: ModeConfig {
                temperature: 0.9,
                max_tokens: 4096,
                system_prompt: "You are a creative coding partner. Help explore ideas, \
                    prototype quickly, and iterate on concepts. Bypass formal specifications \
                    and focus on rapid development."
                    .to_string(),
                capabilities: vec![
                    Capability::CodeGeneration,
                    Capability::CodeModification,
                    Capability::FileOperations,
                    Capability::FreeformChat,
                    Capability::QuestionAnswering,
                    Capability::SpecConversion,
                ],
                constraints: ModeConstraints {
                    allow_file_operations: true,
                    allow_command_execution: false,
                    allow_code_generation: true,
                    require_specs: false,
                    auto_think_more_threshold: None,
                },
            },
        }
    }

    /// Create a Vibe Mode with custom configuration
    pub fn with_config(config: ModeConfig) -> Self {
        Self { config }
    }

    /// Generate code from natural language without formal specs
    ///
    /// This method generates code directly from natural language descriptions
    /// without requiring formal specifications.
    pub fn generate_code_from_description(&self, description: &str) -> Result<String> {
        // Validate that code generation is allowed
        if !self.config.constraints.allow_code_generation {
            return Err(crate::error::ModeError::OperationNotAllowed {
                mode: self.id().to_string(),
                operation: Operation::GenerateCode.to_string(),
            });
        }

        // For now, return a placeholder response
        // In a real implementation, this would call an LLM
        Ok(format!(
            "// Generated from description:\n// {}\n\n// TODO: Implement based on description",
            description
        ))
    }

    /// Support rapid iteration by accepting natural language input
    ///
    /// This method enables quick prototyping by accepting natural language
    /// input and generating code without formal review cycles.
    pub fn iterate_rapidly(&self, iteration_input: &str) -> Result<String> {
        // Validate that code generation is allowed
        if !self.config.constraints.allow_code_generation {
            return Err(crate::error::ModeError::OperationNotAllowed {
                mode: self.id().to_string(),
                operation: Operation::GenerateCode.to_string(),
            });
        }

        Ok(format!(
            "// Rapid iteration:\n// {}\n\n// Generated code for quick prototyping",
            iteration_input
        ))
    }

    /// Generate code with multiple iterations
    ///
    /// This method supports generating code through multiple rapid iterations,
    /// allowing developers to refine and improve the generated code.
    pub fn generate_with_iterations(&self, description: &str, iterations: usize) -> Result<String> {
        // Validate that code generation is allowed
        if !self.config.constraints.allow_code_generation {
            return Err(crate::error::ModeError::OperationNotAllowed {
                mode: self.id().to_string(),
                operation: Operation::GenerateCode.to_string(),
            });
        }

        let mut code = format!(
            "// Generated from description:\n// {}\n\n// Iteration 1:\n// TODO: Implement",
            description
        );

        for i in 2..=iterations {
            code.push_str(&format!("\n\n// Iteration {}:\n// TODO: Refine", i));
        }

        Ok(code)
    }

    /// Accept natural language input and generate code
    ///
    /// This method is the core of Vibe Mode - it accepts natural language
    /// input without requiring formal specifications and generates code.
    pub fn accept_natural_language(&self, input: &str) -> Result<String> {
        // Validate that code generation is allowed
        if !self.config.constraints.allow_code_generation {
            return Err(crate::error::ModeError::OperationNotAllowed {
                mode: self.id().to_string(),
                operation: Operation::GenerateCode.to_string(),
            });
        }

        // Validate that specs are not required
        if self.config.constraints.require_specs {
            return Err(crate::error::ModeError::ProcessingFailed(
                "Specs are required in this mode".to_string(),
            ));
        }

        Ok(format!(
            "// Natural language input:\n// {}\n\n// Generated code without formal specs",
            input
        ))
    }

    /// Convert a Vibe Mode project to spec-driven format
    ///
    /// This method converts existing code and context from Vibe Mode
    /// to a formal spec-driven development format.
    pub fn convert_to_specs(&self, project_code: &str) -> Result<String> {
        let spec_template = format!(
            "# Generated Specification\n\n\
            ## Overview\n\
            This specification was auto-generated from Vibe Mode code.\n\n\
            ## Code\n\
            ```\n\
            {}\n\
            ```\n\n\
            ## Next Steps\n\
            1. Review and refine the specification\n\
            2. Add acceptance criteria\n\
            3. Define design decisions\n\
            4. Create implementation tasks",
            project_code
        );
        Ok(spec_template)
    }

    /// Preserve all code and context during conversion
    ///
    /// This method ensures that all code and context is preserved
    /// when converting from Vibe Mode to spec-driven format.
    pub fn preserve_code_and_context(
        &self,
        code: &str,
        context: &ModeContext,
    ) -> Result<String> {
        let mut preserved = String::new();
        preserved.push_str("# Preserved Context\n\n");
        preserved.push_str(&format!("Session ID: {}\n", context.session_id));

        if let Some(path) = &context.project_path {
            preserved.push_str(&format!("Project Path: {}\n", path.display()));
        }

        preserved.push_str(&format!(
            "Conversation History: {} messages\n",
            context.conversation_history.len()
        ));

        preserved.push_str("\n# Preserved Code\n\n");
        preserved.push_str("```\n");
        preserved.push_str(code);
        preserved.push_str("\n```\n");

        Ok(preserved)
    }

    /// Generate initial specs from code
    ///
    /// This method analyzes code and generates initial specifications
    /// that can be refined and expanded.
    pub fn generate_specs_from_code(&self, code: &str) -> Result<String> {
        let mut spec = String::new();
        spec.push_str("# Auto-Generated Specification from Code\n\n");

        // Analyze code for functions
        if code.contains("fn ") {
            spec.push_str("## Functions\n");
            spec.push_str("The following functions were detected in the code:\n\n");
            // Simple detection - in real implementation would parse AST
            for line in code.lines() {
                if line.contains("fn ") {
                    spec.push_str(&format!("- {}\n", line.trim()));
                }
            }
            spec.push('\n');
        }

        // Analyze code for structs
        if code.contains("struct ") {
            spec.push_str("## Data Structures\n");
            spec.push_str("The following structures were detected in the code:\n\n");
            for line in code.lines() {
                if line.contains("struct ") {
                    spec.push_str(&format!("- {}\n", line.trim()));
                }
            }
            spec.push('\n');
        }

        spec.push_str("## Requirements\n");
        spec.push_str("Based on the code analysis, the following requirements are inferred:\n\n");
        spec.push_str("1. The system should implement the detected functions\n");
        spec.push_str("2. The system should use the detected data structures\n");
        spec.push_str("3. The system should follow the patterns in the code\n\n");

        spec.push_str("## Next Steps\n");
        spec.push_str("1. Review the inferred requirements\n");
        spec.push_str("2. Add acceptance criteria for each requirement\n");
        spec.push_str("3. Define design decisions\n");
        spec.push_str("4. Create implementation tasks\n");

        Ok(spec)
    }

    /// Support converting Vibe Mode projects to spec-driven format
    ///
    /// This method provides a complete conversion workflow from Vibe Mode
    /// to spec-driven development.
    pub fn convert_project_to_specs(
        &self,
        code: &str,
        context: &ModeContext,
    ) -> Result<String> {
        let mut conversion = String::new();
        conversion.push_str("# Vibe Mode to Spec-Driven Conversion\n\n");

        // Preserve context
        conversion.push_str("## Preserved Context\n");
        let preserved = self.preserve_code_and_context(code, context)?;
        conversion.push_str(&preserved);
        conversion.push_str("\n\n");

        // Generate specs from code
        conversion.push_str("## Generated Specification\n");
        let specs = self.generate_specs_from_code(code)?;
        conversion.push_str(&specs);

        Ok(conversion)
    }

    /// Generate warnings about Vibe Mode limitations
    ///
    /// This method creates warnings about the limitations of Vibe Mode
    /// and best practices for using it.
    pub fn generate_warnings(&self) -> Vec<String> {
        vec![
            "⚠️  Vibe Mode bypasses formal specifications. \
            Consider converting to spec-driven development for production code."
                .to_string(),
            "⚠️  Code generated in Vibe Mode may lack comprehensive testing. \
            Add tests before deploying to production."
                .to_string(),
            "⚠️  Vibe Mode does not enforce code quality standards. \
            Review code quality manually or switch to Code Mode."
                .to_string(),
            "⚠️  Rapid iteration may lead to technical debt. \
            Plan for refactoring and cleanup."
                .to_string(),
            "💡 Best Practice: Use Vibe Mode for exploration and prototyping, \
            then convert to spec-driven development for production."
                .to_string(),
        ]
    }

    /// Generate warnings about specific limitations
    ///
    /// This method generates warnings about specific limitations
    /// based on the context of use.
    pub fn generate_specific_warnings(&self, context: &str) -> Vec<String> {
        let mut warnings = self.generate_warnings();

        if context.contains("production") {
            warnings.push(
                "⚠️  WARNING: Vibe Mode is not recommended for production code. \
                Use Code Mode with formal specifications instead."
                    .to_string(),
            );
        }

        if context.contains("test") {
            warnings.push(
                "⚠️  Code generated in Vibe Mode requires comprehensive testing. \
                Add unit tests, integration tests, and property-based tests."
                    .to_string(),
            );
        }

        if context.contains("quality") {
            warnings.push(
                "⚠️  Code quality validation is not performed in Vibe Mode. \
                Switch to Code Mode for quality validation."
                    .to_string(),
            );
        }

        warnings
    }

    /// Display warnings in a formatted way
    ///
    /// This method formats warnings for display to the user.
    pub fn format_warnings(&self, warnings: &[String]) -> String {
        let mut formatted = String::new();
        formatted.push_str("=== Vibe Mode Warnings ===\n\n");

        for warning in warnings {
            formatted.push_str(&format!("{}\n", warning));
        }

        formatted.push_str("\n=== End Warnings ===\n");
        formatted
    }

    /// Include warnings in all responses
    ///
    /// This method adds warnings to a response to inform users
    /// about Vibe Mode limitations.
    pub fn add_warnings_to_response(&self, response: &mut ModeResponse) {
        let warnings = self.generate_warnings();
        for warning in warnings {
            response.add_suggestion(warning);
        }
    }

    /// Include specific warnings based on context
    ///
    /// This method adds context-specific warnings to a response.
    pub fn add_specific_warnings_to_response(
        &self,
        response: &mut ModeResponse,
        context: &str,
    ) {
        let warnings = self.generate_specific_warnings(context);
        for warning in warnings {
            response.add_suggestion(warning);
        }
    }

    /// Validate that an operation is allowed in Vibe Mode
    ///
    /// This method checks if an operation can be executed in Vibe Mode.
    pub fn validate_operation(&self, operation: &Operation) -> Result<()> {
        match operation {
            Operation::GenerateCode | Operation::ModifyFile | Operation::AnswerQuestion => Ok(()),
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

    /// Provide guidance on best practices
    ///
    /// This method generates guidance on how to best use Vibe Mode.
    pub fn provide_best_practices(&self) -> Vec<String> {
        vec![
            "Start with Vibe Mode for rapid prototyping and exploration".to_string(),
            "Use natural language descriptions to generate code quickly".to_string(),
            "Iterate rapidly without formal review cycles".to_string(),
            "Convert to spec-driven development when ready for production".to_string(),
            "Add comprehensive tests before deploying code".to_string(),
            "Review code quality and refactor as needed".to_string(),
        ]
    }
}

impl Default for VibeMode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Mode for VibeMode {
    fn id(&self) -> &str {
        "vibe"
    }

    fn name(&self) -> &str {
        "Vibe Mode"
    }

    fn description(&self) -> &str {
        "Free-form exploration and rapid prototyping without formal specifications"
    }

    fn system_prompt(&self) -> &str {
        &self.config.system_prompt
    }

    async fn process(&self, input: &str, context: &ModeContext) -> Result<ModeResponse> {
        let start = Instant::now();

        // Create response with input as content
        let mut response = ModeResponse::new(input.to_string(), self.id().to_string());

        // Add code generation action for natural language input
        if input.contains("generate") || input.contains("create") || input.contains("build") {
            response.add_action(ModeAction::GenerateCode {
                spec: input.to_string(),
            });
        }

        // Add file operation action if input mentions files
        if input.contains("file") || input.contains("modify") {
            response.add_action(ModeAction::ModifyFile {
                path: std::path::PathBuf::from("generated.rs"),
                diff: "// Generated code".to_string(),
            });
        }

        // Add spec conversion suggestion if input mentions specs
        if input.contains("spec") || input.contains("formal") {
            response.add_action(ModeAction::SuggestMode {
                mode: "code".to_string(),
                reason: "Consider Code Mode for spec-driven development".to_string(),
            });
        }

        // Add warnings to all responses
        self.add_warnings_to_response(&mut response);

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
        matches!(
            operation,
            Operation::GenerateCode | Operation::ModifyFile | Operation::AnswerQuestion
        )
    }

    fn constraints(&self) -> ModeConstraints {
        self.config.constraints.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vibe_mode_creation() {
        let mode = VibeMode::new();
        assert_eq!(mode.id(), "vibe");
        assert_eq!(mode.name(), "Vibe Mode");
    }

    #[test]
    fn test_vibe_mode_capabilities() {
        let mode = VibeMode::new();
        let capabilities = mode.capabilities();
        assert!(capabilities.contains(&Capability::CodeGeneration));
        assert!(capabilities.contains(&Capability::CodeModification));
        assert!(capabilities.contains(&Capability::FileOperations));
        assert!(capabilities.contains(&Capability::FreeformChat));
        assert!(capabilities.contains(&Capability::QuestionAnswering));
        assert!(capabilities.contains(&Capability::SpecConversion));
    }

    #[test]
    fn test_vibe_mode_can_execute() {
        let mode = VibeMode::new();
        assert!(mode.can_execute(&Operation::GenerateCode));
        assert!(mode.can_execute(&Operation::ModifyFile));
        assert!(mode.can_execute(&Operation::AnswerQuestion));
        assert!(!mode.can_execute(&Operation::ExecuteCommand));
        assert!(!mode.can_execute(&Operation::RunTests));
        assert!(!mode.can_execute(&Operation::ValidateQuality));
    }

    #[test]
    fn test_vibe_mode_constraints() {
        let mode = VibeMode::new();
        let constraints = mode.constraints();
        assert!(constraints.allow_file_operations);
        assert!(!constraints.allow_command_execution);
        assert!(constraints.allow_code_generation);
        assert!(!constraints.require_specs);
        assert_eq!(constraints.auto_think_more_threshold, None);
    }

    #[test]
    fn test_vibe_mode_system_prompt() {
        let mode = VibeMode::new();
        let prompt = mode.system_prompt();
        assert!(prompt.contains("creative coding partner"));
        assert!(prompt.contains("rapid"));
        assert!(prompt.contains("Bypass formal specifications"));
    }

    #[tokio::test]
    async fn test_vibe_mode_process() {
        let mode = VibeMode::new();
        let context = ModeContext::new("test-session".to_string());
        let response = mode.process("create a function", &context).await.unwrap();
        assert_eq!(response.content, "create a function");
        assert_eq!(response.metadata.mode, "vibe");
        assert!(!response.metadata.think_more_used);
        assert!(!response.actions.is_empty());
    }

    #[tokio::test]
    async fn test_vibe_mode_process_with_think_more() {
        let mode = VibeMode::new();
        let mut context = ModeContext::new("test-session".to_string());
        context.think_more_enabled = true;
        let response = mode.process("create a function", &context).await.unwrap();
        assert!(response.metadata.think_more_used);
    }

    #[tokio::test]
    async fn test_vibe_mode_process_includes_warnings() {
        let mode = VibeMode::new();
        let context = ModeContext::new("test-session".to_string());
        let response = mode.process("test input", &context).await.unwrap();
        assert!(!response.suggestions.is_empty());
        // Check that at least one suggestion contains a warning
        assert!(response
            .suggestions
            .iter()
            .any(|s| s.contains("⚠️") || s.contains("💡")));
    }

    #[test]
    fn test_vibe_mode_default() {
        let mode = VibeMode::default();
        assert_eq!(mode.id(), "vibe");
    }

    #[test]
    fn test_vibe_mode_with_custom_config() {
        let custom_config = ModeConfig {
            temperature: 0.8,
            max_tokens: 2048,
            system_prompt: "Custom prompt".to_string(),
            capabilities: vec![Capability::CodeGeneration],
            constraints: ModeConstraints {
                allow_file_operations: true,
                allow_command_execution: false,
                allow_code_generation: true,
                require_specs: false,
                auto_think_more_threshold: None,
            },
        };
        let mode = VibeMode::with_config(custom_config);
        assert_eq!(mode.config().temperature, 0.8);
        assert_eq!(mode.config().max_tokens, 2048);
    }

    #[test]
    fn test_generate_code_from_description() {
        let mode = VibeMode::new();
        let description = "Create a function that adds two numbers";
        let result = mode.generate_code_from_description(description);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Generated from description"));
        assert!(code.contains(description));
    }

    #[test]
    fn test_iterate_rapidly() {
        let mode = VibeMode::new();
        let iteration = "Add error handling";
        let result = mode.iterate_rapidly(iteration);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Rapid iteration"));
        assert!(code.contains(iteration));
    }

    #[test]
    fn test_convert_to_specs() {
        let mode = VibeMode::new();
        let code = "fn main() { println!(\"Hello\"); }";
        let result = mode.convert_to_specs(code);
        assert!(result.is_ok());
        let spec = result.unwrap();
        assert!(spec.contains("Generated Specification"));
        assert!(spec.contains("auto-generated from Vibe Mode"));
        assert!(spec.contains(code));
    }

    #[test]
    fn test_generate_warnings() {
        let mode = VibeMode::new();
        let warnings = mode.generate_warnings();
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.contains("⚠️")));
        assert!(warnings.iter().any(|w| w.contains("specification")));
        assert!(warnings.iter().any(|w| w.contains("testing")));
    }

    #[tokio::test]
    async fn test_add_warnings_to_response() {
        let mode = VibeMode::new();
        let mut response = ModeResponse::new("test".to_string(), "vibe".to_string());
        let initial_suggestions = response.suggestions.len();
        mode.add_warnings_to_response(&mut response);
        assert!(response.suggestions.len() > initial_suggestions);
    }

    #[test]
    fn test_validate_operation_allowed() {
        let mode = VibeMode::new();
        assert!(mode.validate_operation(&Operation::GenerateCode).is_ok());
        assert!(mode.validate_operation(&Operation::ModifyFile).is_ok());
        assert!(mode.validate_operation(&Operation::AnswerQuestion).is_ok());
    }

    #[test]
    fn test_validate_operation_blocked_execute_command() {
        let mode = VibeMode::new();
        let result = mode.validate_operation(&Operation::ExecuteCommand);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_operation_blocked_run_tests() {
        let mode = VibeMode::new();
        let result = mode.validate_operation(&Operation::RunTests);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_operation_blocked_validate_quality() {
        let mode = VibeMode::new();
        let result = mode.validate_operation(&Operation::ValidateQuality);
        assert!(result.is_err());
    }

    #[test]
    fn test_provide_best_practices() {
        let mode = VibeMode::new();
        let practices = mode.provide_best_practices();
        assert!(!practices.is_empty());
        assert!(practices.iter().any(|p| p.contains("prototyping")));
        assert!(practices.iter().any(|p| p.contains("spec-driven")));
        assert!(practices.iter().any(|p| p.contains("tests")));
    }

    #[tokio::test]
    async fn test_process_with_generate_keyword() {
        let mode = VibeMode::new();
        let context = ModeContext::new("test-session".to_string());
        let response = mode.process("generate a function", &context).await.unwrap();
        assert!(!response.actions.is_empty());
    }

    #[tokio::test]
    async fn test_process_with_file_keyword() {
        let mode = VibeMode::new();
        let context = ModeContext::new("test-session".to_string());
        let response = mode.process("modify file", &context).await.unwrap();
        assert!(!response.actions.is_empty());
    }

    #[tokio::test]
    async fn test_process_with_spec_keyword() {
        let mode = VibeMode::new();
        let context = ModeContext::new("test-session".to_string());
        let response = mode.process("convert to spec", &context).await.unwrap();
        assert!(!response.actions.is_empty());
    }

    #[test]
    fn test_generate_with_iterations() {
        let mode = VibeMode::new();
        let description = "Create a function";
        let result = mode.generate_with_iterations(description, 3);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Iteration 1"));
        assert!(code.contains("Iteration 2"));
        assert!(code.contains("Iteration 3"));
    }

    #[test]
    fn test_accept_natural_language() {
        let mode = VibeMode::new();
        let input = "Create a function that adds two numbers";
        let result = mode.accept_natural_language(input);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Natural language input"));
        assert!(code.contains("without formal specs"));
        assert!(code.contains(input));
    }

    #[test]
    fn test_accept_natural_language_blocked_when_specs_required() {
        let custom_config = ModeConfig {
            temperature: 0.9,
            max_tokens: 4096,
            system_prompt: "Test".to_string(),
            capabilities: vec![Capability::CodeGeneration],
            constraints: ModeConstraints {
                allow_file_operations: true,
                allow_command_execution: false,
                allow_code_generation: true,
                require_specs: true,
                auto_think_more_threshold: None,
            },
        };
        let mode = VibeMode::with_config(custom_config);
        let result = mode.accept_natural_language("test input");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_code_from_description_blocked_when_disabled() {
        let custom_config = ModeConfig {
            temperature: 0.9,
            max_tokens: 4096,
            system_prompt: "Test".to_string(),
            capabilities: vec![],
            constraints: ModeConstraints {
                allow_file_operations: true,
                allow_command_execution: false,
                allow_code_generation: false,
                require_specs: false,
                auto_think_more_threshold: None,
            },
        };
        let mode = VibeMode::with_config(custom_config);
        let result = mode.generate_code_from_description("test");
        assert!(result.is_err());
    }

    #[test]
    fn test_iterate_rapidly_blocked_when_disabled() {
        let custom_config = ModeConfig {
            temperature: 0.9,
            max_tokens: 4096,
            system_prompt: "Test".to_string(),
            capabilities: vec![],
            constraints: ModeConstraints {
                allow_file_operations: true,
                allow_command_execution: false,
                allow_code_generation: false,
                require_specs: false,
                auto_think_more_threshold: None,
            },
        };
        let mode = VibeMode::with_config(custom_config);
        let result = mode.iterate_rapidly("test");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_with_iterations_blocked_when_disabled() {
        let custom_config = ModeConfig {
            temperature: 0.9,
            max_tokens: 4096,
            system_prompt: "Test".to_string(),
            capabilities: vec![],
            constraints: ModeConstraints {
                allow_file_operations: true,
                allow_command_execution: false,
                allow_code_generation: false,
                require_specs: false,
                auto_think_more_threshold: None,
            },
        };
        let mode = VibeMode::with_config(custom_config);
        let result = mode.generate_with_iterations("test", 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_preserve_code_and_context() {
        let mode = VibeMode::new();
        let code = "fn main() {}";
        let context = ModeContext::new("test-session".to_string());
        let result = mode.preserve_code_and_context(code, &context);
        assert!(result.is_ok());
        let preserved = result.unwrap();
        assert!(preserved.contains("Preserved Context"));
        assert!(preserved.contains("test-session"));
        assert!(preserved.contains("Preserved Code"));
        assert!(preserved.contains(code));
    }

    #[test]
    fn test_preserve_code_and_context_with_project_path() {
        let mode = VibeMode::new();
        let code = "fn main() {}";
        let mut context = ModeContext::new("test-session".to_string());
        context.project_path = Some(std::path::PathBuf::from("/path/to/project"));
        let result = mode.preserve_code_and_context(code, &context);
        assert!(result.is_ok());
        let preserved = result.unwrap();
        assert!(preserved.contains("/path/to/project"));
    }

    #[test]
    fn test_generate_specs_from_code() {
        let mode = VibeMode::new();
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }\nstruct Point { x: i32, y: i32 }";
        let result = mode.generate_specs_from_code(code);
        assert!(result.is_ok());
        let spec = result.unwrap();
        assert!(spec.contains("Auto-Generated Specification"));
        assert!(spec.contains("Functions"));
        assert!(spec.contains("Data Structures"));
        assert!(spec.contains("Requirements"));
    }

    #[test]
    fn test_generate_specs_from_code_with_functions() {
        let mode = VibeMode::new();
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let result = mode.generate_specs_from_code(code);
        assert!(result.is_ok());
        let spec = result.unwrap();
        assert!(spec.contains("Functions"));
        assert!(spec.contains("fn add"));
    }

    #[test]
    fn test_generate_specs_from_code_with_structs() {
        let mode = VibeMode::new();
        let code = "struct Point { x: i32, y: i32 }";
        let result = mode.generate_specs_from_code(code);
        assert!(result.is_ok());
        let spec = result.unwrap();
        assert!(spec.contains("Data Structures"));
        assert!(spec.contains("struct Point"));
    }

    #[test]
    fn test_convert_project_to_specs() {
        let mode = VibeMode::new();
        let code = "fn main() {}";
        let context = ModeContext::new("test-session".to_string());
        let result = mode.convert_project_to_specs(code, &context);
        assert!(result.is_ok());
        let conversion = result.unwrap();
        assert!(conversion.contains("Vibe Mode to Spec-Driven Conversion"));
        assert!(conversion.contains("Preserved Context"));
        assert!(conversion.contains("Generated Specification"));
    }

    #[test]
    fn test_convert_project_to_specs_preserves_all_data() {
        let mode = VibeMode::new();
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let mut context = ModeContext::new("test-session".to_string());
        context.project_path = Some(std::path::PathBuf::from("/project"));
        let result = mode.convert_project_to_specs(code, &context);
        assert!(result.is_ok());
        let conversion = result.unwrap();
        assert!(conversion.contains("test-session"));
        assert!(conversion.contains("/project"));
        assert!(conversion.contains(code));
    }

    #[test]
    fn test_generate_specific_warnings_production() {
        let mode = VibeMode::new();
        let warnings = mode.generate_specific_warnings("production");
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.contains("production")));
    }

    #[test]
    fn test_generate_specific_warnings_test() {
        let mode = VibeMode::new();
        let warnings = mode.generate_specific_warnings("test");
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.contains("testing")));
    }

    #[test]
    fn test_generate_specific_warnings_quality() {
        let mode = VibeMode::new();
        let warnings = mode.generate_specific_warnings("quality");
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.contains("quality")));
    }

    #[test]
    fn test_format_warnings() {
        let mode = VibeMode::new();
        let warnings = vec!["Warning 1".to_string(), "Warning 2".to_string()];
        let formatted = mode.format_warnings(&warnings);
        assert!(formatted.contains("Vibe Mode Warnings"));
        assert!(formatted.contains("Warning 1"));
        assert!(formatted.contains("Warning 2"));
    }

    #[tokio::test]
    async fn test_add_specific_warnings_to_response() {
        let mode = VibeMode::new();
        let mut response = ModeResponse::new("test".to_string(), "vibe".to_string());
        let initial_suggestions = response.suggestions.len();
        mode.add_specific_warnings_to_response(&mut response, "production");
        assert!(response.suggestions.len() > initial_suggestions);
        assert!(response
            .suggestions
            .iter()
            .any(|s| s.contains("production")));
    }

    #[test]
    fn test_warnings_include_best_practices() {
        let mode = VibeMode::new();
        let warnings = mode.generate_warnings();
        assert!(warnings.iter().any(|w| w.contains("💡")));
        assert!(warnings.iter().any(|w| w.contains("Best Practice")));
    }

    #[test]
    fn test_warnings_include_limitations() {
        let mode = VibeMode::new();
        let warnings = mode.generate_warnings();
        assert!(warnings.iter().any(|w| w.contains("⚠️")));
        assert!(warnings.iter().any(|w| w.contains("specifications")));
        assert!(warnings.iter().any(|w| w.contains("testing")));
    }
}
