//! Configuration management for continuous improvement pipeline

pub use crate::types::*;

/// Load configuration from file or environment
pub fn load_config() -> Result<ContinuousImprovementConfig, ConfigError> {
    // In a real implementation, this would load from TOML/JSON files
    // and environment variables. For now, return defaults.
    Ok(ContinuousImprovementConfig::default())
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}