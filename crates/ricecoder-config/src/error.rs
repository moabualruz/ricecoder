//! Configuration error types

use thiserror::Error;

/// Configuration result type
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Config file not found: {0}")]
    NotFound(String),

    #[error("Environment error: {0}")]
    Env(String),

    #[error("Config library error: {0}")]
    ConfigLib(#[from] ::config::ConfigError),

    #[error("TOML deserialize error: {0}")]
    TomlDe(#[from] ::toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSer(#[from] ::toml::ser::Error),
}