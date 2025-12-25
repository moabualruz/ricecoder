//! Specification Aggregate Root
//!
//! Specification aggregate with full DDD compliance
//! - Aggregate root with encapsulated Requirement and Task entities
//! - Invariant enforcement (requirements have AC, tasks trace to requirements)
//! - Domain event emission for all state changes
//! - Immutable identity
//! - Completion percentage calculation

mod requirement;
mod specification;
mod task;
#[cfg(test)]
mod tests;

pub use requirement::Requirement;
pub use specification::{SpecStatus, Specification};
pub use task::{Task, TaskStatus};
