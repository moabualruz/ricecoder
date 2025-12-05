//! Test runner for executing tests and parsing results
//!
//! Provides test framework detection, test execution, and result parsing.
//! Supports Rust (cargo test), TypeScript (npm test), and Python (pytest).

use crate::error::{ExecutionError, ExecutionResult};
use crate::models::{TestFailure, TestFramework, TestResults};
use std::path::Path;
use std::process::Command;
use tracing::{debug, error, info};

/// Test runner for executing tests with framework detection
pub struct TestRunner {
    /// Project root directory
    project_root: std::path::PathBuf,
}

impl TestRunner {
    /// Create a new test runner for the given project root
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
        }
    }

    /// Create a test runner for the current directory
    pub fn current_dir() -> ExecutionResult<Self> {
        let project_root = std::env::current_dir().map_err(|e| {
            ExecutionError::ValidationError(format!("Failed to get current dir: {}", e))
        })?;
        Ok(Self { project_root })
    }

    /// Detect the test framework based on project structure
    pub fn detect_framework(&self) -> ExecutionResult<TestFramework> {
        debug!(project_root = ?self.project_root, "Detecting test framework");

        // Check for Rust (Cargo.toml)
        if self.project_root.join("Cargo.toml").exists() {
            debug!("Detected Rust project");
            return Ok(TestFramework::Rust);
        }

        // Check for TypeScript/Node.js (package.json)
        if self.project_root.join("package.json").exists() {
            debug!("Detected TypeScript/Node.js project");
            return Ok(TestFramework::TypeScript);
        }

        // Check for Python (pytest.ini or setup.py)
        if self.project_root.join("pytest.ini").exists()
            || self.project_root.join("setup.py").exists()
        {
            debug!("Detected Python project");
            return Ok(TestFramework::Python);
        }

        Err(ExecutionError::ValidationError(
            "Could not detect test framework".to_string(),
        ))
    }

    /// Run tests with optional pattern filtering
    ///
    /// # Arguments
    /// * `pattern` - Optional test pattern to filter tests
    ///
    /// # Returns
    /// Test results including pass/fail counts and failure details
    pub fn run_tests(&self, pattern: Option<&str>) -> ExecutionResult<TestResults> {
        let framework = self.detect_framework()?;
        info!(framework = ?framework, pattern = ?pattern, "Running tests");

        let (command, args) = self.build_test_command(&framework, pattern)?;

        // Execute the test command
        let output = Command::new(&command)
            .args(&args)
            .current_dir(&self.project_root)
            .output()
            .map_err(|e| {
                ExecutionError::StepFailed(format!(
                    "Failed to execute test command {}: {}",
                    command, e
                ))
            })?;

        // Parse test results
        let test_output = String::from_utf8_lossy(&output.stdout);
        let test_stderr = String::from_utf8_lossy(&output.stderr);

        let mut results = TestResults {
            passed: 0,
            failed: 0,
            skipped: 0,
            failures: Vec::new(),
            framework,
        };

        // Parse results based on framework
        match framework {
            TestFramework::Rust => {
                self.parse_rust_output(&test_output, &test_stderr, &mut results)?;
            }
            TestFramework::TypeScript => {
                self.parse_typescript_output(&test_output, &test_stderr, &mut results)?;
            }
            TestFramework::Python => {
                self.parse_python_output(&test_output, &test_stderr, &mut results)?;
            }
            TestFramework::Other => {
                debug!("Unknown test framework, skipping output parsing");
            }
        }

        // Check if tests failed
        if results.failed > 0 {
            error!(failed = results.failed, "Tests failed");
            return Err(ExecutionError::TestsFailed(results.failed));
        }

        info!(passed = results.passed, "Tests passed");
        Ok(results)
    }

    /// Build test command for the detected framework
    pub fn build_test_command(
        &self,
        framework: &TestFramework,
        pattern: Option<&str>,
    ) -> ExecutionResult<(String, Vec<String>)> {
        match framework {
            TestFramework::Rust => {
                let mut args = vec![
                    "test".to_string(),
                    "--".to_string(),
                    "--nocapture".to_string(),
                ];
                if let Some(p) = pattern {
                    args.push(p.to_string());
                }
                Ok(("cargo".to_string(), args))
            }
            TestFramework::TypeScript => {
                let mut args = vec!["test".to_string()];
                if let Some(p) = pattern {
                    args.push("--".to_string());
                    args.push(p.to_string());
                }
                Ok(("npm".to_string(), args))
            }
            TestFramework::Python => {
                let mut args = vec![];
                if let Some(p) = pattern {
                    args.push(p.to_string());
                }
                Ok(("pytest".to_string(), args))
            }
            TestFramework::Other => Err(ExecutionError::ValidationError(
                "Cannot build test command for unknown framework".to_string(),
            )),
        }
    }

    /// Parse Rust test output
    fn parse_rust_output(
        &self,
        stdout: &str,
        _stderr: &str,
        results: &mut TestResults,
    ) -> ExecutionResult<()> {
        debug!("Parsing Rust test output");

        // Parse test results from cargo test output
        // Format: "test result: ok. X passed; Y failed; Z ignored"
        for line in stdout.lines() {
            if line.contains("test result:") {
                if line.contains("ok.") {
                    // Extract counts from the line
                    if let Some(passed_str) = line.split("passed;").next() {
                        if let Some(num_str) = passed_str.split_whitespace().last() {
                            if let Ok(num) = num_str.parse::<usize>() {
                                results.passed = num;
                            }
                        }
                    }
                } else if line.contains("FAILED") {
                    // Extract failure counts
                    if let Some(failed_str) = line.split("failed;").next() {
                        if let Some(num_str) = failed_str.split_whitespace().last() {
                            if let Ok(num) = num_str.parse::<usize>() {
                                results.failed = num;
                            }
                        }
                    }
                }
            }

            // Parse individual test failures
            if line.contains("FAILED") && line.contains("::") {
                let test_name = line.split("FAILED").nth(1).unwrap_or("").trim().to_string();
                results.failures.push(TestFailure {
                    name: test_name,
                    message: "Test failed".to_string(),
                    location: None,
                });
            }
        }

        Ok(())
    }

    /// Parse TypeScript test output
    fn parse_typescript_output(
        &self,
        stdout: &str,
        _stderr: &str,
        results: &mut TestResults,
    ) -> ExecutionResult<()> {
        debug!("Parsing TypeScript test output");

        // Parse Jest/npm test output
        // Look for patterns like "Tests: X passed, Y failed"
        for line in stdout.lines() {
            if line.contains("passed") && line.contains("failed") {
                // Try to extract pass/fail counts
                if let Some(passed_part) = line.split("passed").next() {
                    if let Some(num_str) = passed_part.split_whitespace().last() {
                        if let Ok(num) = num_str.parse::<usize>() {
                            results.passed = num;
                        }
                    }
                }

                if let Some(failed_part) = line.split("failed").next() {
                    if let Some(num_str) = failed_part.split_whitespace().last() {
                        if let Ok(num) = num_str.parse::<usize>() {
                            results.failed = num;
                        }
                    }
                }
            }

            // Parse individual test failures
            if line.contains("✕") || line.contains("FAIL") {
                let test_name = line.trim().to_string();
                results.failures.push(TestFailure {
                    name: test_name,
                    message: "Test failed".to_string(),
                    location: None,
                });
            }
        }

        Ok(())
    }

    /// Parse Python test output
    fn parse_python_output(
        &self,
        stdout: &str,
        _stderr: &str,
        results: &mut TestResults,
    ) -> ExecutionResult<()> {
        debug!("Parsing Python test output");

        // Parse pytest output
        // Look for patterns like "X passed, Y failed"
        for line in stdout.lines() {
            if line.contains("passed") || line.contains("failed") {
                // Try to extract pass/fail counts
                if let Some(passed_part) = line.split("passed").next() {
                    if let Some(num_str) = passed_part.split_whitespace().last() {
                        if let Ok(num) = num_str.parse::<usize>() {
                            results.passed = num;
                        }
                    }
                }

                if let Some(failed_part) = line.split("failed").next() {
                    if let Some(num_str) = failed_part.split_whitespace().last() {
                        if let Ok(num) = num_str.parse::<usize>() {
                            results.failed = num;
                        }
                    }
                }
            }

            // Parse individual test failures
            if line.contains("FAILED") {
                let test_name = line.trim().to_string();
                results.failures.push(TestFailure {
                    name: test_name,
                    message: "Test failed".to_string(),
                    location: None,
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_rust_framework() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Cargo.toml"), "").unwrap();

        let runner = TestRunner::new(temp_dir.path());
        let framework = runner.detect_framework().unwrap();
        assert_eq!(framework, TestFramework::Rust);
    }

    #[test]
    fn test_detect_typescript_framework() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("package.json"), "").unwrap();

        let runner = TestRunner::new(temp_dir.path());
        let framework = runner.detect_framework().unwrap();
        assert_eq!(framework, TestFramework::TypeScript);
    }

    #[test]
    fn test_detect_python_framework_pytest() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("pytest.ini"), "").unwrap();

        let runner = TestRunner::new(temp_dir.path());
        let framework = runner.detect_framework().unwrap();
        assert_eq!(framework, TestFramework::Python);
    }

    #[test]
    fn test_detect_python_framework_setup() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("setup.py"), "").unwrap();

        let runner = TestRunner::new(temp_dir.path());
        let framework = runner.detect_framework().unwrap();
        assert_eq!(framework, TestFramework::Python);
    }

    #[test]
    fn test_detect_no_framework() {
        let temp_dir = TempDir::new().unwrap();
        let runner = TestRunner::new(temp_dir.path());
        let result = runner.detect_framework();
        assert!(result.is_err());
    }

    #[test]
    fn test_build_rust_test_command() {
        let temp_dir = TempDir::new().unwrap();
        let runner = TestRunner::new(temp_dir.path());

        let (cmd, args) = runner
            .build_test_command(&TestFramework::Rust, None)
            .unwrap();
        assert_eq!(cmd, "cargo");
        assert!(args.contains(&"test".to_string()));
    }

    #[test]
    fn test_build_rust_test_command_with_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let runner = TestRunner::new(temp_dir.path());

        let (cmd, args) = runner
            .build_test_command(&TestFramework::Rust, Some("my_test"))
            .unwrap();
        assert_eq!(cmd, "cargo");
        assert!(args.contains(&"my_test".to_string()));
    }

    #[test]
    fn test_build_typescript_test_command() {
        let temp_dir = TempDir::new().unwrap();
        let runner = TestRunner::new(temp_dir.path());

        let (cmd, args) = runner
            .build_test_command(&TestFramework::TypeScript, None)
            .unwrap();
        assert_eq!(cmd, "npm");
        assert!(args.contains(&"test".to_string()));
    }

    #[test]
    fn test_build_python_test_command() {
        let temp_dir = TempDir::new().unwrap();
        let runner = TestRunner::new(temp_dir.path());

        let (cmd, _args) = runner
            .build_test_command(&TestFramework::Python, None)
            .unwrap();
        assert_eq!(cmd, "pytest");
    }

    #[test]
    fn test_parse_rust_output() {
        let temp_dir = TempDir::new().unwrap();
        let runner = TestRunner::new(temp_dir.path());

        let stdout = "test result: ok. 5 passed; 0 failed; 1 ignored";
        let mut results = TestResults {
            passed: 0,
            failed: 0,
            skipped: 0,
            failures: Vec::new(),
            framework: TestFramework::Rust,
        };

        runner.parse_rust_output(stdout, "", &mut results).unwrap();
        assert_eq!(results.passed, 5);
        assert_eq!(results.failed, 0);
    }

    #[test]
    fn test_parse_typescript_output() {
        let temp_dir = TempDir::new().unwrap();
        let runner = TestRunner::new(temp_dir.path());

        let stdout = "Tests: 3 passed, 0 failed";
        let mut results = TestResults {
            passed: 0,
            failed: 0,
            skipped: 0,
            failures: Vec::new(),
            framework: TestFramework::TypeScript,
        };

        runner
            .parse_typescript_output(stdout, "", &mut results)
            .unwrap();
        assert_eq!(results.passed, 3);
        assert_eq!(results.failed, 0);
    }

    #[test]
    fn test_parse_python_output() {
        let temp_dir = TempDir::new().unwrap();
        let runner = TestRunner::new(temp_dir.path());

        let stdout = "4 passed, 0 failed";
        let mut results = TestResults {
            passed: 0,
            failed: 0,
            skipped: 0,
            failures: Vec::new(),
            framework: TestFramework::Python,
        };

        runner
            .parse_python_output(stdout, "", &mut results)
            .unwrap();
        assert_eq!(results.passed, 4);
        assert_eq!(results.failed, 0);
    }
}
