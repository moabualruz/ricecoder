#![warn(missing_docs)]

//! Ricecoder Agentic Workflows
//!
//! Provides declarative workflow definitions with state management, error handling,
//! and human approval gates for multi-step agentic operations.

pub mod models;
pub mod error;
pub mod parser;
pub mod engine;
pub mod state;
pub mod executor;
pub mod resolver;
pub mod condition;
pub mod error_handler;
pub mod rollback;
pub mod approval;
pub mod parameters;
pub mod parameter_substitution;
pub mod progress;
pub mod activity_log;
pub mod status_reporter;
pub mod risk_scoring;
pub mod safety_constraints;
pub mod storage_integration;
pub mod agent_executor;
pub mod command_executor;
pub mod parallel_executor;

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

pub use models::*;
pub use error::*;
pub use parser::WorkflowParser;
pub use engine::WorkflowEngine;
pub use state::StateManager;
pub use executor::StepExecutor;
pub use resolver::DependencyResolver;
pub use condition::ConditionEvaluator;
pub use error_handler::{ErrorHandler, ErrorHistoryEntry, RetryState};
pub use rollback::{RollbackManager, RollbackPlan};
pub use approval::{ApprovalGate, ApprovalRequest, ApprovalDecision};
pub use parameters::{ParameterValidator, ParameterSubstitutor, ParameterDef, ParameterType};
pub use parameter_substitution::StepConfigSubstitutor;
pub use progress::{ProgressTracker, StatusReport};
pub use activity_log::{ActivityLogger, ActivityLogEntry, ActivityType};
pub use status_reporter::{StatusReporter, StatusUpdateListener};
pub use risk_scoring::RiskScorer;
pub use safety_constraints::SafetyConstraints;
pub use storage_integration::StorageIntegration;
pub use agent_executor::AgentExecutor;
pub use command_executor::CommandExecutor;
pub use parallel_executor::ParallelExecutor;
