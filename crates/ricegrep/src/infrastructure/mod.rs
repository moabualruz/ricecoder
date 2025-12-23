//! Infrastructure Layer - Concrete Implementations
//!
//! This module contains infrastructure adapters that implement the application
//! layer's repository traits using real I/O operations.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────┐
//! │   MCP Protocol Layer        │
//! ├─────────────────────────────┤
//! │   Application Layer         │ ← Defines traits (FileRepository, IndexRepository)
//! ├─────────────────────────────┤
//! │   Domain Layer              │
//! ├─────────────────────────────┤
//! │   Infrastructure Layer      │ ← THIS LAYER: Implements traits with real I/O
//! └─────────────────────────────┘
//! ```
//!
//! # Adapters
//! - `FsFileRepository` - File operations using `std::fs`
//! - `MetadataIndexRepository` - Index operations using existing `metadata_gating` module
//! - `TracingEventPublisher` - Event publishing using `tracing` crate

pub mod file_repository;
pub mod index_repository;
pub mod event_publisher;

// Re-export for ergonomic imports
pub use file_repository::FsFileRepository;
pub use index_repository::MetadataIndexRepository;
pub use event_publisher::TracingEventPublisher;
