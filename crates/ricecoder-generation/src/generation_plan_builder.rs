//! Generation plan builder for creating and validating generation plans
//!
//! Builds generation plans from spec requirements, determines task dependencies,
//! orders steps for correct execution, and validates plan completeness.

use crate::error::GenerationError;
use crate::spec_processor::{ConstraintType, GenerationPlan, GenerationStep};
use ricecoder_specs::models::{Priority, Requirement};
use std::collections::{HashMap, HashSet};

/// Builds generation plans from specifications
#[derive(Debug, Clone)]
pub struct GenerationPlanBuilder {
    /// Maximum number of steps allowed in a plan
    max_steps: usize,
}

/// Validation result for a generation plan
#[derive(Debug, Clone)]
pub struct PlanValidation {
    /// Whether the plan is valid
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

impl GenerationPlanBuilder {
    /// Creates a new GenerationPlanBuilder
    pub fn new() -> Self {
        Self { max_steps: 1000 }
    }

    /// Sets the maximum number of steps allowed
    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// Creates generation steps from spec requirements
    ///
    /// # Arguments
    ///
    /// * `requirements` - The requirements to convert to steps
    ///
    /// # Returns
    ///
    /// A vector of generation steps
    pub fn create_steps(
        &self,
        requirements: &[Requirement],
    ) -> Result<Vec<GenerationStep>, GenerationError> {
        if requirements.len() > self.max_steps {
            return Err(GenerationError::SpecError(format!(
                "Too many requirements: {} exceeds maximum of {}",
                requirements.len(),
                self.max_steps
            )));
        }

        let mut steps = Vec::new();

        for (idx, requirement) in requirements.iter().enumerate() {
            let step = GenerationStep {
                id: format!("step-{}", requirement.id),
                description: requirement.user_story.clone(),
                requirement_ids: vec![requirement.id.clone()],
                acceptance_criteria: requirement.acceptance_criteria.clone(),
                priority: requirement.priority,
                optional: requirement.priority == Priority::Could,
                sequence: idx,
            };
            steps.push(step);
        }

        Ok(steps)
    }

    /// Determines task dependencies between steps
    ///
    /// # Arguments
    ///
    /// * `steps` - The generation steps
    ///
    /// # Returns
    ///
    /// A vector of dependencies as (from_step_id, to_step_id) tuples
    pub fn determine_dependencies(
        &self,
        steps: &[GenerationStep],
    ) -> Result<Vec<(String, String)>, GenerationError> {
        let mut dependencies = Vec::new();

        // Create a map of step IDs to their sequence numbers
        let mut step_sequence: HashMap<String, usize> = HashMap::new();
        for step in steps {
            step_sequence.insert(step.id.clone(), step.sequence);
        }

        // For now, enforce sequential ordering
        // Steps are executed in order of their sequence number
        for i in 0..steps.len().saturating_sub(1) {
            let current_step = &steps[i];
            let next_step = &steps[i + 1];

            // Only add dependency if next step has higher sequence
            if next_step.sequence > current_step.sequence {
                dependencies.push((current_step.id.clone(), next_step.id.clone()));
            }
        }

        Ok(dependencies)
    }

    /// Orders steps for correct execution
    ///
    /// # Arguments
    ///
    /// * `steps` - The generation steps to order
    /// * `dependencies` - The dependencies between steps
    ///
    /// # Returns
    ///
    /// An ordered vector of steps
    pub fn order_steps(
        &self,
        mut steps: Vec<GenerationStep>,
        dependencies: &[(String, String)],
    ) -> Result<Vec<GenerationStep>, GenerationError> {
        // Build dependency graph
        let mut dep_graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Initialize all steps in the graph
        for step in &steps {
            dep_graph.insert(step.id.clone(), Vec::new());
            in_degree.insert(step.id.clone(), 0);
        }

        // Add edges
        for (from, to) in dependencies {
            if let Some(deps) = dep_graph.get_mut(from) {
                deps.push(to.clone());
            }
            *in_degree.get_mut(to).unwrap_or(&mut 0) += 1;
        }

        // Topological sort using Kahn's algorithm
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut ordered = Vec::new();

        while !queue.is_empty() {
            queue.sort(); // For deterministic ordering
            let current = queue.remove(0);

            // Find the step with this ID
            if let Some(pos) = steps.iter().position(|s| s.id == current) {
                ordered.push(steps.remove(pos));
            }

            // Process neighbors
            if let Some(neighbors) = dep_graph.get(&current) {
                for neighbor in neighbors.clone() {
                    let degree = in_degree.get_mut(&neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(neighbor);
                    }
                }
            }
        }

        if !steps.is_empty() {
            return Err(GenerationError::SpecError(
                "Circular dependency detected in generation steps".to_string(),
            ));
        }

        Ok(ordered)
    }

    /// Validates plan completeness
    ///
    /// # Arguments
    ///
    /// * `plan` - The generation plan to validate
    ///
    /// # Returns
    ///
    /// A validation result
    pub fn validate_plan(&self, plan: &GenerationPlan) -> PlanValidation {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check that plan has steps
        if plan.steps.is_empty() {
            errors.push("Generation plan has no steps".to_string());
        }

        // Check that all steps have unique IDs
        let mut seen_ids = HashSet::new();
        for step in &plan.steps {
            if !seen_ids.insert(&step.id) {
                errors.push(format!("Duplicate step ID: {}", step.id));
            }
        }

        // Check that all dependencies reference existing steps
        let step_ids: HashSet<_> = plan.steps.iter().map(|s| &s.id).collect();
        for (from, to) in &plan.dependencies {
            if !step_ids.contains(from) {
                errors.push(format!("Dependency references non-existent step: {}", from));
            }
            if !step_ids.contains(to) {
                errors.push(format!("Dependency references non-existent step: {}", to));
            }
        }

        // Check that all steps have acceptance criteria
        for step in &plan.steps {
            if step.acceptance_criteria.is_empty() && !step.optional {
                warnings.push(format!("Step {} has no acceptance criteria", step.id));
            }
        }

        // Check that constraints are present for code quality requirements
        let has_quality_constraints = plan
            .constraints
            .iter()
            .any(|c| c.constraint_type == ConstraintType::CodeQuality);

        if !plan.steps.is_empty() && !has_quality_constraints {
            warnings.push("No code quality constraints found in plan".to_string());
        }

        let is_valid = errors.is_empty();

        PlanValidation {
            is_valid,
            errors,
            warnings,
        }
    }
}

impl Default for GenerationPlanBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_specs::models::AcceptanceCriterion;

    fn create_test_requirement(id: &str, priority: Priority) -> Requirement {
        Requirement {
            id: id.to_string(),
            user_story: format!("User story for {}", id),
            acceptance_criteria: vec![AcceptanceCriterion {
                id: format!("{}-ac-1", id),
                when: "condition".to_string(),
                then: "outcome".to_string(),
            }],
            priority,
        }
    }

    #[test]
    fn test_create_steps_from_requirements() {
        let builder = GenerationPlanBuilder::new();
        let requirements = vec![
            create_test_requirement("req-1", Priority::Must),
            create_test_requirement("req-2", Priority::Should),
        ];

        let steps = builder
            .create_steps(&requirements)
            .expect("Failed to create steps");

        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].id, "step-req-1");
        assert_eq!(steps[1].id, "step-req-2");
        assert_eq!(steps[0].sequence, 0);
        assert_eq!(steps[1].sequence, 1);
    }

    #[test]
    fn test_create_steps_marks_optional() {
        let builder = GenerationPlanBuilder::new();
        let requirements = vec![
            create_test_requirement("req-1", Priority::Must),
            create_test_requirement("req-2", Priority::Could),
        ];

        let steps = builder
            .create_steps(&requirements)
            .expect("Failed to create steps");

        assert!(!steps[0].optional);
        assert!(steps[1].optional);
    }

    #[test]
    fn test_determine_dependencies_sequential() {
        let builder = GenerationPlanBuilder::new();
        let steps = vec![
            GenerationStep {
                id: "step-1".to_string(),
                description: "Step 1".to_string(),
                requirement_ids: vec!["req-1".to_string()],
                acceptance_criteria: vec![],
                priority: Priority::Must,
                optional: false,
                sequence: 0,
            },
            GenerationStep {
                id: "step-2".to_string(),
                description: "Step 2".to_string(),
                requirement_ids: vec!["req-2".to_string()],
                acceptance_criteria: vec![],
                priority: Priority::Must,
                optional: false,
                sequence: 1,
            },
        ];

        let deps = builder
            .determine_dependencies(&steps)
            .expect("Failed to determine dependencies");

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], ("step-1".to_string(), "step-2".to_string()));
    }

    #[test]
    fn test_order_steps_topological_sort() {
        let builder = GenerationPlanBuilder::new();
        let steps = vec![
            GenerationStep {
                id: "step-1".to_string(),
                description: "Step 1".to_string(),
                requirement_ids: vec![],
                acceptance_criteria: vec![],
                priority: Priority::Must,
                optional: false,
                sequence: 0,
            },
            GenerationStep {
                id: "step-2".to_string(),
                description: "Step 2".to_string(),
                requirement_ids: vec![],
                acceptance_criteria: vec![],
                priority: Priority::Must,
                optional: false,
                sequence: 1,
            },
        ];

        let dependencies = vec![("step-1".to_string(), "step-2".to_string())];

        let ordered = builder
            .order_steps(steps, &dependencies)
            .expect("Failed to order steps");

        assert_eq!(ordered[0].id, "step-1");
        assert_eq!(ordered[1].id, "step-2");
    }

    #[test]
    fn test_validate_plan_empty_steps() {
        let builder = GenerationPlanBuilder::new();
        let plan = GenerationPlan {
            id: "plan-1".to_string(),
            spec_id: "spec-1".to_string(),
            steps: vec![],
            dependencies: vec![],
            constraints: vec![],
        };

        let validation = builder.validate_plan(&plan);

        assert!(!validation.is_valid);
        assert!(validation.errors.iter().any(|e| e.contains("no steps")));
    }

    #[test]
    fn test_validate_plan_duplicate_ids() {
        let builder = GenerationPlanBuilder::new();
        let plan = GenerationPlan {
            id: "plan-1".to_string(),
            spec_id: "spec-1".to_string(),
            steps: vec![
                GenerationStep {
                    id: "step-1".to_string(),
                    description: "Step 1".to_string(),
                    requirement_ids: vec![],
                    acceptance_criteria: vec![],
                    priority: Priority::Must,
                    optional: false,
                    sequence: 0,
                },
                GenerationStep {
                    id: "step-1".to_string(),
                    description: "Step 1 duplicate".to_string(),
                    requirement_ids: vec![],
                    acceptance_criteria: vec![],
                    priority: Priority::Must,
                    optional: false,
                    sequence: 1,
                },
            ],
            dependencies: vec![],
            constraints: vec![],
        };

        let validation = builder.validate_plan(&plan);

        assert!(!validation.is_valid);
        assert!(validation.errors.iter().any(|e| e.contains("Duplicate")));
    }

    #[test]
    fn test_validate_plan_invalid_dependencies() {
        let builder = GenerationPlanBuilder::new();
        let plan = GenerationPlan {
            id: "plan-1".to_string(),
            spec_id: "spec-1".to_string(),
            steps: vec![GenerationStep {
                id: "step-1".to_string(),
                description: "Step 1".to_string(),
                requirement_ids: vec![],
                acceptance_criteria: vec![],
                priority: Priority::Must,
                optional: false,
                sequence: 0,
            }],
            dependencies: vec![("step-1".to_string(), "step-nonexistent".to_string())],
            constraints: vec![],
        };

        let validation = builder.validate_plan(&plan);

        assert!(!validation.is_valid);
        assert!(validation.errors.iter().any(|e| e.contains("non-existent")));
    }
}
