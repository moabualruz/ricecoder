//! Core data structures for external LSP integration

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Configuration for an external LSP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspServerConfig {
    /// Language identifier (e.g., "rust", "typescript")
    pub language: String,
    /// File extensions this server handles
    pub extensions: Vec<String>,
    /// Executable path (can use $PATH)
    pub executable: String,
    /// Command line arguments
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Initialization options (sent in initialize request)
    pub init_options: Option<serde_json::Value>,
    /// Whether this server is enabled
    pub enabled: bool,
    /// Timeout for requests in milliseconds
    pub timeout_ms: u64,
    /// Maximum restart attempts
    pub max_restarts: u32,
    /// Idle timeout before shutdown (0 = never)
    pub idle_timeout_ms: u64,
    /// Output mapping rules for transforming LSP responses
    pub output_mapping: Option<OutputMappingConfig>,
}

/// Configuration for mapping LSP server output to ricecoder models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMappingConfig {
    /// Mapping rules for completion items
    pub completion: Option<CompletionMappingRules>,
    /// Mapping rules for diagnostics
    pub diagnostics: Option<DiagnosticsMappingRules>,
    /// Mapping rules for hover information
    pub hover: Option<HoverMappingRules>,
    /// Custom transformation functions (by name)
    pub custom_transforms: Option<HashMap<String, String>>,
}

/// Mapping rules for completion items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionMappingRules {
    /// JSON path to completion items array
    pub items_path: String,
    /// Field mappings for each completion item
    pub field_mappings: HashMap<String, String>,
    /// Optional transformation function name
    pub transform: Option<String>,
}

/// Mapping rules for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsMappingRules {
    /// JSON path to diagnostics array
    pub items_path: String,
    /// Field mappings for each diagnostic
    pub field_mappings: HashMap<String, String>,
    /// Optional transformation function name
    pub transform: Option<String>,
}

/// Mapping rules for hover information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverMappingRules {
    /// JSON path to hover content
    pub content_path: String,
    /// Field mappings for hover data
    pub field_mappings: HashMap<String, String>,
    /// Optional transformation function name
    pub transform: Option<String>,
}

/// Registry of all configured LSP servers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LspServerRegistry {
    /// Map of language to server configurations
    pub servers: HashMap<String, Vec<LspServerConfig>>,
    /// Global settings
    pub global: GlobalLspSettings,
}

/// Global LSP settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalLspSettings {
    /// Maximum concurrent LSP server processes
    pub max_processes: usize,
    /// Default request timeout
    pub default_timeout_ms: u64,
    /// Enable fallback to internal providers
    pub enable_fallback: bool,
    /// Health check interval
    pub health_check_interval_ms: u64,
}

impl Default for GlobalLspSettings {
    fn default() -> Self {
        Self {
            max_processes: 5,
            default_timeout_ms: 5000,
            enable_fallback: true,
            health_check_interval_ms: 30000,
        }
    }
}

/// State of an LSP client connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    /// Not started
    Stopped,
    /// Starting up
    Starting,
    /// Running and healthy
    Running,
    /// Unhealthy (failed health checks)
    Unhealthy,
    /// Shutting down
    ShuttingDown,
    /// Crashed (will attempt restart)
    Crashed,
}

/// Health status of an LSP server
#[derive(Debug, Clone)]
pub enum HealthStatus {
    /// Server is healthy
    Healthy { latency: std::time::Duration },
    /// Server is unhealthy
    Unhealthy { reason: String },
}

/// Source of an LSP result
#[derive(Debug, Clone)]
pub enum ResultSource {
    /// From external LSP server
    External { server: String },
    /// From internal fallback provider
    Internal,
    /// Merged from multiple sources
    Merged { sources: Vec<String> },
}

/// Result from external LSP with source tracking
pub struct ExternalLspResult<T> {
    /// The result data
    pub data: T,
    /// Source of the result
    pub source: ResultSource,
    /// Time taken to get result
    pub latency: std::time::Duration,
}

/// Configuration for merging results from multiple sources
#[derive(Debug, Clone)]
pub struct MergeConfig {
    /// Include internal provider results
    pub include_internal: bool,
    /// Deduplicate results
    pub deduplicate: bool,
}

impl Default for MergeConfig {
    fn default() -> Self {
        Self {
            include_internal: true,
            deduplicate: true,
        }
    }
}
