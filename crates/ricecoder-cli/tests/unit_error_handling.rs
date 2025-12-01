// Unit tests for error handling
// **Feature: ricecoder-cli, Tests for Requirements 1.2, 5.1**

use ricecoder_cli::error::{CliError, CliResult};
use std::io;

// ============================================================================
// CliError Creation Tests
// ============================================================================

#[test]
fn test_command_not_found_error() {
    let error = CliError::CommandNotFound {
        command: "invalid".to_string(),
        suggestion: "init".to_string(),
    };
    
    match error {
        CliError::CommandNotFound { command, suggestion } => {
            assert_eq!(command, "invalid");
            assert_eq!(suggestion, "init");
        }
        _ => panic!("Expected CommandNotFound error"),
    }
}

#[test]
fn test_invalid_argument_error() {
    let error = CliError::InvalidArgument {
        message: "Missing required argument".to_string(),
    };
    
    match error {
        CliError::InvalidArgument { message } => {
            assert_eq!(message, "Missing required argument");
        }
        _ => panic!("Expected InvalidArgument error"),
    }
}

#[test]
fn test_config_error() {
    let error = CliError::Config("Config file not found".to_string());
    
    match error {
        CliError::Config(msg) => {
            assert_eq!(msg, "Config file not found");
        }
        _ => panic!("Expected Config error"),
    }
}

#[test]
fn test_provider_error() {
    let error = CliError::Provider("Unsupported provider".to_string());
    
    match error {
        CliError::Provider(msg) => {
            assert_eq!(msg, "Unsupported provider");
        }
        _ => panic!("Expected Provider error"),
    }
}

#[test]
fn test_generation_error() {
    let error = CliError::Generation("Failed to generate code".to_string());
    
    match error {
        CliError::Generation(msg) => {
            assert_eq!(msg, "Failed to generate code");
        }
        _ => panic!("Expected Generation error"),
    }
}

#[test]
fn test_storage_error() {
    let error = CliError::Storage("Failed to write to storage".to_string());
    
    match error {
        CliError::Storage(msg) => {
            assert_eq!(msg, "Failed to write to storage");
        }
        _ => panic!("Expected Storage error"),
    }
}

#[test]
fn test_internal_error() {
    let error = CliError::Internal("Unexpected error".to_string());
    
    match error {
        CliError::Internal(msg) => {
            assert_eq!(msg, "Unexpected error");
        }
        _ => panic!("Expected Internal error"),
    }
}

// ============================================================================
// IO Error Conversion Tests
// ============================================================================

#[test]
fn test_io_error_conversion() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let cli_error = CliError::from(io_error);
    
    match cli_error {
        CliError::Io(_) => {
            // Successfully converted
            assert!(true);
        }
        _ => panic!("Expected Io error"),
    }
}

#[test]
fn test_io_error_permission_denied() {
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
    let cli_error = CliError::from(io_error);
    
    match cli_error {
        CliError::Io(_) => {
            assert!(true);
        }
        _ => panic!("Expected Io error"),
    }
}

// ============================================================================
// User Message Tests
// ============================================================================

#[test]
fn test_command_not_found_user_message() {
    let error = CliError::CommandNotFound {
        command: "invalid".to_string(),
        suggestion: "init".to_string(),
    };
    
    let message = error.user_message();
    
    assert!(message.contains("invalid"));
    assert!(message.contains("init"));
    assert!(message.contains("Did you mean"));
}

#[test]
fn test_invalid_argument_user_message() {
    let error = CliError::InvalidArgument {
        message: "Missing spec file".to_string(),
    };
    
    let message = error.user_message();
    
    assert!(message.contains("Invalid argument"));
    assert!(message.contains("Missing spec file"));
    assert!(message.contains("help"));
}

#[test]
fn test_config_error_user_message() {
    let error = CliError::Config("Invalid configuration".to_string());
    
    let message = error.user_message();
    
    assert!(message.contains("Configuration error"));
    assert!(message.contains("Invalid configuration"));
    assert!(message.contains("rice config"));
}

#[test]
fn test_provider_error_user_message() {
    let error = CliError::Provider("Provider not available".to_string());
    
    let message = error.user_message();
    
    assert!(message.contains("Provider error"));
    assert!(message.contains("Provider not available"));
    assert!(message.contains("configuration"));
}

#[test]
fn test_generation_error_user_message() {
    let error = CliError::Generation("Syntax error in spec".to_string());
    
    let message = error.user_message();
    
    assert!(message.contains("Code generation failed"));
    assert!(message.contains("Syntax error in spec"));
}

#[test]
fn test_storage_error_user_message() {
    let error = CliError::Storage("Disk full".to_string());
    
    let message = error.user_message();
    
    assert!(message.contains("Storage error"));
    assert!(message.contains("Disk full"));
}

#[test]
fn test_internal_error_user_message() {
    let error = CliError::Internal("Unexpected panic".to_string());
    
    let message = error.user_message();
    
    assert!(message.contains("Internal error"));
    assert!(message.contains("Unexpected panic"));
    assert!(message.contains("report"));
}

#[test]
fn test_io_error_user_message() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error = CliError::from(io_error);
    
    let message = error.user_message();
    
    assert!(message.contains("File operation failed"));
}

// ============================================================================
// Error Message Helpfulness Tests (Property 6)
// ============================================================================

#[test]
fn test_error_message_includes_context() {
    let error = CliError::CommandNotFound {
        command: "gen".to_string(),
        suggestion: "generate".to_string(),
    };
    
    let message = error.user_message();
    
    // Should include the invalid command
    assert!(message.contains("gen"));
    // Should include the suggestion
    assert!(message.contains("generate"));
    // Should include helpful context
    assert!(message.contains("Did you mean"));
}

#[test]
fn test_error_message_includes_suggestions() {
    let error = CliError::InvalidArgument {
        message: "Missing required argument: spec".to_string(),
    };
    
    let message = error.user_message();
    
    // Should include the problem
    assert!(message.contains("Missing required argument"));
    // Should include a suggestion
    assert!(message.contains("help"));
}

#[test]
fn test_error_message_includes_documentation_link() {
    let error = CliError::Config("Invalid config format".to_string());
    
    let message = error.user_message();
    
    // Should include a reference to documentation or help
    assert!(message.contains("rice config") || message.contains("help"));
}

#[test]
fn test_all_error_types_have_helpful_messages() {
    let errors = vec![
        CliError::CommandNotFound {
            command: "test".to_string(),
            suggestion: "init".to_string(),
        },
        CliError::InvalidArgument {
            message: "test".to_string(),
        },
        CliError::Config("test".to_string()),
        CliError::Provider("test".to_string()),
        CliError::Generation("test".to_string()),
        CliError::Storage("test".to_string()),
        CliError::Internal("test".to_string()),
    ];
    
    for error in errors {
        let message = error.user_message();
        
        // All messages should be non-empty
        assert!(!message.is_empty());
        // All messages should contain helpful information
        assert!(message.len() > 10);
    }
}

// ============================================================================
// Technical Details Tests
// ============================================================================

#[test]
fn test_technical_details_command_not_found() {
    let error = CliError::CommandNotFound {
        command: "invalid".to_string(),
        suggestion: "init".to_string(),
    };
    
    let details = error.technical_details();
    
    assert!(details.contains("CommandNotFound"));
}

#[test]
fn test_technical_details_invalid_argument() {
    let error = CliError::InvalidArgument {
        message: "test".to_string(),
    };
    
    let details = error.technical_details();
    
    assert!(details.contains("InvalidArgument"));
}

#[test]
fn test_technical_details_config() {
    let error = CliError::Config("test".to_string());
    
    let details = error.technical_details();
    
    assert!(details.contains("Config"));
}

#[test]
fn test_technical_details_provider() {
    let error = CliError::Provider("test".to_string());
    
    let details = error.technical_details();
    
    assert!(details.contains("Provider"));
}

#[test]
fn test_technical_details_generation() {
    let error = CliError::Generation("test".to_string());
    
    let details = error.technical_details();
    
    assert!(details.contains("Generation"));
}

#[test]
fn test_technical_details_storage() {
    let error = CliError::Storage("test".to_string());
    
    let details = error.technical_details();
    
    assert!(details.contains("Storage"));
}

#[test]
fn test_technical_details_internal() {
    let error = CliError::Internal("test".to_string());
    
    let details = error.technical_details();
    
    assert!(details.contains("Internal"));
}

// ============================================================================
// CliResult Type Tests
// ============================================================================

#[test]
fn test_cli_result_ok() {
    let result: CliResult<i32> = Ok(42);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_cli_result_err() {
    let error = CliError::Internal("test".to_string());
    let result: CliResult<i32> = Err(error);
    
    assert!(result.is_err());
}

#[test]
fn test_cli_result_map() {
    let result: CliResult<i32> = Ok(5);
    let mapped = result.map(|x| x * 2);
    
    assert_eq!(mapped.unwrap(), 10);
}

#[test]
fn test_cli_result_map_err() {
    let error = CliError::Internal("test".to_string());
    let result: CliResult<i32> = Err(error);
    
    let mapped = result.map_err(|_| CliError::Internal("mapped".to_string()));
    
    assert!(mapped.is_err());
}

// ============================================================================
// Error Display Tests
// ============================================================================

#[test]
fn test_error_display_command_not_found() {
    let error = CliError::CommandNotFound {
        command: "invalid".to_string(),
        suggestion: "init".to_string(),
    };
    
    let display = format!("{}", error);
    
    assert!(display.contains("invalid"));
    assert!(display.contains("init"));
}

#[test]
fn test_error_display_invalid_argument() {
    let error = CliError::InvalidArgument {
        message: "test message".to_string(),
    };
    
    let display = format!("{}", error);
    
    assert!(display.contains("test message"));
}

#[test]
fn test_error_display_config() {
    let error = CliError::Config("test".to_string());
    
    let display = format!("{}", error);
    
    assert!(display.contains("Configuration error"));
}

#[test]
fn test_error_display_provider() {
    let error = CliError::Provider("test".to_string());
    
    let display = format!("{}", error);
    
    assert!(display.contains("Provider error"));
}

// ============================================================================
// Error Debug Tests
// ============================================================================

#[test]
fn test_error_debug_format() {
    let error = CliError::Internal("test".to_string());
    
    let debug = format!("{:?}", error);
    
    assert!(debug.contains("Internal"));
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn test_error_user_message_idempotent() {
    let error = CliError::Config("test".to_string());
    
    let message1 = error.user_message();
    let message2 = error.user_message();
    
    assert_eq!(message1, message2);
}

#[test]
fn test_error_technical_details_idempotent() {
    let error = CliError::Provider("test".to_string());
    
    let details1 = error.technical_details();
    let details2 = error.technical_details();
    
    assert_eq!(details1, details2);
}

#[test]
fn test_all_error_types_have_messages() {
    let errors = vec![
        CliError::CommandNotFound {
            command: "test".to_string(),
            suggestion: "init".to_string(),
        },
        CliError::InvalidArgument {
            message: "test".to_string(),
        },
        CliError::Config("test".to_string()),
        CliError::Provider("test".to_string()),
        CliError::Generation("test".to_string()),
        CliError::Storage("test".to_string()),
        CliError::Internal("test".to_string()),
    ];
    
    for error in errors {
        let user_msg = error.user_message();
        let tech_details = error.technical_details();
        
        // Both should be non-empty
        assert!(!user_msg.is_empty());
        assert!(!tech_details.is_empty());
    }
}

#[test]
fn test_error_messages_are_distinct() {
    let error1 = CliError::Config("error1".to_string());
    let error2 = CliError::Config("error2".to_string());
    
    let msg1 = error1.user_message();
    let msg2 = error2.user_message();
    
    // Messages should be different for different errors
    assert_ne!(msg1, msg2);
}
