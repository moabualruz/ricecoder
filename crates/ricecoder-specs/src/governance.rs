//! Governance file loading and management

use std::{fs, path::Path};

use crate::{
    error::{Severity, SpecError, ValidationError},
    models::{Standard, Governance, GovernanceRule, TemplateRef},
};

/// Loads and merges Governance documents
pub struct GovernanceLoader;

impl GovernanceLoader {
    /// Load Governance from a directory
    ///
    /// Searches for YAML and Markdown files in the directory and loads them.
    /// Returns a merged Governance document with all rules, standards, and templates.
    ///
    /// # Arguments
    /// * `path` - Directory path to search for Governance files
    ///
    /// # Returns
    /// * `Ok(Governance)` - Merged Governance document
    /// * `Err(SpecError)` - If loading or parsing fails
    pub fn load(path: &Path) -> Result<Governance, SpecError> {
        if !path.exists() {
            return Ok(Governance {
                rules: vec![],
                standards: vec![],
                templates: vec![],
            });
        }

        if !path.is_dir() {
            return Err(SpecError::InvalidFormat(format!(
                "Governance path is not a directory: {}",
                path.display()
            )));
        }

        let mut all_rules = vec![];
        let mut all_standards = vec![];
        let mut all_templates = vec![];

        // Recursively search for Governance files
        Self::load_from_directory(path, &mut all_rules, &mut all_standards, &mut all_templates)?;

        Ok(Governance {
            rules: all_rules,
            standards: all_standards,
            templates: all_templates,
        })
    }

    /// Load Governance files from a directory recursively
    fn load_from_directory(
        dir: &Path,
        rules: &mut Vec<GovernanceRule>,
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
                    let Governance = Self::parse_yaml(&content, &path)?;
                    rules.extend(Governance.rules);
                    standards.extend(Governance.standards);
                    templates.extend(Governance.templates);
                } else if file_name.ends_with(".md") {
                    let content = fs::read_to_string(&path).map_err(SpecError::IoError)?;
                    let Governance = Self::parse_markdown(&content, &path)?;
                    rules.extend(Governance.rules);
                    standards.extend(Governance.standards);
                    templates.extend(Governance.templates);
                }
            }
        }

        Ok(())
    }

    /// Parse YAML Governance file
    fn parse_yaml(content: &str, path: &Path) -> Result<Governance, SpecError> {
        serde_yaml::from_str(content).map_err(|e| SpecError::ParseError {
            path: path.display().to_string(),
            line: e.location().map(|l| l.line()).unwrap_or(0),
            message: e.to_string(),
        })
    }

    /// Parse Markdown Governance file
    fn parse_markdown(content: &str, path: &Path) -> Result<Governance, SpecError> {
        // For now, try to parse as YAML embedded in markdown
        // In a full implementation, this would extract YAML code blocks
        // or parse markdown-specific Governance format

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

        // If no YAML block found, return empty Governance
        Ok(Governance {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        })
    }

    /// Merge global and project Governance
    ///
    /// Project Governance takes precedence over global Governance.
    /// Rules, standards, and templates are merged with project items overriding global items
    /// when they have the same ID.
    ///
    /// # Arguments
    /// * `global` - Global Governance document
    /// * `project` - Project Governance document
    ///
    /// # Returns
    /// * `Ok(Governance)` - Merged Governance document with project taking precedence
    /// * `Err(SpecError)` - If merge fails
    pub fn merge(global: &Governance, project: &Governance) -> Result<Governance, SpecError> {
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

        Ok(Governance {
            rules: merged_rules,
            standards: merged_standards,
            templates: merged_templates,
        })
    }

    /// Validate Governance syntax and semantics
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
    /// * `Governance` - Governance document to validate
    ///
    /// # Returns
    /// * `Ok(())` - If Governance is valid
    /// * `Err(SpecError)` - If validation fails
    pub fn validate(Governance: &Governance) -> Result<(), SpecError> {
        let mut errors = vec![];

        // Check for duplicate rule IDs
        let mut rule_ids = std::collections::HashSet::new();
        for (idx, rule) in Governance.rules.iter().enumerate() {
            if !rule_ids.insert(&rule.id) {
                errors.push(ValidationError {
                    path: "Governance".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Duplicate rule ID: {}", rule.id),
                    severity: Severity::Error,
                });
            }

            // Check rule has description
            if rule.description.is_empty() {
                errors.push(ValidationError {
                    path: "Governance".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Rule {} has empty description", rule.id),
                    severity: Severity::Warning,
                });
            }

            // Check rule has pattern
            if rule.pattern.is_empty() {
                errors.push(ValidationError {
                    path: "Governance".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Rule {} has empty pattern", rule.id),
                    severity: Severity::Warning,
                });
            }
        }

        // Check for duplicate standard IDs
        let mut standard_ids = std::collections::HashSet::new();
        for (idx, standard) in Governance.standards.iter().enumerate() {
            if !standard_ids.insert(&standard.id) {
                errors.push(ValidationError {
                    path: "Governance".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Duplicate standard ID: {}", standard.id),
                    severity: Severity::Error,
                });
            }

            // Check standard has description
            if standard.description.is_empty() {
                errors.push(ValidationError {
                    path: "Governance".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Standard {} has empty description", standard.id),
                    severity: Severity::Warning,
                });
            }
        }

        // Check for duplicate template IDs
        let mut template_ids = std::collections::HashSet::new();
        for (idx, template) in Governance.templates.iter().enumerate() {
            if !template_ids.insert(&template.id) {
                errors.push(ValidationError {
                    path: "Governance".to_string(),
                    line: idx + 1,
                    column: 0,
                    message: format!("Duplicate template ID: {}", template.id),
                    severity: Severity::Error,
                });
            }

            // Check template has path
            if template.path.is_empty() {
                errors.push(ValidationError {
                    path: "Governance".to_string(),
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
    fn test_governance_loader_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = GovernanceLoader::load(temp_dir.path());
        assert!(result.is_ok());
        let Governance = result.unwrap();
        assert!(Governance.rules.is_empty());
        assert!(Governance.standards.is_empty());
        assert!(Governance.templates.is_empty());
    }

    #[test]
    fn test_governance_loader_nonexistent_directory() {
        let path = Path::new("/nonexistent/Governance/path");
        let result = GovernanceLoader::load(path);
        assert!(result.is_ok());
        let Governance = result.unwrap();
        assert!(Governance.rules.is_empty());
    }

    #[test]
    fn test_governance_loader_yaml_file() {
        let temp_dir = TempDir::new().unwrap();
        let governance_file = temp_dir.path().join("Governance.yaml");

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

        fs::write(&governance_file, yaml_content).unwrap();

        let result = GovernanceLoader::load(temp_dir.path());
        assert!(result.is_ok());
        let Governance = result.unwrap();
        assert_eq!(Governance.rules.len(), 1);
        assert_eq!(Governance.standards.len(), 1);
        assert_eq!(Governance.templates.len(), 1);
        assert_eq!(Governance.rules[0].id, "rule-1");
        assert_eq!(Governance.standards[0].id, "std-1");
        assert_eq!(Governance.templates[0].id, "tpl-1");
    }

    #[test]
    fn test_governance_merge_project_overrides_global() {
        let global = Governance {
            rules: vec![
                GovernanceRule {
                    id: "rule-1".to_string(),
                    description: "Global rule".to_string(),
                    pattern: "global".to_string(),
                    action: "enforce".to_string(),
                },
                GovernanceRule {
                    id: "rule-2".to_string(),
                    description: "Only in global".to_string(),
                    pattern: "pattern".to_string(),
                    action: "enforce".to_string(),
                },
            ],
            standards: vec![],
            templates: vec![],
        };

        let project = Governance {
            rules: vec![
                GovernanceRule {
                    id: "rule-1".to_string(),
                    description: "Project rule".to_string(),
                    pattern: "project".to_string(),
                    action: "enforce".to_string(),
                },
                GovernanceRule {
                    id: "rule-3".to_string(),
                    description: "Only in project".to_string(),
                    pattern: "pattern".to_string(),
                    action: "enforce".to_string(),
                },
            ],
            standards: vec![],
            templates: vec![],
        };

        let result = GovernanceLoader::merge(&global, &project);
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
    fn test_governance_merge_empty_global() {
        let global = Governance {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let project = Governance {
            rules: vec![GovernanceRule {
                id: "rule-1".to_string(),
                description: "Project rule".to_string(),
                pattern: "pattern".to_string(),
                action: "enforce".to_string(),
            }],
            standards: vec![],
            templates: vec![],
        };

        let result = GovernanceLoader::merge(&global, &project);
        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged.rules.len(), 1);
        assert_eq!(merged.rules[0].id, "rule-1");
    }

    #[test]
    fn test_governance_merge_empty_project() {
        let global = Governance {
            rules: vec![GovernanceRule {
                id: "rule-1".to_string(),
                description: "Global rule".to_string(),
                pattern: "pattern".to_string(),
                action: "enforce".to_string(),
            }],
            standards: vec![],
            templates: vec![],
        };

        let project = Governance {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let result = GovernanceLoader::merge(&global, &project);
        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged.rules.len(), 1);
        assert_eq!(merged.rules[0].id, "rule-1");
    }

    #[test]
    fn test_governance_validate_valid() {
        let Governance = Governance {
            rules: vec![GovernanceRule {
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

        let result = GovernanceLoader::validate(&Governance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_governance_validate_duplicate_rule_ids() {
        let Governance = Governance {
            rules: vec![
                GovernanceRule {
                    id: "rule-1".to_string(),
                    description: "First rule".to_string(),
                    pattern: "pattern".to_string(),
                    action: "enforce".to_string(),
                },
                GovernanceRule {
                    id: "rule-1".to_string(),
                    description: "Duplicate rule".to_string(),
                    pattern: "pattern".to_string(),
                    action: "enforce".to_string(),
                },
            ],
            standards: vec![],
            templates: vec![],
        };

        let result = GovernanceLoader::validate(&Governance);
        assert!(result.is_err());
    }

    #[test]
    fn test_governance_validate_empty_rule_description() {
        let Governance = Governance {
            rules: vec![GovernanceRule {
                id: "rule-1".to_string(),
                description: String::new(),
                pattern: "pattern".to_string(),
                action: "enforce".to_string(),
            }],
            standards: vec![],
            templates: vec![],
        };

        let result = GovernanceLoader::validate(&Governance);
        // Should succeed but with warnings
        assert!(result.is_ok());
    }

    #[test]
    fn test_governance_validate_empty_template_path() {
        let Governance = Governance {
            rules: vec![],
            standards: vec![],
            templates: vec![TemplateRef {
                id: "tpl-1".to_string(),
                path: String::new(),
            }],
        };

        let result = GovernanceLoader::validate(&Governance);
        // Should succeed but with warnings
        assert!(result.is_ok());
    }
}
