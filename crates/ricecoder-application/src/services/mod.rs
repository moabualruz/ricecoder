//! Application layer services
//!
//! These services orchestrate domain aggregates and implement use cases.
//! All services are stateless and use constructor injection for dependencies.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Application Services                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ProjectService       - Project lifecycle management         │
//! │  SessionService       - Session lifecycle and messages       │
//! │  SpecificationService - Spec validation and task tracking    │
//! │  CodeService          - Code generation orchestration        │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Responsibilities
//!
//! - Orchestrate multi-step use cases
//! - Manage transaction boundaries via UnitOfWork
//! - Emit application events for external consumers
//! - Map domain entities to DTOs
//! - Coordinate domain services
//!
//! # Non-Goals
//!
//! - Domain logic (belongs in Domain Layer)
//! - Direct I/O (belongs in Infrastructure Layer)

mod project_service;
mod session_service;
mod specification_service;
mod code_service;

pub use project_service::ProjectService;
pub use session_service::SessionService;
pub use specification_service::SpecificationService;
pub use code_service::CodeService;
