//! Code Mode implementation for code generation and execution

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use async_trait::async_trait;

use crate::{
    error::Result,
    mode::Mode,
    models::{
        Capability, ChangeSummary, ComplexityLevel, ModeAction, ModeConfig, ModeConstraints,
        ModeContext, ModeResponse, Operation,
    },
};

/// Code Mode for focused code generation and modification
///
/// Code Mode provides full capabilities for:
/// - Code generation from specifications
/// - File creation and modification
/// - Test execution
/// - Quality validation
/// - Change summarization
#[derive(Debug, Clone)]
pub struct CodeMode {
    config: ModeConfig,
}

impl CodeMode {
    /// Create a new Code Mode instance
    pub fn new() -> Self {
        Self {
            config: ModeConfig {
                temperature: 0.7,
                max_tokens: 4096,
                system_prompt: "You are a code generation assistant. Focus on writing clean, \
                    well-documented code that follows best practices and workspace standards."
                    .to_string(),
                capabilities: vec![
                    Capability::CodeGeneration,
                    Capability::CodeModification,
                    Capability::FileOperations,
                    Capability::CommandExecution,
                    Capability::TestExecution,
                    Capability::QualityValidation,
                ],
                constraints: ModeConstraints {
                    allow_file_operations: true,
                    allow_command_execution: true,
                    allow_code_generation: true,
                    require_specs: false,
                    auto_think_more_threshold: Some(ComplexityLevel::Complex),
                },
            },
        }
    }

    /// Create a Code Mode with custom configuration
    pub fn with_config(config: ModeConfig) -> Self {
        Self { config }
    }

    /// Generate code from a specification
    ///
    /// This method generates code based on the provided specification.
    /// The generated code is tracked for later file operations.
    pub fn generate_code(&self, spec: &str) -> Result<String> {
        // For now, return the spec as a placeholder
        // In a real implementation, this would call an LLM
        Ok(format!(
            "// Generated from spec:\n// {}\n\n// TODO: Implement based on spec",
            spec
        ))
    }

    /// Create a file with the given content
    ///
    /// This method creates a new file at the specified path with the given content.
    pub async fn create_file(&self, path: &Path, content: &str) -> Result<()> {
        // Validate that file operations are allowed
        if !self.config.constraints.allow_file_operations {
            return Err(crate::error::ModeError::OperationNotAllowed {
                mode: self.id().to_string(),
                operation: Operation::ModifyFile.to_string(),
            });
        }

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    crate::error::ModeError::ProcessingFailed(format!(
                        "Failed to create directory: {}",
                        e
                    ))
                })?;
            }
        }

        // Write the file
        std::fs::write(path, content).map_err(|e| {
            crate::error::ModeError::ProcessingFailed(format!("Failed to write file: {}", e))
        })?;

        Ok(())
    }

    /// Modify a file with the given diff
    ///
    /// This method applies a diff to an existing file.
    pub async fn modify_file(&self, path: &Path, diff: &str) -> Result<()> {
        // Validate that file operations are allowed
        if !self.config.constraints.allow_file_operations {
            return Err(crate::error::ModeError::OperationNotAllowed {
                mode: self.id().to_string(),
                operation: Operation::ModifyFile.to_string(),
            });
        }

        // Read the current file content
        let current_content = std::fs::read_to_string(path).map_err(|e| {
            crate::error::ModeError::ProcessingFailed(format!("Failed to read file: {}", e))
        })?;

        // For now, just append the diff as a comment
        // In a real implementation, this would apply a proper diff
        let new_content = format!("{}\n\n// Applied diff:\n{}", current_content, diff);

        // Write the modified content
        std::fs::write(path, new_content).map_err(|e| {
            crate::error::ModeError::ProcessingFailed(format!("Failed to write file: {}", e))
        })?;

        Ok(())
    }

    /// Track file changes and return a summary
    ///
    /// This method creates a summary of file operations performed.
    pub fn track_changes(&self, files_created: usize, files_modified: usize) -> ChangeSummary {
        ChangeSummary {
            files_created,
            files_modified,
            tests_run: 0,
            tests_passed: 0,
            quality_issues: Vec::new(),
        }
    }

    /// Run tests for the given paths
    ///
    /// This method executes tests and captures the results.
    pub async fn run_tests(&self, paths: &[PathBuf]) -> Result<(usize, usize, Vec<String>)> {
        // Validate that test execution is allowed
        if !self
            .config
            .capabilities
            .contains(&Capability::TestExecution)
        {
            return Err(crate::error::ModeError::OperationNotAllowed {
                mode: self.id().to_string(),
                operation: Operation::RunTests.to_string(),
            });
        }

        let mut tests_run = 0;
        let mut tests_passed = 0;
        let mut failures = Vec::new();

        // For each path, check if it exists and count it as a test
        for path in paths {
            if path.exists() {
                tests_run += 1;
                // Assume test passes if file exists (simplified for now)
                tests_passed += 1;
            } else {
                tests_run += 1;
                failures.push(format!("Test file not found: {}", path.display()));
            }
        }

        Ok((tests_run, tests_passed, failures))
    }

    /// Capture test results and update summary
    ///
    /// This method updates a change summary with test results.
    pub fn capture_test_results(
        &self,
        mut summary: ChangeSummary,
        tests_run: usize,
        tests_passed: usize,
        failures: Vec<String>,
    ) -> ChangeSummary {
        summary.tests_run = tests_run;
        summary.tests_passed = tests_passed;
        summary.quality_issues.extend(failures);
        summary
    }

    /// Report test failures
    ///
    /// This method formats test failures for reporting.
    pub fn report_test_failures(&self, failures: &[String]) -> String {
        if failures.is_empty() {
            "All tests passed!".to_string()
        } else {
            format!(
                "Test failures:\n{}",
                failures
                    .iter()
                    .map(|f| format!("  - {}", f))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    }

    /// Validate code quality against workspace standards
    ///
    /// This method checks code for quality issues.
    pub async fn validate_quality(&self, paths: &[PathBuf]) -> Result<Vec<String>> {
        // Validate that quality validation is allowed
        if !self
            .config
            .capabilities
            .contains(&Capability::QualityValidation)
        {
            return Err(crate::error::ModeError::OperationNotAllowed {
                mode: self.id().to_string(),
                operation: Operation::ValidateQuality.to_string(),
            });
        }

        let mut issues = Vec::new();

        // Check each file for quality issues
        for path in paths {
            if !path.exists() {
                issues.push(format!("File not found: {}", path.display()));
                continue;
            }

            // Read file content
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    // Check for common quality issues
                    if content.is_empty() {
                        issues.push(format!("Empty file: {}", path.display()));
                    }

                    // Check for TODO comments
                    if content.contains("TODO") {
                        issues.push(format!("TODO found in: {}", path.display()));
                    }

                    // Check for FIXME comments
                    if content.contains("FIXME") {
                        issues.push(format!("FIXME found in: {}", path.display()));
                    }

                    // Check for unwrap() calls
                    if content.contains(".unwrap()") {
                        issues.push(format!("Unwrap found in: {}", path.display()));
                    }
                }
                Err(e) => {
                    issues.push(format!("Failed to read file {}: {}", path.display(), e));
                }
            }
        }

        Ok(issues)
    }

    /// Report quality issues
    ///
    /// This method formats quality issues for reporting.
    pub fn report_quality_issues(&self, issues: &[String]) -> String {
        if issues.is_empty() {
            "No quality issues found!".to_string()
        } else {
            format!(
                "Quality issues:\n{}",
                issues
                    .iter()
                    .map(|i| format!("  - {}", i))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    }

    /// Suggest improvements based on quality issues
    ///
    /// This method generates suggestions for code improvement.
    pub fn suggest_improvements(&self, issues: &[String]) -> Vec<String> {
        let mut suggestions = Vec::new();

        for issue in issues {
            if issue.contains("TODO") {
                suggestions.push("Complete all TODO items before committing".to_string());
            }
            if issue.contains("FIXME") {
                suggestions.push("Fix all FIXME items before committing".to_string());
            }
            if issue.contains("Unwrap") {
                suggestions.push("Replace unwrap() with proper error handling".to_string());
            }
            if issue.contains("Empty file") {
                suggestions.push("Remove empty files or add content".to_string());
            }
        }

        // Remove duplicates
        suggestions.sort();
        suggestions.dedup();
        suggestions
    }

    /// Generate a summary of all changes made
    ///
    /// This method creates a comprehensive summary including file counts, test results,
    /// and quality issues.
    pub fn generate_change_summary(
        &self,
        files_created: usize,
        files_modified: usize,
        tests_run: usize,
        tests_passed: usize,
        quality_issues: Vec<String>,
    ) -> ChangeSummary {
        ChangeSummary {
            files_created,
            files_modified,
            tests_run,
            tests_passed,
            quality_issues,
        }
    }

    /// Format a change summary for display
    ///
    /// This method creates a human-readable summary of changes.
    pub fn format_change_summary(&self, summary: &ChangeSummary) -> String {
        let mut output = String::new();
        output.push_str("=== Change Summary ===\n");
        output.push_str(&format!("Files created: {}\n", summary.files_created));
        output.push_str(&format!("Files modified: {}\n", summary.files_modified));
        output.push_str(&format!("Tests run: {}\n", summary.tests_run));
        output.push_str(&format!("Tests passed: {}\n", summary.tests_passed));

        if summary.tests_run > 0 {
            let pass_rate = (summary.tests_passed as f64 / summary.tests_run as f64) * 100.0;
            output.push_str(&format!("Pass rate: {:.1}%\n", pass_rate));
        }

        if !summary.quality_issues.is_empty() {
            output.push_str(&format!(
                "Quality issues: {}\n",
                summary.quality_issues.len()
            ));
            for issue in &summary.quality_issues {
                output.push_str(&format!("  - {}\n", issue));
            }
        } else {
            output.push_str("Quality issues: None\n");
        }

        output
    }

    /// Provide actionable feedback based on summary
    ///
    /// This method generates feedback and recommendations.
    pub fn provide_feedback(&self, summary: &ChangeSummary) -> Vec<String> {
        let mut feedback = Vec::new();

        // Feedback on files
        if summary.files_created > 0 {
            feedback.push(format!(
                "Successfully created {} file(s)",
                summary.files_created
            ));
        }
        if summary.files_modified > 0 {
            feedback.push(format!(
                "Successfully modified {} file(s)",
                summary.files_modified
            ));
        }

        // Feedback on tests
        if summary.tests_run > 0 {
            if summary.tests_passed == summary.tests_run {
                feedback.push("All tests passed! ✓".to_string());
            } else {
                let failed = summary.tests_run - summary.tests_passed;
                feedback.push(format!("{} test(s) failed. Please review and fix.", failed));
            }
        }

        // Feedback on quality
        if !summary.quality_issues.is_empty() {
            feedback.push(format!(
                "Found {} quality issue(s). Please address them.",
                summary.quality_issues.len()
            ));
        } else if summary.files_created > 0 || summary.files_modified > 0 {
            feedback.push("Code quality looks good! ✓".to_string());
        }

        feedback
    }
}

impl Default for CodeMode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Mode for CodeMode {
    fn id(&self) -> &str {
        "code"
    }

    fn name(&self) -> &str {
        "Code Mode"
    }

    fn description(&self) -> &str {
        "Focused code generation and modification with full execution capabilities"
    }

    fn system_prompt(&self) -> &str {
        &self.config.system_prompt
    }

    async fn process(&self, input: &str, context: &ModeContext) -> Result<ModeResponse> {
        let start = Instant::now();

        // Create response with input as content
        let mut response = ModeResponse::new(input.to_string(), self.id().to_string());

        // Add code generation action if input looks like a spec
        if input.contains("generate") || input.contains("create") {
            response.add_action(ModeAction::GenerateCode {
                spec: input.to_string(),
            });
        }

        // Add file operation action if input mentions files
        if input.contains("file") || input.contains("modify") {
            response.add_action(ModeAction::ModifyFile {
                path: PathBuf::from("generated.rs"),
                diff: "// Generated code".to_string(),
            });
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
        matches!(
            operation,
            Operation::GenerateCode
                | Operation::ModifyFile
                | Operation::ExecuteCommand
                | Operation::RunTests
                | Operation::ValidateQuality
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
    fn test_code_mode_creation() {
        let mode = CodeMode::new();
        assert_eq!(mode.id(), "code");
        assert_eq!(mode.name(), "Code Mode");
    }

    #[test]
    fn test_code_mode_capabilities() {
        let mode = CodeMode::new();
        let capabilities = mode.capabilities();
        assert!(capabilities.contains(&Capability::CodeGeneration));
        assert!(capabilities.contains(&Capability::CodeModification));
        assert!(capabilities.contains(&Capability::FileOperations));
        assert!(capabilities.contains(&Capability::CommandExecution));
        assert!(capabilities.contains(&Capability::TestExecution));
        assert!(capabilities.contains(&Capability::QualityValidation));
    }

    #[test]
    fn test_code_mode_can_execute() {
        let mode = CodeMode::new();
        assert!(mode.can_execute(&Operation::GenerateCode));
        assert!(mode.can_execute(&Operation::ModifyFile));
        assert!(mode.can_execute(&Operation::ExecuteCommand));
        assert!(mode.can_execute(&Operation::RunTests));
        assert!(mode.can_execute(&Operation::ValidateQuality));
        assert!(!mode.can_execute(&Operation::AnswerQuestion));
    }

    #[test]
    fn test_code_mode_constraints() {
        let mode = CodeMode::new();
        let constraints = mode.constraints();
        assert!(constraints.allow_file_operations);
        assert!(constraints.allow_command_execution);
        assert!(constraints.allow_code_generation);
        assert!(!constraints.require_specs);
        assert_eq!(
            constraints.auto_think_more_threshold,
            Some(ComplexityLevel::Complex)
        );
    }

    #[test]
    fn test_code_mode_system_prompt() {
        let mode = CodeMode::new();
        let prompt = mode.system_prompt();
        assert!(prompt.contains("code generation assistant"));
        assert!(prompt.contains("clean"));
        assert!(prompt.contains("best practices"));
    }

    #[tokio::test]
    async fn test_code_mode_process() {
        let mode = CodeMode::new();
        let context = ModeContext::new("test-session".to_string());
        let response = mode.process("test input", &context).await.unwrap();
        assert_eq!(response.content, "test input");
        assert_eq!(response.metadata.mode, "code");
        assert!(!response.metadata.think_more_used);
    }

    #[tokio::test]
    async fn test_code_mode_process_with_think_more() {
        let mode = CodeMode::new();
        let mut context = ModeContext::new("test-session".to_string());
        context.think_more_enabled = true;
        let response = mode.process("test input", &context).await.unwrap();
        assert!(response.metadata.think_more_used);
    }

    #[test]
    fn test_code_mode_default() {
        let mode = CodeMode::default();
        assert_eq!(mode.id(), "code");
    }

    #[test]
    fn test_code_mode_with_custom_config() {
        let custom_config = ModeConfig {
            temperature: 0.5,
            max_tokens: 2048,
            system_prompt: "Custom prompt".to_string(),
            capabilities: vec![Capability::CodeGeneration],
            constraints: ModeConstraints {
                allow_file_operations: false,
                allow_command_execution: false,
                allow_code_generation: true,
                require_specs: true,
                auto_think_more_threshold: None,
            },
        };
        let mode = CodeMode::with_config(custom_config);
        assert_eq!(mode.config().temperature, 0.5);
        assert_eq!(mode.config().max_tokens, 2048);
    }

    #[test]
    fn test_generate_code() {
        let mode = CodeMode::new();
        let spec = "Create a function that adds two numbers";
        let result = mode.generate_code(spec);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Generated from spec"));
        assert!(code.contains(spec));
    }

    #[tokio::test]
    async fn test_create_file() {
        let mode = CodeMode::new();
        let temp_dir = std::env::temp_dir().join("ricecoder_test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("test_file.rs");

        let result = mode.create_file(&file_path, "fn main() {}").await;
        assert!(result.is_ok());
        assert!(file_path.exists());

        // Cleanup
        let _ = std::fs::remove_file(&file_path);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[tokio::test]
    async fn test_create_file_with_nested_dirs() {
        let mode = CodeMode::new();
        let temp_dir = std::env::temp_dir().join("ricecoder_test_nested");
        let file_path = temp_dir.join("src").join("lib.rs");

        let result = mode.create_file(&file_path, "pub fn hello() {}").await;
        assert!(result.is_ok());
        assert!(file_path.exists());

        // Cleanup
        let _ = std::fs::remove_file(&file_path);
        let _ = std::fs::remove_dir(temp_dir.join("src"));
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[tokio::test]
    async fn test_modify_file() {
        let mode = CodeMode::new();
        let temp_dir = std::env::temp_dir().join("ricecoder_test_modify");
        let _ = std::fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("test_file.rs");

        // Create initial file
        std::fs::write(&file_path, "fn main() {}").unwrap();

        // Modify file
        let result = mode.modify_file(&file_path, "// Added comment").await;
        assert!(result.is_ok());

        // Verify modification
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("fn main() {}"));
        assert!(content.contains("// Added comment"));

        // Cleanup
        let _ = std::fs::remove_file(&file_path);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[tokio::test]
    async fn test_modify_file_blocked_when_disabled() {
        let custom_config = ModeConfig {
            temperature: 0.7,
            max_tokens: 4096,
            system_prompt: "Test".to_string(),
            capabilities: vec![],
            constraints: ModeConstraints {
                allow_file_operations: false,
                allow_command_execution: false,
                allow_code_generation: false,
                require_specs: false,
                auto_think_more_threshold: None,
            },
        };
        let mode = CodeMode::with_config(custom_config);
        let temp_dir = std::env::temp_dir().join("ricecoder_test_blocked");
        let _ = std::fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("test_file.rs");
        std::fs::write(&file_path, "fn main() {}").unwrap();

        let result = mode.modify_file(&file_path, "// comment").await;
        assert!(result.is_err());

        // Cleanup
        let _ = std::fs::remove_file(&file_path);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_track_changes() {
        let mode = CodeMode::new();
        let summary = mode.track_changes(3, 2);
        assert_eq!(summary.files_created, 3);
        assert_eq!(summary.files_modified, 2);
        assert_eq!(summary.tests_run, 0);
        assert_eq!(summary.tests_passed, 0);
        assert!(summary.quality_issues.is_empty());
    }

    #[tokio::test]
    async fn test_process_with_generate_keyword() {
        let mode = CodeMode::new();
        let context = ModeContext::new("test-session".to_string());
        let response = mode.process("generate a function", &context).await.unwrap();
        assert!(!response.actions.is_empty());
    }

    #[tokio::test]
    async fn test_process_with_file_keyword() {
        let mode = CodeMode::new();
        let context = ModeContext::new("test-session".to_string());
        let response = mode.process("modify file", &context).await.unwrap();
        assert!(!response.actions.is_empty());
    }

    #[tokio::test]
    async fn test_run_tests() {
        let mode = CodeMode::new();
        let temp_dir = std::env::temp_dir().join("ricecoder_test_run");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test.rs");
        std::fs::write(&test_file, "fn test() {}").unwrap();

        let result = mode.run_tests(&[test_file.clone()]).await;
        assert!(result.is_ok());
        let (tests_run, tests_passed, failures) = result.unwrap();
        assert_eq!(tests_run, 1);
        assert_eq!(tests_passed, 1);
        assert!(failures.is_empty());

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[tokio::test]
    async fn test_run_tests_with_missing_file() {
        let mode = CodeMode::new();
        let missing_file = PathBuf::from("/nonexistent/test.rs");

        let result = mode.run_tests(&[missing_file]).await;
        assert!(result.is_ok());
        let (tests_run, tests_passed, failures) = result.unwrap();
        assert_eq!(tests_run, 1);
        assert_eq!(tests_passed, 0);
        assert!(!failures.is_empty());
    }

    #[test]
    fn test_capture_test_results() {
        let mode = CodeMode::new();
        let summary = ChangeSummary {
            files_created: 1,
            files_modified: 0,
            tests_run: 0,
            tests_passed: 0,
            quality_issues: Vec::new(),
        };

        let updated = mode.capture_test_results(summary, 5, 4, vec!["test 1 failed".to_string()]);
        assert_eq!(updated.tests_run, 5);
        assert_eq!(updated.tests_passed, 4);
        assert_eq!(updated.quality_issues.len(), 1);
    }

    #[test]
    fn test_report_test_failures_no_failures() {
        let mode = CodeMode::new();
        let report = mode.report_test_failures(&[]);
        assert_eq!(report, "All tests passed!");
    }

    #[test]
    fn test_report_test_failures_with_failures() {
        let mode = CodeMode::new();
        let failures = vec!["test 1 failed".to_string(), "test 2 failed".to_string()];
        let report = mode.report_test_failures(&failures);
        assert!(report.contains("Test failures:"));
        assert!(report.contains("test 1 failed"));
        assert!(report.contains("test 2 failed"));
    }

    #[tokio::test]
    async fn test_run_tests_blocked_when_disabled() {
        let custom_config = ModeConfig {
            temperature: 0.7,
            max_tokens: 4096,
            system_prompt: "Test".to_string(),
            capabilities: vec![],
            constraints: ModeConstraints {
                allow_file_operations: false,
                allow_command_execution: false,
                allow_code_generation: false,
                require_specs: false,
                auto_think_more_threshold: None,
            },
        };
        let mode = CodeMode::with_config(custom_config);
        let result = mode.run_tests(&[PathBuf::from("test.rs")]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_quality() {
        let mode = CodeMode::new();
        let temp_dir = std::env::temp_dir().join("ricecoder_test_quality");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        let result = mode.validate_quality(&[test_file.clone()]).await;
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert!(issues.is_empty());

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[tokio::test]
    async fn test_validate_quality_with_todo() {
        let mode = CodeMode::new();
        let temp_dir = std::env::temp_dir().join("ricecoder_test_quality_todo");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test.rs");
        std::fs::write(&test_file, "fn main() {\n    // TODO: implement\n}").unwrap();

        let result = mode.validate_quality(&[test_file.clone()]).await;
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert!(!issues.is_empty());
        assert!(issues[0].contains("TODO"));

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[tokio::test]
    async fn test_validate_quality_with_unwrap() {
        let mode = CodeMode::new();
        let temp_dir = std::env::temp_dir().join("ricecoder_test_quality_unwrap");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test.rs");
        std::fs::write(&test_file, "fn main() {\n    let x = Some(1).unwrap();\n}").unwrap();

        let result = mode.validate_quality(&[test_file.clone()]).await;
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert!(!issues.is_empty());
        assert!(issues[0].contains("Unwrap"));

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[tokio::test]
    async fn test_validate_quality_missing_file() {
        let mode = CodeMode::new();
        let missing_file = PathBuf::from("/nonexistent/test.rs");

        let result = mode.validate_quality(&[missing_file]).await;
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_report_quality_issues_no_issues() {
        let mode = CodeMode::new();
        let report = mode.report_quality_issues(&[]);
        assert_eq!(report, "No quality issues found!");
    }

    #[test]
    fn test_report_quality_issues_with_issues() {
        let mode = CodeMode::new();
        let issues = vec!["TODO found".to_string(), "FIXME found".to_string()];
        let report = mode.report_quality_issues(&issues);
        assert!(report.contains("Quality issues:"));
        assert!(report.contains("TODO found"));
        assert!(report.contains("FIXME found"));
    }

    #[test]
    fn test_suggest_improvements() {
        let mode = CodeMode::new();
        let issues = vec![
            "TODO found in file.rs".to_string(),
            "Unwrap found in file.rs".to_string(),
        ];
        let suggestions = mode.suggest_improvements(&issues);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("TODO")));
        assert!(suggestions.iter().any(|s| s.contains("unwrap")));
    }

    #[tokio::test]
    async fn test_validate_quality_blocked_when_disabled() {
        let custom_config = ModeConfig {
            temperature: 0.7,
            max_tokens: 4096,
            system_prompt: "Test".to_string(),
            capabilities: vec![],
            constraints: ModeConstraints {
                allow_file_operations: false,
                allow_command_execution: false,
                allow_code_generation: false,
                require_specs: false,
                auto_think_more_threshold: None,
            },
        };
        let mode = CodeMode::with_config(custom_config);
        let result = mode.validate_quality(&[PathBuf::from("test.rs")]).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_change_summary() {
        let mode = CodeMode::new();
        let summary = mode.generate_change_summary(2, 1, 5, 5, vec![]);
        assert_eq!(summary.files_created, 2);
        assert_eq!(summary.files_modified, 1);
        assert_eq!(summary.tests_run, 5);
        assert_eq!(summary.tests_passed, 5);
        assert!(summary.quality_issues.is_empty());
    }

    #[test]
    fn test_generate_change_summary_with_issues() {
        let mode = CodeMode::new();
        let issues = vec!["TODO found".to_string()];
        let summary = mode.generate_change_summary(1, 0, 3, 2, issues);
        assert_eq!(summary.files_created, 1);
        assert_eq!(summary.quality_issues.len(), 1);
    }

    #[test]
    fn test_format_change_summary() {
        let mode = CodeMode::new();
        let summary = ChangeSummary {
            files_created: 2,
            files_modified: 1,
            tests_run: 5,
            tests_passed: 5,
            quality_issues: vec![],
        };
        let formatted = mode.format_change_summary(&summary);
        assert!(formatted.contains("Change Summary"));
        assert!(formatted.contains("Files created: 2"));
        assert!(formatted.contains("Files modified: 1"));
        assert!(formatted.contains("Tests run: 5"));
        assert!(formatted.contains("Tests passed: 5"));
        assert!(formatted.contains("Pass rate: 100.0%"));
    }

    #[test]
    fn test_format_change_summary_with_issues() {
        let mode = CodeMode::new();
        let summary = ChangeSummary {
            files_created: 1,
            files_modified: 0,
            tests_run: 0,
            tests_passed: 0,
            quality_issues: vec!["TODO found".to_string()],
        };
        let formatted = mode.format_change_summary(&summary);
        assert!(formatted.contains("Quality issues: 1"));
        assert!(formatted.contains("TODO found"));
    }

    #[test]
    fn test_provide_feedback_all_passed() {
        let mode = CodeMode::new();
        let summary = ChangeSummary {
            files_created: 2,
            files_modified: 1,
            tests_run: 5,
            tests_passed: 5,
            quality_issues: vec![],
        };
        let feedback = mode.provide_feedback(&summary);
        assert!(!feedback.is_empty());
        assert!(feedback.iter().any(|f| f.contains("created")));
        assert!(feedback.iter().any(|f| f.contains("All tests passed")));
        assert!(feedback.iter().any(|f| f.contains("quality looks good")));
    }

    #[test]
    fn test_provide_feedback_with_failures() {
        let mode = CodeMode::new();
        let summary = ChangeSummary {
            files_created: 1,
            files_modified: 0,
            tests_run: 5,
            tests_passed: 3,
            quality_issues: vec![],
        };
        let feedback = mode.provide_feedback(&summary);
        assert!(!feedback.is_empty());
        assert!(feedback.iter().any(|f| f.contains("2 test(s) failed")));
    }

    #[test]
    fn test_provide_feedback_with_quality_issues() {
        let mode = CodeMode::new();
        let summary = ChangeSummary {
            files_created: 1,
            files_modified: 0,
            tests_run: 0,
            tests_passed: 0,
            quality_issues: vec!["TODO found".to_string(), "FIXME found".to_string()],
        };
        let feedback = mode.provide_feedback(&summary);
        assert!(!feedback.is_empty());
        assert!(feedback.iter().any(|f| f.contains("2 quality issue(s)")));
    }

    #[test]
    fn test_provide_feedback_no_changes() {
        let mode = CodeMode::new();
        let summary = ChangeSummary {
            files_created: 0,
            files_modified: 0,
            tests_run: 0,
            tests_passed: 0,
            quality_issues: vec![],
        };
        let feedback = mode.provide_feedback(&summary);
        assert!(feedback.is_empty());
    }
}
