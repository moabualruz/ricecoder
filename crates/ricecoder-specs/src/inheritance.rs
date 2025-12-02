//! Spec inheritance and hierarchy resolution

use crate::error::SpecError;
use crate::models::{Spec, SpecInheritance};
use std::collections::{HashMap, HashSet};

/// Manages hierarchical spec resolution with explicit precedence
pub struct SpecInheritanceResolver;

impl SpecInheritanceResolver {
    /// Resolve spec hierarchy (project > feature > task)
    ///
    /// Organizes specs by their precedence level and returns them in order
    /// from highest precedence (project level, 0) to lowest (task level, 2).
    ///
    /// # Arguments
    /// * `specs` - Collection of specs to resolve
    ///
    /// # Returns
    /// * `Ok(Vec<Spec>)` - Specs sorted by precedence level (highest first)
    /// * `Err(SpecError)` - If circular dependencies or conflicts are detected
    pub fn resolve(specs: &[Spec]) -> Result<Vec<Spec>, SpecError> {
        // Validate the chain first to detect circular dependencies
        Self::validate_chain(specs)?;

        // Sort specs by precedence level (0 = project, 1 = feature, 2 = task)
        let mut sorted_specs = specs.to_vec();
        sorted_specs.sort_by_key(|spec| {
            spec.inheritance
                .as_ref()
                .map(|inh| inh.precedence_level)
                .unwrap_or(0)
        });

        Ok(sorted_specs)
    }

    /// Merge two specs with precedence rules
    ///
    /// Merges a child spec into a parent spec, with the parent taking precedence.
    /// Higher precedence level specs override lower precedence level specs.
    ///
    /// # Arguments
    /// * `parent` - Parent spec (higher precedence)
    /// * `child` - Child spec (lower precedence)
    ///
    /// # Returns
    /// * `Ok(Spec)` - Merged spec with parent values taking precedence
    /// * `Err(SpecError)` - If merge would create conflicts
    pub fn merge(parent: &Spec, child: &Spec) -> Result<Spec, SpecError> {
        // Validate precedence levels
        let parent_level = parent
            .inheritance
            .as_ref()
            .map(|inh| inh.precedence_level)
            .unwrap_or(0);
        let child_level = child
            .inheritance
            .as_ref()
            .map(|inh| inh.precedence_level)
            .unwrap_or(0);

        if parent_level >= child_level {
            return Err(SpecError::InheritanceConflict(
                "Parent precedence level must be lower than child precedence level".to_string(),
            ));
        }

        // Create merged spec with parent values taking precedence
        let mut merged = child.clone();

        // Override with parent values where parent has higher precedence
        if !parent.id.is_empty() && parent.id != child.id {
            // Keep child ID but track parent in inheritance
        }

        if !parent.name.is_empty() {
            merged.name = parent.name.clone();
        }

        if !parent.version.is_empty() {
            merged.version = parent.version.clone();
        }

        // Merge requirements: parent requirements take precedence
        if !parent.requirements.is_empty() {
            merged.requirements = parent.requirements.clone();
        }

        // Merge design: parent design takes precedence
        if parent.design.is_some() {
            merged.design = parent.design.clone();
        }

        // Merge tasks: parent tasks take precedence
        if !parent.tasks.is_empty() {
            merged.tasks = parent.tasks.clone();
        }

        // Merge metadata: parent metadata takes precedence
        merged.metadata.author = parent.metadata.author.clone().or(merged.metadata.author);
        merged.metadata.phase = parent.metadata.phase;
        merged.metadata.status = parent.metadata.status;
        merged.metadata.updated_at = chrono::Utc::now();

        // Update inheritance information
        let mut merged_from = child
            .inheritance
            .as_ref()
            .and_then(|inh| {
                if inh.merged_from.is_empty() {
                    Some(vec![child.id.clone()])
                } else {
                    Some(inh.merged_from.clone())
                }
            })
            .unwrap_or_else(|| vec![child.id.clone()]);

        if !merged_from.contains(&parent.id) {
            merged_from.insert(0, parent.id.clone());
        }

        merged.inheritance = Some(SpecInheritance {
            parent_id: Some(parent.id.clone()),
            precedence_level: parent_level,
            merged_from,
        });

        Ok(merged)
    }

    /// Validate inheritance chain for conflicts and circular dependencies
    ///
    /// Checks that:
    /// 1. No circular dependencies exist
    /// 2. Precedence levels are consistent
    /// 3. Parent-child relationships are valid
    ///
    /// # Arguments
    /// * `specs` - Collection of specs to validate
    ///
    /// # Returns
    /// * `Ok(())` - If chain is valid
    /// * `Err(SpecError)` - If circular dependencies or conflicts are detected
    pub fn validate_chain(specs: &[Spec]) -> Result<(), SpecError> {
        // Build a map of spec IDs to specs for quick lookup
        let spec_map: HashMap<String, &Spec> = specs.iter().map(|s| (s.id.clone(), s)).collect();

        // Check for circular dependencies
        for spec in specs {
            let mut visited = HashSet::new();
            let mut current_id = Some(spec.id.clone());

            while let Some(id) = current_id {
                if visited.contains(&id) {
                    // Circular dependency detected
                    let mut cycle = vec![id.clone()];
                    let mut cycle_id = Some(id);

                    while let Some(cid) = cycle_id {
                        if let Some(s) = spec_map.get(&cid) {
                            if let Some(inh) = &s.inheritance {
                                if let Some(parent_id) = &inh.parent_id {
                                    if parent_id == cycle.first().unwrap() {
                                        break;
                                    }
                                    cycle.push(parent_id.clone());
                                    cycle_id = Some(parent_id.clone());
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    return Err(SpecError::CircularDependency { specs: cycle });
                }

                visited.insert(id.clone());

                // Move to parent
                if let Some(inh) = &spec_map.get(&id).and_then(|s| s.inheritance.as_ref()) {
                    current_id = inh.parent_id.clone();
                } else {
                    current_id = None;
                }
            }
        }

        // Check precedence level consistency
        for spec in specs {
            if let Some(inh) = &spec.inheritance {
                if let Some(parent_id) = &inh.parent_id {
                    if let Some(parent) = spec_map.get(parent_id) {
                        let parent_level = parent
                            .inheritance
                            .as_ref()
                            .map(|p| p.precedence_level)
                            .unwrap_or(0);

                        if parent_level >= inh.precedence_level {
                            return Err(SpecError::InheritanceConflict(
                                format!(
                                    "Parent {} has precedence level {} but child {} has level {}",
                                    parent_id, parent_level, spec.id, inh.precedence_level
                                ),
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SpecMetadata, SpecPhase, SpecStatus};
    use chrono::Utc;

    fn create_spec(id: &str, precedence_level: u32, parent_id: Option<&str>) -> Spec {
        Spec {
            id: id.to_string(),
            name: format!("Spec {}", id),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: Some(SpecInheritance {
                parent_id: parent_id.map(|s| s.to_string()),
                precedence_level,
                merged_from: vec![],
            }),
        }
    }

    #[test]
    fn test_resolve_empty_specs() {
        let specs = vec![];
        let result = SpecInheritanceResolver::resolve(&specs);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_resolve_single_spec() {
        let specs = vec![create_spec("project", 0, None)];
        let result = SpecInheritanceResolver::resolve(&specs);
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].id, "project");
    }

    #[test]
    fn test_resolve_hierarchy_ordering() {
        let specs = vec![
            create_spec("task", 2, Some("feature")),
            create_spec("project", 0, None),
            create_spec("feature", 1, Some("project")),
        ];

        let result = SpecInheritanceResolver::resolve(&specs);
        assert!(result.is_ok());
        let resolved = result.unwrap();

        // Should be ordered by precedence level (0, 1, 2)
        assert_eq!(resolved[0].id, "project");
        assert_eq!(resolved[1].id, "feature");
        assert_eq!(resolved[2].id, "task");
    }

    #[test]
    fn test_validate_chain_no_circular_dependency() {
        let specs = vec![
            create_spec("project", 0, None),
            create_spec("feature", 1, Some("project")),
            create_spec("task", 2, Some("feature")),
        ];

        let result = SpecInheritanceResolver::validate_chain(&specs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_chain_circular_dependency() {
        let mut specs = vec![
            create_spec("project", 0, Some("task")), // Creates cycle
            create_spec("feature", 1, Some("project")),
            create_spec("task", 2, Some("feature")),
        ];

        // Manually set up circular dependency
        if let Some(inh) = &mut specs[0].inheritance {
            inh.parent_id = Some("task".to_string());
        }

        let result = SpecInheritanceResolver::validate_chain(&specs);
        assert!(result.is_err());
        match result {
            Err(SpecError::CircularDependency { specs: cycle }) => {
                assert!(!cycle.is_empty());
            }
            _ => panic!("Expected CircularDependency error"),
        }
    }

    #[test]
    fn test_validate_chain_invalid_precedence() {
        let mut specs = vec![
            create_spec("project", 0, None),
            create_spec("feature", 1, Some("project")),
            create_spec("task", 2, Some("feature")),
        ];

        // Make feature have higher precedence than project (invalid)
        if let Some(inh) = &mut specs[1].inheritance {
            inh.precedence_level = 0;
        }

        let result = SpecInheritanceResolver::validate_chain(&specs);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_parent_overrides_child() {
        let parent = Spec {
            id: "parent".to_string(),
            name: "Parent Name".to_string(),
            version: "2.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Parent Author".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Design,
                status: SpecStatus::Approved,
            },
            inheritance: Some(SpecInheritance {
                parent_id: None,
                precedence_level: 0,
                merged_from: vec![],
            }),
        };

        let child = Spec {
            id: "child".to_string(),
            name: "Child Name".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Child Author".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: Some(SpecInheritance {
                parent_id: Some("parent".to_string()),
                precedence_level: 1,
                merged_from: vec![],
            }),
        };

        let result = SpecInheritanceResolver::merge(&parent, &child);
        assert!(result.is_ok());

        let merged = result.unwrap();
        // Parent values should override child values
        assert_eq!(merged.name, "Parent Name");
        assert_eq!(merged.version, "2.0.0");
        assert_eq!(merged.metadata.phase, SpecPhase::Design);
        assert_eq!(merged.metadata.status, SpecStatus::Approved);
    }

    #[test]
    fn test_merge_invalid_precedence() {
        let parent = create_spec("parent", 1, None);
        let child = create_spec("child", 0, Some("parent"));

        let result = SpecInheritanceResolver::merge(&parent, &child);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_updates_inheritance() {
        let parent = create_spec("parent", 0, None);
        let child = create_spec("child", 1, Some("parent"));

        let result = SpecInheritanceResolver::merge(&parent, &child);
        assert!(result.is_ok());

        let merged = result.unwrap();
        assert!(merged.inheritance.is_some());

        let inh = merged.inheritance.unwrap();
        assert_eq!(inh.parent_id, Some("parent".to_string()));
        assert_eq!(inh.precedence_level, 0);
        assert!(inh.merged_from.contains(&"parent".to_string()));
    }
}
