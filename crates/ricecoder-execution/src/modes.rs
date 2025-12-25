//! Execution modes for controlling how plans are executed
//!
//! Supports three execution modes:
//! - Automatic: Execute all steps without user intervention
//! - StepByStep: Require approval for each step
//! - DryRun: Preview changes without applying them

use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::{
    error::{ExecutionError, ExecutionResult},
    models::{ExecutionMode, ExecutionPlan, ExecutionStep, StepAction},
};

/// Configuration for execution modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    /// Default execution mode
    pub default_mode: ExecutionMode,
    /// Whether to skip approval gates for Low/Medium risk
    pub skip_low_medium_approval: bool,
    /// Whether to always require approval for Critical risk
    pub always_approve_critical: bool,
    /// Path to configuration file
    pub config_path: Option<String>,
}

impl Default for ModeConfig {
    fn default() -> Self {
        Self {
            default_mode: ExecutionMode::Automatic,
            skip_low_medium_approval: true,
            always_approve_critical: true,
            config_path: None,
        }
    }
}

/// Automatic mode executor
///
/// Executes all steps without user intervention. Skips approval gates
/// except for Critical risk level.
pub struct AutomaticModeExecutor {
    config: ModeConfig,
}

impl AutomaticModeExecutor {
    /// Create a new automatic mode executor
    pub fn new(config: ModeConfig) -> Self {
        Self { config }
    }

    /// Check if approval is required for a plan
    ///
    /// In automatic mode, approval is only required for Critical risk.
    pub fn requires_approval(&self, plan: &ExecutionPlan) -> bool {
        if !self.config.always_approve_critical {
            return false;
        }

        plan.risk_score.level == crate::models::RiskLevel::Critical
    }

    /// Execute plan in automatic mode
    ///
    /// Executes all steps sequentially without user intervention.
    pub fn execute(&self, plan: &ExecutionPlan) -> ExecutionResult<()> {
        info!(
            plan_id = %plan.id,
            step_count = plan.steps.len(),
            "Executing plan in automatic mode"
        );

        for (index, step) in plan.steps.iter().enumerate() {
            debug!(
                step_index = index,
                step_id = %step.id,
                description = %step.description,
                "Executing step in automatic mode"
            );

            // In automatic mode, we execute all steps
            // Actual execution is handled by StepExecutor
        }

        info!(
            plan_id = %plan.id,
            "Automatic mode execution completed"
        );

        Ok(())
    }
}

/// Step-by-step mode executor
///
/// Requires approval for each step. Allows skipping individual steps
/// and supports pause/resume between steps.
pub struct StepByStepModeExecutor {
    #[allow(dead_code)]
    config: ModeConfig,
    /// Steps that have been approved
    approved_steps: Vec<String>,
    /// Steps that have been skipped
    skipped_steps: Vec<String>,
}

impl StepByStepModeExecutor {
    /// Create a new step-by-step mode executor
    pub fn new(config: ModeConfig) -> Self {
        Self {
            config,
            approved_steps: Vec::new(),
            skipped_steps: Vec::new(),
        }
    }

    /// Request approval for a step
    ///
    /// Returns true if the step is approved, false if rejected.
    pub fn request_approval(&mut self, step: &ExecutionStep) -> ExecutionResult<bool> {
        debug!(
            step_id = %step.id,
            description = %step.description,
            "Requesting approval for step"
        );

        // In a real implementation, this would show a UI prompt
        // For now, we'll return true (approved)
        self.approved_steps.push(step.id.clone());

        info!(
            step_id = %step.id,
            "Step approved"
        );

        Ok(true)
    }

    /// Skip a step
    pub fn skip_step(&mut self, step_id: &str) -> ExecutionResult<()> {
        debug!(step_id = %step_id, "Skipping step");

        self.skipped_steps.push(step_id.to_string());

        info!(
            step_id = %step_id,
            "Step skipped"
        );

        Ok(())
    }

    /// Check if a step has been approved
    pub fn is_approved(&self, step_id: &str) -> bool {
        self.approved_steps.contains(&step_id.to_string())
    }

    /// Check if a step has been skipped
    pub fn is_skipped(&self, step_id: &str) -> bool {
        self.skipped_steps.contains(&step_id.to_string())
    }

    /// Get approved steps
    pub fn approved_steps(&self) -> &[String] {
        &self.approved_steps
    }

    /// Get skipped steps
    pub fn skipped_steps(&self) -> &[String] {
        &self.skipped_steps
    }

    /// Execute plan in step-by-step mode
    ///
    /// Requires approval for each step before execution.
    pub fn execute(&mut self, plan: &ExecutionPlan) -> ExecutionResult<()> {
        info!(
            plan_id = %plan.id,
            step_count = plan.steps.len(),
            "Executing plan in step-by-step mode"
        );

        for (index, step) in plan.steps.iter().enumerate() {
            debug!(
                step_index = index,
                step_id = %step.id,
                description = %step.description,
                "Processing step in step-by-step mode"
            );

            // Request approval for this step
            self.request_approval(step)?;
        }

        info!(
            plan_id = %plan.id,
            approved_count = self.approved_steps.len(),
            skipped_count = self.skipped_steps.len(),
            "Step-by-step mode execution completed"
        );

        Ok(())
    }
}

/// Dry-run mode executor
///
/// Previews all changes without applying them. Shows what would be
/// created, modified, or deleted.
pub struct DryRunModeExecutor {
    #[allow(dead_code)]
    config: ModeConfig,
    /// Changes that would be made
    preview_changes: Vec<PreviewChange>,
}

/// A change that would be made in dry-run mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewChange {
    /// Step ID
    pub step_id: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Path affected
    pub path: String,
    /// Description of the change
    pub description: String,
}

/// Type of change in dry-run mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// File would be created
    Create,
    /// File would be modified
    Modify,
    /// File would be deleted
    Delete,
    /// Command would be executed
    Command,
    /// Tests would be run
    Test,
}

impl DryRunModeExecutor {
    /// Create a new dry-run mode executor
    pub fn new(config: ModeConfig) -> Self {
        Self {
            config,
            preview_changes: Vec::new(),
        }
    }

    /// Preview a step without executing it
    pub fn preview_step(&mut self, step: &ExecutionStep) -> ExecutionResult<()> {
        debug!(
            step_id = %step.id,
            description = %step.description,
            "Previewing step in dry-run mode"
        );

        let change = match &step.action {
            StepAction::CreateFile { path, content } => PreviewChange {
                step_id: step.id.clone(),
                change_type: ChangeType::Create,
                path: path.clone(),
                description: format!("Create file with {} bytes", content.len()),
            },
            StepAction::ModifyFile { path, diff } => PreviewChange {
                step_id: step.id.clone(),
                change_type: ChangeType::Modify,
                path: path.clone(),
                description: format!("Modify file with diff ({} bytes)", diff.len()),
            },
            StepAction::DeleteFile { path } => PreviewChange {
                step_id: step.id.clone(),
                change_type: ChangeType::Delete,
                path: path.clone(),
                description: "Delete file".to_string(),
            },
            StepAction::RunCommand { command, args } => PreviewChange {
                step_id: step.id.clone(),
                change_type: ChangeType::Command,
                path: command.clone(),
                description: format!("Run command with {} args", args.len()),
            },
            StepAction::RunShellCommand { command, .. } => PreviewChange {
                step_id: step.id.clone(),
                change_type: ChangeType::Command,
                path: command.clone(),
                description: "Run shell command".to_string(),
            },
            StepAction::RunTests { pattern } => PreviewChange {
                step_id: step.id.clone(),
                change_type: ChangeType::Test,
                path: pattern.clone().unwrap_or_else(|| "all".to_string()),
                description: "Run tests".to_string(),
            },
        };

        self.preview_changes.push(change);

        info!(
            step_id = %step.id,
            "Step previewed in dry-run mode"
        );

        Ok(())
    }

    /// Get all preview changes
    pub fn preview_changes(&self) -> &[PreviewChange] {
        &self.preview_changes
    }

    /// Get summary of changes
    pub fn get_summary(&self) -> DryRunSummary {
        let mut creates = 0;
        let mut modifies = 0;
        let mut deletes = 0;
        let mut commands = 0;
        let mut tests = 0;

        for change in &self.preview_changes {
            match change.change_type {
                ChangeType::Create => creates += 1,
                ChangeType::Modify => modifies += 1,
                ChangeType::Delete => deletes += 1,
                ChangeType::Command => commands += 1,
                ChangeType::Test => tests += 1,
            }
        }

        DryRunSummary {
            total_changes: self.preview_changes.len(),
            creates,
            modifies,
            deletes,
            commands,
            tests,
        }
    }

    /// Execute plan in dry-run mode
    ///
    /// Previews all changes without applying them.
    pub fn execute(&mut self, plan: &ExecutionPlan) -> ExecutionResult<()> {
        info!(
            plan_id = %plan.id,
            step_count = plan.steps.len(),
            "Executing plan in dry-run mode"
        );

        for step in &plan.steps {
            self.preview_step(step)?;
        }

        let summary = self.get_summary();
        info!(
            plan_id = %plan.id,
            total_changes = summary.total_changes,
            creates = summary.creates,
            modifies = summary.modifies,
            deletes = summary.deletes,
            commands = summary.commands,
            tests = summary.tests,
            "Dry-run mode execution completed"
        );

        Ok(())
    }
}

/// Summary of changes in dry-run mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunSummary {
    /// Total number of changes
    pub total_changes: usize,
    /// Number of files to create
    pub creates: usize,
    /// Number of files to modify
    pub modifies: usize,
    /// Number of files to delete
    pub deletes: usize,
    /// Number of commands to run
    pub commands: usize,
    /// Number of test runs
    pub tests: usize,
}

/// Mode persistence for remembering user preferences
pub struct ModePersistence {
    config_path: String,
}

impl ModePersistence {
    /// Create a new mode persistence handler
    pub fn new(config_path: String) -> Self {
        Self { config_path }
    }

    /// Load mode configuration from file
    pub fn load_mode(&self) -> ExecutionResult<ExecutionMode> {
        debug!(config_path = %self.config_path, "Loading execution mode from config");

        if !Path::new(&self.config_path).exists() {
            debug!(config_path = %self.config_path, "Config file not found, using default");
            return Ok(ExecutionMode::default());
        }

        let content = std::fs::read_to_string(&self.config_path).map_err(|e| {
            ExecutionError::ValidationError(format!("Failed to read mode config: {}", e))
        })?;

        let config: ModeConfig = serde_yaml::from_str(&content).map_err(|e| {
            ExecutionError::ValidationError(format!("Failed to parse mode config: {}", e))
        })?;

        info!(
            config_path = %self.config_path,
            mode = ?config.default_mode,
            "Execution mode loaded from config"
        );

        Ok(config.default_mode)
    }

    /// Save mode configuration to file
    pub fn save_mode(&self, mode: ExecutionMode) -> ExecutionResult<()> {
        debug!(config_path = %self.config_path, mode = ?mode, "Saving execution mode to config");

        let config = ModeConfig {
            default_mode: mode,
            skip_low_medium_approval: true,
            always_approve_critical: true,
            config_path: Some(self.config_path.clone()),
        };

        let yaml = serde_yaml::to_string(&config).map_err(|e| {
            ExecutionError::ValidationError(format!("Failed to serialize mode config: {}", e))
        })?;

        std::fs::write(&self.config_path, yaml).map_err(|e| {
            ExecutionError::ValidationError(format!("Failed to write mode config: {}", e))
        })?;

        info!(
            config_path = %self.config_path,
            mode = ?mode,
            "Execution mode saved to config"
        );

        Ok(())
    }

    /// Load mode with fallback to default
    pub fn load_mode_or_default(&self) -> ExecutionMode {
        match self.load_mode() {
            Ok(mode) => mode,
            Err(e) => {
                warn!(error = %e, "Failed to load mode, using default");
                ExecutionMode::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ComplexityLevel, RiskLevel, RiskScore};

    fn create_test_plan(risk_level: RiskLevel) -> ExecutionPlan {
        ExecutionPlan {
            id: "test-plan".to_string(),
            name: "Test Plan".to_string(),
            steps: vec![],
            risk_score: RiskScore {
                level: risk_level,
                score: 0.5,
                factors: vec![],
            },
            estimated_duration: std::time::Duration::from_secs(10),
            estimated_complexity: ComplexityLevel::Simple,
            requires_approval: false,
            editable: true,
        }
    }

    #[test]
    fn test_automatic_mode_low_risk() {
        let config = ModeConfig::default();
        let executor = AutomaticModeExecutor::new(config);
        let plan = create_test_plan(RiskLevel::Low);

        assert!(!executor.requires_approval(&plan));
    }

    #[test]
    fn test_automatic_mode_critical_risk() {
        let config = ModeConfig::default();
        let executor = AutomaticModeExecutor::new(config);
        let plan = create_test_plan(RiskLevel::Critical);

        assert!(executor.requires_approval(&plan));
    }

    #[test]
    fn test_automatic_mode_execute() {
        let config = ModeConfig::default();
        let executor = AutomaticModeExecutor::new(config);
        let plan = create_test_plan(RiskLevel::Low);

        let result = executor.execute(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_step_by_step_mode_approval() {
        let config = ModeConfig::default();
        let mut executor = StepByStepModeExecutor::new(config);

        let step = ExecutionStep::new(
            "Test step".to_string(),
            StepAction::RunCommand {
                command: "echo".to_string(),
                args: vec!["test".to_string()],
            },
        );

        let result = executor.request_approval(&step);
        assert!(result.is_ok());
        assert!(executor.is_approved(&step.id));
    }

    #[test]
    fn test_step_by_step_mode_skip() {
        let config = ModeConfig::default();
        let mut executor = StepByStepModeExecutor::new(config);

        let step_id = "test-step";
        let result = executor.skip_step(step_id);
        assert!(result.is_ok());
        assert!(executor.is_skipped(step_id));
    }

    #[test]
    fn test_dry_run_mode_preview_create() {
        let config = ModeConfig::default();
        let mut executor = DryRunModeExecutor::new(config);

        let step = ExecutionStep::new(
            "Create file".to_string(),
            StepAction::CreateFile {
                path: "/tmp/test.txt".to_string(),
                content: "test content".to_string(),
            },
        );

        let result = executor.preview_step(&step);
        assert!(result.is_ok());
        assert_eq!(executor.preview_changes().len(), 1);
        assert_eq!(
            executor.preview_changes()[0].change_type,
            ChangeType::Create
        );
    }

    #[test]
    fn test_dry_run_mode_preview_delete() {
        let config = ModeConfig::default();
        let mut executor = DryRunModeExecutor::new(config);

        let step = ExecutionStep::new(
            "Delete file".to_string(),
            StepAction::DeleteFile {
                path: "/tmp/test.txt".to_string(),
            },
        );

        let result = executor.preview_step(&step);
        assert!(result.is_ok());
        assert_eq!(executor.preview_changes().len(), 1);
        assert_eq!(
            executor.preview_changes()[0].change_type,
            ChangeType::Delete
        );
    }

    #[test]
    fn test_dry_run_mode_summary() {
        let config = ModeConfig::default();
        let mut executor = DryRunModeExecutor::new(config);

        let step1 = ExecutionStep::new(
            "Create file".to_string(),
            StepAction::CreateFile {
                path: "/tmp/test1.txt".to_string(),
                content: "content".to_string(),
            },
        );

        let step2 = ExecutionStep::new(
            "Delete file".to_string(),
            StepAction::DeleteFile {
                path: "/tmp/test2.txt".to_string(),
            },
        );

        executor.preview_step(&step1).unwrap();
        executor.preview_step(&step2).unwrap();

        let summary = executor.get_summary();
        assert_eq!(summary.total_changes, 2);
        assert_eq!(summary.creates, 1);
        assert_eq!(summary.deletes, 1);
    }

    #[test]
    fn test_mode_config_default() {
        let config = ModeConfig::default();
        assert_eq!(config.default_mode, ExecutionMode::Automatic);
        assert!(config.skip_low_medium_approval);
        assert!(config.always_approve_critical);
    }

    #[test]
    fn test_mode_persistence_default() {
        let persistence = ModePersistence::new("/tmp/nonexistent_config.yaml".to_string());
        let mode = persistence.load_mode_or_default();
        assert_eq!(mode, ExecutionMode::Automatic);
    }
}
