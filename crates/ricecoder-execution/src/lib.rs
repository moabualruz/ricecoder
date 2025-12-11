#![warn(missing_docs)]

//! Ricecoder Execution Plans
//!
//! Provides execution plans, test running integration, and step-by-step task execution
//! with user approval gates and rollback support.

pub mod approval;
pub mod approval_ui;
pub mod error;
pub mod file_operations;
pub mod manager;
pub mod models;
pub mod modes;
pub mod plan_builder;
pub mod progress_tracker;
pub mod risk_scorer;
pub mod rollback_actions;
pub mod rollback_handler;
pub mod step_action_handler;
pub mod step_creator;
pub mod step_executor;
pub mod test_runner;
pub mod validation;

pub use approval::{ApprovalManager, ApprovalSummary};
pub use approval_ui::{ApprovalUI, ApprovalUIBuilder, ApprovalUIState};
pub use error::{ExecutionError, ExecutionResult};
pub use file_operations::FileOperations;
pub use manager::ExecutionManager;
pub use models::{
    BatchExecutionConfig, BatchExecutionOutput, BatchExecutionResult, BatchExecutionSummary,
    CommandOutput, ComplexityLevel, ExecutionMode, ExecutionPlan, ExecutionResult as ExecutionResultData,
    ExecutionState, ExecutionStatus, ExecutionStep, RiskFactor, RiskLevel, RiskScore,
    RollbackAction, RollbackType, StepAction, StepResult, StepStatus, TestFailure, TestFramework,
    TestResults,
};
pub use modes::{
    AutomaticModeExecutor, ChangeType, DryRunModeExecutor, DryRunSummary, ModeConfig,
    ModePersistence, PreviewChange, StepByStepModeExecutor,
};
pub use plan_builder::PlanBuilder;
pub use progress_tracker::{ProgressCallback, ProgressTracker, ProgressUpdate};
pub use risk_scorer::ExecutionRiskScorer;
pub use rollback_actions::{RestoreFileHandler, UndoCommandHandler};
pub use rollback_handler::{RollbackHandler, RollbackResult};
pub use step_action_handler::{
    CommandHandler, CreateFileHandler, DeleteFileHandler, ModifyFileHandler, TestHandler,
};
pub use step_creator::StepCreator;
pub use step_executor::StepExecutor;
pub use test_runner::TestRunner;
pub use validation::ExecutionValidator;
