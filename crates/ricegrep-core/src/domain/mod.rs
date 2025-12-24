//! Domain Layer - Pure Business Logic
//!
//! This module contains the core domain types and business rules for RiceGrep.
//! It follows Domain-Driven Design patterns with:
//! - Value Objects: Immutable, validated types
//! - Aggregate Roots: Entities with business invariants
//! - Domain Events: Published when aggregates change
//!
//! # Design Principles
//! - No external dependencies except std lib
//! - Pure functions, no side effects
//! - Testable without mocks
//! - Immutable types (mutations return new instances)

pub mod value_objects;
pub mod aggregates;
pub mod events;
pub mod errors;

pub use value_objects::*;
pub use aggregates::*;
pub use events::*;
pub use errors::*;
