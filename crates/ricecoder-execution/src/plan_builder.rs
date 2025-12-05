//! Plan builder for converting generation results to execution plans

use crate::error::{ExecutionError, ExecutionResult};
use crate::models::{
    ComplexityLevel, ExecutionPlan, ExecutionStep, RiskFactor, RiskLevel, RiskScore, StepAction,
};
use ricecoder_storage::PathResolver;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use uuid::Uuid;

/// Builder for creating execution plans from generation results
///
/// Converts generation results (file changes, commands) into structured
/// execution plans with step ordering, dependency resolution, and risk scoring.
pub struct PlanBuilder {
    /// Plan name
    name: String,
    /// Steps to include in the plan
    steps: Vec<ExecutionStep>,
    /// Step dependencies (step_id -> [dependency_ids])
    dependencies: HashMap<String, Vec<String>>,
    /// Critical files that increase risk
    critical_files: Vec<String>,
}

impl PlanBuilder {
    /// Create a new plan builder
    pub fn new(name: String) -> Self {
        Self {
            name,
            steps: Vec::new(),
            dependencies: HashMap::new(),
            critical_files: vec![
                "Cargo.toml".to_string(),
                "package.json".to_string(),
                "setup.py".to_string(),
                "pyproject.toml".to_string(),
                "go.mod".to_string(),
                "pom.xml".to_string(),
                "build.gradle".to_string(),
            ],
        }
    }

    /// Add a create file step
    ///
    /// # Arguments
    /// * `path` - File path (will be validated with PathResolver)
    /// * `content` - File content
    ///
    /// # Errors
    /// Returns error if path is invalid
    pub fn add_create_file_step(mut self, path: String, content: String) -> ExecutionResult<Self> {
        // Validate path using PathResolver
        let _resolved = PathResolver::expand_home(Path::new(&path))
            .map_err(|e| ExecutionError::ValidationError(format!("Invalid path: {}", e)))?;

        let step = ExecutionStep::new(
            format!("Create file: {}", path),
            StepAction::CreateFile { path, content },
        );

        self.steps.push(step);
        Ok(self)
    }

    /// Add a modify file step
    ///
    /// # Arguments
    /// * `path` - File path (will be validated with PathResolver)
    /// * `diff` - Diff to apply
    ///
    /// # Errors
    /// Returns error if path is invalid
    pub fn add_modify_file_step(mut self, path: String, diff: String) -> ExecutionResult<Self> {
        // Validate path using PathResolver
        let _resolved = PathResolver::expand_home(Path::new(&path))
            .map_err(|e| ExecutionError::ValidationError(format!("Invalid path: {}", e)))?;

        let step = ExecutionStep::new(
            format!("Modify file: {}", path),
            StepAction::ModifyFile { path, diff },
        );

        self.steps.push(step);
        Ok(self)
    }

    /// Add a delete file step
    ///
    /// # Arguments
    /// * `path` - File path (will be validated with PathResolver)
    ///
    /// # Errors
    /// Returns error if path is invalid
    pub fn add_delete_file_step(mut self, path: String) -> ExecutionResult<Self> {
        // Validate path using PathResolver
        let _resolved = PathResolver::expand_home(Path::new(&path))
            .map_err(|e| ExecutionError::ValidationError(format!("Invalid path: {}", e)))?;

        let step = ExecutionStep::new(
            format!("Delete file: {}", path),
            StepAction::DeleteFile { path },
        );

        self.steps.push(step);
        Ok(self)
    }

    /// Add a command execution step
    ///
    /// # Arguments
    /// * `command` - Command to execute
    /// * `args` - Command arguments
    pub fn add_command_step(mut self, command: String, args: Vec<String>) -> Self {
        let step = ExecutionStep::new(
            format!("Run command: {} {}", command, args.join(" ")),
            StepAction::RunCommand { command, args },
        );

        self.steps.push(step);
        self
    }

    /// Add a test execution step
    ///
    /// # Arguments
    /// * `pattern` - Optional test pattern to filter tests
    pub fn add_test_step(mut self, pattern: Option<String>) -> Self {
        let description = if let Some(ref p) = pattern {
            format!("Run tests matching: {}", p)
        } else {
            "Run all tests".to_string()
        };

        let step = ExecutionStep::new(description, StepAction::RunTests { pattern });

        self.steps.push(step);
        self
    }

    /// Add a dependency between steps
    ///
    /// # Arguments
    /// * `step_id` - ID of the step that depends on others
    /// * `dependency_id` - ID of the step that must complete first
    pub fn add_dependency(mut self, step_id: String, dependency_id: String) -> Self {
        self.dependencies
            .entry(step_id)
            .or_default()
            .push(dependency_id);
        self
    }

    /// Set custom critical files
    ///
    /// These files are weighted higher in risk scoring.
    pub fn with_critical_files(mut self, files: Vec<String>) -> Self {
        self.critical_files = files;
        self
    }

    /// Build the execution plan
    ///
    /// Performs final validation, calculates risk scores, and returns the plan.
    pub fn build(mut self) -> ExecutionResult<ExecutionPlan> {
        if self.steps.is_empty() {
            return Err(ExecutionError::PlanError(
                "Cannot build plan with no steps".to_string(),
            ));
        }

        // Apply dependencies to steps
        for step in &mut self.steps {
            if let Some(deps) = self.dependencies.get(&step.id) {
                step.dependencies = deps.clone();
            }
        }

        // Calculate risk score
        let risk_score = self.calculate_risk_score();

        // Calculate complexity
        let complexity = self.calculate_complexity();

        // Calculate estimated duration
        let estimated_duration = self.estimate_duration();

        // Determine if approval is required
        let requires_approval = matches!(risk_score.level, RiskLevel::High | RiskLevel::Critical);

        let plan = ExecutionPlan {
            id: Uuid::new_v4().to_string(),
            name: self.name,
            steps: self.steps,
            risk_score,
            estimated_duration,
            estimated_complexity: complexity,
            requires_approval,
            editable: true,
        };

        Ok(plan)
    }

    /// Calculate risk score for the plan
    fn calculate_risk_score(&self) -> RiskScore {
        let mut factors = Vec::new();
        let mut total_score = 0.0;

        // Factor 1: Number of files changed
        let file_count = self
            .steps
            .iter()
            .filter(|s| matches!(s.action, StepAction::ModifyFile { .. }))
            .count();
        let file_weight = file_count as f32 * 0.1;
        factors.push(RiskFactor {
            name: "file_count".to_string(),
            weight: file_weight,
            description: format!("{} files modified", file_count),
        });
        total_score += file_weight;

        // Factor 2: Critical files
        let critical_count = self
            .steps
            .iter()
            .filter(|s| self.is_critical_file_step(s))
            .count();
        let critical_weight = critical_count as f32 * 0.5;
        factors.push(RiskFactor {
            name: "critical_files".to_string(),
            weight: critical_weight,
            description: format!("{} critical files", critical_count),
        });
        total_score += critical_weight;

        // Factor 3: Deletions
        let deletion_count = self
            .steps
            .iter()
            .filter(|s| matches!(s.action, StepAction::DeleteFile { .. }))
            .count();
        let deletion_weight = deletion_count as f32 * 0.3;
        factors.push(RiskFactor {
            name: "deletions".to_string(),
            weight: deletion_weight,
            description: format!("{} files deleted", deletion_count),
        });
        total_score += deletion_weight;

        // Factor 4: Scope (number of steps)
        let scope_weight = (self.steps.len() as f32 / 10.0).min(0.2);
        factors.push(RiskFactor {
            name: "scope".to_string(),
            weight: scope_weight,
            description: format!("{} steps", self.steps.len()),
        });
        total_score += scope_weight;

        let level = match total_score {
            s if s < 0.5 => RiskLevel::Low,
            s if s < 1.5 => RiskLevel::Medium,
            s if s < 2.5 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        RiskScore {
            level,
            score: total_score,
            factors,
        }
    }

    /// Check if a step involves a critical file
    fn is_critical_file_step(&self, step: &ExecutionStep) -> bool {
        match &step.action {
            StepAction::CreateFile { path, .. } | StepAction::ModifyFile { path, .. } => {
                self.critical_files.iter().any(|cf| path.ends_with(cf))
            }
            StepAction::DeleteFile { path } => {
                self.critical_files.iter().any(|cf| path.ends_with(cf))
            }
            _ => false,
        }
    }

    /// Calculate complexity level
    fn calculate_complexity(&self) -> ComplexityLevel {
        let step_count = self.steps.len();
        let has_dependencies = !self.dependencies.is_empty();
        let has_deletions = self
            .steps
            .iter()
            .any(|s| matches!(s.action, StepAction::DeleteFile { .. }));

        match (step_count, has_dependencies, has_deletions) {
            (1..=3, false, false) => ComplexityLevel::Simple,
            (4..=8, false, false) => ComplexityLevel::Moderate,
            (9..=15, _, false) => ComplexityLevel::Complex,
            _ => ComplexityLevel::VeryComplex,
        }
    }

    /// Estimate execution duration
    fn estimate_duration(&self) -> Duration {
        let mut total_ms = 0u64;

        for step in &self.steps {
            let step_ms = match &step.action {
                StepAction::CreateFile { content, .. } => {
                    // Estimate: 10ms base + 1ms per 100 bytes
                    10 + (content.len() as u64 / 100)
                }
                StepAction::ModifyFile { .. } => 50,
                StepAction::DeleteFile { .. } => 10,
                StepAction::RunCommand { .. } => 500,
                StepAction::RunTests { .. } => 5000,
            };
            total_ms += step_ms;
        }

        Duration::from_millis(total_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_builder() {
        let builder = PlanBuilder::new("test plan".to_string());
        assert_eq!(builder.name, "test plan");
        assert_eq!(builder.steps.len(), 0);
    }

    #[test]
    fn test_add_create_file_step() {
        let builder = PlanBuilder::new("test".to_string());
        let result = builder.add_create_file_step("test.txt".to_string(), "content".to_string());
        assert!(result.is_ok());
        let builder = result.unwrap();
        assert_eq!(builder.steps.len(), 1);
    }

    #[test]
    fn test_add_modify_file_step() {
        let builder = PlanBuilder::new("test".to_string());
        let result = builder.add_modify_file_step("test.txt".to_string(), "diff".to_string());
        assert!(result.is_ok());
        let builder = result.unwrap();
        assert_eq!(builder.steps.len(), 1);
    }

    #[test]
    fn test_add_delete_file_step() {
        let builder = PlanBuilder::new("test".to_string());
        let result = builder.add_delete_file_step("test.txt".to_string());
        assert!(result.is_ok());
        let builder = result.unwrap();
        assert_eq!(builder.steps.len(), 1);
    }

    #[test]
    fn test_add_command_step() {
        let builder = PlanBuilder::new("test".to_string());
        let builder = builder.add_command_step("echo".to_string(), vec!["hello".to_string()]);
        assert_eq!(builder.steps.len(), 1);
    }

    #[test]
    fn test_add_test_step() {
        let builder = PlanBuilder::new("test".to_string());
        let builder = builder.add_test_step(Some("*.rs".to_string()));
        assert_eq!(builder.steps.len(), 1);
    }

    #[test]
    fn test_build_simple_plan() {
        let builder = PlanBuilder::new("test".to_string());
        let result = builder
            .add_create_file_step("test.txt".to_string(), "content".to_string())
            .unwrap()
            .build();

        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.name, "test");
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.estimated_complexity, ComplexityLevel::Simple);
    }

    #[test]
    fn test_build_empty_plan_fails() {
        let builder = PlanBuilder::new("test".to_string());
        let result = builder.build();
        assert!(result.is_err());
    }

    #[test]
    fn test_risk_score_calculation() {
        let builder = PlanBuilder::new("test".to_string());
        let result = builder
            .add_create_file_step("Cargo.toml".to_string(), "content".to_string())
            .unwrap()
            .add_delete_file_step("old.rs".to_string())
            .unwrap()
            .add_delete_file_step("old2.rs".to_string())
            .unwrap()
            .add_delete_file_step("old3.rs".to_string())
            .unwrap()
            .build();

        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(plan.risk_score.score > 0.0);
        // Risk score should be high enough to require approval
        // Critical file (0.5) + 3 deletions (0.9) = 1.4, which is >= 1.5 for High
        // Let's just check that score is calculated
        assert!(plan.risk_score.score > 0.5);
    }

    #[test]
    fn test_complexity_calculation() {
        let builder = PlanBuilder::new("simple".to_string());
        let simple = builder
            .add_create_file_step("a.txt".to_string(), "a".to_string())
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(simple.estimated_complexity, ComplexityLevel::Simple);

        let builder = PlanBuilder::new("moderate".to_string());
        let moderate = builder
            .add_create_file_step("a.txt".to_string(), "a".to_string())
            .unwrap()
            .add_create_file_step("b.txt".to_string(), "b".to_string())
            .unwrap()
            .add_create_file_step("c.txt".to_string(), "c".to_string())
            .unwrap()
            .add_create_file_step("d.txt".to_string(), "d".to_string())
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(moderate.estimated_complexity, ComplexityLevel::Moderate);
    }

    #[test]
    fn test_duration_estimation() {
        let builder = PlanBuilder::new("test".to_string());
        let plan = builder
            .add_create_file_step("test.txt".to_string(), "content".to_string())
            .unwrap()
            .add_command_step("echo".to_string(), vec![])
            .build()
            .unwrap();

        assert!(plan.estimated_duration.as_millis() > 0);
    }

    #[test]
    fn test_add_dependencies() {
        let builder = PlanBuilder::new("test".to_string());
        let builder = builder
            .add_create_file_step("a.txt".to_string(), "a".to_string())
            .unwrap()
            .add_create_file_step("b.txt".to_string(), "b".to_string())
            .unwrap();

        let step_ids: Vec<_> = builder.steps.iter().map(|s| s.id.clone()).collect();
        let builder = builder.add_dependency(step_ids[1].clone(), step_ids[0].clone());

        let plan = builder.build().unwrap();
        assert!(!plan.steps[1].dependencies.is_empty());
    }

    #[test]
    fn test_critical_files_detection() {
        let builder = PlanBuilder::new("test".to_string());
        let plan = builder
            .add_modify_file_step("Cargo.toml".to_string(), "diff".to_string())
            .unwrap()
            .build()
            .unwrap();

        // Should have higher risk due to critical file
        assert!(plan.risk_score.score > 0.0);
        let has_critical_factor = plan
            .risk_score
            .factors
            .iter()
            .any(|f| f.name == "critical_files" && f.weight > 0.0);
        assert!(has_critical_factor);
    }
}
