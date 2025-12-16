//! Test execution and validation

use crate::error::BenchmarkError;
use crate::exercise::Exercise;
use std::path::Path;
use tokio::process::Command;
use std::process::Stdio;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug)]
pub struct TestResult {
    pub passed: bool,
    pub output: String,
    pub timeout: bool,
}

pub struct Evaluator;

impl Evaluator {
    pub async fn run_tests(exercise: &Exercise, test_dir: &Path) -> Result<TestResult, BenchmarkError> {
        let timeout_duration = Duration::from_secs(180); // 3 minutes

        let result = timeout(timeout_duration, Self::run_tests_inner(exercise, test_dir)).await;

        match result {
            Ok(Ok(test_result)) => Ok(test_result),
            Ok(Err(e)) => Err(e),
            Err(_) => Ok(TestResult {
                passed: false,
                output: "Tests timed out!".to_string(),
                timeout: true,
            }),
        }
    }

    async fn run_tests_inner(exercise: &Exercise, test_dir: &Path) -> Result<TestResult, BenchmarkError> {
        // Copy test files to test directory
        for test_file in &exercise.get_test_files() {
            if test_file.exists() {
                let dest = test_dir.join(test_file.file_name().unwrap());
                std::fs::create_dir_all(dest.parent().unwrap())?;
                std::fs::copy(test_file, dest)?;
            }
        }

        // Determine test command based on language
        let (command, args) = Self::get_test_command(&exercise.language)?;

        // Remove @Disabled annotations from Java test files
        if exercise.language == "java" {
            Self::enable_java_tests(test_dir)?;
        }

        // Run the test command
        let output = Command::new(command)
            .args(&args)
            .current_dir(test_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let full_output = format!("{}{}", stdout, stderr);

        let passed = output.status.success();

        // Clean up output for consistency
        let cleaned_output = Self::cleanup_test_output(&full_output, test_dir);

        Ok(TestResult {
            passed,
            output: cleaned_output,
            timeout: false,
        })
    }

    fn get_test_command(language: &str) -> Result<(String, Vec<String>), BenchmarkError> {
        match language {
            "python" => Ok(("pytest".to_string(), vec![])),
            "rust" => Ok(("cargo".to_string(), vec!["test".to_string(), "--".to_string(), "--include-ignored".to_string()])),
            "go" => Ok(("go".to_string(), vec!["test".to_string(), "./...".to_string()])),
            "javascript" => Ok(("npm".to_string(), vec!["test".to_string()])),
            "cpp" => Ok(("make".to_string(), vec!["test".to_string()])), // Assuming Makefile
            "java" => Ok(("./gradlew".to_string(), vec!["test".to_string()])),
            _ => Err(BenchmarkError::TestExecution(format!("Unsupported language: {}", language))),
        }
    }

    fn enable_java_tests(test_dir: &Path) -> Result<(), BenchmarkError> {
        use walkdir::WalkDir;

        for entry in WalkDir::new(test_dir) {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("java") {
                let content = std::fs::read_to_string(entry.path())?;
                let new_content = regex::Regex::new(r"@Disabled\([^)]*\)\s*\n")?
                    .replace_all(&content, "");
                std::fs::write(entry.path(), new_content.as_ref())?;
            }
        }

        Ok(())
    }

    fn cleanup_test_output(output: &str, test_dir: &Path) -> String {
        // Remove timing info to avoid randomizing responses
        let output = regex::Regex::new(r"\bin \d+\.\d+s\b").unwrap()
            .replace_all(output, "");

        // Replace test directory path with just the name
        output.replace(&test_dir.to_string_lossy().to_string(),
                      &test_dir.file_name().unwrap_or_default().to_string_lossy().to_string())
    }
}