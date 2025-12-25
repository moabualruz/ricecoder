//! Core domain entities with business logic and validation
//!
//! This module contains the domain entities organized by responsibility:
//! - `project`: Project entity (legacy representation)
//! - `code_file`: Source code file entity
//! - `analysis`: Code analysis results and metrics
//! - `session`: Session entity (legacy representation)
//! - `security`: Security context, events, and alerts
//! - `compliance`: SOC 2 compliance and audit
//! - `gdpr`: GDPR data protection entities
//! - `performance`: Performance metrics and benchmarks
//! - `user`: User entity
//! - `provider`: AI provider configuration

mod analysis;
mod code_file;
mod compliance;
mod gdpr;
mod performance;
mod project;
mod provider;
mod security;
mod session;
mod user;

// Re-export all entities for backward compatibility
pub use analysis::*;
pub use code_file::*;
pub use compliance::*;
pub use gdpr::*;
pub use performance::*;
pub use project::*;
pub use provider::*;
pub use security::*;
pub use session::*;
pub use user::*;
