//! Data models for local model management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Information about a local model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalModel {
    /// Model name/ID (e.g., "mistral:latest")
    pub name: String,

    /// Model size in bytes
    pub size: u64,

    /// Model digest/hash
    pub digest: String,

    /// When the model was last modified
    pub modified_at: DateTime<Utc>,

    /// Model metadata
    pub metadata: ModelMetadata,
}

/// Metadata about a model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelMetadata {
    /// Model format (e.g., "gguf")
    pub format: String,

    /// Model family (e.g., "llama", "mistral")
    pub family: String,

    /// Parameter size (e.g., "7B", "13B")
    pub parameter_size: String,

    /// Quantization level (e.g., "Q4_0", "Q5_K_M")
    pub quantization_level: String,
}

/// Progress information for model pull operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullProgress {
    /// Model name being pulled
    pub model: String,

    /// Current status message
    pub status: String,

    /// Model digest
    pub digest: String,

    /// Total bytes to download
    pub total: u64,

    /// Bytes downloaded so far
    pub completed: u64,
}

impl PullProgress {
    /// Get the progress percentage (0-100)
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.completed as f64 / self.total as f64) * 100.0
        }
    }

    /// Check if pull is complete
    pub fn is_complete(&self) -> bool {
        self.completed >= self.total && self.total > 0
    }
}

// Tests moved to tests/models.rs per project test organization policy
