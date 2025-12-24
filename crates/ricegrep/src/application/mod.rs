//! Application Layer - Orchestration and Ports
//!
//! This module re-exports the application layer from `ricegrep-core`.
//! RiceGrep uses `ricegrep-core` as the single source of truth for
//! application logic, following hexagonal architecture (ports and adapters).
//!
//! # Architecture
//! ```text
//! ricegrep (MCP, CLI, Infrastructure)
//!     └── depends on ──> ricegrep-core (Domain, Application)
//! ```
//!
//! # Re-exported Types
//! - Errors: `AppError`, `AppResult`, `IoOperation`
//! - Repository Traits: `FileRepository`, `IndexRepository`, `EventPublisher`, `FileIndexEntry`
//! - Use Cases: `EditFileUseCase`, `SearchFilesUseCase`, `WriteFileUseCase` + Request/Response types
//! - Services: `AppServices`, `AppServicesBuilder`

// Re-export everything from ricegrep-core's application module
pub use ricegrep_core::application::*;

// Also re-export at module level for backwards compatibility
pub use ricegrep_core::{
    // Errors
    AppError, AppResult, IoOperation,
    // Repository Traits (Ports)
    FileRepository, IndexRepository, EventPublisher, FileIndexEntry,
    // Use Cases
    EditFileUseCase, EditFileRequest, EditFileResponse,
    SearchFilesUseCase, SearchFilesRequest, SearchFilesResponse,
    WriteFileUseCase, WriteFileRequest, WriteFileResponse,
    // Services
    AppServices, AppServicesBuilder,
};

// Re-export use_cases module for explicit imports like `use crate::application::use_cases::*`
pub mod use_cases {
    pub use ricegrep_core::application::use_cases::*;
}
