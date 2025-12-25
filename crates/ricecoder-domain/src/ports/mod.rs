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
//! - `config`: Configuration source types and traits
//! - `http`: HTTP client types and traits
//! - `scheduler`: Task scheduler types and traits

pub mod ai;
pub mod cache;
pub mod config;
pub mod file;
pub mod http;
pub mod scheduler;

#[cfg(test)]
mod tests;

// Re-export all types for backward compatibility
pub use ai::*;
pub use cache::*;
pub use config::*;
pub use file::*;
pub use http::*;
pub use scheduler::*;
