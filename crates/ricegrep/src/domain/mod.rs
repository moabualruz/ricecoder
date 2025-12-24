//! Domain Layer - Pure Business Logic
//!
//! This module re-exports the core domain types from `ricegrep-core`.
//! RiceGrep uses `ricegrep-core` as the single source of truth for
//! domain logic, following DDD and Clean Architecture principles.
//!
//! # Architecture
//! ```text
//! ricegrep (MCP, CLI, Infrastructure)
//!     └── depends on ──> ricegrep-core (Domain, Application)
//! ```
//!
//! # Re-exported Types
//! - Value Objects: `FilePath`, `EditPattern`, `SearchQuery`
//! - Aggregates: `FileEdit`, `SearchResult`, `SearchMatch`
//! - Events: `DomainEvent`
//! - Errors: `DomainError`, `DomainResult`

// Re-export everything from ricegrep-core's domain module
pub use ricegrep_core::domain::*;

// Also re-export at module level for backwards compatibility
pub use ricegrep_core::{
    // Value Objects
    FilePath, EditPattern, SearchQuery,
    // Aggregates  
    FileEdit, SearchResult, SearchMatch,
    // Events
    DomainEvent,
    // Errors
    DomainError, DomainResult,
};
