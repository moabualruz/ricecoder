//! Safety constraints for high-risk workflow operations

use crate::models::{SafetyViolation, WorkflowStep};
use std::time::Duration;

/// Safety constraints for high-risk operations
#[derive(Debug, Clone)]
pub struct SafetyConstraints {
    /// Maximum execution timeout for high-risk operations (milliseconds)
    pub max_timeout_ms: u64,
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Maximum CPU percentage (0-100)
    pub max_cpu_percent: u8,
    /// Maximum file handles
    pub max_file_handles: u32,
}

impl SafetyConstraints {
    /// Create default safety constraints
    pub fn new() -> Self {
        Self {
            max_timeout_ms: 300_000, // 5 minutes
            max_memory_mb: 1024,     // 1 GB
            max_cpu_percent: 80,
            max_file_handles: 1024,
        }
    }

    /// Create safety constraints with custom timeout
    pub fn with_timeout(timeout_ms: u64) -> Self {
        Self {
            max_timeout_ms: timeout_ms,
            ..Self::new()
        }
    }

    /// Apply safety constraints to a high-risk step
    pub fn apply_to_step(&self, step: &WorkflowStep, _risk_score: u8) -> Vec<SafetyViolation> {
        let mut violations = Vec::new();

        // Check if step has timeout configured
        if let crate::models::StepType::Command(cmd_step) = &step.step_type {
            if cmd_step.timeout > self.max_timeout_ms {
                violations.push(SafetyViolation {
                    step_id: step.id.clone(),
                    violation_type: "timeout_exceeded".to_string(),
                    description: format!(
                        "Step timeout {} ms exceeds maximum {} ms",
                        cmd_step.timeout, self.max_timeout_ms
                    ),
                });
            }
        }

        violations
    }

    /// Enforce timeout on a high-risk operation
    pub fn enforce_timeout(&self, step: &WorkflowStep) -> Duration {
        match &step.step_type {
            crate::models::StepType::Command(cmd_step) => {
                let timeout_ms = cmd_step.timeout.min(self.max_timeout_ms);
                Duration::from_millis(timeout_ms)
            }
            _ => Duration::from_millis(self.max_timeout_ms),
        }
    }

    /// Check if rollback capability is maintained
    pub fn has_rollback_capability(&self, step: &WorkflowStep) -> bool {
        // Rollback is maintained if the step has rollback error action
        matches!(step.on_error, crate::models::ErrorAction::Rollback)
    }
}

impl Default for SafetyConstraints {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CommandStep, ErrorAction, RiskFactors, StepConfig, StepType};

    fn create_command_step(id: &str, timeout: u64) -> WorkflowStep {
        WorkflowStep {
            id: id.to_string(),
            name: format!("Step {}", id),
            step_type: StepType::Command(CommandStep {
                command: "test".to_string(),
                args: vec![],
                timeout,
            }),
            config: StepConfig {
                config: serde_json::json!({}),
            },
            dependencies: Vec::new(),
            approval_required: false,
            on_error: ErrorAction::Rollback,
            risk_score: None,
            risk_factors: RiskFactors::default(),
        }
    }

    #[test]
    fn test_safety_constraints_default() {
        let constraints = SafetyConstraints::new();
        assert_eq!(constraints.max_timeout_ms, 300_000);
        assert_eq!(constraints.max_memory_mb, 1024);
        assert_eq!(constraints.max_cpu_percent, 80);
    }

    #[test]
    fn test_enforce_timeout_within_limit() {
        let constraints = SafetyConstraints::new();
        let step = create_command_step("1", 100_000);
        let timeout = constraints.enforce_timeout(&step);
        assert_eq!(timeout.as_millis(), 100_000);
    }

    #[test]
    fn test_enforce_timeout_exceeds_limit() {
        let constraints = SafetyConstraints::new();
        let step = create_command_step("1", 500_000);
        let timeout = constraints.enforce_timeout(&step);
        assert_eq!(timeout.as_millis(), 300_000);
    }

    #[test]
    fn test_timeout_violation_detection() {
        let constraints = SafetyConstraints::new();
        let step = create_command_step("1", 500_000);
        let violations = constraints.apply_to_step(&step, 80);
        assert!(!violations.is_empty());
        assert_eq!(violations[0].violation_type, "timeout_exceeded");
    }

    #[test]
    fn test_rollback_capability() {
        let constraints = SafetyConstraints::new();
        let step = create_command_step("1", 100_000);
        assert!(constraints.has_rollback_capability(&step));
    }
}
