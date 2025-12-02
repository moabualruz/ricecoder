#![warn(missing_docs)]

//! Ricecoder Specification System
//!
//! Provides spec-driven development capabilities including spec discovery, loading,
//! validation, querying, and AI-assisted spec writing.

pub mod models;
pub mod error;
pub mod parsers;
pub mod manager;
pub mod validation;
pub mod query;
pub mod inheritance;
pub mod change_tracking;
pub mod steering;
pub mod format_conversion;
pub mod conversation;
pub mod approval;
pub mod ai_writer;
pub mod workflow;

pub use models::*;
pub use error::*;
pub use manager::SpecManager;
pub use validation::ValidationEngine;
pub use query::SpecQueryEngine;
pub use inheritance::SpecInheritanceResolver;
pub use change_tracking::ChangeTracker;
pub use steering::SteeringLoader;
pub use format_conversion::FormatConverter;
pub use conversation::ConversationManager;
pub use approval::ApprovalManager;
pub use ai_writer::{AISpecWriter, GapAnalysis};
pub use workflow::WorkflowOrchestrator;
