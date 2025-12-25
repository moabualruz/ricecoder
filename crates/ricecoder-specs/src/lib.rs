#![warn(missing_docs)]

//! Ricecoder Specification System
//!
//! Provides spec-driven development capabilities including spec discovery, loading,
//! validation, querying, and AI-assisted spec writing.

pub mod ai_writer;
pub mod approval;
pub mod cache;
pub mod change_tracking;
pub mod conversation;
pub mod di;
pub mod error;
pub mod format_conversion;
pub mod inheritance;
pub mod manager;
pub mod models;
pub mod parsers;
pub mod query;
pub mod governance;
pub mod validation;
pub mod workflow;

pub use ai_writer::{AISpecWriter, GapAnalysis};
pub use approval::ApprovalManager;
pub use cache::SpecCache;
pub use change_tracking::ChangeTracker;
pub use conversation::ConversationManager;
pub use error::*;
pub use format_conversion::FormatConverter;
pub use inheritance::SpecInheritanceResolver;
pub use manager::SpecManager;
pub use models::*;
pub use query::SpecQueryEngine;
pub use governance::GovernanceLoader;
pub use validation::ValidationEngine;
pub use workflow::WorkflowOrchestrator;
