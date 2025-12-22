//! Error types for the research system

use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur during research operations
#[derive(Debug, Error)]
pub enum ResearchError {
    /// Project not found at the specified path
    #[error("Project not found at {path}: {reason}")]
    ProjectNotFound {
        /// Path where project was expected
        path: PathBuf,
        /// Reason why project was not found
        reason: String,
    },

    /// Codebase scan failed
    #[error("Codebase scan failed: {reason}")]
    ScanFailed {
        /// Reason for scan failure
        reason: String,
    },

    /// Analysis operation failed
    #[error("Analysis failed: {reason}. Context: {context}")]
    AnalysisFailed {
        /// Reason for analysis failure
        reason: String,
        /// Additional context about what was being analyzed
        context: String,
    },

    /// Dependency parsing failed for a specific language
    #[error("Dependency parsing failed for {language}: {reason}")]
    DependencyParsingFailed {
        /// Programming language that failed to parse
        language: String,
        /// Path to the manifest file (stored but not displayed in error message)
        path: Option<PathBuf>,
        /// Reason for parsing failure
        reason: String,
    },

    /// Semantic index operation failed
    #[error("Semantic index error: {reason}. Operation: {operation}")]
    IndexError {
        /// Reason for index error
        reason: String,
        /// Operation that was being performed
        operation: String,
    },

    /// Search operation failed
    #[error("Search failed for query '{query}': {reason}")]
    SearchFailed {
        /// The search query that failed
        query: String,
        /// Reason for search failure
        reason: String,
    },

    /// Cache operation failed
    #[error("Cache error during {operation}: {reason}")]
    CacheError {
        /// Operation being performed (get, set, clear, etc.)
        operation: String,
        /// Reason for cache error
        reason: String,
    },

    /// IO error with context
    #[error("IO error: {reason}")]
    IoError {
        /// Reason for IO error
        reason: String,
    },

    /// Serialization error
    #[error("Serialization error: {reason}. Format: {format}")]
    SerializationError {
        /// Reason for serialization error
        reason: String,
        /// Format being serialized (JSON, TOML, YAML, etc.)
        format: String,
    },

    /// Invalid configuration
    #[error("Invalid configuration: {reason}. Expected: {expected}")]
    InvalidConfiguration {
        /// Reason why configuration is invalid
        reason: String,
        /// What was expected
        expected: String,
    },
}

impl From<serde_json::Error> for ResearchError {
    fn from(err: serde_json::Error) -> Self {
        ResearchError::SerializationError {
            reason: err.to_string(),
            format: "JSON".to_string(),
        }
    }
}

impl From<toml::de::Error> for ResearchError {
    fn from(err: toml::de::Error) -> Self {
        ResearchError::SerializationError {
            reason: err.to_string(),
            format: "TOML".to_string(),
        }
    }
}

impl From<serde_yaml::Error> for ResearchError {
    fn from(err: serde_yaml::Error) -> Self {
        ResearchError::SerializationError {
            reason: err.to_string(),
            format: "YAML".to_string(),
        }
    }
}
