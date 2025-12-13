//! Comprehensive error types for the RiceCoder TUI
//!
//! This module provides a hierarchical error type system for the TUI crate,
//! covering all major error scenarios that can occur during TUI operations.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for TUI operations
pub type TuiResult<T> = Result<T, TuiError>;

/// Main error type for TUI operations
#[derive(Error, Debug)]
pub enum TuiError {
    /// IO errors (file operations, network, etc.)
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Theme errors
    #[error("Theme error: {message}")]
    Theme { message: String },

    /// Rendering errors
    #[error("Rendering error: {message}")]
    Render { message: String },

    /// Widget errors
    #[error("Widget error: {message}")]
    Widget { message: String },

    /// Event handling errors
    #[error("Event error: {message}")]
    Event { message: String },

    /// Command execution errors
    #[error("Command error: {message}")]
    Command { message: String },

    /// Session management errors
    #[error("Session error: {0}")]
    Session(#[from] SessionError),

    /// Provider integration errors
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    /// Tool execution errors
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),

    /// VCS integration errors
    #[error("VCS error: {message}")]
    Vcs { message: String },

    /// LSP integration errors
    #[error("LSP error: {message}")]
    Lsp { message: String },

    /// Clipboard errors
    #[error("Clipboard error: {0}")]
    Clipboard(#[from] ClipboardError),

    /// Plugin system errors
    #[error("Plugin error: {message}")]
    Plugin { message: String },

    /// Terminal/Crossterm errors
    #[error("Terminal error: {message}")]
    Terminal { message: String },

    /// Image processing errors
    #[error("Image error: {message}")]
    Image { message: String },

    /// Markdown processing errors
    #[error("Markdown error: {message}")]
    Markdown { message: String },

    /// JSON parsing/serialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML parsing/serialization errors
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// TOML parsing/serialization errors
    #[error("TOML error: {message}")]
    Toml { message: String },

    /// Regex compilation errors
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Task scheduling errors
    #[error("Task error: {message}")]
    Task { message: String },

    /// Performance monitoring errors
    #[error("Performance error: {message}")]
    Performance { message: String },

    /// Accessibility errors
    #[error("Accessibility error: {message}")]
    Accessibility { message: String },

    /// Security-related errors
    #[error("Security error: {message}")]
    Security { message: String },

    /// Network communication errors
    #[error("Network error: {message}")]
    Network { message: String },

    /// Database/storage errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Keybind errors
    #[error("Keybind error: {0}")]
    Keybind(#[from] KeybindError),

    /// Validation errors
    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    /// State management errors
    #[error("State error: {message}")]
    State { message: String },

    /// Initialization errors
    #[error("Initialization error: {message}")]
    Init { message: String },

    /// Shutdown errors
    #[error("Shutdown error: {message}")]
    Shutdown { message: String },

    /// Timeout errors
    #[error("Timeout error: {operation} timed out after {duration}ms")]
    Timeout { operation: String, duration: u64 },

    /// Cancellation errors
    #[error("Operation cancelled: {operation}")]
    Cancelled { operation: String },

    /// Resource exhaustion errors
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    /// Version compatibility errors
    #[error("Version error: {message}")]
    Version { message: String },

    /// Generic internal errors
    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Session management errors
#[derive(Error, Debug)]
pub enum SessionError {
    /// Session not found
    #[error("Session not found: {id}")]
    NotFound { id: String },

    /// Session already exists
    #[error("Session already exists: {id}")]
    AlreadyExists { id: String },

    /// Session corrupted
    #[error("Session corrupted: {id} - {reason}")]
    Corrupted { id: String, reason: String },

    /// Session save failed
    #[error("Failed to save session {id}: {source}")]
    SaveFailed {
        id: String,
        source: std::io::Error,
    },

    /// Session load failed
    #[error("Failed to load session {id}: {source}")]
    LoadFailed {
        id: String,
        source: std::io::Error,
    },

    /// Session migration failed
    #[error("Failed to migrate session {id}: {reason}")]
    MigrationFailed { id: String, reason: String },

    /// Session limit exceeded
    #[error("Session limit exceeded: {current}/{max}")]
    LimitExceeded { current: usize, max: usize },

    /// Session locked
    #[error("Session locked: {id}")]
    Locked { id: String },

    /// Session expired
    #[error("Session expired: {id}")]
    Expired { id: String },

    /// Invalid session data
    #[error("Invalid session data: {field} - {reason}")]
    InvalidData { field: String, reason: String },
}

/// Tool execution errors
#[derive(Error, Debug)]
pub enum ToolError {
    /// Tool not found
    #[error("Tool not found: {name}")]
    NotFound { name: String },

    /// Tool execution failed
    #[error("Tool execution failed: {name} - {message}")]
    ExecutionFailed { name: String, message: String },

    /// Tool timeout
    #[error("Tool timeout: {name} after {timeout}ms")]
    Timeout { name: String, timeout: u64 },

    /// Tool permission denied
    #[error("Tool permission denied: {name}")]
    PermissionDenied { name: String },

    /// Tool configuration error
    #[error("Tool configuration error: {name} - {message}")]
    ConfigError { name: String, message: String },

    /// Tool input validation failed
    #[error("Tool input validation failed: {name} - {field}: {reason}")]
    InputValidationFailed { name: String, field: String, reason: String },

    /// Tool output parsing failed
    #[error("Tool output parsing failed: {name} - {reason}")]
    OutputParsingFailed { name: String, reason: String },

    /// Tool resource exhausted
    #[error("Tool resource exhausted: {name} - {resource}")]
    ResourceExhausted { name: String, resource: String },

    /// Tool cancelled
    #[error("Tool cancelled: {name}")]
    Cancelled { name: String },
}

/// Provider integration errors (wrapper around ricecoder_providers::ProviderError)
#[derive(Error, Debug)]
pub enum ProviderError {
    /// Provider not found
    #[error("Provider not found: {0}")]
    NotFound(String),

    /// Authentication failed
    #[error("Authentication failed")]
    AuthError,

    /// Rate limited
    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    /// Context too large
    #[error("Context too large: {0} tokens, max {1}")]
    ContextTooLarge(usize, usize),

    /// Network error
    #[error("Network error")]
    NetworkError,

    /// Generic provider error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid model
    #[error("Invalid model: {0}")]
    InvalidModel(String),

    /// Model not available
    #[error("Model not available: {0}")]
    ModelNotAvailable(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Clipboard operation errors
#[derive(Error, Debug)]
pub enum ClipboardError {
    /// Clipboard not available
    #[error("Clipboard not available")]
    NotAvailable,

    /// Clipboard operation failed
    #[error("Clipboard operation failed: {message}")]
    OperationFailed { message: String },

    /// Content too large
    #[error("Content too large for clipboard: {size} bytes")]
    ContentTooLarge { size: usize },

    /// Unsupported content type
    #[error("Unsupported content type: {content_type}")]
    UnsupportedContentType { content_type: String },

    /// Permission denied
    #[error("Clipboard permission denied")]
    PermissionDenied,
}

/// Storage operation errors (wrapper around ricecoder_storage::StorageError)
#[derive(Error, Debug)]
pub enum StorageError {
    /// IO error
    #[error("IO error: {operation} on {path}: {source}")]
    IoError {
        path: PathBuf,
        operation: String,
        source: std::io::Error,
    },

    /// Parse error
    #[error("Parse error: {path} as {format}: {message}")]
    ParseError {
        path: PathBuf,
        format: String,
        message: String,
    },

    /// Validation error
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    /// Path resolution error
    #[error("Path resolution error: {message}")]
    PathResolutionError { message: String },

    /// Environment variable error
    #[error("Environment variable error: {var_name} - {message}")]
    EnvVarError { var_name: String, message: String },

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Keybind operation errors (wrapper around ricecoder_keybinds errors)
#[derive(Error, Debug)]
pub enum KeybindError {
    /// Registry error
    #[error("Registry error: {message}")]
    Registry { message: String },

    /// Profile error
    #[error("Profile error: {message}")]
    Profile { message: String },

    /// Parse error
    #[error("Parse error: {message}")]
    Parse { message: String },

    /// Persistence error
    #[error("Persistence error: {message}")]
    Persistence { message: String },

    /// Engine error
    #[error("Engine error: {message}")]
    Engine { message: String },
}

// Conversion implementations
impl From<ricecoder_providers::ProviderError> for ProviderError {
    fn from(err: ricecoder_providers::ProviderError) -> Self {
        match err {
            ricecoder_providers::ProviderError::NotFound(s) => ProviderError::NotFound(s),
            ricecoder_providers::ProviderError::AuthError => ProviderError::AuthError,
            ricecoder_providers::ProviderError::RateLimited(d) => ProviderError::RateLimited(d),
            ricecoder_providers::ProviderError::ContextTooLarge(a, b) => ProviderError::ContextTooLarge(a, b),
            ricecoder_providers::ProviderError::NetworkError => ProviderError::NetworkError,
            ricecoder_providers::ProviderError::ProviderError(s) => ProviderError::ProviderError(s),
            ricecoder_providers::ProviderError::ConfigError(s) => ProviderError::ConfigError(s),
            ricecoder_providers::ProviderError::InvalidModel(s) => ProviderError::InvalidModel(s),
            ricecoder_providers::ProviderError::ModelNotAvailable(s) => ProviderError::ModelNotAvailable(s),
            ricecoder_providers::ProviderError::SerializationError(s) => ProviderError::SerializationError(s),
            ricecoder_providers::ProviderError::Internal(s) => ProviderError::Internal(s),
        }
    }
}

impl From<ricecoder_storage::StorageError> for StorageError {
    fn from(err: ricecoder_storage::StorageError) -> Self {
        match err {
            ricecoder_storage::StorageError::IoError { path, operation, source } => {
                StorageError::IoError {
                    path,
                    operation: operation.to_string(),
                    source,
                }
            }
            ricecoder_storage::StorageError::ParseError { path, format, message } => {
                StorageError::ParseError { path, format, message }
            }
            ricecoder_storage::StorageError::ValidationError { field, message } => {
                StorageError::ValidationError { field, message }
            }
            ricecoder_storage::StorageError::PathResolutionError { message } => {
                StorageError::PathResolutionError { message }
            }
            ricecoder_storage::StorageError::EnvVarError { var_name, message } => {
                StorageError::EnvVarError { var_name, message }
            }
            ricecoder_storage::StorageError::Internal(s) => StorageError::Internal(s),
            _ => StorageError::Internal(format!("Unhandled storage error: {:?}", err)),
        }
    }
}

impl From<ricecoder_keybinds::error::EngineError> for KeybindError {
    fn from(err: ricecoder_keybinds::error::EngineError) -> Self {
        KeybindError::Engine { message: err.to_string() }
    }
}

impl From<ricecoder_keybinds::error::RegistryError> for KeybindError {
    fn from(err: ricecoder_keybinds::error::RegistryError) -> Self {
        KeybindError::Registry { message: err.to_string() }
    }
}

impl From<ricecoder_keybinds::error::ProfileError> for KeybindError {
    fn from(err: ricecoder_keybinds::error::ProfileError) -> Self {
        KeybindError::Profile { message: err.to_string() }
    }
}

impl From<ricecoder_keybinds::error::ParseError> for KeybindError {
    fn from(err: ricecoder_keybinds::error::ParseError) -> Self {
        KeybindError::Parse { message: err.to_string() }
    }
}

impl From<ricecoder_keybinds::error::PersistenceError> for KeybindError {
    fn from(err: ricecoder_keybinds::error::PersistenceError) -> Self {
        KeybindError::Persistence { message: err.to_string() }
    }
}

// Convenience constructors
impl TuiError {
    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        TuiError::Config { message: message.into() }
    }

    /// Create a theme error
    pub fn theme(message: impl Into<String>) -> Self {
        TuiError::Theme { message: message.into() }
    }

    /// Create a rendering error
    pub fn render(message: impl Into<String>) -> Self {
        TuiError::Render { message: message.into() }
    }

    /// Create a widget error
    pub fn widget(message: impl Into<String>) -> Self {
        TuiError::Widget { message: message.into() }
    }

    /// Create an event error
    pub fn event(message: impl Into<String>) -> Self {
        TuiError::Event { message: message.into() }
    }

    /// Create a command error
    pub fn command(message: impl Into<String>) -> Self {
        TuiError::Command { message: message.into() }
    }

    /// Create a VCS error
    pub fn vcs(message: impl Into<String>) -> Self {
        TuiError::Vcs { message: message.into() }
    }

    /// Create an LSP error
    pub fn lsp(message: impl Into<String>) -> Self {
        TuiError::Lsp { message: message.into() }
    }

    /// Create a terminal error
    pub fn terminal(message: impl Into<String>) -> Self {
        TuiError::Terminal { message: message.into() }
    }

    /// Create an image error
    pub fn image(message: impl Into<String>) -> Self {
        TuiError::Image { message: message.into() }
    }

    /// Create a markdown error
    pub fn markdown(message: impl Into<String>) -> Self {
        TuiError::Markdown { message: message.into() }
    }

    /// Create a TOML error
    pub fn toml(message: impl Into<String>) -> Self {
        TuiError::Toml { message: message.into() }
    }

    /// Create a task error
    pub fn task(message: impl Into<String>) -> Self {
        TuiError::Task { message: message.into() }
    }

    /// Create a performance error
    pub fn performance(message: impl Into<String>) -> Self {
        TuiError::Performance { message: message.into() }
    }

    /// Create an accessibility error
    pub fn accessibility(message: impl Into<String>) -> Self {
        TuiError::Accessibility { message: message.into() }
    }

    /// Create a security error
    pub fn security(message: impl Into<String>) -> Self {
        TuiError::Security { message: message.into() }
    }

    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        TuiError::Network { message: message.into() }
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        TuiError::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a state error
    pub fn state(message: impl Into<String>) -> Self {
        TuiError::State { message: message.into() }
    }

    /// Create an initialization error
    pub fn init(message: impl Into<String>) -> Self {
        TuiError::Init { message: message.into() }
    }

    /// Create a shutdown error
    pub fn shutdown(message: impl Into<String>) -> Self {
        TuiError::Shutdown { message: message.into() }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, duration: u64) -> Self {
        TuiError::Timeout {
            operation: operation.into(),
            duration,
        }
    }

    /// Create a cancellation error
    pub fn cancelled(operation: impl Into<String>) -> Self {
        TuiError::Cancelled { operation: operation.into() }
    }

    /// Create a resource exhausted error
    pub fn resource_exhausted(resource: impl Into<String>) -> Self {
        TuiError::ResourceExhausted { resource: resource.into() }
    }

    /// Create a version error
    pub fn version(message: impl Into<String>) -> Self {
        TuiError::Version { message: message.into() }
    }

    /// Create a plugin error
    pub fn plugin(message: impl Into<String>) -> Self {
        TuiError::Plugin { message: message.into() }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        TuiError::Internal { message: message.into() }
    }
}

