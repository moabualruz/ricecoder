//! Spec processing for code generation
//!
//! Processes specifications into generation plans by extracting requirements,
//! acceptance criteria, and constraints, then mapping them to generation tasks.

use crate::error::GenerationError;
use ricecoder_specs::models::{AcceptanceCriterion, Priority, Requirement, Spec};
use std::collections::BTreeMap;

/// Processes specifications into generation plans
#[derive(Debug, Clone)]
pub struct SpecProcessor;

/// A generation plan derived from a specification
#[derive(Debug, Clone)]
pub struct GenerationPlan {
    /// Unique identifier for the plan
    pub id: String,
    /// Spec ID this plan was derived from
    pub spec_id: String,
    /// Ordered generation steps
    pub steps: Vec<GenerationStep>,
    /// Dependencies between steps (step_id_a, step_id_b means a must complete before b)
    pub dependencies: Vec<(String, String)>,
    /// Constraints extracted from spec
    pub constraints: Vec<Constraint>,
}

/// A single generation step
#[derive(Debug, Clone)]
pub struct GenerationStep {
    /// Unique identifier for this step
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Requirements this step addresses
    pub requirement_ids: Vec<String>,
    /// Acceptance criteria this step must satisfy
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    /// Priority of this step
    pub priority: Priority,
    /// Whether this step is optional
    pub optional: bool,
    /// Order in execution sequence
    pub sequence: usize,
}

/// A constraint extracted from requirements
#[derive(Debug, Clone)]
pub struct Constraint {
    /// Constraint identifier
    pub id: String,
    /// Constraint description
    pub description: String,
    /// Type of constraint
    pub constraint_type: ConstraintType,
}

/// Types of constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintType {
    /// Naming convention constraint
    NamingConvention,
    /// Code quality constraint
    CodeQuality,
    /// Documentation constraint
    Documentation,
    /// Error handling constraint
    ErrorHandling,
    /// Testing constraint
    Testing,
    /// Other constraint
    Other,
}

impl SpecProcessor {
    /// Creates a new SpecProcessor
    pub fn new() -> Self {
        Self
    }

    /// Processes a spec into a generation plan
    ///
    /// # Arguments
    ///
    /// * `spec` - The specification to process
    ///
    /// # Returns
    ///
    /// A generation plan with ordered steps and dependencies
    ///
    /// # Errors
    ///
    /// Returns an error if the spec is invalid or cannot be processed
    pub fn process(&self, spec: &Spec) -> Result<GenerationPlan, GenerationError> {
        // Extract requirements and build steps
        let mut steps = Vec::new();
        let mut step_map: BTreeMap<String, GenerationStep> = BTreeMap::new();

        for (idx, requirement) in spec.requirements.iter().enumerate() {
            let step = self.requirement_to_step(requirement, idx)?;
            step_map.insert(step.id.clone(), step);
        }

        // Convert to ordered vec
        for (_, step) in step_map {
            steps.push(step);
        }

        // Sort by sequence
        steps.sort_by_key(|s| s.sequence);

        // Extract constraints
        let constraints = self.extract_constraints(spec)?;

        // Determine dependencies based on requirement relationships
        let dependencies = self.determine_dependencies(&steps)?;

        Ok(GenerationPlan {
            id: format!("plan-{}", uuid::Uuid::new_v4()),
            spec_id: spec.id.clone(),
            steps,
            dependencies,
            constraints,
        })
    }

    /// Converts a requirement to a generation step
    fn requirement_to_step(
        &self,
        requirement: &Requirement,
        sequence: usize,
    ) -> Result<GenerationStep, GenerationError> {
        Ok(GenerationStep {
            id: format!("step-{}", requirement.id),
            description: requirement.user_story.clone(),
            requirement_ids: vec![requirement.id.clone()],
            acceptance_criteria: requirement.acceptance_criteria.clone(),
            priority: requirement.priority,
            optional: false,
            sequence,
        })
    }

    /// Extracts constraints from the specification
    fn extract_constraints(&self, spec: &Spec) -> Result<Vec<Constraint>, GenerationError> {
        let mut constraints = Vec::new();

        // Extract constraints from requirements
        for requirement in &spec.requirements {
            for criterion in &requirement.acceptance_criteria {
                // Look for naming convention constraints
                if criterion.then.to_lowercase().contains("naming convention")
                    || criterion.then.to_lowercase().contains("snake_case")
                    || criterion.then.to_lowercase().contains("camelcase")
                    || criterion.then.to_lowercase().contains("pascalcase")
                {
                    constraints.push(Constraint {
                        id: format!("constraint-naming-{}", criterion.id),
                        description: criterion.then.clone(),
                        constraint_type: ConstraintType::NamingConvention,
                    });
                }

                // Look for documentation constraints
                if criterion.then.to_lowercase().contains("doc comment")
                    || criterion.then.to_lowercase().contains("documentation")
                {
                    constraints.push(Constraint {
                        id: format!("constraint-doc-{}", criterion.id),
                        description: criterion.then.clone(),
                        constraint_type: ConstraintType::Documentation,
                    });
                }

                // Look for error handling constraints
                if criterion.then.to_lowercase().contains("error handling")
                    || criterion.then.to_lowercase().contains("error type")
                {
                    constraints.push(Constraint {
                        id: format!("constraint-error-{}", criterion.id),
                        description: criterion.then.clone(),
                        constraint_type: ConstraintType::ErrorHandling,
                    });
                }

                // Look for testing constraints
                if criterion.then.to_lowercase().contains("test")
                    || criterion.then.to_lowercase().contains("unit test")
                {
                    constraints.push(Constraint {
                        id: format!("constraint-test-{}", criterion.id),
                        description: criterion.then.clone(),
                        constraint_type: ConstraintType::Testing,
                    });
                }

                // Look for code quality constraints
                if criterion.then.to_lowercase().contains("quality")
                    || criterion.then.to_lowercase().contains("standard")
                {
                    constraints.push(Constraint {
                        id: format!("constraint-quality-{}", criterion.id),
                        description: criterion.then.clone(),
                        constraint_type: ConstraintType::CodeQuality,
                    });
                }
            }
        }

        Ok(constraints)
    }

    /// Determines dependencies between generation steps
    fn determine_dependencies(
        &self,
        steps: &[GenerationStep],
    ) -> Result<Vec<(String, String)>, GenerationError> {
        let mut dependencies = Vec::new();

        // For now, steps are executed in sequence
        // In the future, this could analyze requirement relationships
        for i in 0..steps.len().saturating_sub(1) {
            dependencies.push((steps[i].id.clone(), steps[i + 1].id.clone()));
        }

        Ok(dependencies)
    }
}

impl Default for SpecProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ricecoder_specs::models::{SpecMetadata, SpecPhase, SpecStatus};

    fn create_test_spec() -> Spec {
        Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![
                Requirement {
                    id: "req-1".to_string(),
                    user_story: "As a user, I want feature X".to_string(),
                    acceptance_criteria: vec![
                        AcceptanceCriterion {
                            id: "ac-1-1".to_string(),
                            when: "I do action A".to_string(),
                            then: "The system SHALL use snake_case naming convention".to_string(),
                        },
                        AcceptanceCriterion {
                            id: "ac-1-2".to_string(),
                            when: "I do action B".to_string(),
                            then: "The system SHALL include doc comments for all public functions"
                                .to_string(),
                        },
                    ],
                    priority: Priority::Must,
                },
                Requirement {
                    id: "req-2".to_string(),
                    user_story: "As a user, I want feature Y".to_string(),
                    acceptance_criteria: vec![AcceptanceCriterion {
                        id: "ac-2-1".to_string(),
                        when: "I do action C".to_string(),
                        then: "The system SHALL include error handling".to_string(),
                    }],
                    priority: Priority::Should,
                },
            ],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Test Author".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Approved,
            },
            inheritance: None,
        }
    }

    #[test]
    fn test_process_creates_plan_with_steps() {
        let processor = SpecProcessor::new();
        let spec = create_test_spec();

        let plan = processor.process(&spec).expect("Failed to process spec");

        assert_eq!(plan.spec_id, "test-spec");
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].requirement_ids, vec!["req-1"]);
        assert_eq!(plan.steps[1].requirement_ids, vec!["req-2"]);
    }

    #[test]
    fn test_process_extracts_constraints() {
        let processor = SpecProcessor::new();
        let spec = create_test_spec();

        let plan = processor.process(&spec).expect("Failed to process spec");

        // Should extract naming, documentation, and error handling constraints
        assert!(!plan.constraints.is_empty());

        let has_naming = plan
            .constraints
            .iter()
            .any(|c| c.constraint_type == ConstraintType::NamingConvention);
        let has_doc = plan
            .constraints
            .iter()
            .any(|c| c.constraint_type == ConstraintType::Documentation);
        let has_error = plan
            .constraints
            .iter()
            .any(|c| c.constraint_type == ConstraintType::ErrorHandling);

        assert!(has_naming, "Should extract naming constraint");
        assert!(has_doc, "Should extract documentation constraint");
        assert!(has_error, "Should extract error handling constraint");
    }

    #[test]
    fn test_process_determines_dependencies() {
        let processor = SpecProcessor::new();
        let spec = create_test_spec();

        let plan = processor.process(&spec).expect("Failed to process spec");

        // With 2 steps, should have 1 dependency
        assert_eq!(plan.dependencies.len(), 1);
        assert_eq!(plan.dependencies[0].0, plan.steps[0].id);
        assert_eq!(plan.dependencies[0].1, plan.steps[1].id);
    }

    #[test]
    fn test_requirement_to_step_preserves_priority() {
        let processor = SpecProcessor::new();
        let requirement = Requirement {
            id: "req-test".to_string(),
            user_story: "Test story".to_string(),
            acceptance_criteria: vec![],
            priority: Priority::Must,
        };

        let step = processor
            .requirement_to_step(&requirement, 0)
            .expect("Failed to convert requirement");

        assert_eq!(step.priority, Priority::Must);
        assert_eq!(step.sequence, 0);
    }

    #[test]
    fn test_extract_constraints_identifies_all_types() {
        let processor = SpecProcessor::new();
        let spec = create_test_spec();

        let constraints = processor
            .extract_constraints(&spec)
            .expect("Failed to extract constraints");

        // Verify we have constraints of different types
        let types: Vec<_> = constraints.iter().map(|c| c.constraint_type).collect();
        assert!(types.contains(&ConstraintType::NamingConvention));
        assert!(types.contains(&ConstraintType::Documentation));
        assert!(types.contains(&ConstraintType::ErrorHandling));
    }
}
