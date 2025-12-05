//! Error types for keybind operations

use thiserror::Error;

/// Errors that can occur during keybind parsing
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid JSON syntax: {0}")]
    InvalidJson(String),

    #[error("Invalid Markdown syntax: {0}")]
    InvalidMarkdown(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid key syntax: {0}")]
    InvalidKeySyntax(String),

    #[error("Invalid modifier: {0}")]
    InvalidModifier(String),

    #[error("Duplicate keybind definition: {0}")]
    DuplicateDefinition(String),

    #[error("Parse error at line {line}: {message}")]
    LineError { line: usize, message: String },
}

/// Errors that can occur in the keybind registry
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Duplicate action ID: {0}")]
    DuplicateActionId(String),

    #[error("Invalid action ID format: {0}")]
    InvalidActionIdFormat(String),

    #[error("Action not found: {0}")]
    ActionNotFound(String),

    #[error("Key combination not found")]
    KeyNotFound,
}

/// Errors that can occur in profile management
#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Profile already exists: {0}")]
    ProfileAlreadyExists(String),

    #[error("Cannot delete active profile: {0}")]
    CannotDeleteActiveProfile(String),

    #[error("Invalid profile name: {0}")]
    InvalidProfileName(String),

    #[error("No active profile set")]
    NoActiveProfile,
}

/// Errors that can occur in the keybind engine
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Registry error: {0}")]
    RegistryError(#[from] RegistryError),

    #[error("Profile error: {0}")]
    ProfileError(#[from] ProfileError),

    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("Persistence error: {0}")]
    PersistenceError(#[from] PersistenceError),

    #[error("No keybind for action: {0}")]
    NoKeybindForAction(String),

    #[error("Invalid key combination")]
    InvalidKeyCombo,

    #[error("Engine not initialized")]
    NotInitialized,

    #[error("Defaults load error: {0}")]
    DefaultsLoadError(String),
}

/// Errors that can occur in the persistence layer
#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Corrupted JSON: {0}")]
    CorruptedJson(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
}
