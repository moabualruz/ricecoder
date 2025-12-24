//! Application Use Cases
//!
//! Use cases orchestrate domain logic and infrastructure through repository traits.
//! Each use case represents a single business operation.
//!
//! # Architecture Pattern
//! - Use cases depend on repository traits (ports), not concrete implementations
//! - Use cases return domain types, never infrastructure types
//! - Use cases publish domain events for observability
//! - Use cases handle validation and error mapping
//!
//! # Available Use Cases
//! - `EditFileUseCase` - Find and replace patterns in files
//! - `SearchFilesUseCase` - Search for patterns across files
//! - `WriteFileUseCase` - Write content to files with validation

pub mod edit_file;
pub mod search_files;
pub mod write_file;

// Re-export use cases for ergonomic imports
pub use edit_file::{EditFileUseCase, EditFileRequest, EditFileResponse};
pub use search_files::{SearchFilesUseCase, SearchFilesRequest, SearchFilesResponse};
pub use write_file::{WriteFileUseCase, WriteFileRequest, WriteFileResponse};
