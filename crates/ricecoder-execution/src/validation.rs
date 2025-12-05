//! Input validation for execution plans, steps, and configurations

use crate::error::{ExecutionError, ExecutionResult};
use crate::models::{ExecutionPlan, ExecutionStep, StepAction};

/// Validator for execution plans and their components
pub struct ExecutionValidator;

impl ExecutionValidator {
    /// Validate an entire execution plan
    ///
    /// # Arguments
    /// * `plan` - The execution plan to validate
    ///
    /// # Returns
    /// * `Ok(())` if the plan is valid
    /// * `Err(ExecutionError)` if validation fails
    pub fn validate_plan(plan: &ExecutionPlan) -> ExecutionResult<()> {
        // Validate plan name
        Self::validate_plan_name(&plan.name)?;

        // Validate plan has at least one step
        if plan.steps.is_empty() {
            return Err(ExecutionError::ValidationError(
                "Execution plan must contain at least one step".to_string(),
            ));
        }

        // Validate each step
        for step in &plan.steps {
            Self::validate_step(step)?;
        }

        // Validate step dependencies
        Self::validate_dependencies(plan)?;

        Ok(())
    }

    /// Validate a single execution step
    ///
    /// # Arguments
    /// * `step` - The execution step to validate
    ///
    /// # Returns
    /// * `Ok(())` if the step is valid
    /// * `Err(ExecutionError)` if validation fails
    pub fn validate_step(step: &ExecutionStep) -> ExecutionResult<()> {
        // Validate step description
        if step.description.is_empty() {
            return Err(ExecutionError::ValidationError(
                "Step description cannot be empty".to_string(),
            ));
        }

        if step.description.len() > 1000 {
            return Err(ExecutionError::ValidationError(
                "Step description cannot exceed 1000 characters".to_string(),
            ));
        }

        // Validate step action
        Self::validate_step_action(&step.action)?;

        Ok(())
    }

    /// Validate a step action
    ///
    /// # Arguments
    /// * `action` - The step action to validate
    ///
    /// # Returns
    /// * `Ok(())` if the action is valid
    /// * `Err(ExecutionError)` if validation fails
    pub fn validate_step_action(action: &StepAction) -> ExecutionResult<()> {
        match action {
            StepAction::CreateFile { path, content } => {
                Self::validate_file_path(path)?;
                Self::validate_file_content(content)?;
            }
            StepAction::ModifyFile { path, diff } => {
                Self::validate_file_path(path)?;
                Self::validate_diff(diff)?;
            }
            StepAction::DeleteFile { path } => {
                Self::validate_file_path(path)?;
            }
            StepAction::RunCommand { command, args } => {
                Self::validate_command(command)?;
                Self::validate_command_args(args)?;
            }
            StepAction::RunTests { pattern } => {
                if let Some(p) = pattern {
                    Self::validate_test_pattern(p)?;
                }
            }
        }

        Ok(())
    }

    /// Validate a file path
    ///
    /// # Arguments
    /// * `path` - The file path to validate
    ///
    /// # Returns
    /// * `Ok(())` if the path is valid
    /// * `Err(ExecutionError)` if validation fails
    pub fn validate_file_path(path: &str) -> ExecutionResult<()> {
        // Path cannot be empty
        if path.is_empty() {
            return Err(ExecutionError::ValidationError(
                "File path cannot be empty".to_string(),
            ));
        }

        // Path cannot exceed reasonable length
        if path.len() > 4096 {
            return Err(ExecutionError::ValidationError(
                "File path cannot exceed 4096 characters".to_string(),
            ));
        }

        // Path cannot contain null bytes
        if path.contains('\0') {
            return Err(ExecutionError::ValidationError(
                "File path cannot contain null bytes".to_string(),
            ));
        }

        // Path should not start with absolute path indicators (security check)
        // Allow relative paths and paths resolved by PathResolver
        if path.starts_with('/') && !path.starts_with("./") && !path.starts_with("../") {
            // Absolute paths are allowed but should be validated by PathResolver
            // This is a warning-level check
        }

        Ok(())
    }

    /// Validate file content
    ///
    /// # Arguments
    /// * `content` - The file content to validate
    ///
    /// # Returns
    /// * `Ok(())` if the content is valid
    /// * `Err(ExecutionError)` if validation fails
    pub fn validate_file_content(content: &str) -> ExecutionResult<()> {
        // Content can be empty (empty files are valid)
        // Content cannot exceed reasonable size (100MB)
        if content.len() > 100 * 1024 * 1024 {
            return Err(ExecutionError::ValidationError(
                "File content cannot exceed 100MB".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate a diff
    ///
    /// # Arguments
    /// * `diff` - The diff to validate
    ///
    /// # Returns
    /// * `Ok(())` if the diff is valid
    /// * `Err(ExecutionError)` if validation fails
    pub fn validate_diff(diff: &str) -> ExecutionResult<()> {
        // Diff cannot be empty
        if diff.is_empty() {
            return Err(ExecutionError::ValidationError(
                "Diff cannot be empty".to_string(),
            ));
        }

        // Diff cannot exceed reasonable size (10MB)
        if diff.len() > 10 * 1024 * 1024 {
            return Err(ExecutionError::ValidationError(
                "Diff cannot exceed 10MB".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate a shell command
    ///
    /// # Arguments
    /// * `command` - The command to validate
    ///
    /// # Returns
    /// * `Ok(())` if the command is valid
    /// * `Err(ExecutionError)` if validation fails
    pub fn validate_command(command: &str) -> ExecutionResult<()> {
        // Command cannot be empty
        if command.is_empty() {
            return Err(ExecutionError::ValidationError(
                "Command cannot be empty".to_string(),
            ));
        }

        // Command cannot exceed reasonable length
        if command.len() > 4096 {
            return Err(ExecutionError::ValidationError(
                "Command cannot exceed 4096 characters".to_string(),
            ));
        }

        // Command cannot contain null bytes
        if command.contains('\0') {
            return Err(ExecutionError::ValidationError(
                "Command cannot contain null bytes".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate command arguments
    ///
    /// # Arguments
    /// * `args` - The command arguments to validate
    ///
    /// # Returns
    /// * `Ok(())` if the arguments are valid
    /// * `Err(ExecutionError)` if validation fails
    pub fn validate_command_args(args: &[String]) -> ExecutionResult<()> {
        // Arguments list cannot exceed reasonable size
        if args.len() > 1000 {
            return Err(ExecutionError::ValidationError(
                "Command arguments cannot exceed 1000 items".to_string(),
            ));
        }

        // Each argument must be valid
        for arg in args {
            if arg.len() > 4096 {
                return Err(ExecutionError::ValidationError(
                    "Command argument cannot exceed 4096 characters".to_string(),
                ));
            }

            if arg.contains('\0') {
                return Err(ExecutionError::ValidationError(
                    "Command argument cannot contain null bytes".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate a test pattern
    ///
    /// # Arguments
    /// * `pattern` - The test pattern to validate
    ///
    /// # Returns
    /// * `Ok(())` if the pattern is valid
    /// * `Err(ExecutionError)` if validation fails
    pub fn validate_test_pattern(pattern: &str) -> ExecutionResult<()> {
        // Pattern cannot be empty
        if pattern.is_empty() {
            return Err(ExecutionError::ValidationError(
                "Test pattern cannot be empty".to_string(),
            ));
        }

        // Pattern cannot exceed reasonable length
        if pattern.len() > 1024 {
            return Err(ExecutionError::ValidationError(
                "Test pattern cannot exceed 1024 characters".to_string(),
            ));
        }

        // Pattern cannot contain null bytes
        if pattern.contains('\0') {
            return Err(ExecutionError::ValidationError(
                "Test pattern cannot contain null bytes".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate plan name
    ///
    /// # Arguments
    /// * `name` - The plan name to validate
    ///
    /// # Returns
    /// * `Ok(())` if the name is valid
    /// * `Err(ExecutionError)` if validation fails
    pub fn validate_plan_name(name: &str) -> ExecutionResult<()> {
        // Name cannot be empty
        if name.is_empty() {
            return Err(ExecutionError::ValidationError(
                "Plan name cannot be empty".to_string(),
            ));
        }

        // Name cannot exceed reasonable length
        if name.len() > 256 {
            return Err(ExecutionError::ValidationError(
                "Plan name cannot exceed 256 characters".to_string(),
            ));
        }

        // Name cannot contain null bytes
        if name.contains('\0') {
            return Err(ExecutionError::ValidationError(
                "Plan name cannot contain null bytes".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate step dependencies
    ///
    /// # Arguments
    /// * `plan` - The execution plan to validate dependencies for
    ///
    /// # Returns
    /// * `Ok(())` if dependencies are valid
    /// * `Err(ExecutionError)` if validation fails
    fn validate_dependencies(plan: &ExecutionPlan) -> ExecutionResult<()> {
        // Create a set of valid step IDs
        let valid_ids: std::collections::HashSet<_> =
            plan.steps.iter().map(|s| s.id.as_str()).collect();

        // Check each step's dependencies
        for step in &plan.steps {
            for dep_id in &step.dependencies {
                // Dependency must reference an existing step
                if !valid_ids.contains(dep_id.as_str()) {
                    return Err(ExecutionError::ValidationError(format!(
                        "Step {} references non-existent dependency: {}",
                        step.id, dep_id
                    )));
                }

                // Dependency cannot be self-referential
                if dep_id == &step.id {
                    return Err(ExecutionError::ValidationError(format!(
                        "Step {} has self-referential dependency",
                        step.id
                    )));
                }
            }
        }

        // Check for circular dependencies
        Self::check_circular_dependencies(plan)?;

        Ok(())
    }

    /// Check for circular dependencies in the plan
    ///
    /// # Arguments
    /// * `plan` - The execution plan to check
    ///
    /// # Returns
    /// * `Ok(())` if no circular dependencies exist
    /// * `Err(ExecutionError)` if circular dependencies are found
    fn check_circular_dependencies(plan: &ExecutionPlan) -> ExecutionResult<()> {
        // Build a map of step ID to dependencies
        let mut dep_map: std::collections::HashMap<&str, Vec<&str>> =
            std::collections::HashMap::new();

        for step in &plan.steps {
            dep_map.insert(
                &step.id,
                step.dependencies.iter().map(|s| s.as_str()).collect(),
            );
        }

        // Check each step for cycles
        for step in &plan.steps {
            let mut visited = std::collections::HashSet::new();
            let mut rec_stack = std::collections::HashSet::new();

            if Self::has_cycle(&step.id, &dep_map, &mut visited, &mut rec_stack) {
                return Err(ExecutionError::ValidationError(format!(
                    "Circular dependency detected involving step: {}",
                    step.id
                )));
            }
        }

        Ok(())
    }

    /// Helper function to detect cycles using DFS
    fn has_cycle(
        node: &str,
        dep_map: &std::collections::HashMap<&str, Vec<&str>>,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        let node_str = node.to_string();

        visited.insert(node_str.clone());
        rec_stack.insert(node_str.clone());

        if let Some(deps) = dep_map.get(node) {
            for dep in deps {
                let dep_str = dep.to_string();

                if !visited.contains(&dep_str) {
                    if Self::has_cycle(dep, dep_map, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(&dep_str) {
                    return true;
                }
            }
        }

        rec_stack.remove(&node_str);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_plan_name_empty() {
        let result = ExecutionValidator::validate_plan_name("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_plan_name_valid() {
        let result = ExecutionValidator::validate_plan_name("My Execution Plan");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_plan_name_too_long() {
        let long_name = "a".repeat(257);
        let result = ExecutionValidator::validate_plan_name(&long_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_path_empty() {
        let result = ExecutionValidator::validate_file_path("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_path_valid() {
        let result = ExecutionValidator::validate_file_path("src/main.rs");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_path_with_null_byte() {
        let result = ExecutionValidator::validate_file_path("src/main\0.rs");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_content_empty() {
        let result = ExecutionValidator::validate_file_content("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_content_valid() {
        let result = ExecutionValidator::validate_file_content("fn main() {}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_command_empty() {
        let result = ExecutionValidator::validate_command("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_command_valid() {
        let result = ExecutionValidator::validate_command("cargo build");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_command_args_valid() {
        let args = vec!["--release".to_string(), "--verbose".to_string()];
        let result = ExecutionValidator::validate_command_args(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_test_pattern_empty() {
        let result = ExecutionValidator::validate_test_pattern("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_test_pattern_valid() {
        let result = ExecutionValidator::validate_test_pattern("test_*");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_step_action_create_file() {
        let action = StepAction::CreateFile {
            path: "src/lib.rs".to_string(),
            content: "pub fn hello() {}".to_string(),
        };
        let result = ExecutionValidator::validate_step_action(&action);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_step_action_create_file_empty_path() {
        let action = StepAction::CreateFile {
            path: "".to_string(),
            content: "pub fn hello() {}".to_string(),
        };
        let result = ExecutionValidator::validate_step_action(&action);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_step_action_modify_file() {
        let action = StepAction::ModifyFile {
            path: "src/lib.rs".to_string(),
            diff: "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-old\n+new".to_string(),
        };
        let result = ExecutionValidator::validate_step_action(&action);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_step_action_delete_file() {
        let action = StepAction::DeleteFile {
            path: "src/old.rs".to_string(),
        };
        let result = ExecutionValidator::validate_step_action(&action);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_step_action_run_command() {
        let action = StepAction::RunCommand {
            command: "cargo".to_string(),
            args: vec!["test".to_string()],
        };
        let result = ExecutionValidator::validate_step_action(&action);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_step_action_run_tests() {
        let action = StepAction::RunTests {
            pattern: Some("test_*".to_string()),
        };
        let result = ExecutionValidator::validate_step_action(&action);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_step_valid() {
        let step = ExecutionStep::new(
            "Create a new file".to_string(),
            StepAction::CreateFile {
                path: "src/lib.rs".to_string(),
                content: "pub fn hello() {}".to_string(),
            },
        );
        let result = ExecutionValidator::validate_step(&step);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_step_empty_description() {
        let mut step = ExecutionStep::new(
            "".to_string(),
            StepAction::CreateFile {
                path: "src/lib.rs".to_string(),
                content: "pub fn hello() {}".to_string(),
            },
        );
        step.description = "".to_string();
        let result = ExecutionValidator::validate_step(&step);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_plan_empty_steps() {
        let plan = ExecutionPlan::new("Test Plan".to_string(), vec![]);
        let result = ExecutionValidator::validate_plan(&plan);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least one step"));
    }

    #[test]
    fn test_validate_plan_valid() {
        let step = ExecutionStep::new(
            "Create a new file".to_string(),
            StepAction::CreateFile {
                path: "src/lib.rs".to_string(),
                content: "pub fn hello() {}".to_string(),
            },
        );
        let plan = ExecutionPlan::new("Test Plan".to_string(), vec![step]);
        let result = ExecutionValidator::validate_plan(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_plan_invalid_dependency() {
        let mut step1 = ExecutionStep::new(
            "Create a new file".to_string(),
            StepAction::CreateFile {
                path: "src/lib.rs".to_string(),
                content: "pub fn hello() {}".to_string(),
            },
        );

        let step2 = ExecutionStep::new(
            "Modify the file".to_string(),
            StepAction::ModifyFile {
                path: "src/lib.rs".to_string(),
                diff: "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-old\n+new".to_string(),
            },
        );

        // Add invalid dependency
        step1.dependencies.push("non-existent-id".to_string());

        let plan = ExecutionPlan::new("Test Plan".to_string(), vec![step1, step2]);
        let result = ExecutionValidator::validate_plan(&plan);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("non-existent dependency"));
    }

    #[test]
    fn test_validate_plan_self_referential_dependency() {
        let mut step = ExecutionStep::new(
            "Create a new file".to_string(),
            StepAction::CreateFile {
                path: "src/lib.rs".to_string(),
                content: "pub fn hello() {}".to_string(),
            },
        );

        // Add self-referential dependency
        step.dependencies.push(step.id.clone());

        let plan = ExecutionPlan::new("Test Plan".to_string(), vec![step]);
        let result = ExecutionValidator::validate_plan(&plan);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("self-referential"));
    }

    #[test]
    fn test_validate_plan_circular_dependency() {
        let mut step1 = ExecutionStep::new(
            "Step 1".to_string(),
            StepAction::CreateFile {
                path: "src/lib.rs".to_string(),
                content: "pub fn hello() {}".to_string(),
            },
        );

        let mut step2 = ExecutionStep::new(
            "Step 2".to_string(),
            StepAction::CreateFile {
                path: "src/main.rs".to_string(),
                content: "fn main() {}".to_string(),
            },
        );

        let step1_id = step1.id.clone();
        let step2_id = step2.id.clone();

        // Create circular dependency: step1 -> step2 -> step1
        step1.dependencies.push(step2_id.clone());
        step2.dependencies.push(step1_id);

        let plan = ExecutionPlan::new("Test Plan".to_string(), vec![step1, step2]);
        let result = ExecutionValidator::validate_plan(&plan);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
    }
}
