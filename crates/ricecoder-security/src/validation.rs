//! Input validation and sanitization utilities

use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use serde_json;
use std::sync::Arc;

use crate::{audit::AuditLogger, Result, SecurityError};

/// Validated input wrapper
#[derive(Debug, Clone)]
pub struct ValidatedInput {
    pub content: String,
    pub is_sanitized: bool,
}

/// Validation error types
#[derive(Debug, Clone)]
pub enum ValidationError {
    EmptyInput,
    TooLong(usize),
    InvalidCharacters(String),
    SuspiciousPattern(String),
    CodeInjectionAttempt,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyInput => write!(f, "Input cannot be empty"),
            ValidationError::TooLong(len) => write!(f, "Input too long: {} characters", len),
            ValidationError::InvalidCharacters(chars) => write!(f, "Invalid characters: {}", chars),
            ValidationError::SuspiciousPattern(pattern) => {
                write!(f, "Suspicious pattern detected: {}", pattern)
            }
            ValidationError::CodeInjectionAttempt => {
                write!(f, "Potential code injection attempt detected")
            }
        }
    }
}

/// Validate and sanitize input
pub fn validate_input(input: &str) -> Result<ValidatedInput> {
    // Check for empty input
    if input.trim().is_empty() {
        return Err(SecurityError::Validation {
            message: "Input cannot be empty".to_string(),
        });
    }

    // Check length limits (reasonable for code/API keys)
    if input.len() > 10000 {
        return Err(SecurityError::Validation {
            message: format!("Input too long: {} characters (max 10000)", input.len()),
        });
    }

    // Check for suspicious patterns
    check_suspicious_patterns(input)?;

    // Sanitize the input
    let sanitized = sanitize_input(input);

    Ok(ValidatedInput {
        content: sanitized.clone(),
        is_sanitized: sanitized != input,
    })
}

/// Check for suspicious patterns that might indicate security issues
fn check_suspicious_patterns(input: &str) -> Result<()> {
    // Common injection patterns
    let injection_patterns = [
        r"<script[^>]*>.*?</script>", // Script tags
        r"javascript:",               // JavaScript URLs
        r"data:text/html",            // Data URLs
        r"vbscript:",                 // VBScript
        r"on\w+\s*=",                 // Event handlers
        r"eval\s*\(",                 // Eval calls
        r"document\.cookie",          // Cookie access
        r"localStorage",              // Local storage access
        r"sessionStorage",            // Session storage access
    ];

    for pattern in &injection_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(input) {
                return Err(SecurityError::Validation {
                    message: format!("Suspicious pattern detected: {}", pattern),
                });
            }
        }
    }

    // Check for potential path traversal
    if input.contains("..") || input.contains("\\") {
        let path_parts: Vec<&str> = input.split('/').collect();
        for part in path_parts {
            if part.contains("..") {
                return Err(SecurityError::Validation {
                    message: "Path traversal attempt detected".to_string(),
                });
            }
        }
    }

    Ok(())
}

/// Sanitize input by removing or escaping potentially dangerous content
fn sanitize_input(input: &str) -> String {
    let mut sanitized = input.to_string();

    // Remove null bytes
    sanitized = sanitized.replace('\0', "");

    // Remove control characters except common whitespace
    sanitized = sanitized
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .collect();

    // Trim excessive whitespace
    sanitized = sanitized.trim().to_string();

    sanitized
}

/// Validate API key format
pub fn validate_api_key_format(api_key: &str) -> Result<()> {
    // Basic API key format validation (adjust based on provider requirements)
    if api_key.len() < 20 {
        return Err(SecurityError::Validation {
            message: "API key too short".to_string(),
        });
    }

    if api_key.len() > 200 {
        return Err(SecurityError::Validation {
            message: "API key too long".to_string(),
        });
    }

    // Check for valid characters (alphanumeric, hyphens, underscores, dots)
    let valid_chars = api_key
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

    if !valid_chars {
        return Err(SecurityError::Validation {
            message: "API key contains invalid characters".to_string(),
        });
    }

    Ok(())
}

/// Validate code input for potential security issues
pub fn validate_code_input(code: &str) -> Result<ValidatedInput> {
    let validated = validate_input(code)?;

    // Additional checks for code
    if code.contains("import os") && code.contains("system(") {
        return Err(SecurityError::Validation {
            message: "Potentially dangerous code pattern detected".to_string(),
        });
    }

    Ok(validated)
}

/// Validate file path for security
pub fn validate_file_path(path: &str) -> Result<()> {
    // Check for absolute paths that might be dangerous
    if path.starts_with('/') || path.starts_with('\\') {
        return Err(SecurityError::Validation {
            message: "Absolute paths not allowed".to_string(),
        });
    }

    // Check for path traversal
    if path.contains("..") {
        return Err(SecurityError::Validation {
            message: "Path traversal not allowed".to_string(),
        });
    }

    // Check for suspicious extensions
    let suspicious_extensions = ["exe", "bat", "cmd", "sh", "dll", "so"];
    if let Some(ext) = std::path::Path::new(path).extension() {
        if let Some(ext_str) = ext.to_str() {
            if suspicious_extensions.contains(&ext_str) {
                return Err(SecurityError::Validation {
                    message: format!("Suspicious file extension: {}", ext_str),
                });
            }
        }
    }

    Ok(())
}

/// Security validator with audit logging
pub struct ValidationEngine {
    audit_logger: Arc<AuditLogger>,
}

impl ValidationEngine {
    /// Create a new security validator
    pub fn new(audit_logger: Arc<AuditLogger>) -> Self {
        Self { audit_logger }
    }

    /// Validate SQL input
    pub async fn validate_sql_input(&self, input: &str) -> Result<String> {
        // Basic SQL injection detection
        let sql_patterns = [
            r"(?i)union\s+select",
            r"(?i)drop\s+table",
            r"(?i)delete\s+from",
            r"(?i)insert\s+into",
            r"(?i)update\s+.*\s+set",
            r"--",
            r";",
        ];

        for pattern in &sql_patterns {
            if Regex::new(pattern).unwrap().is_match(input) {
                // Log security violation
                self.audit_logger
                    .log_security_violation(
                        "sql_injection_attempt",
                        serde_json::json!({
                            "input": input,
                            "pattern": pattern
                        }),
                    )
                    .await?;
                return Err(SecurityError::Validation {
                    message: "Potential SQL injection detected".to_string(),
                });
            }
        }

        // Sanitize by removing dangerous keywords
        let mut sanitized = input.to_string();
        sanitized = sanitized
            .replace("DROP", "")
            .replace("DELETE", "")
            .replace("UNION", "")
            .replace("--", "");

        Ok(sanitized)
    }

    /// Validate HTML input
    pub async fn validate_html_input(&self, input: &str) -> Result<String> {
        if input.contains("<script") || input.contains("javascript:") {
            self.audit_logger
                .log_security_violation(
                    "xss_attempt",
                    serde_json::json!({
                        "input": input
                    }),
                )
                .await?;
            return Err(SecurityError::Validation {
                message: "Potential XSS detected".to_string(),
            });
        }
        Ok(input.to_string())
    }

    /// Validate JavaScript input
    pub async fn validate_javascript_input(&self, input: &str) -> Result<String> {
        if input.contains("eval(") || input.contains("Function(") {
            self.audit_logger
                .log_security_violation(
                    "code_injection_attempt",
                    serde_json::json!({
                        "input": input
                    }),
                )
                .await?;
            return Err(SecurityError::Validation {
                message: "Potential code injection detected".to_string(),
            });
        }
        Ok(input.to_string())
    }

    /// Validate file path
    pub async fn validate_file_path(&self, path: &str) -> Result<String> {
        if path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
            self.audit_logger
                .log_security_violation(
                    "path_traversal_attempt",
                    serde_json::json!({
                        "path": path
                    }),
                )
                .await?;
            return Err(SecurityError::Validation {
                message: "Path traversal detected".to_string(),
            });
        }
        Ok(path.to_string())
    }

    /// Validate system command
    pub async fn validate_system_command(&self, command: &str) -> Result<String> {
        let dangerous_commands = ["rm", "del", "format", "shutdown"];
        for cmd in &dangerous_commands {
            if command.contains(cmd) {
                self.audit_logger
                    .log_security_violation(
                        "dangerous_command_attempt",
                        serde_json::json!({
                            "command": command
                        }),
                    )
                    .await?;
                return Err(SecurityError::Validation {
                    message: "Dangerous command detected".to_string(),
                });
            }
        }
        Ok(command.to_string())
    }

    /// Validate input size
    pub async fn validate_input_size(&self, input: &str) -> Result<String> {
        if input.len() > 10000 {
            self.audit_logger
                .log_security_violation(
                    "input_size_exceeded",
                    serde_json::json!({
                        "size": input.len()
                    }),
                )
                .await?;
            return Err(SecurityError::Validation {
                message: "Input size exceeded".to_string(),
            });
        }
        Ok(input.to_string())
    }

    /// Validate credentials
    pub async fn validate_credentials(&self, username: &str, password: &str) -> Result<()> {
        if username.is_empty() || password.is_empty() {
            return Err(SecurityError::Validation {
                message: "Empty credentials".to_string(),
            });
        }
        if password.len() < 8 {
            return Err(SecurityError::Validation {
                message: "Password too short".to_string(),
            });
        }
        Ok(())
    }

    /// Validate JSON input
    pub async fn validate_json_input(&self, input: &str) -> Result<String> {
        serde_json::from_str::<serde_json::Value>(input).map_err(|_| {
            SecurityError::Validation {
                message: "Invalid JSON".to_string(),
            }
        })?;
        Ok(input.to_string())
    }

    /// Validate encoded input
    pub async fn validate_encoded_input(&self, input: &str) -> Result<String> {
        // Basic check for base64-like input
        if input.contains("=")
            && input
                .chars()
                .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '=')
        {
            // Try to decode
            if general_purpose::STANDARD.decode(input).is_err() {
                return Err(SecurityError::Validation {
                    message: "Invalid base64".to_string(),
                });
            }
        }
        Ok(input.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_input_basic() {
        let input = "valid input";
        let result = validate_input(input).unwrap();
        assert_eq!(result.content, input);
        assert!(!result.is_sanitized);
    }

    #[test]
    fn test_validate_input_empty() {
        let result = validate_input("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_input_too_long() {
        let long_input = "a".repeat(10001);
        let result = validate_input(&long_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_input_script_injection() {
        let malicious_input = "<script>alert('xss')</script>";
        let result = validate_input(malicious_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_api_key_format() {
        // Valid API key
        let valid_key = "sk-test12345678901234567890123456789012";
        assert!(validate_api_key_format(valid_key).is_ok());

        // Too short
        let short_key = "short";
        assert!(validate_api_key_format(short_key).is_err());

        // Invalid characters
        let invalid_key = "sk-test@invalid#chars";
        assert!(validate_api_key_format(invalid_key).is_err());
    }

    #[test]
    fn test_validate_file_path() {
        // Valid relative path
        assert!(validate_file_path("src/main.rs").is_ok());

        // Absolute path
        assert!(validate_file_path("/etc/passwd").is_err());

        // Path traversal
        assert!(validate_file_path("../secret.txt").is_err());

        // Suspicious extension
        assert!(validate_file_path("malware.exe").is_err());
    }

    #[test]
    fn test_sanitize_input() {
        let input = "  hello\x00world\t\n  ";
        let result = validate_input(input).unwrap();
        assert_eq!(result.content, "helloworld"); // null byte should be removed
        assert!(result.is_sanitized);
    }
}
