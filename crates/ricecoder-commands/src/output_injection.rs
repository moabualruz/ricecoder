use crate::error::Result;
use crate::types::CommandExecutionResult;

/// Output injection configuration
#[derive(Debug, Clone)]
pub struct OutputInjectionConfig {
    /// Whether to inject stdout
    pub inject_stdout: bool,

    /// Whether to inject stderr
    pub inject_stderr: bool,

    /// Maximum output length (0 = unlimited)
    pub max_length: usize,

    /// Whether to include exit code in output
    pub include_exit_code: bool,

    /// Whether to include execution time in output
    pub include_duration: bool,

    /// Format for injected output
    pub format: OutputFormat,
}

/// Output format for injection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Plain text format
    Plain,

    /// Markdown format with code blocks
    Markdown,

    /// JSON format
    Json,
}

/// Output injector for command results
pub struct OutputInjector;

impl OutputInjector {
    /// Inject command output into chat format
    pub fn inject(
        result: &CommandExecutionResult,
        config: &OutputInjectionConfig,
    ) -> Result<String> {
        match config.format {
            OutputFormat::Plain => Self::format_plain(result, config),
            OutputFormat::Markdown => Self::format_markdown(result, config),
            OutputFormat::Json => Self::format_json(result, config),
        }
    }

    /// Format output as plain text
    fn format_plain(
        result: &CommandExecutionResult,
        config: &OutputInjectionConfig,
    ) -> Result<String> {
        let mut output = String::new();

        if config.include_exit_code {
            output.push_str(&format!("Exit Code: {}\n", result.exit_code));
        }

        if config.include_duration {
            output.push_str(&format!("Duration: {}ms\n", result.duration_ms));
        }

        if config.inject_stdout && !result.stdout.is_empty() {
            let stdout = if config.max_length > 0 && result.stdout.len() > config.max_length {
                format!("{}... (truncated)", &result.stdout[..config.max_length])
            } else {
                result.stdout.clone()
            };
            output.push_str(&format!("Output:\n{}\n", stdout));
        }

        if config.inject_stderr && !result.stderr.is_empty() {
            let stderr = if config.max_length > 0 && result.stderr.len() > config.max_length {
                format!("{}... (truncated)", &result.stderr[..config.max_length])
            } else {
                result.stderr.clone()
            };
            output.push_str(&format!("Error:\n{}\n", stderr));
        }

        Ok(output.trim().to_string())
    }

    /// Format output as markdown
    fn format_markdown(
        result: &CommandExecutionResult,
        config: &OutputInjectionConfig,
    ) -> Result<String> {
        let mut output = String::new();

        if config.include_exit_code {
            output.push_str(&format!("**Exit Code:** {}\n", result.exit_code));
        }

        if config.include_duration {
            output.push_str(&format!("**Duration:** {}ms\n", result.duration_ms));
        }

        if config.inject_stdout && !result.stdout.is_empty() {
            let stdout = if config.max_length > 0 && result.stdout.len() > config.max_length {
                format!("{}... (truncated)", &result.stdout[..config.max_length])
            } else {
                result.stdout.clone()
            };
            output.push_str(&format!("```\n{}\n```\n", stdout));
        }

        if config.inject_stderr && !result.stderr.is_empty() {
            let stderr = if config.max_length > 0 && result.stderr.len() > config.max_length {
                format!("{}... (truncated)", &result.stderr[..config.max_length])
            } else {
                result.stderr.clone()
            };
            output.push_str(&format!("**Error:**\n```\n{}\n```\n", stderr));
        }

        Ok(output.trim().to_string())
    }

    /// Format output as JSON
    fn format_json(
        result: &CommandExecutionResult,
        config: &OutputInjectionConfig,
    ) -> Result<String> {
        let mut json = serde_json::json!({
            "command_id": result.command_id,
            "exit_code": result.exit_code,
            "success": result.success,
        });

        if config.include_duration {
            json["duration_ms"] = serde_json::json!(result.duration_ms);
        }

        if config.inject_stdout {
            let stdout = if config.max_length > 0 && result.stdout.len() > config.max_length {
                format!("{}... (truncated)", &result.stdout[..config.max_length])
            } else {
                result.stdout.clone()
            };
            json["stdout"] = serde_json::json!(stdout);
        }

        if config.inject_stderr {
            let stderr = if config.max_length > 0 && result.stderr.len() > config.max_length {
                format!("{}... (truncated)", &result.stderr[..config.max_length])
            } else {
                result.stderr.clone()
            };
            json["stderr"] = serde_json::json!(stderr);
        }

        Ok(serde_json::to_string_pretty(&json)?)
    }
}

impl Default for OutputInjectionConfig {
    fn default() -> Self {
        Self {
            inject_stdout: true,
            inject_stderr: true,
            max_length: 5000,
            include_exit_code: true,
            include_duration: true,
            format: OutputFormat::Markdown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_result() -> CommandExecutionResult {
        CommandExecutionResult::new("test-cmd", 0)
            .with_stdout("Hello World")
            .with_stderr("")
            .with_duration(100)
    }

    #[test]
    fn test_format_plain() {
        let result = create_test_result();
        let config = OutputInjectionConfig {
            format: OutputFormat::Plain,
            ..Default::default()
        };

        let output = OutputInjector::inject(&result, &config).unwrap();
        assert!(output.contains("Exit Code: 0"));
        assert!(output.contains("Duration: 100ms"));
        assert!(output.contains("Hello World"));
    }

    #[test]
    fn test_format_markdown() {
        let result = create_test_result();
        let config = OutputInjectionConfig {
            format: OutputFormat::Markdown,
            ..Default::default()
        };

        let output = OutputInjector::inject(&result, &config).unwrap();
        assert!(output.contains("**Exit Code:** 0"));
        assert!(output.contains("**Duration:** 100ms"));
        assert!(output.contains("```"));
        assert!(output.contains("Hello World"));
    }

    #[test]
    fn test_format_json() {
        let result = create_test_result();
        let config = OutputInjectionConfig {
            format: OutputFormat::Json,
            ..Default::default()
        };

        let output = OutputInjector::inject(&result, &config).unwrap();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["command_id"], "test-cmd");
        assert_eq!(json["exit_code"], 0);
        assert!(json["success"].as_bool().unwrap());
    }

    #[test]
    fn test_truncate_output() {
        let mut result = create_test_result();
        result.stdout = "a".repeat(10000);

        let config = OutputInjectionConfig {
            format: OutputFormat::Plain,
            max_length: 100,
            ..Default::default()
        };

        let output = OutputInjector::inject(&result, &config).unwrap();
        assert!(output.contains("(truncated)"));
        assert!(!output.contains(&"a".repeat(10000)));
    }

    #[test]
    fn test_exclude_stderr() {
        let mut result = create_test_result();
        result.stderr = "Error message".to_string();

        let config = OutputInjectionConfig {
            format: OutputFormat::Plain,
            inject_stderr: false,
            ..Default::default()
        };

        let output = OutputInjector::inject(&result, &config).unwrap();
        assert!(!output.contains("Error message"));
    }

    #[test]
    fn test_exclude_exit_code() {
        let result = create_test_result();
        let config = OutputInjectionConfig {
            format: OutputFormat::Plain,
            include_exit_code: false,
            ..Default::default()
        };

        let output = OutputInjector::inject(&result, &config).unwrap();
        assert!(!output.contains("Exit Code"));
    }
}
