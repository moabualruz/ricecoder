//! Error types for the updates crate

use std::fmt;
use thiserror::Error;

/// Result type alias for update operations
pub type Result<T> = std::result::Result<T, UpdateError>;

/// Comprehensive error type for update operations
#[derive(Debug, Error)]
pub enum UpdateError {
    /// Network-related errors during update checking or downloading
    #[error("Network error: {source}")]
    Network {
        #[from]
        source: reqwest::Error,
    },

    /// I/O errors during file operations
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// JSON parsing errors
    #[error("JSON parsing error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },

    /// YAML parsing errors
    #[error("YAML parsing error: {source}")]
    Yaml {
        #[from]
        source: serde_yaml::Error,
    },

    /// URL parsing errors
    #[error("URL parsing error: {source}")]
    Url {
        #[from]
        source: url::ParseError,
    },

    /// Semver parsing errors
    #[error("Version parsing error: {source}")]
    Semver {
        #[from]
        source: semver::Error,
    },

    /// Security validation errors
    #[error("Security validation failed: {message}")]
    SecurityValidation { message: String },

    /// Policy violation errors
    #[error("Policy violation: {message}")]
    PolicyViolation { message: String },

    /// Update check errors
    #[error("Update check failed: {message}")]
    UpdateCheck { message: String },

    /// Download errors
    #[error("Download failed: {message}")]
    Download { message: String },

    /// Installation errors
    #[error("Installation failed: {message}")]
    Installation { message: String },

    /// Rollback errors
    #[error("Rollback failed: {message}")]
    Rollback { message: String },

    /// Analytics errors
    #[error("Analytics error: {message}")]
    Analytics { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Generic update errors
    #[error("Update error: {message}")]
    Generic { message: String },
}

impl UpdateError {
    /// Create a new security validation error
    pub fn security_validation<S: Into<String>>(message: S) -> Self {
        Self::SecurityValidation {
            message: message.into(),
        }
    }

    /// Create a new policy violation error
    pub fn policy_violation<S: Into<String>>(message: S) -> Self {
        Self::PolicyViolation {
            message: message.into(),
        }
    }

    /// Create a new update check error
    pub fn update_check<S: Into<String>>(message: S) -> Self {
        Self::UpdateCheck {
            message: message.into(),
        }
    }

    /// Create a new download error
    pub fn download<S: Into<String>>(message: S) -> Self {
        Self::Download {
            message: message.into(),
        }
    }

    /// Create a new installation error
    pub fn installation<S: Into<String>>(message: S) -> Self {
        Self::Installation {
            message: message.into(),
        }
    }

    /// Create a new rollback error
    pub fn rollback<S: Into<String>>(message: S) -> Self {
        Self::Rollback {
            message: message.into(),
        }
    }

    /// Create a new analytics error
    pub fn analytics<S: Into<String>>(message: S) -> Self {
        Self::Analytics {
            message: message.into(),
        }
    }

    /// Create a new configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a new generic error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }
}