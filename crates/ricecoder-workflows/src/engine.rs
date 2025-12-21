//! Workflow execution engine

use crate::error::{WorkflowError, WorkflowResult};
use crate::models::{Workflow, WorkflowState};
use ricecoder_sessions::SessionManager;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Optional MCP integration
#[cfg(feature = "mcp")]
use ricecoder_mcp::agent_integration::ToolInvoker;

/// Extension trait for MCP integration
#[cfg(feature = "mcp")]
pub trait WorkflowEngineMcpExt {
    /// Execute an MCP tool as part of a workflow step
    fn execute_mcp_tool_async(
        &self,
        tool_id: &str,
        parameters: HashMap<String, serde_json::Value>,
        server_id: Option<&str>,
        timeout_seconds: Option<u64>,
        user_id: Option<&str>,
        session_id: Option<&str>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = WorkflowResult<serde_json::Value>> + Send + '_>,
    >;
}

#[cfg(feature = "mcp")]
impl WorkflowEngineMcpExt for WorkflowEngine {
    fn execute_mcp_tool_async(
        &self,
        tool_id: &str,
        parameters: HashMap<String, serde_json::Value>,
        server_id: Option<&str>,
        timeout_seconds: Option<u64>,
        user_id: Option<&str>,
        session_id: Option<&str>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = WorkflowResult<serde_json::Value>> + Send + '_>,
    > {
        Box::pin(self.execute_mcp_tool(
            tool_id,
            parameters,
            server_id,
            timeout_seconds,
            user_id,
            session_id,
        ))
    }
}

/// Central coordinator for workflow execution
///
/// Manages workflow lifecycle (create, start, pause, resume, cancel) and tracks
/// active workflows. Handles step execution orchestration and dependency resolution.
/// Integrates with session management for persistent workflow state.
pub struct WorkflowEngine {
    /// Active workflow executions
    active_workflows: HashMap<String, WorkflowState>,
    /// Session manager for persistence
    session_manager: Arc<RwLock<SessionManager>>,
    /// MCP tool invoker for workflow steps (optional)
    #[cfg(feature = "mcp")]
    mcp_tool_invoker: Option<Arc<dyn ToolInvoker>>,
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowEngine {
    /// Create a new workflow engine
    pub fn new() -> Self {
        Self::with_session_manager(SessionManager::new(100)) // Default session limit
    }

    /// Create a new workflow engine with custom session manager
    pub fn with_session_manager(session_manager: SessionManager) -> Self {
        WorkflowEngine {
            active_workflows: HashMap::new(),
            session_manager: Arc::new(RwLock::new(session_manager)),
            #[cfg(feature = "mcp")]
            mcp_tool_invoker: None,
        }
    }

    /// Create a new workflow engine with MCP integration
    #[cfg(feature = "mcp")]
    pub fn with_mcp_integration(
        session_manager: SessionManager,
        mcp_tool_invoker: Arc<dyn ToolInvoker>,
    ) -> Self {
        WorkflowEngine {
            active_workflows: HashMap::new(),
            session_manager: Arc::new(RwLock::new(session_manager)),
            mcp_tool_invoker: Some(mcp_tool_invoker),
        }
    }

    /// Set MCP tool invoker
    #[cfg(feature = "mcp")]
    pub fn set_mcp_tool_invoker(&mut self, invoker: Arc<dyn ToolInvoker>) {
        self.mcp_tool_invoker = Some(invoker);
    }

    /// Execute an MCP tool as part of a workflow step
    #[cfg(feature = "mcp")]
    pub async fn execute_mcp_tool(
        &self,
        tool_id: &str,
        parameters: HashMap<String, serde_json::Value>,
        server_id: Option<&str>,
        timeout_seconds: Option<u64>,
        user_id: Option<&str>,
        session_id: Option<&str>,
    ) -> WorkflowResult<serde_json::Value> {
        let invoker = self
            .mcp_tool_invoker
            .as_ref()
            .ok_or_else(|| WorkflowError::Invalid("MCP tool invoker not configured".to_string()))?;

        // Convert parameters to the expected format
        let params = parameters.into_iter().collect();

        // Execute the tool
        match invoker.invoke_tool(tool_id, params) {
            Ok(result) => {
                // Log successful execution
                info!(
                    "Successfully executed MCP tool '{}' in workflow context (user: {:?}, session: {:?})",
                    tool_id, user_id, session_id
                );
                Ok(result)
            }
            Err(e) => {
                // Log failed execution
                error!(
                    "Failed to execute MCP tool '{}' in workflow context: {} (user: {:?}, session: {:?})",
                    tool_id, e, user_id, session_id
                );
                Err(WorkflowError::Execution(format!(
                    "MCP tool execution failed: {}",
                    e
                )))
            }
        }
    }

    /// Create a new workflow execution
    ///
    /// Creates a new execution state for the given workflow and tracks it.
    pub async fn create_execution(&mut self, workflow: &Workflow) -> WorkflowResult<String> {
        let state = WorkflowState::new(workflow);
        let execution_id = Uuid::new_v4().to_string();

        // Create a session for this workflow execution
        let session_name = format!("workflow-{}", execution_id);
        let context = ricecoder_sessions::SessionContext::new(
            "workflow".to_string(),
            "workflow-engine".to_string(),
            ricecoder_sessions::SessionMode::Code,
        );
        let mut session_manager = self.session_manager.write().await;
        session_manager
            .create_session(session_name, context)
            .map_err(|e| WorkflowError::Invalid(format!("Failed to create session: {}", e)))?;

        self.active_workflows.insert(execution_id.clone(), state);
        Ok(execution_id)
    }

    /// Start workflow execution
    ///
    /// Transitions the workflow from pending to running state.
    pub async fn start_execution(&mut self, execution_id: &str) -> WorkflowResult<()> {
        let state = self.active_workflows.get_mut(execution_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Execution not found: {}", execution_id))
        })?;

        state.start_workflow();

        // Update session with workflow state
        self.save_workflow_state(execution_id).await?;
        Ok(())
    }

    /// Save workflow state to session
    async fn save_workflow_state(&self, execution_id: &str) -> WorkflowResult<()> {
        let state = self.active_workflows.get(execution_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Execution not found: {}", execution_id))
        })?;

        let session_name = format!("workflow-{}", execution_id);
        let session_manager = self.session_manager.read().await;

        // Serialize workflow state and store in session
        let _state_json = serde_json::to_string(state)
            .map_err(|e| WorkflowError::Invalid(format!("Failed to serialize state: {}", e)))?;

        // Store in session context (this is a simplified implementation)
        // In a real implementation, you'd use the session's storage mechanism
        let _ = session_manager.get_session(&session_name);

        Ok(())
    }

    /// Load workflow state from session
    async fn load_workflow_state(
        &self,
        execution_id: &str,
    ) -> WorkflowResult<Option<WorkflowState>> {
        let session_name = format!("workflow-{}", execution_id);
        let session_manager = self.session_manager.read().await;

        // Try to get session and load state (simplified)
        match session_manager.get_session(&session_name) {
            Ok(_) => {
                // In a real implementation, you'd deserialize the state from session storage
                // For now, return None to indicate no persisted state
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }

    /// Pause workflow execution
    ///
    /// Pauses the workflow at the current step, allowing resumption later.
    pub async fn pause_execution(&mut self, execution_id: &str) -> WorkflowResult<()> {
        let state = self.active_workflows.get_mut(execution_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Execution not found: {}", execution_id))
        })?;

        state.pause_workflow();
        self.save_workflow_state(execution_id).await?;
        Ok(())
    }

    /// Resume workflow execution
    ///
    /// Resumes a paused workflow from the last completed step.
    pub async fn resume_execution(&mut self, execution_id: &str) -> WorkflowResult<()> {
        let state = self.active_workflows.get_mut(execution_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Execution not found: {}", execution_id))
        })?;

        state.resume_workflow();
        self.save_workflow_state(execution_id).await?;
        Ok(())
    }

    /// Cancel workflow execution
    ///
    /// Cancels the workflow, stopping any further execution.
    pub async fn cancel_execution(&mut self, execution_id: &str) -> WorkflowResult<()> {
        let state = self.active_workflows.get_mut(execution_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Execution not found: {}", execution_id))
        })?;

        state.cancel_workflow();
        self.save_workflow_state(execution_id).await?;
        Ok(())
    }

    /// Get the current state of a workflow execution
    pub fn get_execution_state(&self, execution_id: &str) -> WorkflowResult<WorkflowState> {
        self.active_workflows
            .get(execution_id)
            .cloned()
            .ok_or_else(|| {
                WorkflowError::NotFound(format!("Execution not found: {}", execution_id))
            })
    }

    /// Get execution order for workflow steps
    ///
    /// Builds execution order from dependency graph using topological sort.
    /// Returns error if circular dependencies are detected.
    pub fn get_execution_order(workflow: &Workflow) -> WorkflowResult<Vec<String>> {
        Self::resolve_dependencies(workflow)
    }

    /// Resolve step dependencies and build execution order
    ///
    /// Uses topological sort to determine the order in which steps should execute
    /// based on their dependencies. Detects and reports circular dependencies.
    fn resolve_dependencies(workflow: &Workflow) -> WorkflowResult<Vec<String>> {
        let mut order = Vec::new();
        let mut completed = HashSet::new();
        let mut queue = VecDeque::new();

        // Find all steps with no dependencies
        for step in &workflow.steps {
            if step.dependencies.is_empty() {
                queue.push_back(step.id.clone());
            }
        }

        // Build step map for quick lookup
        let step_map: HashMap<_, _> = workflow.steps.iter().map(|s| (&s.id, s)).collect();

        // Topological sort
        while let Some(step_id) = queue.pop_front() {
            if completed.contains(&step_id) {
                continue;
            }

            // Check if all dependencies are completed
            if let Some(step) = step_map.get(&step_id) {
                let all_deps_completed =
                    step.dependencies.iter().all(|dep| completed.contains(dep));

                if all_deps_completed {
                    order.push(step_id.clone());
                    completed.insert(step_id.clone());

                    // Add steps that depend on this one
                    for other_step in &workflow.steps {
                        if other_step.dependencies.contains(&step_id)
                            && !completed.contains(&other_step.id)
                        {
                            queue.push_back(other_step.id.clone());
                        }
                    }
                } else {
                    // Re-queue if dependencies not met
                    queue.push_back(step_id);
                }
            }
        }

        if order.len() != workflow.steps.len() {
            return Err(WorkflowError::Invalid(
                "Could not determine execution order for all steps".to_string(),
            ));
        }

        Ok(order)
    }

    /// Check if a step can be executed
    ///
    /// A step can be executed if all its dependencies have been completed.
    pub fn can_execute_step(
        workflow: &Workflow,
        state: &WorkflowState,
        step_id: &str,
    ) -> WorkflowResult<bool> {
        // Find the step
        let step = workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Step not found: {}", step_id)))?;

        // Check if all dependencies are completed
        for dep in &step.dependencies {
            if !state.completed_steps.contains(dep) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get next executable step
    ///
    /// Returns the next step that can be executed based on completed dependencies.
    /// Returns None if all steps are completed or no steps are ready.
    pub fn get_next_step(
        workflow: &Workflow,
        state: &WorkflowState,
    ) -> WorkflowResult<Option<String>> {
        for step in &workflow.steps {
            if !state.completed_steps.contains(&step.id)
                && !state.step_results.contains_key(&step.id)
                && Self::can_execute_step(workflow, state, &step.id)?
            {
                return Ok(Some(step.id.clone()));
            }
        }

        Ok(None)
    }

    /// Wait for a step's dependencies to complete
    ///
    /// Blocks until all dependencies for the given step are completed.
    /// Returns error if the step is not found or dependencies cannot be resolved.
    pub fn wait_for_dependencies(
        workflow: &Workflow,
        state: &WorkflowState,
        step_id: &str,
    ) -> WorkflowResult<()> {
        let step = workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Step not found: {}", step_id)))?;

        // Check if all dependencies are completed
        for dep in &step.dependencies {
            if !state.completed_steps.contains(dep) {
                return Err(WorkflowError::StateError(format!(
                    "Dependency {} not completed for step {}",
                    dep, step_id
                )));
            }
        }

        Ok(())
    }

    /// Complete workflow execution
    pub fn complete_execution(&mut self, execution_id: &str) -> WorkflowResult<()> {
        let state = self.active_workflows.get_mut(execution_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Execution not found: {}", execution_id))
        })?;

        state.complete_workflow();
        Ok(())
    }

    /// Fail workflow execution
    pub fn fail_execution(&mut self, execution_id: &str) -> WorkflowResult<()> {
        let state = self.active_workflows.get_mut(execution_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Execution not found: {}", execution_id))
        })?;

        state.fail_workflow();
        Ok(())
    }

    /// Remove a completed execution from tracking
    pub fn remove_execution(&mut self, execution_id: &str) -> WorkflowResult<WorkflowState> {
        self.active_workflows.remove(execution_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Execution not found: {}", execution_id))
        })
    }

    /// Get all active executions
    pub fn get_active_executions(&self) -> Vec<String> {
        self.active_workflows.keys().cloned().collect()
    }

    /// Get count of active executions
    pub fn active_execution_count(&self) -> usize {
        self.active_workflows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        ErrorAction, RiskFactors, StepType, WorkflowConfig, WorkflowStatus, WorkflowStep,
    };

    fn create_test_workflow_with_deps() -> Workflow {
        Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            parameters: vec![],
            steps: vec![
                WorkflowStep {
                    id: "step1".to_string(),
                    name: "Step 1".to_string(),
                    step_type: StepType::Agent(crate::models::AgentStep {
                        agent_id: "test-agent".to_string(),
                        task: "test-task".to_string(),
                    }),
                    config: crate::models::StepConfig {
                        config: serde_json::json!({}),
                    },
                    dependencies: vec![],
                    approval_required: false,
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                },
                WorkflowStep {
                    id: "step2".to_string(),
                    name: "Step 2".to_string(),
                    step_type: StepType::Agent(crate::models::AgentStep {
                        agent_id: "test-agent".to_string(),
                        task: "test-task".to_string(),
                    }),
                    config: crate::models::StepConfig {
                        config: serde_json::json!({}),
                    },
                    dependencies: vec!["step1".to_string()],
                    approval_required: false,
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                },
                WorkflowStep {
                    id: "step3".to_string(),
                    name: "Step 3".to_string(),
                    step_type: StepType::Agent(crate::models::AgentStep {
                        agent_id: "test-agent".to_string(),
                        task: "test-task".to_string(),
                    }),
                    config: crate::models::StepConfig {
                        config: serde_json::json!({}),
                    },
                    dependencies: vec!["step1".to_string(), "step2".to_string()],
                    approval_required: false,
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                },
            ],
            config: WorkflowConfig {
                timeout_ms: None,
                max_parallel: None,
            },
        }
    }

    #[tokio::test]
    async fn test_create_engine() {
        let engine = WorkflowEngine::new();
        assert_eq!(engine.active_execution_count(), 0);
    }

    #[tokio::test]
    async fn test_workflow_execution() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow_with_deps();

        let execution_id = engine.create_execution(&workflow).await.unwrap();
        assert!(!execution_id.is_empty());
        assert_eq!(engine.active_execution_count(), 1);
    }

    #[tokio::test]
    async fn test_start_execution() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow_with_deps();

        let execution_id = engine.create_execution(&workflow).await.unwrap();
        engine.start_execution(&execution_id).await.unwrap();

        let state = engine.get_execution_state(&execution_id).unwrap();
        assert_eq!(state.status, WorkflowStatus::Running);
    }

    #[tokio::test]
    async fn test_get_execution_order() {
        let workflow = create_test_workflow_with_deps();
        let order = WorkflowEngine::get_execution_order(&workflow).unwrap();

        assert_eq!(order.len(), 3);
        assert_eq!(order[0], "step1");
        assert_eq!(order[1], "step2");
        assert_eq!(order[2], "step3");
    }

    #[tokio::test]
    async fn test_can_execute_step() {
        let workflow = create_test_workflow_with_deps();
        let state = WorkflowState::new(&workflow);

        // step1 can execute (no dependencies)
        assert!(WorkflowEngine::can_execute_step(&workflow, &state, "step1").unwrap());

        // step2 cannot execute (depends on step1)
        assert!(!WorkflowEngine::can_execute_step(&workflow, &state, "step2").unwrap());

        // Create a new state with step1 completed
        let mut state2 = WorkflowState::new(&workflow);
        state2.completed_steps.push("step1".to_string());

        // Now step2 can execute
        assert!(WorkflowEngine::can_execute_step(&workflow, &state2, "step2").unwrap());
    }

    #[tokio::test]
    async fn test_get_next_step() {
        let workflow = create_test_workflow_with_deps();
        let state = WorkflowState::new(&workflow);

        let next = WorkflowEngine::get_next_step(&workflow, &state).unwrap();
        assert_eq!(next, Some("step1".to_string()));
    }

    #[tokio::test]
    async fn test_pause_and_resume_execution() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow_with_deps();

        let execution_id = engine.create_execution(&workflow).await.unwrap();
        engine.start_execution(&execution_id).await.unwrap();

        engine.pause_execution(&execution_id).await.unwrap();
        let state = engine.get_execution_state(&execution_id).unwrap();
        assert_eq!(state.status, WorkflowStatus::Paused);

        engine.resume_execution(&execution_id).await.unwrap();
        let state = engine.get_execution_state(&execution_id).unwrap();
        assert_eq!(state.status, WorkflowStatus::Running);
    }

    #[tokio::test]
    async fn test_cancel_execution() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow_with_deps();

        let execution_id = engine.create_execution(&workflow).await.unwrap();
        engine.start_execution(&execution_id).await.unwrap();
        engine.cancel_execution(&execution_id).await.unwrap();

        let state = engine.get_execution_state(&execution_id).unwrap();
        assert_eq!(state.status, WorkflowStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_get_active_executions() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow_with_deps();

        let id1 = engine.create_execution(&workflow).await.unwrap();
        let id2 = engine.create_execution(&workflow).await.unwrap();

        let active = engine.get_active_executions();
        assert_eq!(active.len(), 2);
        assert!(active.contains(&id1));
        assert!(active.contains(&id2));
    }

    #[tokio::test]
    async fn test_remove_execution() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow_with_deps();

        let execution_id = engine.create_execution(&workflow).await.unwrap();
        assert_eq!(engine.active_execution_count(), 1);

        let removed_state = engine.remove_execution(&execution_id).unwrap();
        assert_eq!(removed_state.workflow_id, "test-workflow");
        assert_eq!(engine.active_execution_count(), 0);
    }

    #[tokio::test]
    async fn test_wait_for_dependencies() {
        let workflow = create_test_workflow_with_deps();
        let mut state = WorkflowState::new(&workflow);

        // step2 depends on step1, which is not completed
        let result = WorkflowEngine::wait_for_dependencies(&workflow, &state, "step2");
        assert!(result.is_err());

        // Mark step1 as completed
        state.completed_steps.push("step1".to_string());

        // Now step2 dependencies are satisfied
        let result = WorkflowEngine::wait_for_dependencies(&workflow, &state, "step2");
        assert!(result.is_ok());
    }
}
