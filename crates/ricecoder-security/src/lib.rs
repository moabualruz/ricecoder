//! # RiceCoder Security
//!
//! Security utilities and cryptographic operations for RiceCoder.
//!
//! This crate provides:
//! - API key encryption and secure storage
//! - Input validation and sanitization
//! - Authentication helpers
//! - Audit logging system
//! - Access control and permission management

pub mod encryption;
pub mod validation;
pub mod audit;
pub mod access_control;
pub mod error;

pub use encryption::{KeyManager, EncryptedData};
pub use validation::{validate_input, ValidatedInput, ValidationError};
pub use audit::{AuditLogger, AuditEvent, AuditRecord};
pub use access_control::{Permission, AccessControl, PermissionCheck};
pub use error::SecurityError;

/// Re-export commonly used types
pub type Result<T> = std::result::Result<T, SecurityError>;

// Property-based tests can be added here in the future
// For now, unit tests provide comprehensive coverage