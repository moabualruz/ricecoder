use thiserror::Error;

/// Errors that can occur in the commands system
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Invalid command name: {0}")]
    InvalidCommandName(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Template parsing error: {0}")]
    TemplateError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CommandError>;
