//! Security-related error types

use thiserror::Error;

/// Security operation errors
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Encryption error: {message}")]
    Encryption { message: String },

    #[error("Decryption error: {message}")]
    Decryption { message: String },

    #[error("Key derivation error: {message}")]
    KeyDerivation { message: String },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Access denied: {message}")]
    AccessDenied { message: String },

    #[error("Audit logging error: {message}")]
    Audit { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}