//! RiceCoder Multi-Project Orchestration Module
//!
//! This module provides workspace-level operations across multiple projects,
//! enabling coordinated changes, dependency management, and cross-project analysis.
//!
//! # Overview
//!
//! The orchestration module coordinates workspace-level operations including:
//! - Workspace scanning and project discovery
//! - Cross-project dependency analysis
//! - Batch operations across multiple projects
//! - Impact analysis for changes
//! - Configuration and rules management
//! - Version coordination
//! - Status reporting
//!
//! # Core Components
//!
//! - [`OrchestrationManager`]: Central coordinator for all workspace operations
//! - [`WorkspaceScanner`]: Discovers projects and their metadata
//! - [`models`]: Core data structures for workspace orchestration
//! - [`error`]: Error types for orchestration operations
//!
//! # Path Resolution
//!
//! All path operations use `ricecoder_storage::PathResolver` for consistent
//! workspace navigation and portability across different environments.
//!
//! # Example Usage
//!
//! ```ignore
//! use ricecoder_orchestration::{OrchestrationManager, WorkspaceScanner};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let workspace_root = PathBuf::from("/path/to/workspace");
//!
//!     // Create and initialize the orchestration manager
//!     let manager = OrchestrationManager::new(workspace_root.clone());
//!     manager.initialize().await?;
//!
//!     // Scan the workspace for projects
//!     let scanner = WorkspaceScanner::new(workspace_root);
//!     let projects = scanner.scan_workspace().await?;
//!
//!     println!("Found {} projects", projects.len());
//!
//!     // Shutdown the manager
//!     manager.shutdown().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod analyzers;
pub mod error;
pub mod managers;
pub mod models;

// Re-export commonly used types
pub use analyzers::{
    Change, ChangeDetails, ChangePropagationTracker, ChangeType, DependencyAnalyzer,
    DependencyGraph, DependencyInfo, DependencyValidator, ImpactAnalyzer, ProjectChange,
    ProjectDetector, ValidationReport, Version, VersionConstraint, VersionValidator,
    WorkspaceScanner,
};
pub use error::{OrchestrationError, Result};
pub use managers::{
    AggregatedMetrics, BatchExecutionConfig, BatchExecutionResult, BatchExecutor,
    ComplianceSummary, ConfigLoadResult, ConfigManager, ConfigSchema, ConfigSource,
    ConflictResolution, ExecutionLevel, ExecutionOrderer, ExecutionPlan, OrchestrationManager,
    ParallelizationStrategy, ProjectHealthIndicator, ProjectOperation, RuleViolation,
    RulesValidator, StatusReport, StatusReporter, SyncConflict, SyncLogEntry, SyncManager,
    ValidationResult, ValidationRule, VersionCoordinator, VersionUpdatePlan, VersionUpdateResult,
    VersionUpdateStep, ViolationSeverity,
};
pub use models::{
    DependencyType, HealthStatus, ImpactDetail, ImpactLevel, ImpactReport, Operation, Project,
    ProjectDependency, ProjectStatus, RuleType, Transaction, TransactionState, Workspace,
    WorkspaceConfig, WorkspaceMetrics, WorkspaceRule,
};
