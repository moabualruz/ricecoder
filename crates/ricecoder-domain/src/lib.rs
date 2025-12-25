//! # RiceCoder Domain
#![forbid(unsafe_code)]

//!
//! Core domain entities and business logic for RiceCoder.
//! This crate contains the central business rules, entities, and domain services
//! that form the foundation of the RiceCoder system.
//!
//! ## Architecture
//!
//! This crate follows Domain-Driven Design principles:
//! - **Entities**: Core business objects with identity
//! - **Value Objects**: Immutable objects representing concepts
//! - **Domain Services**: Business logic that doesn't belong to entities
//! - **Repositories**: Interfaces for data persistence (defined here, implemented in infrastructure)

pub mod di;
pub mod entities;
pub mod errors;
pub mod events;
pub mod ports;
pub mod project;
pub mod repositories;
pub mod services;
pub mod session;
pub mod specification;
pub mod value_objects;

pub use entities::*;
pub use errors::*;
pub use events::*;
pub use ports::*;
pub use project::*;
pub use repositories::*;
pub use services::*;
pub use session::*;
pub use specification::*;
pub use value_objects::*;
