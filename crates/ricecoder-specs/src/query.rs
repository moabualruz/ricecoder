//! Query engine for spec discovery and filtering

use crate::models::{Priority, Spec, SpecQuery, SpecType};
use regex::Regex;

/// Enables efficient spec discovery and filtering
pub struct SpecQueryEngine;

impl SpecQueryEngine {
    /// Execute a query against specs
    ///
    /// Filters specs by name (exact or partial match), type, status, priority, phase,
    /// and custom criteria. Returns all specs matching all filter criteria.
    ///
    /// # Arguments
    ///
    /// * `specs` - Slice of specs to query
    /// * `query` - Query with filter criteria
    ///
    /// # Returns
    ///
    /// Vector of specs matching all filter criteria
    pub fn query(specs: &[Spec], query: &SpecQuery) -> Vec<Spec> {
        specs
            .iter()
            .filter(|spec| Self::matches_query(spec, query))
            .cloned()
            .collect()
    }

    /// Check if a spec matches all query criteria
    fn matches_query(spec: &Spec, query: &SpecQuery) -> bool {
        // Check name filter (exact or partial match)
        if let Some(ref name_filter) = query.name {
            if !Self::matches_name(spec, name_filter) {
                return false;
            }
        }

        // Check type filter
        if let Some(spec_type) = query.spec_type {
            if !Self::matches_type(spec, spec_type) {
                return false;
            }
        }

        // Check status filter
        if let Some(status) = query.status {
            if spec.metadata.status != status {
                return false;
            }
        }

        // Check priority filter
        if let Some(priority) = query.priority {
            if !Self::matches_priority(spec, priority) {
                return false;
            }
        }

        // Check phase filter
        if let Some(phase) = query.phase {
            if spec.metadata.phase != phase {
                return false;
            }
        }

        // Check custom filters
        for (field, value) in &query.custom_filters {
            if !Self::matches_custom_filter(spec, field, value) {
                return false;
            }
        }

        true
    }

    /// Check if spec name matches filter (exact or partial match)
    fn matches_name(spec: &Spec, filter: &str) -> bool {
        // Try exact match first
        if spec.name.eq_ignore_ascii_case(filter) {
            return true;
        }

        // Try partial match (case-insensitive)
        if spec.name.to_lowercase().contains(&filter.to_lowercase()) {
            return true;
        }

        // Try ID match
        if spec.id.eq_ignore_ascii_case(filter) {
            return true;
        }

        // Try regex match
        if let Ok(regex) = Regex::new(filter) {
            if regex.is_match(&spec.name) || regex.is_match(&spec.id) {
                return true;
            }
        }

        false
    }

    /// Check if spec type matches filter
    fn matches_type(spec: &Spec, spec_type: SpecType) -> bool {
        // Infer type from spec structure
        let inferred_type = if !spec.requirements.is_empty() {
            SpecType::Feature
        } else if spec.design.is_some() {
            SpecType::Component
        } else if !spec.tasks.is_empty() {
            SpecType::Task
        } else {
            SpecType::Feature // Default
        };

        inferred_type == spec_type
    }

    /// Check if spec has any requirement with matching priority
    fn matches_priority(spec: &Spec, priority: Priority) -> bool {
        spec.requirements.iter().any(|req| req.priority == priority)
    }

    /// Check if spec matches custom filter
    fn matches_custom_filter(spec: &Spec, field: &str, value: &str) -> bool {
        match field.to_lowercase().as_str() {
            "author" => spec
                .metadata
                .author
                .as_ref()
                .map(|a| a.contains(value))
                .unwrap_or(false),
            "version" => spec.version.contains(value),
            "id" => spec.id.contains(value),
            _ => false,
        }
    }

    /// Resolve dependencies for a spec
    ///
    /// Returns IDs of all specs that this spec depends on (via requirement references).
    ///
    /// # Arguments
    ///
    /// * `spec` - Spec to resolve dependencies for
    /// * `all_specs` - All available specs
    ///
    /// # Returns
    ///
    /// Vector of spec IDs that this spec depends on
    pub fn resolve_dependencies(spec: &Spec, all_specs: &[Spec]) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Collect all requirement IDs referenced by tasks
        let mut referenced_req_ids = std::collections::HashSet::new();
        for task in &spec.tasks {
            Self::collect_requirement_ids(task, &mut referenced_req_ids);
        }

        // Find specs that provide these requirements
        for other_spec in all_specs {
            if other_spec.id == spec.id {
                continue;
            }

            for req in &other_spec.requirements {
                if referenced_req_ids.contains(&req.id) {
                    dependencies.push(other_spec.id.clone());
                    break;
                }
            }
        }

        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    /// Collect all requirement IDs from a task and its subtasks
    fn collect_requirement_ids(
        task: &crate::models::Task,
        ids: &mut std::collections::HashSet<String>,
    ) {
        for req_id in &task.requirements {
            ids.insert(req_id.clone());
        }
        for subtask in &task.subtasks {
            Self::collect_requirement_ids(subtask, ids);
        }
    }

    /// Detect circular dependencies
    ///
    /// Returns a vector of cycles, where each cycle is a vector of spec IDs
    /// forming a circular dependency chain.
    ///
    /// # Arguments
    ///
    /// * `specs` - All specs to check
    ///
    /// # Returns
    ///
    /// Vector of cycles (each cycle is a vector of spec IDs)
    pub fn detect_circular_dependencies(specs: &[Spec]) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        for spec in specs {
            if !visited.contains(&spec.id) {
                Self::dfs_detect_cycle(
                    &spec.id,
                    specs,
                    &mut visited,
                    &mut rec_stack,
                    &mut Vec::new(),
                    &mut cycles,
                );
            }
        }

        cycles
    }

    /// Depth-first search to detect cycles
    fn dfs_detect_cycle(
        spec_id: &str,
        all_specs: &[Spec],
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(spec_id.to_string());
        rec_stack.insert(spec_id.to_string());
        path.push(spec_id.to_string());

        // Find the spec
        if let Some(spec) = all_specs.iter().find(|s| s.id == spec_id) {
            let dependencies = Self::resolve_dependencies(spec, all_specs);

            for dep_id in dependencies {
                if !visited.contains(&dep_id) {
                    Self::dfs_detect_cycle(&dep_id, all_specs, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(&dep_id) {
                    // Found a cycle
                    if let Some(pos) = path.iter().position(|id| id == &dep_id) {
                        let cycle = path[pos..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }

        rec_stack.remove(spec_id);
        path.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SpecMetadata, SpecPhase, SpecStatus};
    use chrono::Utc;

    fn create_test_spec(id: &str, name: &str, status: SpecStatus, phase: SpecPhase) -> Spec {
        Spec {
            id: id.to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Test Author".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase,
                status,
            },
            inheritance: None,
        }
    }

    // ============================================================================
    // Query Tests
    // ============================================================================

    #[test]
    fn test_query_empty_specs() {
        let specs = vec![];
        let query = SpecQuery::default();
        let results = SpecQueryEngine::query(&specs, &query);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_no_filters() {
        let specs = vec![
            create_test_spec(
                "spec-1",
                "Feature One",
                SpecStatus::Draft,
                SpecPhase::Requirements,
            ),
            create_test_spec(
                "spec-2",
                "Feature Two",
                SpecStatus::Approved,
                SpecPhase::Design,
            ),
        ];
        let query = SpecQuery::default();
        let results = SpecQueryEngine::query(&specs, &query);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_by_name_exact_match() {
        let specs = vec![
            create_test_spec(
                "spec-1",
                "Feature One",
                SpecStatus::Draft,
                SpecPhase::Requirements,
            ),
            create_test_spec(
                "spec-2",
                "Feature Two",
                SpecStatus::Approved,
                SpecPhase::Design,
            ),
        ];
        let query = SpecQuery {
            name: Some("Feature One".to_string()),
            ..Default::default()
        };
        let results = SpecQueryEngine::query(&specs, &query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "spec-1");
    }

    #[test]
    fn test_query_by_name_partial_match() {
        let specs = vec![
            create_test_spec(
                "spec-1",
                "Feature One",
                SpecStatus::Draft,
                SpecPhase::Requirements,
            ),
            create_test_spec(
                "spec-2",
                "Feature Two",
                SpecStatus::Approved,
                SpecPhase::Design,
            ),
        ];
        let query = SpecQuery {
            name: Some("Feature".to_string()),
            ..Default::default()
        };
        let results = SpecQueryEngine::query(&specs, &query);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_by_name_case_insensitive() {
        let specs = vec![create_test_spec(
            "spec-1",
            "Feature One",
            SpecStatus::Draft,
            SpecPhase::Requirements,
        )];
        let query = SpecQuery {
            name: Some("feature one".to_string()),
            ..Default::default()
        };
        let results = SpecQueryEngine::query(&specs, &query);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_by_status() {
        let specs = vec![
            create_test_spec(
                "spec-1",
                "Feature One",
                SpecStatus::Draft,
                SpecPhase::Requirements,
            ),
            create_test_spec(
                "spec-2",
                "Feature Two",
                SpecStatus::Approved,
                SpecPhase::Design,
            ),
        ];
        let query = SpecQuery {
            status: Some(SpecStatus::Approved),
            ..Default::default()
        };
        let results = SpecQueryEngine::query(&specs, &query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "spec-2");
    }

    #[test]
    fn test_query_by_phase() {
        let specs = vec![
            create_test_spec(
                "spec-1",
                "Feature One",
                SpecStatus::Draft,
                SpecPhase::Requirements,
            ),
            create_test_spec(
                "spec-2",
                "Feature Two",
                SpecStatus::Approved,
                SpecPhase::Design,
            ),
        ];
        let query = SpecQuery {
            phase: Some(SpecPhase::Design),
            ..Default::default()
        };
        let results = SpecQueryEngine::query(&specs, &query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "spec-2");
    }

    #[test]
    fn test_query_by_multiple_filters() {
        let specs = vec![
            create_test_spec(
                "spec-1",
                "Feature One",
                SpecStatus::Draft,
                SpecPhase::Requirements,
            ),
            create_test_spec(
                "spec-2",
                "Feature Two",
                SpecStatus::Approved,
                SpecPhase::Design,
            ),
            create_test_spec(
                "spec-3",
                "Feature Three",
                SpecStatus::Approved,
                SpecPhase::Requirements,
            ),
        ];
        let query = SpecQuery {
            status: Some(SpecStatus::Approved),
            phase: Some(SpecPhase::Design),
            ..Default::default()
        };
        let results = SpecQueryEngine::query(&specs, &query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "spec-2");
    }

    #[test]
    fn test_query_by_custom_filter_author() {
        let mut spec1 = create_test_spec(
            "spec-1",
            "Feature One",
            SpecStatus::Draft,
            SpecPhase::Requirements,
        );
        spec1.metadata.author = Some("Alice".to_string());

        let mut spec2 = create_test_spec(
            "spec-2",
            "Feature Two",
            SpecStatus::Approved,
            SpecPhase::Design,
        );
        spec2.metadata.author = Some("Bob".to_string());

        let specs = vec![spec1, spec2];
        let query = SpecQuery {
            custom_filters: vec![("author".to_string(), "Alice".to_string())],
            ..Default::default()
        };
        let results = SpecQueryEngine::query(&specs, &query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "spec-1");
    }

    #[test]
    fn test_query_by_custom_filter_version() {
        let mut spec1 = create_test_spec(
            "spec-1",
            "Feature One",
            SpecStatus::Draft,
            SpecPhase::Requirements,
        );
        spec1.version = "1.0.0".to_string();

        let mut spec2 = create_test_spec(
            "spec-2",
            "Feature Two",
            SpecStatus::Approved,
            SpecPhase::Design,
        );
        spec2.version = "2.0.0".to_string();

        let specs = vec![spec1, spec2];
        let query = SpecQuery {
            custom_filters: vec![("version".to_string(), "2.0".to_string())],
            ..Default::default()
        };
        let results = SpecQueryEngine::query(&specs, &query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "spec-2");
    }

    // ============================================================================
    // Dependency Resolution Tests
    // ============================================================================

    #[test]
    fn test_resolve_dependencies_no_dependencies() {
        let spec = create_test_spec(
            "spec-1",
            "Feature One",
            SpecStatus::Draft,
            SpecPhase::Requirements,
        );
        let all_specs = vec![spec.clone()];
        let deps = SpecQueryEngine::resolve_dependencies(&spec, &all_specs);
        assert_eq!(deps.len(), 0);
    }

    #[test]
    fn test_resolve_dependencies_with_task_requirements() {
        let mut spec1 = create_test_spec(
            "spec-1",
            "Feature One",
            SpecStatus::Draft,
            SpecPhase::Requirements,
        );
        spec1.requirements = vec![crate::models::Requirement {
            id: "REQ-1".to_string(),
            user_story: "Test".to_string(),
            acceptance_criteria: vec![],
            priority: Priority::Must,
        }];

        let mut spec2 = create_test_spec(
            "spec-2",
            "Feature Two",
            SpecStatus::Approved,
            SpecPhase::Design,
        );
        spec2.tasks = vec![crate::models::Task {
            id: "1".to_string(),
            description: "Task 1".to_string(),
            subtasks: vec![],
            requirements: vec!["REQ-1".to_string()],
            status: crate::models::TaskStatus::NotStarted,
            optional: false,
        }];

        let all_specs = vec![spec1, spec2.clone()];
        let deps = SpecQueryEngine::resolve_dependencies(&spec2, &all_specs);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], "spec-1");
    }

    // ============================================================================
    // Circular Dependency Detection Tests
    // ============================================================================

    #[test]
    fn test_detect_circular_dependencies_no_cycles() {
        let specs = vec![
            create_test_spec(
                "spec-1",
                "Feature One",
                SpecStatus::Draft,
                SpecPhase::Requirements,
            ),
            create_test_spec(
                "spec-2",
                "Feature Two",
                SpecStatus::Approved,
                SpecPhase::Design,
            ),
        ];
        let cycles = SpecQueryEngine::detect_circular_dependencies(&specs);
        assert_eq!(cycles.len(), 0);
    }

    #[test]
    fn test_query_consistency_multiple_executions() {
        let specs = vec![
            create_test_spec(
                "spec-1",
                "Feature One",
                SpecStatus::Draft,
                SpecPhase::Requirements,
            ),
            create_test_spec(
                "spec-2",
                "Feature Two",
                SpecStatus::Approved,
                SpecPhase::Design,
            ),
        ];
        let query = SpecQuery {
            status: Some(SpecStatus::Approved),
            ..Default::default()
        };

        let results1 = SpecQueryEngine::query(&specs, &query);
        let results2 = SpecQueryEngine::query(&specs, &query);

        assert_eq!(results1.len(), results2.len());
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            assert_eq!(r1.id, r2.id);
        }
    }
}
