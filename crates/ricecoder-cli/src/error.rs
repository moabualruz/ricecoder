// Adapted from automation/src/cli/error.rs

use thiserror::Error;

/// CLI-specific errors
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
}

impl CliError {
    /// Get a user-friendly error message with suggestions
    pub fn user_message(&self) -> String {
        match self {
            CliError::CommandNotFound { command, suggestion } => {
                format!(
                    "Command '{}' not found.\n\nDid you mean: {}\n\nRun 'rice help' for available commands.",
                    command, suggestion
                )
            }
            CliError::InvalidArgument { message } => {
                format!("Invalid argument: {}\n\nRun 'rice help' for usage information.", message)
            }
            CliError::Io(e) => {
                format!("File operation failed: {}", e)
            }
            CliError::Config(msg) => {
                format!("Configuration error: {}\n\nRun 'rice config' to check your configuration.", msg)
            }
            CliError::Provider(msg) => {
                format!("Provider error: {}\n\nCheck your provider configuration with 'rice config'.", msg)
            }
            CliError::Generation(msg) => {
                format!("Code generation failed: {}", msg)
            }
            CliError::Storage(msg) => {
                format!("Storage error: {}\n\nCheck your storage configuration.", msg)
            }
            CliError::Internal(msg) => {
                format!("Internal error: {}\n\nPlease report this issue.", msg)
            }
        }
    }

    /// Get technical details for verbose mode
    pub fn technical_details(&self) -> String {
        format!("{:?}", self)
    }
}

pub type CliResult<T> = Result<T, CliError>;
