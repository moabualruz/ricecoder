//! Manager components for orchestration

pub mod batch_executor;
pub mod config_manager;
pub mod execution_ordering;
pub mod orchestration_manager;
pub mod rules_validator;
pub mod status_reporter;
pub mod sync_manager;
pub mod version_coordinator;

pub use batch_executor::{
    BatchExecutionConfig, BatchExecutionResult, BatchExecutor, ProjectOperation,
};
pub use config_manager::{
    ConfigLoadResult, ConfigManager, ConfigSchema, ConfigSource, ValidationRule,
};
pub use execution_ordering::{
    ExecutionLevel, ExecutionOrderer, ExecutionPlan, ParallelizationStrategy,
};
pub use orchestration_manager::OrchestrationManager;
pub use rules_validator::{RuleViolation, RulesValidator, ValidationResult, ViolationSeverity};
pub use status_reporter::{
    AggregatedMetrics, ComplianceSummary, ProjectHealthIndicator, StatusReport, StatusReporter,
};
pub use sync_manager::{ConflictResolution, SyncConflict, SyncLogEntry, SyncManager};
pub use version_coordinator::{
    VersionCoordinator, VersionUpdatePlan, VersionUpdateResult, VersionUpdateStep,
};
