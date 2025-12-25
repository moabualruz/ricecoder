//! Error handling tests
//!
//! Tests for CLI error types and user-friendly messages.

use ricecoder_cli::error::CliError;

#[test]
fn test_command_not_found_user_message() {
    let error = CliError::CommandNotFound {
        command: "unknwon".to_string(),
        suggestion: "unknown".to_string(),
    };
    let msg = error.user_message();
    assert!(msg.contains("unknwon"));
    assert!(msg.contains("unknown"));
    assert!(msg.contains("Did you mean"));
}

#[test]
fn test_invalid_argument_user_message() {
    let error = CliError::InvalidArgument {
        message: "Invalid path format".to_string(),
    };
    let msg = error.user_message();
    assert!(msg.contains("Invalid argument"));
    assert!(msg.contains("Invalid path format"));
}

#[test]
fn test_config_error_user_message() {
    let error = CliError::Config("Missing API key".to_string());
    let msg = error.user_message();
    assert!(msg.contains("Configuration error"));
    assert!(msg.contains("Missing API key"));
    assert!(msg.contains("rice config"));
}

#[test]
fn test_provider_error_user_message() {
    let error = CliError::Provider("Rate limit exceeded".to_string());
    let msg = error.user_message();
    assert!(msg.contains("Provider error"));
    assert!(msg.contains("Rate limit exceeded"));
}

#[test]
fn test_generation_error_user_message() {
    let error = CliError::Generation("Template parsing failed".to_string());
    let msg = error.user_message();
    assert!(msg.contains("generation failed"));
    assert!(msg.contains("Template parsing failed"));
}

#[test]
fn test_storage_error_user_message() {
    let error = CliError::Storage("Database connection failed".to_string());
    let msg = error.user_message();
    assert!(msg.contains("Storage error"));
    assert!(msg.contains("Database connection failed"));
}

#[test]
fn test_internal_error_user_message() {
    let error = CliError::Internal("Unexpected state".to_string());
    let msg = error.user_message();
    assert!(msg.contains("Internal error"));
    assert!(msg.contains("Unexpected state"));
    assert!(msg.contains("report"));
}

#[test]
fn test_file_not_found_user_message() {
    let error = CliError::FileNotFound {
        path: "/path/to/file.txt".to_string(),
    };
    let msg = error.user_message();
    assert!(msg.contains("File not found"));
    assert!(msg.contains("/path/to/file.txt"));
}

#[test]
fn test_permission_denied_user_message() {
    let error = CliError::PermissionDenied {
        path: "/etc/passwd".to_string(),
    };
    let msg = error.user_message();
    assert!(msg.contains("Permission denied"));
    assert!(msg.contains("/etc/passwd"));
    assert!(msg.contains("chmod"));
}

#[test]
fn test_invalid_config_format_user_message() {
    let error = CliError::InvalidConfigFormat {
        details: "Expected YAML, got JSON".to_string(),
    };
    let msg = error.user_message();
    assert!(msg.contains("Invalid configuration format"));
    assert!(msg.contains("Expected YAML, got JSON"));
}

#[test]
fn test_missing_field_user_message() {
    let error = CliError::MissingField {
        field: "api_key".to_string(),
    };
    let msg = error.user_message();
    assert!(msg.contains("Missing required field"));
    assert!(msg.contains("api_key"));
}

#[test]
fn test_network_error_user_message() {
    let error = CliError::NetworkError {
        details: "Connection timed out".to_string(),
    };
    let msg = error.user_message();
    assert!(msg.contains("Network error"));
    assert!(msg.contains("Connection timed out"));
}

#[test]
fn test_timeout_user_message() {
    let error = CliError::Timeout {
        operation: "API request".to_string(),
    };
    let msg = error.user_message();
    assert!(msg.contains("Timeout"));
    assert!(msg.contains("API request"));
}

#[test]
fn test_validation_error_user_message() {
    let error = CliError::Validation {
        message: "Invalid email format".to_string(),
    };
    let msg = error.user_message();
    assert!(msg.contains("Validation error"));
    assert!(msg.contains("Invalid email format"));
}

#[test]
fn test_short_message() {
    let error = CliError::FileNotFound {
        path: "/path/to/file".to_string(),
    };
    let short = error.short_message();
    assert!(short.contains("File not found"));
    assert!(short.contains("/path/to/file"));
    // Short message should be more concise than user_message
    assert!(short.len() < error.user_message().len());
}

#[test]
fn test_suggestions_for_command_not_found() {
    let error = CliError::CommandNotFound {
        command: "chta".to_string(),
        suggestion: "chat".to_string(),
    };
    let suggestions = error.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("help")));
}

#[test]
fn test_suggestions_for_config_error() {
    let error = CliError::Config("Invalid".to_string());
    let suggestions = error.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("config")));
}

#[test]
fn test_suggestions_for_provider_error() {
    let error = CliError::Provider("Failed".to_string());
    let suggestions = error.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("API key") || s.contains("network")));
}

#[test]
fn test_suggestions_for_file_not_found() {
    let error = CliError::FileNotFound {
        path: "missing.txt".to_string(),
    };
    let suggestions = error.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("path") || s.contains("exists")));
}

#[test]
fn test_technical_details() {
    let error = CliError::Internal("Debug info".to_string());
    let details = error.technical_details();
    assert!(details.contains("Internal"));
    assert!(details.contains("Debug info"));
}

#[test]
fn test_io_error_conversion() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let cli_error: CliError = io_error.into();
    assert!(matches!(cli_error, CliError::Io(_)));
}
