//! Data Transfer Objects (DTOs) for layer boundary crossing
//!
//! DTOs prevent domain model leakage to the presentation layer.
//! They provide a stable API contract while allowing domain internals to evolve.

pub mod project;
pub mod session;
pub mod specification;

// Re-export commonly used DTOs
pub use project::*;
pub use session::*;
pub use specification::*;
