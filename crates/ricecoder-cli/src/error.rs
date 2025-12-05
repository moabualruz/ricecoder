// Adapted from automation/src/cli/error.rs
// Enhanced with better error messages, suggestions, and documentation links

use thiserror::Error;

/// CLI-specific errors with enhanced context and suggestions
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Command not found: {command}. Did you mean: {suggestion}?")]
    CommandNotFound { command: String, suggestion: String },

    #[error("Invalid argument: {message}")]
    InvalidArgument { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Generation error: {0}")]
    Generation(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Invalid configuration format: {details}")]
    InvalidConfigFormat { details: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Network error: {details}")]
    NetworkError { details: String },

    #[error("Timeout: {operation}")]
    Timeout { operation: String },
}

impl CliError {
    /// Get a user-friendly error message with suggestions and documentation links
    pub fn user_message(&self) -> String {
        match self {
            CliError::CommandNotFound {
                command,
                suggestion,
            } => {
                format!(
                    "❌ Command '{}' not found.\n\n💡 Did you mean: {}\n\n📚 Run 'rice help' for available commands.\n📖 Documentation: https://ricecoder.dev/docs/commands",
                    command, suggestion
                )
            }
            CliError::InvalidArgument { message } => {
                format!(
                    "❌ Invalid argument: {}\n\n💡 Suggestion: Check the argument syntax and try again.\n\n📚 Run 'rice help' for usage information.\n📖 Documentation: https://ricecoder.dev/docs/cli-usage",
                    message
                )
            }
            CliError::Io(e) => {
                let suggestion = match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        "💡 Suggestion: Check that the file or directory exists.\n📖 Documentation: https://ricecoder.dev/docs/file-operations"
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        "💡 Suggestion: Check file permissions or run with appropriate privileges.\n📖 Documentation: https://ricecoder.dev/docs/permissions"
                    }
                    _ => {
                        "💡 Suggestion: Check your file system and try again.\n📖 Documentation: https://ricecoder.dev/docs/troubleshooting"
                    }
                };
                format!(
                    "❌ File operation failed: {}\n\n{}\n\n🔧 Technical details: {}",
                    e, suggestion, e
                )
            }
            CliError::Config(msg) => {
                format!(
                    "❌ Configuration error: {}\n\n💡 Suggestion: Run 'rice config' to check your configuration.\n\n📚 Common issues:\n  • Missing RICECODER_HOME environment variable\n  • Invalid configuration file format\n  • Missing required configuration fields\n\n📖 Documentation: https://ricecoder.dev/docs/configuration",
                    msg
                )
            }
            CliError::Provider(msg) => {
                format!(
                    "❌ Provider error: {}\n\n💡 Suggestion: Check your provider configuration with 'rice config'.\n\n📚 Common issues:\n  • Invalid API key\n  • Provider service unavailable\n  • Network connectivity issues\n\n📖 Documentation: https://ricecoder.dev/docs/providers",
                    msg
                )
            }
            CliError::Generation(msg) => {
                format!(
                    "❌ Code generation failed: {}\n\n💡 Suggestion: Check your specification and try again.\n\n📚 Common issues:\n  • Invalid specification format\n  • Missing required fields in specification\n  • Provider rate limit exceeded\n\n📖 Documentation: https://ricecoder.dev/docs/generation",
                    msg
                )
            }
            CliError::Storage(msg) => {
                format!(
                    "❌ Storage error: {}\n\n💡 Suggestion: Check your storage configuration.\n\n📚 Common issues:\n  • Insufficient disk space\n  • Invalid storage path\n  • Permission issues\n\n📖 Documentation: https://ricecoder.dev/docs/storage",
                    msg
                )
            }
            CliError::Internal(msg) => {
                format!(
                    "❌ Internal error: {}\n\n💡 This is unexpected. Please report this issue.\n\n📚 How to report:\n  1. Run 'rice --verbose' to get more details\n  2. Include the output in your bug report\n  3. Visit: https://github.com/ricecoder/ricecoder/issues\n\n📖 Documentation: https://ricecoder.dev/docs/troubleshooting",
                    msg
                )
            }
            CliError::FileNotFound { path } => {
                format!(
                    "❌ File not found: {}\n\n💡 Suggestion: Check that the file exists and the path is correct.\n\n📚 Common issues:\n  • Typo in file path\n  • File was deleted or moved\n  • Relative path is incorrect\n\n📖 Documentation: https://ricecoder.dev/docs/file-operations",
                    path
                )
            }
            CliError::PermissionDenied { path } => {
                format!(
                    "❌ Permission denied: {}\n\n💡 Suggestion: Check file permissions or run with appropriate privileges.\n\n📚 To fix:\n  • Check file ownership: ls -l {}\n  • Change permissions: chmod u+r {}\n  • Or run with sudo (not recommended)\n\n📖 Documentation: https://ricecoder.dev/docs/permissions",
                    path, path, path
                )
            }
            CliError::InvalidConfigFormat { details } => {
                format!(
                    "❌ Invalid configuration format: {}\n\n💡 Suggestion: Check your configuration file syntax.\n\n📚 Supported formats:\n  • YAML (.yaml, .yml)\n  • TOML (.toml)\n  • JSON (.json)\n\n📖 Documentation: https://ricecoder.dev/docs/configuration-format",
                    details
                )
            }
            CliError::MissingField { field } => {
                format!(
                    "❌ Missing required field: {}\n\n💡 Suggestion: Add the missing field to your configuration.\n\n📚 Required fields depend on your use case.\n\n📖 Documentation: https://ricecoder.dev/docs/configuration-reference",
                    field
                )
            }
            CliError::NetworkError { details } => {
                format!(
                    "❌ Network error: {}\n\n💡 Suggestion: Check your internet connection and try again.\n\n📚 Common issues:\n  • No internet connection\n  • Firewall blocking the connection\n  • Provider service is down\n\n📖 Documentation: https://ricecoder.dev/docs/network-troubleshooting",
                    details
                )
            }
            CliError::Timeout { operation } => {
                format!(
                    "❌ Timeout: {} took too long\n\n💡 Suggestion: Try again or increase the timeout.\n\n📚 Common issues:\n  • Slow internet connection\n  • Provider service is slow\n  • Large input data\n\n📖 Documentation: https://ricecoder.dev/docs/performance",
                    operation
                )
            }
        }
    }

    /// Get technical details for verbose mode
    pub fn technical_details(&self) -> String {
        format!("{:?}", self)
    }

    /// Get a short error message (for inline display)
    pub fn short_message(&self) -> String {
        match self {
            CliError::CommandNotFound { command, .. } => {
                format!("Command '{}' not found", command)
            }
            CliError::InvalidArgument { message } => {
                format!("Invalid argument: {}", message)
            }
            CliError::Io(e) => format!("File operation failed: {}", e),
            CliError::Config(msg) => format!("Configuration error: {}", msg),
            CliError::Provider(msg) => format!("Provider error: {}", msg),
            CliError::Generation(msg) => format!("Generation failed: {}", msg),
            CliError::Storage(msg) => format!("Storage error: {}", msg),
            CliError::Internal(msg) => format!("Internal error: {}", msg),
            CliError::FileNotFound { path } => format!("File not found: {}", path),
            CliError::PermissionDenied { path } => format!("Permission denied: {}", path),
            CliError::InvalidConfigFormat { details } => {
                format!("Invalid config format: {}", details)
            }
            CliError::MissingField { field } => format!("Missing field: {}", field),
            CliError::NetworkError { details } => format!("Network error: {}", details),
            CliError::Timeout { operation } => format!("Timeout: {}", operation),
        }
    }

    /// Get actionable suggestions for this error
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            CliError::CommandNotFound { .. } => vec![
                "Run 'rice help' to see available commands".to_string(),
                "Check the command spelling".to_string(),
            ],
            CliError::InvalidArgument { .. } => vec![
                "Check the argument syntax".to_string(),
                "Run 'rice help <command>' for usage".to_string(),
            ],
            CliError::Config(_) => vec![
                "Run 'rice config' to check configuration".to_string(),
                "Check RICECODER_HOME environment variable".to_string(),
                "Verify configuration file format".to_string(),
            ],
            CliError::Provider(_) => vec![
                "Check provider API key".to_string(),
                "Verify provider is available".to_string(),
                "Check network connectivity".to_string(),
            ],
            CliError::FileNotFound { .. } => vec![
                "Check file path spelling".to_string(),
                "Verify file exists".to_string(),
                "Use absolute path if relative path fails".to_string(),
            ],
            CliError::PermissionDenied { .. } => vec![
                "Check file permissions".to_string(),
                "Run with appropriate privileges".to_string(),
            ],
            CliError::NetworkError { .. } => vec![
                "Check internet connection".to_string(),
                "Check firewall settings".to_string(),
                "Try again later if service is down".to_string(),
            ],
            CliError::Timeout { .. } => vec![
                "Try again".to_string(),
                "Check internet speed".to_string(),
                "Increase timeout if available".to_string(),
            ],
            _ => vec!["Check documentation for more details".to_string()],
        }
    }
}

pub type CliResult<T> = Result<T, CliError>;
