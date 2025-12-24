//! RiceCoder Application Layer
//!
//! The Application Layer implements use cases by orchestrating domain aggregates
//! and domain services. It provides stateless services that coordinate multi-step
//! workflows, manage transaction boundaries, and emit application events.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        Application Layer                                 │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │  Services          │ DTOs              │ Ports              │ Events    │
//! │  ─────────         │ ────              │ ─────              │ ──────    │
//! │  ProjectService    │ CreateProjectCmd  │ UnitOfWork         │ AppEvent  │
//! │  SessionService    │ ProjectDto        │ EventPublisher     │           │
//! │  SpecService       │ SessionDto        │                    │           │
//! │  CodeService       │ SpecDto           │                    │           │
//! └─────────────────────────────────────────────────────────────────────────┘
//!                              ▲
//!                              │ depends on
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                         Domain Layer                                     │
//! │  Aggregates, Entities, Value Objects, Domain Events, Repository Traits  │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Responsibilities
//!
//! - **Use Case Orchestration**: Coordinate multi-step workflows
//! - **Transaction Boundaries**: Wrap atomic operations in UnitOfWork
//! - **DTO Mapping**: Convert domain objects to/from presentation-safe DTOs
//! - **Application Events**: Emit use-case-level events for external consumers
//! - **Error Mapping**: Translate domain errors to application-level errors
//!
//! # Non-Goals
//!
//! - Domain logic (belongs in Domain Layer)
//! - Direct I/O operations (belongs in Infrastructure Layer)
//! - HTTP/CLI handling (belongs in Presentation Layer)

pub mod di;
pub mod dto;
pub mod errors;
pub mod events;
pub mod ports;
pub mod services;

// Re-export commonly used types
pub use dto::*;
pub use errors::ApplicationError;
pub use events::{ApplicationEvent, EventPublisher};
pub use ports::UnitOfWork;
pub use services::*;
