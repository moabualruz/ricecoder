#![warn(missing_docs)]

//! Ricecoder Agentic Workflows
//!
//! Provides declarative workflow definitions with state management, error handling,
//! and human approval gates for multi-step agentic operations.

pub mod activity_log;
pub mod agent_executor;
pub mod approval;
pub mod command_executor;
pub mod condition;
pub mod engine;
pub mod error;
pub mod error_handler;
pub mod executor;
pub mod models;
pub mod parallel_executor;
pub mod parameter_substitution;
pub mod parameters;
pub mod parser;
pub mod progress;
pub mod resolver;
pub mod risk_scoring;
pub mod rollback;
pub mod safety_constraints;
pub mod state;
pub mod status_reporter;
pub mod storage_integration;

#[cfg(test)]
mod condition_properties;

#[cfg(test)]
mod error_handling_properties;

#[cfg(test)]
mod approval_properties;

#[cfg(test)]
mod parameter_substitution_properties;

#[cfg(test)]
mod state_persistence_properties;

#[cfg(test)]
mod pause_resume_properties;

#[cfg(test)]
mod status_reporting_properties;

#[cfg(test)]
mod activity_logging_properties;

#[cfg(test)]
mod risk_scoring_properties;

#[cfg(test)]
mod safety_constraints_properties;

pub use activity_log::{ActivityLogEntry, ActivityLogger, ActivityType};
pub use agent_executor::AgentExecutor;
pub use approval::{ApprovalDecision, ApprovalGate, ApprovalRequest};
pub use command_executor::CommandExecutor;
pub use condition::ConditionEvaluator;
pub use engine::WorkflowEngine;
pub use error::*;
pub use error_handler::{ErrorHandler, ErrorHistoryEntry, RetryState};
pub use executor::StepExecutor;
pub use models::*;
pub use parallel_executor::ParallelExecutor;
pub use parameter_substitution::StepConfigSubstitutor;
pub use parameters::{ParameterDef, ParameterSubstitutor, ParameterType, ParameterValidator};
pub use parser::WorkflowParser;
pub use progress::{ProgressTracker, StatusReport};
pub use resolver::DependencyResolver;
pub use risk_scoring::RiskScorer;
pub use rollback::{RollbackManager, RollbackPlan};
pub use safety_constraints::SafetyConstraints;
pub use state::StateManager;
pub use status_reporter::{StatusReporter, StatusUpdateListener};
pub use storage_integration::StorageIntegration;
