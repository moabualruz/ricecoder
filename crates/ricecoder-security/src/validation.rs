//! Input validation and sanitization utilities

use regex::Regex;

use crate::{SecurityError, Result};

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
            ValidationError::SuspiciousPattern(pattern) => write!(f, "Suspicious pattern detected: {}", pattern),
            ValidationError::CodeInjectionAttempt => write!(f, "Potential code injection attempt detected"),
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
        r"<script[^>]*>.*?</script>",  // Script tags
        r"javascript:",                // JavaScript URLs
        r"data:text/html",             // Data URLs
        r"vbscript:",                  // VBScript
        r"on\w+\s*=",                  // Event handlers
        r"eval\s*\(",                  // Eval calls
        r"document\.cookie",           // Cookie access
        r"localStorage",               // Local storage access
        r"sessionStorage",             // Session storage access
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
    let valid_chars = api_key.chars().all(|c| {
        c.is_alphanumeric() || c == '-' || c == '_' || c == '.'
    });

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