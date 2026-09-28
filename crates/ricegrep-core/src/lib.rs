//! RiceGrep Core Library
//!
//! This crate provides the core domain and application layer for RiceGrep,
//! following Domain-Driven Design (DDD) and Clean Architecture principles.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │  Consumers (MCP, REST, CLI, etc.)   │
//! ├─────────────────────────────────────┤
//! │  Application Layer (this crate)     │  ← Use cases, repository traits
//! ├─────────────────────────────────────┤
//! │  Domain Layer (this crate)          │  ← Pure business logic
//! ├─────────────────────────────────────┤
//! │  Infrastructure (consumer provides) │  ← File I/O, indexing, etc.
//! └─────────────────────────────────────┘
//! ```
//!
//! # Design Principles
//!
//! - **Minimal Dependencies**: Only `regex` for pattern matching
//! - **Pure Domain**: Domain layer has zero external dependencies
//! - **Ports & Adapters**: Repository traits allow custom implementations
//! - **Testable**: Mock implementations provided for all traits
//!
//! # Modules
//!
//! - [`domain`] - Value objects, aggregates, events, and errors
//! - [`application`] - Use cases, repository traits (ports), and services
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use ricegrep_core::domain::{FilePath, EditPattern};
//! use ricegrep_core::application::{
//!     FileRepository, IndexRepository, EventPublisher,
//!     EditFileUseCase, EditFileRequest,
//! };
//!
//! // Implement repository traits for your infrastructure
//! struct MyFileRepo;
//! impl FileRepository for MyFileRepo { /* ... */ }
//!
//! // Use the application layer
//! let use_case = EditFileUseCase::new(my_file_repo, my_event_publisher);
//! let request = EditFileRequest { /* ... */ };
//! let response = use_case.execute(request)?;
//! ```

pub mod application;
pub mod domain;

// Re-export commonly used types at crate root for convenience
pub use domain::{
    // Errors
    DomainError,
    // Events
    DomainEvent,
    DomainResult,
    EditPattern,
    // Aggregates
    FileEdit,
    // Value Objects
    FilePath,
    SearchMatch,
    SearchQuery,
    SearchResult,
};

pub use application::{
    // Errors
    AppError,
    AppResult,
    // Services
    AppServices,
    AppServicesBuilder,
    EditFileRequest,
    EditFileResponse,
    // Use Cases
    EditFileUseCase,
    EventPublisher,
    FileIndexEntry,
    // Repository Traits (Ports)
    FileRepository,
    IndexRepository,
    IoOperation,
    SearchFilesRequest,
    SearchFilesResponse,
    SearchFilesUseCase,
    WriteFileRequest,
    WriteFileResponse,
    WriteFileUseCase,
};
