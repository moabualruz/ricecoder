//! Risk scoring for execution plans
//!
//! Calculates risk scores for execution plans based on:
//! - Number of files changed
//! - Critical files (Cargo.toml, package.json, etc.)
//! - File deletions
//! - Overall scope (number of steps)

use crate::models::{ExecutionPlan, ExecutionStep, RiskFactor, RiskLevel, RiskScore, StepAction};
use std::collections::HashSet;

/// Risk scorer for execution plans
#[derive(Debug, Clone)]
pub struct ExecutionRiskScorer {
    /// Critical file patterns that require higher risk weighting
    critical_files: HashSet<String>,
    /// Approval threshold (0.0 to 1.0+)
    approval_threshold: f32,
}

impl ExecutionRiskScorer {
    /// Create a new risk scorer with default critical files
    pub fn new() -> Self {
        let mut critical_files = HashSet::new();
        // Rust
        critical_files.insert("Cargo.toml".to_string());
        critical_files.insert("Cargo.lock".to_string());
        // Node.js
        critical_files.insert("package.json".to_string());
        critical_files.insert("package-lock.json".to_string());
        critical_files.insert("yarn.lock".to_string());
        // Python
        critical_files.insert("setup.py".to_string());
        critical_files.insert("requirements.txt".to_string());
        critical_files.insert("pyproject.toml".to_string());
        // Go
        critical_files.insert("go.mod".to_string());
        critical_files.insert("go.sum".to_string());
        // Configuration
        critical_files.insert(".env".to_string());
        critical_files.insert(".env.production".to_string());
        critical_files.insert("config.yaml".to_string());
        critical_files.insert("config.yml".to_string());
        // Build/CI
        critical_files.insert("Makefile".to_string());
        critical_files.insert(".github/workflows".to_string());
        critical_files.insert(".gitlab-ci.yml".to_string());

        Self {
            critical_files,
            approval_threshold: 1.5, // High risk threshold
        }
    }

    /// Create a new risk scorer with custom critical files
    pub fn with_critical_files(critical_files: HashSet<String>) -> Self {
        Self {
            critical_files,
            approval_threshold: 1.5,
        }
    }

    /// Set the approval threshold
    pub fn with_approval_threshold(mut self, threshold: f32) -> Self {
        self.approval_threshold = threshold;
        self
    }

    /// Add a critical file pattern
    pub fn add_critical_file(&mut self, pattern: String) {
        self.critical_files.insert(pattern);
    }

    /// Check if a file path matches a critical file pattern
    fn is_critical_file(&self, path: &str) -> bool {
        // Check exact matches
        if self.critical_files.contains(path) {
            return true;
        }

        // Check filename only
        if let Some(filename) = path.split('/').next_back() {
            if self.critical_files.contains(filename) {
                return true;
            }
        }

        // Check directory patterns
        for pattern in &self.critical_files {
            if path.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Calculate risk score for an execution plan
    ///
    /// **Feature: ricecoder-execution, Property 1: Risk Score Consistency**
    /// **Validates: Requirements 1.1, 1.2**
    pub fn score_plan(&self, plan: &ExecutionPlan) -> RiskScore {
        let mut factors = Vec::new();
        let mut total_score = 0.0;

        // Factor 1: Number of files changed
        let file_count = plan
            .steps
            .iter()
            .filter(|s| matches!(s.action, StepAction::ModifyFile { .. }))
            .count();
        let file_count_weight = file_count as f32 * 0.1;
        factors.push(RiskFactor {
            name: "file_count".to_string(),
            weight: file_count_weight,
            description: format!("{} files modified", file_count),
        });
        total_score += file_count_weight;

        // Factor 2: Critical files
        let critical_files_count = plan
            .steps
            .iter()
            .filter(|s| self.is_critical_file_in_step(s))
            .count();
        let critical_files_weight = critical_files_count as f32 * 0.5;
        factors.push(RiskFactor {
            name: "critical_files".to_string(),
            weight: critical_files_weight,
            description: format!("{} critical files", critical_files_count),
        });
        total_score += critical_files_weight;

        // Factor 3: Deletions
        let deletions = plan
            .steps
            .iter()
            .filter(|s| matches!(s.action, StepAction::DeleteFile { .. }))
            .count();
        let deletions_weight = deletions as f32 * 0.3;
        factors.push(RiskFactor {
            name: "deletions".to_string(),
            weight: deletions_weight,
            description: format!("{} files deleted", deletions),
        });
        total_score += deletions_weight;

        // Factor 4: Scope (number of steps)
        let scope_weight = (plan.steps.len() as f32 / 10.0).min(0.2);
        factors.push(RiskFactor {
            name: "scope".to_string(),
            weight: scope_weight,
            description: format!("{} steps", plan.steps.len()),
        });
        total_score += scope_weight;

        let level = self.level_from_score(total_score);

        RiskScore {
            level,
            score: total_score,
            factors,
        }
    }

    /// Determine risk level from score
    fn level_from_score(&self, score: f32) -> RiskLevel {
        match score {
            s if s < 0.5 => RiskLevel::Low,
            s if s < 1.5 => RiskLevel::Medium,
            s if s < 2.5 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }

    /// Check if approval is required based on risk score
    pub fn requires_approval(&self, risk_score: &RiskScore) -> bool {
        risk_score.score > self.approval_threshold
    }

    /// Check if a step contains a critical file
    fn is_critical_file_in_step(&self, step: &ExecutionStep) -> bool {
        match &step.action {
            StepAction::CreateFile { path, .. } => self.is_critical_file(path),
            StepAction::ModifyFile { path, .. } => self.is_critical_file(path),
            StepAction::DeleteFile { path } => self.is_critical_file(path),
            _ => false,
        }
    }
}

impl Default for ExecutionRiskScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ComplexityLevel, ExecutionStep, StepStatus};
    use uuid::Uuid;

    fn create_test_plan(steps: Vec<ExecutionStep>) -> ExecutionPlan {
        ExecutionPlan {
            id: Uuid::new_v4().to_string(),
            name: "Test Plan".to_string(),
            steps,
            risk_score: RiskScore::default(),
            estimated_duration: std::time::Duration::from_secs(0),
            estimated_complexity: ComplexityLevel::Simple,
            requires_approval: false,
            editable: true,
        }
    }

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

    #[test]
    fn test_empty_plan_low_risk() {
        let scorer = ExecutionRiskScorer::new();
        let plan = create_test_plan(vec![]);
        let score = scorer.score_plan(&plan);
        assert_eq!(score.level, RiskLevel::Low);
        assert_eq!(score.score, 0.0);
    }

    #[test]
    fn test_single_file_modification_low_risk() {
        let scorer = ExecutionRiskScorer::new();
        let step = create_test_step(
            "Modify file",
            StepAction::ModifyFile {
                path: "src/main.rs".to_string(),
                diff: "".to_string(),
            },
        );
        let plan = create_test_plan(vec![step]);
        let score = scorer.score_plan(&plan);
        assert_eq!(score.level, RiskLevel::Low);
    }

    #[test]
    fn test_critical_file_modification_high_risk() {
        let scorer = ExecutionRiskScorer::new();
        let step = create_test_step(
            "Modify Cargo.toml",
            StepAction::ModifyFile {
                path: "Cargo.toml".to_string(),
                diff: "".to_string(),
            },
        );
        let plan = create_test_plan(vec![step]);
        let score = scorer.score_plan(&plan);
        assert!(
            score.score > 0.4,
            "Critical file should increase risk score"
        );
    }

    #[test]
    fn test_file_deletion_increases_risk() {
        let scorer = ExecutionRiskScorer::new();
        let step = create_test_step(
            "Delete file",
            StepAction::DeleteFile {
                path: "src/old.rs".to_string(),
            },
        );
        let plan = create_test_plan(vec![step]);
        let score = scorer.score_plan(&plan);
        assert!(
            score.score > 0.2,
            "File deletion should increase risk score"
        );
    }

    #[test]
    fn test_multiple_files_increase_risk() {
        let scorer = ExecutionRiskScorer::new();
        let steps = vec![
            create_test_step(
                "Modify file 1",
                StepAction::ModifyFile {
                    path: "src/a.rs".to_string(),
                    diff: "".to_string(),
                },
            ),
            create_test_step(
                "Modify file 2",
                StepAction::ModifyFile {
                    path: "src/b.rs".to_string(),
                    diff: "".to_string(),
                },
            ),
            create_test_step(
                "Modify file 3",
                StepAction::ModifyFile {
                    path: "src/c.rs".to_string(),
                    diff: "".to_string(),
                },
            ),
        ];
        let plan = create_test_plan(steps);
        let score = scorer.score_plan(&plan);
        assert!(
            score.score > 0.2,
            "Multiple files should increase risk score"
        );
    }

    #[test]
    fn test_approval_threshold() {
        let scorer = ExecutionRiskScorer::new().with_approval_threshold(0.4);
        let step = create_test_step(
            "Modify Cargo.toml",
            StepAction::ModifyFile {
                path: "Cargo.toml".to_string(),
                diff: "".to_string(),
            },
        );
        let plan = create_test_plan(vec![step]);
        let score = scorer.score_plan(&plan);
        assert!(
            scorer.requires_approval(&score),
            "Score {} should require approval with threshold 0.4",
            score.score
        );
    }

    #[test]
    fn test_custom_critical_files() {
        let mut critical_files = HashSet::new();
        critical_files.insert("custom.conf".to_string());
        let scorer = ExecutionRiskScorer::with_critical_files(critical_files);

        let step = create_test_step(
            "Modify custom config",
            StepAction::ModifyFile {
                path: "custom.conf".to_string(),
                diff: "".to_string(),
            },
        );
        let plan = create_test_plan(vec![step]);
        let score = scorer.score_plan(&plan);
        assert!(
            score.score > 0.4,
            "Custom critical file should increase risk"
        );
    }

    #[test]
    fn test_risk_score_consistency() {
        // **Feature: ricecoder-execution, Property 1: Risk Score Consistency**
        // **Validates: Requirements 1.1, 1.2**
        let scorer = ExecutionRiskScorer::new();
        let step = create_test_step(
            "Modify Cargo.toml",
            StepAction::ModifyFile {
                path: "Cargo.toml".to_string(),
                diff: "".to_string(),
            },
        );
        let plan = create_test_plan(vec![step]);

        // Score the same plan multiple times
        let score1 = scorer.score_plan(&plan);
        let score2 = scorer.score_plan(&plan);
        let score3 = scorer.score_plan(&plan);

        // All scores should be identical
        assert_eq!(score1.score, score2.score);
        assert_eq!(score2.score, score3.score);
        assert_eq!(score1.level, score2.level);
        assert_eq!(score2.level, score3.level);
    }
}
