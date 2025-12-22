//! Steering file loading and management

use std::{fs, path::Path};

use crate::{
    error::{Severity, SpecError, ValidationError},
    models::{Standard, Steering, SteeringRule, TemplateRef},
};

/// Loads and merges steering documents
pub struct SteeringLoader;

impl SteeringLoader {
    /// Load steering from a directory
    ///
    /// Searches for YAML and Markdown files in the directory and loads them.
    /// Returns a merged Steering document with all rules, standards, and templates.
    ///
    /// # Arguments
    /// * `path` - Directory path to search for steering files
    ///
    /// # Returns
    /// * `Ok(Steering)` - Merged steering document
    /// * `Err(SpecError)` - If loading or parsing fails
    pub fn load(path: &Path) -> Result<Steering, SpecError> {
        if !path.exists() {
            return Ok(Steering {
                rules: vec![],
                standards: vec![],
                templates: vec![],
            });
        }

        if !path.is_dir() {
            return Err(SpecError::InvalidFormat(format!(
                "Steering path is not a directory: {}",
                path.display()
            )));
        }

        let mut all_rules = vec![];
        let mut all_standards = vec![];
        let mut all_templates = vec![];

        // Recursively search for steering files
        Self::load_from_directory(path, &mut all_rules, &mut all_standards, &mut all_templates)?;

        Ok(Steering {
            rules: all_rules,
            standards: all_standards,
            templates: all_templates,
        })
    }

    /// Load steering files from a directory recursively
    fn load_from_directory(
        dir: &Path,
        rules: &mut Vec<SteeringRule>,
        standards: &mut Vec<Standard>,
        templates: &mut Vec<TemplateRef>,
    ) -> Result<(), SpecError> {
        let entries = fs::read_dir(dir).map_err(SpecError::IoError)?;

        for entry in entries {
            let entry = entry.map_err(SpecError::IoError)?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively load from subdirectories
                Self::load_from_directory(&path, rules, standards, templates)?;
            } else if path.is_file() {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();

                // Check if it's a YAML or Markdown file
                if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
                    let content = fs::read_to_string(&path).map_err(SpecError::IoError)?;
                    let steering = Self::parse_yaml(&content, &path)?;
                    rules.extend(steering.rules);
                    standards.extend(steering.standards);
                    templates.extend(steering.templates);
                } else if file_name.ends_with(".md") {
                    let content = fs::read_to_string(&path).map_err(SpecError::IoError)?;
                    let steering = Self::parse_markdown(&content, &path)?;
                    rules.extend(steering.rules);
                    standards.extend(steering.standards);
                    templates.extend(steering.templates);
                }
            }
        }

        Ok(())
    }

    /// Parse YAML steering file
    fn parse_yaml(content: &str, path: &Path) -> Result<Steering, SpecError> {
        serde_yaml::from_str(content).map_err(|e| SpecError::ParseError {
            path: path.display().to_string(),
            line: e.location().map(|l| l.line()).unwrap_or(0),
            message: e.to_string(),
        })
    }

    /// Parse Markdown steering file
    fn parse_markdown(content: &str, path: &Path) -> Result<Steering, SpecError> {
        // For now, try to parse as YAML embedded in markdown
        // In a full implementation, this would extract YAML code blocks
        // or parse markdown-specific steering format

        // Try to find YAML code block
        if let Some(start) = content.find("```yaml") {
            let after_start = &content[start + 7..];
            if let Some(end) = after_start.find("```") {
                let yaml_content = &after_start[..end];
                return serde_yaml::from_str(yaml_content).map_err(|e| SpecError::ParseError {
                    path: path.display().to_string(),
                    line: e.location().map(|l| l.line()).unwrap_or(0),
                    message: e.to_string(),
                });
            }
        }

        // If no YAML block found, return empty steering
        Ok(Steering {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        })
    }

    /// Merge global and project steering
    ///
    /// Project steering takes precedence over global steering.
    /// Rules, standards, and templates are merged with project items overriding global items
    /// when they have the same ID.
    ///
    /// # Arguments
    /// * `global` - Global steering document
    /// * `project` - Project steering document
    ///
    /// # Returns
    /// * `Ok(Steering)` - Merged steering document with project taking precedence
    /// * `Err(SpecError)` - If merge fails
    pub fn merge(global: &Steering, project: &Steering) -> Result<Steering, SpecError> {
        // Merge rules: project overrides global
        let mut merged_rules = global.rules.clone();
        for project_rule in &project.rules {
            // Remove any global rule with the same ID
            merged_rules.retain(|r| r.id != project_rule.id);
            // Add the project rule
            merged_rules.push(project_rule.clone());
        }

        // Merge standards: project overrides global
        let mut merged_standards = global.standards.clone();
        for project_standard in &project.standards {
            // Remove any global standard with the same ID
            merged_standards.retain(|s| s.id != project_standard.id);
            // Add the project standard
            merged_standards.push(project_standard.clone());
        }

        // Merge templates: project overrides global
        let mut merged_templates = global.templates.clone();
        for project_template in &project.templates {
            // Remove any global template with the same ID
            merged_templates.retain(|t| t.id != project_template.id);
            // Add the project template
            merged_templates.push(project_template.clone());
        }

        Ok(Steering {
            rules: merged_rules,
            standards: merged_standards,
            templates: merged_templates,
        })
    }

    /// Validate steering syntax and semantics
    ///
    /// Checks that:
    /// - All rule IDs are unique
    /// - All standard IDs are unique
    /// - All template IDs are unique
    /// - Rules have non-empty descriptions and patterns
    /// - Standards have non-empty descriptions
    /// - Templates have non-empty paths
    ///
    /// # Arguments
    /// * `steering` - Steering document to validate
    ///
    /// # Returns
    /// * `Ok(())` - If steering is valid
    /// * `Err(SpecError)` - If validation fails
    pub fn validate(steering: &Steering) -> Result<(), SpecError> {
        let mut errors = vec![];

        // Check for duplicate rule IDs
        let mut rule_ids = std::collections::HashSet::new();
        for (idx, rule) in steering.rules.iter().enumerate() {
            if !rule_ids.insert(&rule.id) {
                errors.push(ValidationError {
                    path: "steering".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Duplicate rule ID: {}", rule.id),
                    severity: Severity::Error,
                });
            }

            // Check rule has description
            if rule.description.is_empty() {
                errors.push(ValidationError {
                    path: "steering".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Rule {} has empty description", rule.id),
                    severity: Severity::Warning,
                });
            }

            // Check rule has pattern
            if rule.pattern.is_empty() {
                errors.push(ValidationError {
                    path: "steering".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Rule {} has empty pattern", rule.id),
                    severity: Severity::Warning,
                });
            }
        }

        // Check for duplicate standard IDs
        let mut standard_ids = std::collections::HashSet::new();
        for (idx, standard) in steering.standards.iter().enumerate() {
            if !standard_ids.insert(&standard.id) {
                errors.push(ValidationError {
                    path: "steering".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Duplicate standard ID: {}", standard.id),
                    severity: Severity::Error,
                });
            }

            // Check standard has description
            if standard.description.is_empty() {
                errors.push(ValidationError {
                    path: "steering".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Standard {} has empty description", standard.id),
                    severity: Severity::Warning,
                });
            }
        }

        // Check for duplicate template IDs
        let mut template_ids = std::collections::HashSet::new();
        for (idx, template) in steering.templates.iter().enumerate() {
            if !template_ids.insert(&template.id) {
                errors.push(ValidationError {
                    path: "steering".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Duplicate template ID: {}", template.id),
                    severity: Severity::Error,
                });
            }

            // Check template has path
            if template.path.is_empty() {
                errors.push(ValidationError {
                    path: "steering".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Template {} has empty path", template.id),
                    severity: Severity::Warning,
                });
            }
        }

        // If there are any errors, return them
        if !errors.is_empty() {
            let has_errors = errors.iter().any(|e| e.severity == Severity::Error);
            if has_errors {
                return Err(SpecError::ValidationFailed(errors));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_steering_loader_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = SteeringLoader::load(temp_dir.path());
        assert!(result.is_ok());
        let steering = result.unwrap();
        assert!(steering.rules.is_empty());
        assert!(steering.standards.is_empty());
        assert!(steering.templates.is_empty());
    }

    #[test]
    fn test_steering_loader_nonexistent_directory() {
        let path = Path::new("/nonexistent/steering/path");
        let result = SteeringLoader::load(path);
        assert!(result.is_ok());
        let steering = result.unwrap();
        assert!(steering.rules.is_empty());
    }

    #[test]
    fn test_steering_loader_yaml_file() {
        let temp_dir = TempDir::new().unwrap();
        let steering_file = temp_dir.path().join("steering.yaml");

        let yaml_content = r#"
rules:
  - id: rule-1
    description: Use snake_case for variables
    pattern: "^[a-z_]+$"
    action: enforce
standards:
  - id: std-1
    description: All public APIs must have tests
templates:
  - id: tpl-1
    path: templates/entity.rs
"#;

        fs::write(&steering_file, yaml_content).unwrap();

        let result = SteeringLoader::load(temp_dir.path());
        assert!(result.is_ok());
        let steering = result.unwrap();
        assert_eq!(steering.rules.len(), 1);
        assert_eq!(steering.standards.len(), 1);
        assert_eq!(steering.templates.len(), 1);
        assert_eq!(steering.rules[0].id, "rule-1");
        assert_eq!(steering.standards[0].id, "std-1");
        assert_eq!(steering.templates[0].id, "tpl-1");
    }

    #[test]
    fn test_steering_merge_project_overrides_global() {
        let global = Steering {
            rules: vec![
                SteeringRule {
                    id: "rule-1".to_string(),
                    description: "Global rule".to_string(),
                    pattern: "global".to_string(),
                    action: "enforce".to_string(),
                },
                SteeringRule {
                    id: "rule-2".to_string(),
                    description: "Only in global".to_string(),
                    pattern: "pattern".to_string(),
                    action: "enforce".to_string(),
                },
            ],
            standards: vec![],
            templates: vec![],
        };

        let project = Steering {
            rules: vec![
                SteeringRule {
                    id: "rule-1".to_string(),
                    description: "Project rule".to_string(),
                    pattern: "project".to_string(),
                    action: "enforce".to_string(),
                },
                SteeringRule {
                    id: "rule-3".to_string(),
                    description: "Only in project".to_string(),
                    pattern: "pattern".to_string(),
                    action: "enforce".to_string(),
                },
            ],
            standards: vec![],
            templates: vec![],
        };

        let result = SteeringLoader::merge(&global, &project);
        assert!(result.is_ok());
        let merged = result.unwrap();

        // Should have 3 rules: rule-1 (project version), rule-2 (global), rule-3 (project)
        assert_eq!(merged.rules.len(), 3);

        // Find rule-1 and verify it's the project version
        let rule_1 = merged.rules.iter().find(|r| r.id == "rule-1").unwrap();
        assert_eq!(rule_1.description, "Project rule");
        assert_eq!(rule_1.pattern, "project");
    }

    #[test]
    fn test_steering_merge_empty_global() {
        let global = Steering {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let project = Steering {
            rules: vec![SteeringRule {
                id: "rule-1".to_string(),
                description: "Project rule".to_string(),
                pattern: "pattern".to_string(),
                action: "enforce".to_string(),
            }],
            standards: vec![],
            templates: vec![],
        };

        let result = SteeringLoader::merge(&global, &project);
        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged.rules.len(), 1);
        assert_eq!(merged.rules[0].id, "rule-1");
    }

    #[test]
    fn test_steering_merge_empty_project() {
        let global = Steering {
            rules: vec![SteeringRule {
                id: "rule-1".to_string(),
                description: "Global rule".to_string(),
                pattern: "pattern".to_string(),
                action: "enforce".to_string(),
            }],
            standards: vec![],
            templates: vec![],
        };

        let project = Steering {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let result = SteeringLoader::merge(&global, &project);
        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged.rules.len(), 1);
        assert_eq!(merged.rules[0].id, "rule-1");
    }

    #[test]
    fn test_steering_validate_valid() {
        let steering = Steering {
            rules: vec![SteeringRule {
                id: "rule-1".to_string(),
                description: "Valid rule".to_string(),
                pattern: "pattern".to_string(),
                action: "enforce".to_string(),
            }],
            standards: vec![Standard {
                id: "std-1".to_string(),
                description: "Valid standard".to_string(),
            }],
            templates: vec![TemplateRef {
                id: "tpl-1".to_string(),
                path: "templates/entity.rs".to_string(),
            }],
        };

        let result = SteeringLoader::validate(&steering);
        assert!(result.is_ok());
    }

    #[test]
    fn test_steering_validate_duplicate_rule_ids() {
        let steering = Steering {
            rules: vec![
                SteeringRule {
                    id: "rule-1".to_string(),
                    description: "First rule".to_string(),
                    pattern: "pattern".to_string(),
                    action: "enforce".to_string(),
                },
                SteeringRule {
                    id: "rule-1".to_string(),
                    description: "Duplicate rule".to_string(),
                    pattern: "pattern".to_string(),
                    action: "enforce".to_string(),
                },
            ],
            standards: vec![],
            templates: vec![],
        };

        let result = SteeringLoader::validate(&steering);
        assert!(result.is_err());
    }

    #[test]
    fn test_steering_validate_empty_rule_description() {
        let steering = Steering {
            rules: vec![SteeringRule {
                id: "rule-1".to_string(),
                description: String::new(),
                pattern: "pattern".to_string(),
                action: "enforce".to_string(),
            }],
            standards: vec![],
            templates: vec![],
        };

        let result = SteeringLoader::validate(&steering);
        // Should succeed but with warnings
        assert!(result.is_ok());
    }

    #[test]
    fn test_steering_validate_empty_template_path() {
        let steering = Steering {
            rules: vec![],
            standards: vec![],
            templates: vec![TemplateRef {
                id: "tpl-1".to_string(),
                path: String::new(),
            }],
        };

        let result = SteeringLoader::validate(&steering);
        // Should succeed but with warnings
        assert!(result.is_ok());
    }
}
