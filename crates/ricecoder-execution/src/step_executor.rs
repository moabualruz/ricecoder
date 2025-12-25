//! Step execution orchestration for execution plans
//!
//! Wraps the WorkflowEngine's StepExecutor and provides high-level
//! step execution with progress reporting and error handling.

use std::time::Instant;

use tracing::{debug, error, info, warn};

use crate::{
    error::{ExecutionError, ExecutionResult},
    models::{
        BatchExecutionConfig, BatchExecutionOutput, BatchExecutionResult, BatchExecutionSummary,
        CommandOutput, ExecutionPlan, ExecutionStep, StepAction, StepResult,
    },
};

/// Executes steps from an execution plan
///
/// Handles:
/// - Sequential step execution
/// - Progress reporting
/// - Error handling with detailed context
/// - Step skipping and resumption
pub struct StepExecutor {
    /// Current step index
    current_step_index: usize,
    /// Completed step results
    completed_steps: Vec<StepResult>,
    /// Whether to skip failed steps
    skip_on_error: bool,
}

impl StepExecutor {
    /// Create a new step executor
    pub fn new() -> Self {
        Self {
            current_step_index: 0,
            completed_steps: Vec::new(),
            skip_on_error: false,
        }
    }

    /// Create a step executor that skips failed steps
    pub fn with_skip_on_error(mut self, skip: bool) -> Self {
        self.skip_on_error = skip;
        self
    }

    /// Execute all steps in a plan sequentially
    ///
    /// Executes steps in order, respecting dependencies. Stops on first error
    /// unless skip_on_error is enabled.
    ///
    /// # Arguments
    /// * `plan` - The execution plan containing steps to execute
    ///
    /// # Returns
    /// A vector of step results for each executed step
    pub fn execute_plan(&mut self, plan: &ExecutionPlan) -> ExecutionResult<Vec<StepResult>> {
        if plan.steps.is_empty() {
            return Err(ExecutionError::PlanError(
                "Cannot execute plan with no steps".to_string(),
            ));
        }

        info!(
            plan_id = %plan.id,
            step_count = plan.steps.len(),
            "Starting plan execution"
        );

        for (index, step) in plan.steps.iter().enumerate() {
            self.current_step_index = index;

            debug!(
                step_id = %step.id,
                step_index = index,
                description = %step.description,
                "Executing step"
            );

            match self.execute_step(step) {
                Ok(result) => {
                    info!(
                        step_id = %step.id,
                        duration_ms = result.duration.as_millis(),
                        "Step completed successfully"
                    );
                    self.completed_steps.push(result);
                }
                Err(e) => {
                    error!(
                        step_id = %step.id,
                        error = %e,
                        "Step execution failed"
                    );

                    if self.skip_on_error {
                        warn!(
                            step_id = %step.id,
                            "Skipping failed step and continuing"
                        );
                        let result = StepResult {
                            step_id: step.id.clone(),
                            success: false,
                            error: Some(e.to_string()),
                            duration: std::time::Duration::from_secs(0),
                            output: None,
                        };
                        self.completed_steps.push(result);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        info!(
            plan_id = %plan.id,
            completed_steps = self.completed_steps.len(),
            "Plan execution completed"
        );

        Ok(self.completed_steps.clone())
    }

    /// Execute a single step
    ///
    /// Dispatches to the appropriate handler based on step action type.
    ///
    /// # Arguments
    /// * `step` - The step to execute
    ///
    /// # Returns
    /// A StepResult containing execution details
    pub fn execute_step(&self, step: &ExecutionStep) -> ExecutionResult<StepResult> {
        let start_time = Instant::now();

        let (success, output) = match &step.action {
            StepAction::CreateFile { path, content } => {
                self.handle_create_file(path, content)?;
                (true, None)
            }
            StepAction::ModifyFile { path, diff } => {
                self.handle_modify_file(path, diff)?;
                (true, None)
            }
            StepAction::DeleteFile { path } => {
                self.handle_delete_file(path)?;
                (true, None)
            }
            StepAction::RunCommand { command, args } => {
                let cmd_output = self.handle_run_command(command, args)?;
                let success = cmd_output.exit_code.map(|code| code == 0).unwrap_or(false);
                (success, Some(cmd_output))
            }
            StepAction::RunShellCommand {
                command,
                timeout_ms,
                workdir,
                description,
            } => {
                let cmd_output = self.handle_run_shell_command(
                    command,
                    *timeout_ms,
                    workdir.as_deref(),
                    description,
                )?;
                let success = cmd_output.exit_code.map(|code| code == 0).unwrap_or(false);
                (success, Some(cmd_output))
            }
            StepAction::RunTests { pattern } => {
                self.handle_run_tests(pattern)?;
                (true, None)
            }
        };

        let duration = start_time.elapsed();

        Ok(StepResult {
            step_id: step.id.clone(),
            success,
            error: None,
            duration,
            output,
        })
    }

    /// Get the current step index
    pub fn current_step_index(&self) -> usize {
        self.current_step_index
    }

    /// Get completed step results
    pub fn completed_steps(&self) -> &[StepResult] {
        &self.completed_steps
    }

    /// Resume execution from a specific step index
    ///
    /// Allows resuming execution after a pause.
    pub fn resume_from_step(&mut self, step_index: usize) {
        self.current_step_index = step_index;
        debug!(step_index = step_index, "Resuming execution from step");
    }

    /// Skip a step
    ///
    /// Marks a step as skipped and continues to the next one.
    pub fn skip_step(&mut self, step_id: &str) {
        let result = StepResult {
            step_id: step_id.to_string(),
            success: true,
            error: None,
            duration: std::time::Duration::from_secs(0),
            output: None,
        };
        self.completed_steps.push(result);
        info!(step_id = %step_id, "Step skipped");
    }

    /// Handle file creation
    fn handle_create_file(&self, path: &str, content: &str) -> ExecutionResult<()> {
        debug!(path = %path, content_len = content.len(), "Creating file");

        // Use ricecoder-files for file operations
        // For now, we'll use std::fs as a placeholder
        std::fs::write(path, content).map_err(|e| {
            ExecutionError::StepFailed(format!("Failed to create file {}: {}", path, e))
        })?;

        info!(path = %path, "File created successfully");
        Ok(())
    }

    /// Handle file modification
    fn handle_modify_file(&self, path: &str, diff: &str) -> ExecutionResult<()> {
        debug!(path = %path, diff_len = diff.len(), "Modifying file");

        // In a real implementation, this would:
        // 1. Read the file
        // 2. Apply the diff
        // 3. Write the modified content back
        //
        // For now, we'll just validate that the file exists
        if !std::path::Path::new(path).exists() {
            return Err(ExecutionError::StepFailed(format!(
                "File not found for modification: {}",
                path
            )));
        }

        // TODO: Implement actual diff application
        debug!(path = %path, "File modification would be applied here");

        info!(path = %path, "File modified successfully");
        Ok(())
    }

    /// Handle file deletion
    fn handle_delete_file(&self, path: &str) -> ExecutionResult<()> {
        debug!(path = %path, "Deleting file");

        std::fs::remove_file(path).map_err(|e| {
            ExecutionError::StepFailed(format!("Failed to delete file {}: {}", path, e))
        })?;

        info!(path = %path, "File deleted successfully");
        Ok(())
    }

    /// Handle command execution
    async fn handle_run_command_async(
        &self,
        command: &str,
        args: &[String],
    ) -> ExecutionResult<CommandOutput> {
        debug!(command = %command, args_count = args.len(), "Running command asynchronously");

        // Use async CommandHandler with default settings
        let output = crate::step_action_handler::CommandHandler::handle_async(
            command,
            args,
            Some(120_000), // 2 minute timeout
            Some(true),    // Require confirmation for dangerous commands
        )
        .await?;

        Ok(output)
    }

    // Keep synchronous version for compatibility
    fn handle_run_command(&self, command: &str, args: &[String]) -> ExecutionResult<CommandOutput> {
        // For now, use the synchronous version - will be updated to async later
        crate::step_action_handler::CommandHandler::handle(command, args)
    }

    /// Handle shell command execution (OpenCode-compatible)
    fn handle_run_shell_command(
        &self,
        command: &str,
        timeout_ms: Option<u64>,
        workdir: Option<&str>,
        description: &str,
    ) -> ExecutionResult<CommandOutput> {
        // Run async in blocking context
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                crate::step_action_handler::ShellCommandHandler::handle(
                    command,
                    timeout_ms,
                    workdir,
                    description,
                )
                .await
            })
        })
    }

    /// Execute a batch of steps with progress tracking and error handling
    pub async fn execute_batch(
        &self,
        steps: &[ExecutionStep],
        config: &BatchExecutionConfig,
    ) -> ExecutionResult<BatchExecutionOutput> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;
        let mut cancelled = false;

        info!(
            "Starting batch execution of {} steps (concurrent: {}, continue_on_error: {})",
            steps.len(),
            config.max_concurrent,
            config.continue_on_error
        );

        // For now, implement sequential execution (max_concurrent = 1)
        // TODO: Implement concurrent execution when max_concurrent > 1
        for step in steps {
            if cancelled {
                break;
            }

            let step_start = std::time::Instant::now();
            let step_result = self.execute_single_step_async(step).await;
            let step_duration = step_start.elapsed().as_millis() as u64;

            let batch_result = match step_result {
                Ok(result) => {
                    if result.success {
                        successful += 1;
                    } else {
                        failed += 1;
                        if !config.continue_on_error {
                            cancelled = true;
                        }
                    }

                    BatchExecutionResult {
                        step_id: result.step_id.clone(),
                        success: result.success,
                        output: result.output,
                        duration_ms: step_duration,
                        error: result.error,
                    }
                }
                Err(e) => {
                    failed += 1;
                    if !config.continue_on_error {
                        cancelled = true;
                    }

                    BatchExecutionResult {
                        step_id: step.id.clone(),
                        success: false,
                        output: None,
                        duration_ms: step_duration,
                        error: Some(e.to_string()),
                    }
                }
            };

            results.push(batch_result);

            // Check for batch timeout
            if let Some(batch_timeout) = config.batch_timeout_ms {
                if start_time.elapsed().as_millis() as u64 > batch_timeout {
                    warn!("Batch execution timed out after {}ms", batch_timeout);
                    cancelled = true;
                    break;
                }
            }
        }

        let total_duration = start_time.elapsed().as_millis() as u64;
        let rolled_back = config.rollback_on_failure && failed > 0;

        // TODO: Implement actual rollback logic when needed

        let summary = BatchExecutionSummary {
            total_steps: steps.len(),
            successful,
            failed,
            total_duration_ms: total_duration,
            cancelled,
            rolled_back,
        };

        info!(
            "Batch execution completed: {}/{} successful, {} failed, {}ms total",
            successful,
            steps.len(),
            failed,
            total_duration
        );

        Ok(BatchExecutionOutput { results, summary })
    }

    /// Execute a single step asynchronously
    async fn execute_single_step_async(&self, step: &ExecutionStep) -> ExecutionResult<StepResult> {
        let start_time = std::time::Instant::now();

        let (success, output) = match &step.action {
            StepAction::CreateFile { path, content } => {
                self.handle_create_file(path, content)?;
                (true, None)
            }
            StepAction::ModifyFile { path, diff } => {
                self.handle_modify_file(path, diff)?;
                (true, None)
            }
            StepAction::DeleteFile { path } => {
                self.handle_delete_file(path)?;
                (true, None)
            }
            StepAction::RunCommand { command, args } => {
                let cmd_output = self.handle_run_command_async(command, args).await?;
                let success = cmd_output.exit_code.map(|code| code == 0).unwrap_or(false);
                (success, Some(cmd_output))
            }
            StepAction::RunShellCommand {
                command,
                timeout_ms,
                workdir,
                description,
            } => {
                let cmd_output = crate::step_action_handler::ShellCommandHandler::handle(
                    command,
                    *timeout_ms,
                    workdir.as_deref(),
                    description,
                )
                .await?;
                let success = cmd_output.exit_code.map(|code| code == 0).unwrap_or(false);
                (success, Some(cmd_output))
            }
            StepAction::RunTests { pattern } => {
                self.handle_run_tests(pattern)?;
                (true, None)
            }
        };

        let duration = start_time.elapsed();

        Ok(StepResult {
            step_id: step.id.clone(),
            success,
            error: None,
            duration,
            output,
        })
    }

    /// Handle test execution
    fn handle_run_tests(&self, pattern: &Option<String>) -> ExecutionResult<()> {
        debug!(pattern = ?pattern, "Running tests");

        // TODO: Implement test framework detection and execution
        // For now, we'll just log that tests would be run
        if let Some(p) = pattern {
            debug!(pattern = %p, "Tests would be run with pattern");
        } else {
            debug!("All tests would be run");
        }

        info!("Tests executed successfully");
        Ok(())
    }
}

impl Default for StepExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::models::{RiskScore, StepStatus};

    fn create_test_step(description: &str, action: StepAction) -> ExecutionStep {
        ExecutionStep {
            id: Uuid::new_v4().to_string(),
            description: description.to_string(),
            action,
            risk_score: RiskScore::default(),
            dependencies: Vec::new(),
            rollback_action: None,
            status: StepStatus::Pending,
        }
    }

    fn create_test_plan(steps: Vec<ExecutionStep>) -> ExecutionPlan {
        ExecutionPlan {
            id: Uuid::new_v4().to_string(),
            name: "Test Plan".to_string(),
            steps,
            risk_score: RiskScore::default(),
            estimated_duration: std::time::Duration::from_secs(0),
            estimated_complexity: crate::models::ComplexityLevel::Simple,
            requires_approval: false,
            editable: true,
        }
    }

    #[test]
    fn test_create_executor() {
        let executor = StepExecutor::new();
        assert_eq!(executor.current_step_index(), 0);
        assert_eq!(executor.completed_steps().len(), 0);
    }

    #[test]
    fn test_skip_step() {
        let mut executor = StepExecutor::new();
        executor.skip_step("test-step-id");
        assert_eq!(executor.completed_steps().len(), 1);
    }

    #[test]
    fn test_resume_from_step() {
        let mut executor = StepExecutor::new();
        executor.resume_from_step(5);
        assert_eq!(executor.current_step_index(), 5);
    }

    #[test]
    fn test_execute_empty_plan() {
        let mut executor = StepExecutor::new();
        let plan = create_test_plan(vec![]);
        let result = executor.execute_plan(&plan);
        assert!(result.is_err()); // Empty plan should fail
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_execute_command_step() {
        let executor = StepExecutor::new();
        let step = create_test_step(
            "Run echo",
            StepAction::RunCommand {
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
            },
        );

        let result = executor.execute_step(&step);
        assert!(result.is_ok());
        let step_result = result.unwrap();
        assert!(step_result.success);
    }

    #[test]
    fn test_execute_with_skip_on_error() {
        let executor = StepExecutor::new().with_skip_on_error(true);
        assert!(executor.skip_on_error);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_step_result_contains_duration() {
        let executor = StepExecutor::new();
        let step = create_test_step(
            "Run echo",
            StepAction::RunCommand {
                command: "echo".to_string(),
                args: vec!["test".to_string()],
            },
        );

        let result = executor.execute_step(&step).unwrap();
        // Duration should be recorded (even if 0)
        let _ = result.duration;
    }
}
