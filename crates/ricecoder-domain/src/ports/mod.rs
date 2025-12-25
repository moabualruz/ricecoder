//! Port interfaces for external services
//!
//! Infrastructure implements domain interfaces
//! AI Provider Implementations
//!
//! Ports define the contracts for external system integrations.
//! These are implemented by infrastructure crates.
//!
//! ## Modules
//!
//! - `ai`: AI provider types and traits (chat, completion, streaming)
//! - `file`: File system operation types and traits
//! - `cache`: Cache operation types and traits (includes file watching)

pub mod ai;
pub mod cache;
pub mod file;

#[cfg(test)]
mod tests;

// Re-export all types for backward compatibility
pub use ai::*;
pub use cache::*;
pub use file::*;
