//! Validation engine for refactoring operations

use crate::error::Result;
use crate::types::ValidationResult;
use std::path::Path;
use std::process::Command;

/// Validates refactoring results
pub struct ValidationEngine;

impl ValidationEngine {
    /// Create a new validation engine
    pub fn new() -> Self {
        Self
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of test execution
#[derive(Debug, Clone)]
pub struct TestExecutionResult {
    /// Whether all tests passed
    pub passed: bool,
    /// Number of tests run
    pub tests_run: usize,
    /// Number of tests passed
    pub tests_passed: usize,
    /// Number of tests failed
    pub tests_failed: usize,
    /// Test output
    pub output: String,
    /// Test errors
    pub errors: Vec<String>,
}

impl ValidationEngine {
    /// Validate code syntax
    pub fn validate_syntax(code: &str, language: &str) -> Result<ValidationResult> {
        let mut errors = vec![];
        let mut warnings = vec![];

        // Basic syntax validation based on language
        match language {
            "rust" => {
                Self::validate_rust_syntax(code, &mut errors, &mut warnings);
            }
            "typescript" | "javascript" => {
                Self::validate_typescript_syntax(code, &mut errors, &mut warnings);
            }
            "python" => {
                Self::validate_python_syntax(code, &mut errors, &mut warnings);
            }
            _ => {
                // Generic validation
                Self::validate_generic_syntax(code, &mut errors, &mut warnings);
            }
        }

        Ok(ValidationResult {
            passed: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Validate Rust syntax
    fn validate_rust_syntax(code: &str, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        let open_parens = code.matches('(').count();
        let close_parens = code.matches(')').count();

        if open_braces != close_braces {
            errors.push(format!(
                "Brace mismatch: {} open, {} close",
                open_braces, close_braces
            ));
        }

        if open_parens != close_parens {
            errors.push(format!(
                "Parenthesis mismatch: {} open, {} close",
                open_parens, close_parens
            ));
        }

        if code.contains("unsafe") {
            warnings.push("Code contains unsafe block".to_string());
        }
    }

    /// Validate TypeScript syntax
    fn validate_typescript_syntax(
        code: &str,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        let open_parens = code.matches('(').count();
        let close_parens = code.matches(')').count();

        if open_braces != close_braces {
            errors.push(format!(
                "Brace mismatch: {} open, {} close",
                open_braces, close_braces
            ));
        }

        if open_parens != close_parens {
            errors.push(format!(
                "Parenthesis mismatch: {} open, {} close",
                open_parens, close_parens
            ));
        }

        if code.contains("any") {
            warnings.push("Code uses 'any' type".to_string());
        }
    }

    /// Validate Python syntax
    fn validate_python_syntax(code: &str, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        let open_parens = code.matches('(').count();
        let close_parens = code.matches(')').count();
        let open_brackets = code.matches('[').count();
        let close_brackets = code.matches(']').count();

        if open_parens != close_parens {
            errors.push(format!(
                "Parenthesis mismatch: {} open, {} close",
                open_parens, close_parens
            ));
        }

        if open_brackets != close_brackets {
            errors.push(format!(
                "Bracket mismatch: {} open, {} close",
                open_brackets, close_brackets
            ));
        }

        if code.contains("exec(") {
            warnings.push("Code uses exec() function".to_string());
        }
    }

    /// Generic syntax validation
    fn validate_generic_syntax(code: &str, errors: &mut Vec<String>, _warnings: &mut Vec<String>) {
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        let open_parens = code.matches('(').count();
        let close_parens = code.matches(')').count();

        if open_braces != close_braces {
            errors.push(format!(
                "Brace mismatch: {} open, {} close",
                open_braces, close_braces
            ));
        }

        if open_parens != close_parens {
            errors.push(format!(
                "Parenthesis mismatch: {} open, {} close",
                open_parens, close_parens
            ));
        }
    }

    /// Validate semantic correctness
    pub fn validate_semantics(code: &str, _language: &str) -> Result<ValidationResult> {
        let mut errors = vec![];
        let warnings = vec![];

        // Basic semantic checks
        if code.is_empty() {
            errors.push("Code cannot be empty".to_string());
        }

        Ok(ValidationResult {
            passed: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Run automated tests for a project
    ///
    /// This method attempts to run tests for the specified language/project.
    /// It supports Rust (cargo test), TypeScript/JavaScript (npm test), and Python (pytest).
    pub fn run_tests(project_path: &Path, language: &str) -> Result<TestExecutionResult> {
        match language {
            "rust" => Self::run_rust_tests(project_path),
            "typescript" | "javascript" => Self::run_npm_tests(project_path),
            "python" => Self::run_python_tests(project_path),
            _ => Self::run_generic_tests(project_path),
        }
    }

    /// Run Rust tests using cargo
    fn run_rust_tests(project_path: &Path) -> Result<TestExecutionResult> {
        let output = Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("--test-threads=1")
            .current_dir(project_path)
            .output()
            .map_err(|e| {
                crate::error::RefactoringError::ValidationFailed(format!(
                    "Failed to run cargo tests: {}",
                    e
                ))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let passed = output.status.success();
        let (tests_run, tests_passed, tests_failed) = Self::parse_rust_test_output(&stdout);

        let mut errors = vec![];
        if !passed && !stderr.is_empty() {
            errors.push(stderr);
        }

        Ok(TestExecutionResult {
            passed,
            tests_run,
            tests_passed,
            tests_failed,
            output: stdout,
            errors,
        })
    }

    /// Run npm tests
    fn run_npm_tests(project_path: &Path) -> Result<TestExecutionResult> {
        let output = Command::new("npm")
            .arg("test")
            .current_dir(project_path)
            .output()
            .map_err(|e| {
                crate::error::RefactoringError::ValidationFailed(format!(
                    "Failed to run npm tests: {}",
                    e
                ))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let passed = output.status.success();
        let (tests_run, tests_passed, tests_failed) = Self::parse_npm_test_output(&stdout);

        let mut errors = vec![];
        if !passed && !stderr.is_empty() {
            errors.push(stderr);
        }

        Ok(TestExecutionResult {
            passed,
            tests_run,
            tests_passed,
            tests_failed,
            output: stdout,
            errors,
        })
    }

    /// Run Python tests using pytest
    fn run_python_tests(project_path: &Path) -> Result<TestExecutionResult> {
        let output = Command::new("pytest")
            .arg("-v")
            .current_dir(project_path)
            .output()
            .map_err(|e| {
                crate::error::RefactoringError::ValidationFailed(format!(
                    "Failed to run pytest: {}",
                    e
                ))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let passed = output.status.success();
        let (tests_run, tests_passed, tests_failed) = Self::parse_pytest_output(&stdout);

        let mut errors = vec![];
        if !passed && !stderr.is_empty() {
            errors.push(stderr);
        }

        Ok(TestExecutionResult {
            passed,
            tests_run,
            tests_passed,
            tests_failed,
            output: stdout,
            errors,
        })
    }

    /// Run generic tests (fallback)
    fn run_generic_tests(_project_path: &Path) -> Result<TestExecutionResult> {
        Ok(TestExecutionResult {
            passed: true,
            tests_run: 0,
            tests_passed: 0,
            tests_failed: 0,
            output: "No test runner available for this language".to_string(),
            errors: vec![],
        })
    }

    /// Parse Rust test output to extract test counts
    fn parse_rust_test_output(output: &str) -> (usize, usize, usize) {
        let mut tests_run = 0;
        let mut tests_passed = 0;
        let mut tests_failed = 0;

        for line in output.lines() {
            if line.contains("test result:") {
                // Parse lines like "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
                if let Some(passed_part) = line.split("passed;").next() {
                    if let Some(num_str) = passed_part.split_whitespace().last() {
                        if let Ok(num) = num_str.parse::<usize>() {
                            tests_passed = num;
                        }
                    }
                }

                if let Some(failed_part) = line.split("failed;").next() {
                    if let Some(num_str) = failed_part.split_whitespace().last() {
                        if let Ok(num) = num_str.parse::<usize>() {
                            tests_failed = num;
                        }
                    }
                }

                tests_run = tests_passed + tests_failed;
            }
        }

        (tests_run, tests_passed, tests_failed)
    }

    /// Parse npm test output to extract test counts
    fn parse_npm_test_output(output: &str) -> (usize, usize, usize) {
        let mut tests_passed = 0;
        let mut tests_failed = 0;

        for line in output.lines() {
            // Try to extract numbers from common test output formats
            // Handles formats like "3 passed, 2 failed" or "5 passed"
            let parts: Vec<&str> = line
                .split(|c: char| c == ',' || c.is_whitespace())
                .collect();
            let mut i = 0;
            while i < parts.len() {
                if let Ok(num) = parts[i].parse::<usize>() {
                    if i + 1 < parts.len() {
                        match parts[i + 1] {
                            "passed" => tests_passed = num,
                            "failed" => tests_failed = num,
                            _ => {}
                        }
                    }
                }
                i += 1;
            }
        }

        let tests_run = tests_passed + tests_failed;
        (tests_run, tests_passed, tests_failed)
    }

    /// Parse pytest output to extract test counts
    fn parse_pytest_output(output: &str) -> (usize, usize, usize) {
        let mut tests_passed = 0;
        let mut tests_failed = 0;

        for line in output.lines() {
            // Parse lines like "5 passed in 0.12s" or "3 failed, 2 passed"
            let parts: Vec<&str> = line
                .split(|c: char| c == ',' || c.is_whitespace())
                .collect();
            let mut i = 0;
            while i < parts.len() {
                if let Ok(num) = parts[i].parse::<usize>() {
                    if i + 1 < parts.len() {
                        match parts[i + 1] {
                            "passed" => tests_passed = num,
                            "failed" => tests_failed = num,
                            _ => {}
                        }
                    }
                }
                i += 1;
            }
        }

        let tests_run = tests_passed + tests_failed;
        (tests_run, tests_passed, tests_failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Syntax Validation Tests =====

    #[test]
    fn test_validate_rust_syntax_valid() -> Result<()> {
        let code = "fn main() { println!(\"Hello\"); }";
        let result = ValidationEngine::validate_syntax(code, "rust")?;
        assert!(result.passed);
        assert!(result.errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_validate_rust_syntax_invalid_braces() -> Result<()> {
        let code = "fn main() { println!(\"Hello\"); ";
        let result = ValidationEngine::validate_syntax(code, "rust")?;
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("Brace mismatch"));
        Ok(())
    }

    #[test]
    fn test_validate_rust_syntax_invalid_parens() -> Result<()> {
        let code = "fn main() { println!(\"Hello\"; }";
        let result = ValidationEngine::validate_syntax(code, "rust")?;
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("Parenthesis mismatch"));
        Ok(())
    }

    #[test]
    fn test_validate_rust_syntax_unsafe_warning() -> Result<()> {
        let code = "unsafe { let x = 5; }";
        let result = ValidationEngine::validate_syntax(code, "rust")?;
        assert!(result.passed);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("unsafe"));
        Ok(())
    }

    #[test]
    fn test_validate_typescript_syntax_valid() -> Result<()> {
        let code = "function main() { console.log(\"Hello\"); }";
        let result = ValidationEngine::validate_syntax(code, "typescript")?;
        assert!(result.passed);
        assert!(result.errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_validate_typescript_syntax_invalid_braces() -> Result<()> {
        let code = "function main() { console.log(\"Hello\"); ";
        let result = ValidationEngine::validate_syntax(code, "typescript")?;
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_validate_typescript_syntax_any_warning() -> Result<()> {
        let code = "let x: any = 5;";
        let result = ValidationEngine::validate_syntax(code, "typescript")?;
        assert!(result.passed);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("any"));
        Ok(())
    }

    #[test]
    fn test_validate_python_syntax_valid() -> Result<()> {
        let code = "def main():\n    print(\"Hello\")";
        let result = ValidationEngine::validate_syntax(code, "python")?;
        assert!(result.passed);
        assert!(result.errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_validate_python_syntax_invalid_parens() -> Result<()> {
        let code = "def main():\n    print(\"Hello\"";
        let result = ValidationEngine::validate_syntax(code, "python")?;
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_validate_python_syntax_invalid_brackets() -> Result<()> {
        let code = "x = [1, 2, 3";
        let result = ValidationEngine::validate_syntax(code, "python")?;
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_validate_python_syntax_exec_warning() -> Result<()> {
        let code = "exec(\"print('hello')\")";
        let result = ValidationEngine::validate_syntax(code, "python")?;
        assert!(result.passed);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("exec()"));
        Ok(())
    }

    #[test]
    fn test_validate_generic_syntax_valid() -> Result<()> {
        let code = "some code { with (parens) }";
        let result = ValidationEngine::validate_syntax(code, "unknown")?;
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_validate_generic_syntax_invalid() -> Result<()> {
        let code = "some code { with (parens }";
        let result = ValidationEngine::validate_syntax(code, "unknown")?;
        assert!(!result.passed);
        Ok(())
    }

    // ===== Semantic Validation Tests =====

    #[test]
    fn test_validate_semantics_empty() -> Result<()> {
        let result = ValidationEngine::validate_semantics("", "rust")?;
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("empty"));
        Ok(())
    }

    #[test]
    fn test_validate_semantics_valid() -> Result<()> {
        let result = ValidationEngine::validate_semantics("fn main() {}", "rust")?;
        assert!(result.passed);
        assert!(result.errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_validate_semantics_valid_typescript() -> Result<()> {
        let result = ValidationEngine::validate_semantics("function main() {}", "typescript")?;
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_validate_semantics_valid_python() -> Result<()> {
        let result = ValidationEngine::validate_semantics("def main(): pass", "python")?;
        assert!(result.passed);
        Ok(())
    }

    // ===== Test Output Parsing Tests =====

    #[test]
    fn test_parse_rust_test_output_success() {
        let output = "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out";
        let (tests_run, tests_passed, tests_failed) =
            ValidationEngine::parse_rust_test_output(output);
        assert_eq!(tests_run, 5);
        assert_eq!(tests_passed, 5);
        assert_eq!(tests_failed, 0);
    }

    #[test]
    fn test_parse_rust_test_output_with_failures() {
        let output =
            "test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out";
        let (tests_run, tests_passed, tests_failed) =
            ValidationEngine::parse_rust_test_output(output);
        assert_eq!(tests_run, 5);
        assert_eq!(tests_passed, 3);
        assert_eq!(tests_failed, 2);
    }

    #[test]
    fn test_parse_rust_test_output_no_tests() {
        let output = "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out";
        let (tests_run, tests_passed, tests_failed) =
            ValidationEngine::parse_rust_test_output(output);
        assert_eq!(tests_run, 0);
        assert_eq!(tests_passed, 0);
        assert_eq!(tests_failed, 0);
    }

    #[test]
    fn test_parse_npm_test_output_success() {
        let output = "5 passed";
        let (_tests_run, tests_passed, tests_failed) =
            ValidationEngine::parse_npm_test_output(output);
        assert_eq!(tests_passed, 5);
        assert_eq!(tests_failed, 0);
    }

    #[test]
    fn test_parse_npm_test_output_with_failures() {
        let output = "3 passed, 2 failed";
        let (_tests_run, tests_passed, tests_failed) =
            ValidationEngine::parse_npm_test_output(output);
        assert_eq!(tests_passed, 3);
        assert_eq!(tests_failed, 2);
    }

    #[test]
    fn test_parse_pytest_output_success() {
        let output = "5 passed in 0.12s";
        let (_tests_run, tests_passed, tests_failed) =
            ValidationEngine::parse_pytest_output(output);
        assert_eq!(tests_passed, 5);
        assert_eq!(tests_failed, 0);
    }

    #[test]
    fn test_parse_pytest_output_with_failures() {
        let output = "3 failed, 2 passed in 0.15s";
        let (_tests_run, tests_passed, tests_failed) =
            ValidationEngine::parse_pytest_output(output);
        assert_eq!(tests_passed, 2);
        assert_eq!(tests_failed, 3);
    }

    // ===== Test Execution Result Tests =====

    #[test]
    fn test_test_execution_result_creation() {
        let result = TestExecutionResult {
            passed: true,
            tests_run: 5,
            tests_passed: 5,
            tests_failed: 0,
            output: "All tests passed".to_string(),
            errors: vec![],
        };

        assert!(result.passed);
        assert_eq!(result.tests_run, 5);
        assert_eq!(result.tests_passed, 5);
        assert_eq!(result.tests_failed, 0);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_test_execution_result_with_errors() {
        let result = TestExecutionResult {
            passed: false,
            tests_run: 5,
            tests_passed: 3,
            tests_failed: 2,
            output: "Some tests failed".to_string(),
            errors: vec!["Test 1 failed".to_string(), "Test 2 failed".to_string()],
        };

        assert!(!result.passed);
        assert_eq!(result.tests_run, 5);
        assert_eq!(result.tests_passed, 3);
        assert_eq!(result.tests_failed, 2);
        assert_eq!(result.errors.len(), 2);
    }
}
