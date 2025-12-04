//! Integration with ricecoder-storage for workflow state persistence

use crate::error::{WorkflowError, WorkflowResult};
use crate::models::WorkflowState;
use std::path::{Path, PathBuf};

/// Storage integration for workflow state persistence
pub struct StorageIntegration;

impl StorageIntegration {
    /// Persist workflow state using storage manager
    ///
    /// This function handles storage errors gracefully by:
    /// - Creating necessary directories
    /// - Serializing state to JSON format
    /// - Handling IO errors with context
    pub fn persist_state(state: &WorkflowState, storage_path: &Path) -> WorkflowResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = storage_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                WorkflowError::StateError(format!(
                    "Failed to create storage directory at {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        // Serialize state to JSON
        let json = serde_json::to_string_pretty(state).map_err(|e| {
            WorkflowError::StateError(format!("Failed to serialize workflow state: {}", e))
        })?;

        // Write to file
        std::fs::write(storage_path, json).map_err(|e| {
            WorkflowError::StateError(format!(
                "Failed to write workflow state to {}: {}",
                storage_path.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Load workflow state from storage
    ///
    /// This function handles storage errors gracefully by:
    /// - Checking if file exists
    /// - Reading and deserializing JSON
    /// - Validating state integrity
    pub fn load_state(storage_path: &Path) -> WorkflowResult<WorkflowState> {
        if !storage_path.exists() {
            return Err(WorkflowError::StateError(format!(
                "Workflow state file not found at {}",
                storage_path.display()
            )));
        }

        // Read file
        let content = std::fs::read_to_string(storage_path).map_err(|e| {
            WorkflowError::StateError(format!(
                "Failed to read workflow state from {}: {}",
                storage_path.display(),
                e
            ))
        })?;

        // Try JSON first (primary format)
        if let Ok(state) = serde_json::from_str::<WorkflowState>(&content) {
            return Ok(state);
        }

        // Fall back to YAML for backward compatibility
        serde_yaml::from_str::<WorkflowState>(&content).map_err(|e| {
            WorkflowError::StateError(format!(
                "Failed to deserialize workflow state from {}: {}",
                storage_path.display(),
                e
            ))
        })
    }

    /// Load workflow state with validation
    ///
    /// Validates state integrity after loading to ensure:
    /// - Workflow ID is not empty
    /// - All completed steps have results
    /// - Current step (if any) has a result
    pub fn load_state_validated(storage_path: &Path) -> WorkflowResult<WorkflowState> {
        let state = Self::load_state(storage_path)?;
        Self::validate_state(&state)?;
        Ok(state)
    }

    /// Validate workflow state integrity
    fn validate_state(state: &WorkflowState) -> WorkflowResult<()> {
        // Check that workflow_id is not empty
        if state.workflow_id.is_empty() {
            return Err(WorkflowError::StateError(
                "Workflow ID cannot be empty".to_string(),
            ));
        }

        // Check that all completed steps have results
        for step_id in &state.completed_steps {
            if !state.step_results.contains_key(step_id) {
                return Err(WorkflowError::StateError(format!(
                    "Completed step '{}' has no result",
                    step_id
                )));
            }
        }

        // Check that current step (if any) has a result
        if let Some(current_step) = &state.current_step {
            if !state.step_results.contains_key(current_step) {
                return Err(WorkflowError::StateError(format!(
                    "Current step '{}' has no result",
                    current_step
                )));
            }
        }

        Ok(())
    }

    /// Delete workflow state from storage
    ///
    /// Handles deletion errors gracefully
    pub fn delete_state(storage_path: &Path) -> WorkflowResult<()> {
        if storage_path.exists() {
            std::fs::remove_file(storage_path).map_err(|e| {
                WorkflowError::StateError(format!(
                    "Failed to delete workflow state at {}: {}",
                    storage_path.display(),
                    e
                ))
            })?;
        }
        Ok(())
    }

    /// Check if workflow state exists in storage
    pub fn state_exists(storage_path: &Path) -> bool {
        storage_path.exists()
    }

    /// Get the storage path for a workflow
    ///
    /// Constructs a path like: `{base_path}/workflows/{workflow_id}/state.json`
    pub fn get_workflow_state_path(base_path: &Path, workflow_id: &str) -> PathBuf {
        base_path
            .join("workflows")
            .join(workflow_id)
            .join("state.json")
    }

    /// Get the storage directory for a workflow
    ///
    /// Constructs a path like: `{base_path}/workflows/{workflow_id}/`
    pub fn get_workflow_storage_dir(base_path: &Path, workflow_id: &str) -> PathBuf {
        base_path.join("workflows").join(workflow_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{StepStatus, WorkflowStatus};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_state() -> WorkflowState {
        let mut step_results = HashMap::new();
        step_results.insert(
            "step1".to_string(),
            crate::models::StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"result": "success"})),
                error: None,
                duration_ms: 100,
            },
        );
        step_results.insert(
            "step2".to_string(),
            crate::models::StepResult {
                status: StepStatus::Running,
                output: None,
                error: None,
                duration_ms: 0,
            },
        );

        WorkflowState {
            workflow_id: "test-workflow".to_string(),
            status: WorkflowStatus::Running,
            current_step: Some("step2".to_string()),
            completed_steps: vec!["step1".to_string()],
            step_results,
            started_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_persist_and_load_state() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");

        let original_state = create_test_state();

        // Persist state
        let result = StorageIntegration::persist_state(&original_state, &state_path);
        assert!(result.is_ok());
        assert!(state_path.exists());

        // Load state
        let loaded_state = StorageIntegration::load_state(&state_path).unwrap();
        assert_eq!(loaded_state.workflow_id, original_state.workflow_id);
        assert_eq!(loaded_state.status, original_state.status);
        assert_eq!(loaded_state.completed_steps, original_state.completed_steps);
    }

    #[test]
    fn test_load_nonexistent_state() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("nonexistent.json");

        let result = StorageIntegration::load_state(&state_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_state_success() {
        let state = create_test_state();
        let result = StorageIntegration::validate_state(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_state_empty_workflow_id() {
        let mut state = create_test_state();
        state.workflow_id = String::new();

        let result = StorageIntegration::validate_state(&state);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_state_missing_result() {
        let mut state = create_test_state();
        state.completed_steps.push("step3".to_string());
        // Don't add result for step3

        let result = StorageIntegration::validate_state(&state);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_state() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");

        let state = create_test_state();
        StorageIntegration::persist_state(&state, &state_path).unwrap();
        assert!(state_path.exists());

        let result = StorageIntegration::delete_state(&state_path);
        assert!(result.is_ok());
        assert!(!state_path.exists());
    }

    #[test]
    fn test_state_exists() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");

        assert!(!StorageIntegration::state_exists(&state_path));

        let state = create_test_state();
        StorageIntegration::persist_state(&state, &state_path).unwrap();
        assert!(StorageIntegration::state_exists(&state_path));
    }

    #[test]
    fn test_get_workflow_state_path() {
        let base_path = Path::new("/storage");
        let path = StorageIntegration::get_workflow_state_path(base_path, "my-workflow");

        assert_eq!(
            path,
            PathBuf::from("/storage/workflows/my-workflow/state.json")
        );
    }

    #[test]
    fn test_get_workflow_storage_dir() {
        let base_path = Path::new("/storage");
        let dir = StorageIntegration::get_workflow_storage_dir(base_path, "my-workflow");

        assert_eq!(dir, PathBuf::from("/storage/workflows/my-workflow"));
    }

    #[test]
    fn test_load_state_validated() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");

        let state = create_test_state();
        StorageIntegration::persist_state(&state, &state_path).unwrap();

        let result = StorageIntegration::load_state_validated(&state_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_state_validated_with_invalid_state() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");

        let mut state = create_test_state();
        state.workflow_id = String::new();
        StorageIntegration::persist_state(&state, &state_path).unwrap();

        let result = StorageIntegration::load_state_validated(&state_path);
        assert!(result.is_err());
    }
}
