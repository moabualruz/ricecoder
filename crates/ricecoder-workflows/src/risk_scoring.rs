//! Risk scoring and assessment for workflow steps

use crate::models::{RiskAssessment, RiskAssessmentReport, WorkflowStep, WorkflowState, ApprovalDecisionRecord};
use chrono::Utc;

/// Risk score calculator for workflow steps
#[derive(Debug, Clone)]
pub struct RiskScorer {
    /// Threshold above which approval is required (0-100)
    pub approval_threshold: u8,
    /// Maximum execution timeout for high-risk operations (milliseconds)
    pub max_timeout_ms: u64,
}

impl RiskScorer {
    /// Create a new risk scorer with default settings
    pub fn new() -> Self {
        Self {
            approval_threshold: 70,
            max_timeout_ms: 300_000, // 5 minutes
        }
    }

    /// Create a new risk scorer with custom settings
    pub fn with_threshold(approval_threshold: u8) -> Self {
        Self {
            approval_threshold,
            max_timeout_ms: 300_000,
        }
    }

    /// Calculate risk score for a workflow step
    ///
    /// The risk score is calculated based on three factors:
    /// - Impact (0-100): potential for data loss or system damage
    /// - Reversibility (0-100): ability to undo the operation (inverted: lower is riskier)
    /// - Complexity (0-100): number of dependencies and interactions
    ///
    /// Formula: (impact + (100 - reversibility) + complexity) / 3
    pub fn calculate_risk_score(&self, step: &WorkflowStep) -> u8 {
        let factors = &step.risk_factors;

        // Validate inputs are in range [0, 100]
        let impact = factors.impact.min(100);
        let reversibility = factors.reversibility.min(100);
        let complexity = factors.complexity.min(100);

        // Calculate weighted score
        // Impact: 40% weight
        // Reversibility: 40% weight (inverted: lower reversibility = higher risk)
        // Complexity: 20% weight
        let impact_contribution = (impact as f32) * 0.4;
        let reversibility_contribution = ((100 - reversibility as u16) as f32) * 0.4;
        let complexity_contribution = (complexity as f32) * 0.2;

        let score = (impact_contribution + reversibility_contribution + complexity_contribution) as u8;
        score.min(100)
    }

    /// Check if a step requires approval based on risk score
    pub fn requires_approval(&self, risk_score: u8) -> bool {
        risk_score > self.approval_threshold
    }

    /// Generate a risk assessment for a step
    pub fn assess_step(&self, step: &WorkflowStep) -> RiskAssessment {
        let risk_score = self.calculate_risk_score(step);
        let approval_required = self.requires_approval(risk_score);

        RiskAssessment {
            step_id: step.id.clone(),
            step_name: step.name.clone(),
            risk_score,
            risk_factors: step.risk_factors.clone(),
            approval_required,
            approval_decision: None,
        }
    }

    /// Generate a risk assessment report for a completed workflow
    pub fn generate_report(
        &self,
        workflow_id: &str,
        workflow_name: &str,
        steps: &[WorkflowStep],
        state: &WorkflowState,
    ) -> RiskAssessmentReport {
        let mut step_assessments = Vec::new();
        let mut total_score = 0u32;

        for step in steps {
            let mut assessment = self.assess_step(step);

            // Check if step was executed and has approval decision
            if state.step_results.contains_key(&step.id) {
                // In a real implementation, we would look up the actual approval decision
                // For now, we mark it as approved if the step completed
                if let Some(result) = state.step_results.get(&step.id) {
                    if result.status == crate::models::StepStatus::Completed && assessment.approval_required {
                        assessment.approval_decision = Some(ApprovalDecisionRecord {
                            approved: true,
                            timestamp: Utc::now(),
                            approver: None,
                            comments: None,
                        });
                    }
                }
            }

            total_score += assessment.risk_score as u32;
            step_assessments.push(assessment);
        }

        let overall_risk_score = if step_assessments.is_empty() {
            0
        } else {
            (total_score / step_assessments.len() as u32) as u8
        };

        RiskAssessmentReport {
            workflow_id: workflow_id.to_string(),
            workflow_name: workflow_name.to_string(),
            overall_risk_score,
            step_assessments,
            safety_violations: Vec::new(),
            generated_at: Utc::now(),
        }
    }
}

impl Default for RiskScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{StepConfig, StepType, ErrorAction, AgentStep, RiskFactors};

    fn create_test_step(id: &str, impact: u8, reversibility: u8, complexity: u8) -> WorkflowStep {
        WorkflowStep {
            id: id.to_string(),
            name: format!("Step {}", id),
            step_type: StepType::Agent(AgentStep {
                agent_id: "test-agent".to_string(),
                task: "test-task".to_string(),
            }),
            config: StepConfig {
                config: serde_json::json!({}),
            },
            dependencies: Vec::new(),
            approval_required: false,
            on_error: ErrorAction::Fail,
            risk_score: None,
            risk_factors: RiskFactors {
                impact,
                reversibility,
                complexity,
            },
        }
    }

    #[test]
    fn test_risk_score_calculation_low_risk() {
        let scorer = RiskScorer::new();
        let step = create_test_step("1", 10, 90, 10);
        let score = scorer.calculate_risk_score(&step);
        assert!(score < 30, "Low risk step should have score < 30, got {}", score);
    }

    #[test]
    fn test_risk_score_calculation_high_risk() {
        let scorer = RiskScorer::new();
        let step = create_test_step("1", 90, 10, 90);
        let score = scorer.calculate_risk_score(&step);
        assert!(score > 70, "High risk step should have score > 70, got {}", score);
    }

    #[test]
    fn test_risk_score_in_range() {
        let scorer = RiskScorer::new();
        for impact in [0, 25, 50, 75, 100] {
            for reversibility in [0, 25, 50, 75, 100] {
                for complexity in [0, 25, 50, 75, 100] {
                    let step = create_test_step("1", impact, reversibility, complexity);
                    let score = scorer.calculate_risk_score(&step);
                    assert!(score <= 100, "Risk score should be <= 100, got {}", score);
                }
            }
        }
    }

    #[test]
    fn test_approval_threshold() {
        let scorer = RiskScorer::with_threshold(70);
        assert!(!scorer.requires_approval(69));
        assert!(!scorer.requires_approval(70)); // 70 is not > 70, so no approval required
        assert!(scorer.requires_approval(71));  // 71 > 70, so approval required
    }

    #[test]
    fn test_risk_assessment() {
        let scorer = RiskScorer::new();
        let step = create_test_step("1", 80, 20, 80);
        let assessment = scorer.assess_step(&step);
        assert_eq!(assessment.step_id, "1");
        assert!(assessment.approval_required);
        assert!(assessment.risk_score > 70);
    }
}
