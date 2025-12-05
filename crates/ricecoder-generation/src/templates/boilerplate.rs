//! Boilerplate management for project scaffolding
//!
//! Handles loading, parsing, validating, and applying boilerplates to create new projects.
//! Supports variable customization, file conflict resolution, and custom boilerplate creation.

use crate::models::{Boilerplate, BoilerplateFile, BoilerplateSource, ConflictResolution};
use crate::templates::discovery::BoilerplateDiscovery;
use crate::templates::error::BoilerplateError;
use crate::templates::resolver::{CaseTransform, PlaceholderResolver};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Manages boilerplate operations: loading, validation, and application
///
/// Implements requirements:
/// - Requirement 3.1: Load and parse boilerplate definitions
/// - Requirement 3.2: Validate boilerplate before application
/// - Requirement 3.3: Prompt for variable customization
/// - Requirement 3.4: Handle file conflicts (skip/overwrite/merge)
/// - Requirement 3.5: Support custom boilerplate creation
/// - Requirement 3.6: Validate boilerplate structure
pub struct BoilerplateManager;

impl BoilerplateManager {
    /// Create a new boilerplate manager
    pub fn new() -> Self {
        Self
    }

    /// Load a boilerplate from a directory
    ///
    /// Parses the boilerplate metadata and validates the structure.
    ///
    /// # Arguments
    /// * `boilerplate_path` - Path to the boilerplate directory
    ///
    /// # Returns
    /// Loaded boilerplate or error
    ///
    /// # Requirements
    /// - Requirement 3.1: Load and parse boilerplate definitions
    /// - Requirement 3.2: Validate boilerplate before application
    pub fn load(&self, boilerplate_path: &Path) -> Result<Boilerplate, BoilerplateError> {
        // Validate boilerplate structure
        BoilerplateDiscovery::validate_boilerplate(boilerplate_path)?;

        // Parse metadata
        let boilerplate = BoilerplateDiscovery::parse_metadata(boilerplate_path)?;

        Ok(boilerplate)
    }

    /// Load a boilerplate by name from discovery
    ///
    /// Searches for the boilerplate in project and global scopes.
    ///
    /// # Arguments
    /// * `project_root` - Root directory of the project
    /// * `boilerplate_name` - Name of the boilerplate to load
    ///
    /// # Returns
    /// Loaded boilerplate or error
    pub fn load_by_name(
        &self,
        project_root: &Path,
        boilerplate_name: &str,
    ) -> Result<Boilerplate, BoilerplateError> {
        let discovery_result = BoilerplateDiscovery::discover(project_root)?;

        // Find the boilerplate by name (project scope takes precedence)
        let metadata = discovery_result
            .boilerplates
            .iter()
            .find(|bp| bp.name.to_lowercase() == boilerplate_name.to_lowercase())
            .ok_or_else(|| BoilerplateError::NotFound(boilerplate_name.to_string()))?;

        // Get the path from the source
        let path = match &metadata.source {
            BoilerplateSource::Global(p) | BoilerplateSource::Project(p) => p,
        };

        self.load(path)
    }

    /// Validate a boilerplate structure
    ///
    /// Checks that the boilerplate has required fields and valid structure.
    ///
    /// # Arguments
    /// * `boilerplate` - Boilerplate to validate
    ///
    /// # Returns
    /// Ok if valid, error otherwise
    ///
    /// # Requirements
    /// - Requirement 3.2: Validate boilerplate before application
    pub fn validate(&self, boilerplate: &Boilerplate) -> Result<(), BoilerplateError> {
        // Check required fields
        if boilerplate.id.is_empty() {
            return Err(BoilerplateError::ValidationFailed(
                "Boilerplate ID cannot be empty".to_string(),
            ));
        }

        if boilerplate.name.is_empty() {
            return Err(BoilerplateError::ValidationFailed(
                "Boilerplate name cannot be empty".to_string(),
            ));
        }

        if boilerplate.language.is_empty() {
            return Err(BoilerplateError::ValidationFailed(
                "Boilerplate language cannot be empty".to_string(),
            ));
        }

        // Check that files have valid paths
        for file in &boilerplate.files {
            if file.path.is_empty() {
                return Err(BoilerplateError::ValidationFailed(
                    "Boilerplate file path cannot be empty".to_string(),
                ));
            }

            if file.template.is_empty() {
                return Err(BoilerplateError::ValidationFailed(
                    "Boilerplate file template cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Extract placeholders from a boilerplate
    ///
    /// Scans all template files in the boilerplate and extracts required placeholders.
    ///
    /// # Arguments
    /// * `boilerplate` - Boilerplate to scan
    ///
    /// # Returns
    /// Map of placeholder names to descriptions
    pub fn extract_placeholders(
        &self,
        boilerplate: &Boilerplate,
    ) -> Result<HashMap<String, String>, BoilerplateError> {
        let mut placeholders = HashMap::new();

        // Compile regex once outside the loop
        let re = regex::Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_-]*)\}\}").unwrap();

        for file in &boilerplate.files {
            // Extract placeholders from template content using regex
            for cap in re.captures_iter(&file.template) {
                if let Some(name) = cap.get(1) {
                    let placeholder_name = name.as_str().to_string();
                    placeholders.insert(placeholder_name, format!("Placeholder in {}", file.path));
                }
            }
        }

        Ok(placeholders)
    }

    /// Apply a boilerplate to create a new project
    ///
    /// Creates the project structure from the boilerplate, rendering templates with provided values.
    ///
    /// # Arguments
    /// * `boilerplate` - Boilerplate to apply
    /// * `target_dir` - Directory where to create the project
    /// * `variables` - Variable values for template substitution
    /// * `conflict_resolution` - How to handle file conflicts
    ///
    /// # Returns
    /// Result of scaffolding operation
    ///
    /// # Requirements
    /// - Requirement 3.1: Create project structure from boilerplate
    /// - Requirement 3.3: Render template files with context
    /// - Requirement 3.4: Handle file conflicts (skip/overwrite/merge)
    pub fn apply(
        &self,
        boilerplate: &Boilerplate,
        target_dir: &Path,
        variables: &HashMap<String, String>,
        conflict_resolution: ConflictResolution,
    ) -> Result<ScaffoldingResult, BoilerplateError> {
        // Validate boilerplate
        self.validate(boilerplate)?;

        // Create target directory if it doesn't exist
        fs::create_dir_all(target_dir).map_err(BoilerplateError::IoError)?;

        let mut created_files = Vec::new();
        let mut skipped_files = Vec::new();
        let mut conflicts = Vec::new();

        // Create files from boilerplate
        for file in &boilerplate.files {
            // Evaluate condition if present
            if let Some(condition) = &file.condition {
                if !self.evaluate_condition(condition, variables) {
                    continue;
                }
            }

            // Render template with variables
            let mut resolver = PlaceholderResolver::new();
            resolver.add_values(variables.clone());
            let rendered_content = self.render_template(&file.template, &resolver)?;

            // Determine target file path
            let file_path = target_dir.join(&file.path);

            // Create parent directories
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).map_err(BoilerplateError::IoError)?;
            }

            // Handle file conflicts
            if file_path.exists() {
                match conflict_resolution {
                    ConflictResolution::Skip => {
                        skipped_files.push(file.path.clone());
                        continue;
                    }
                    ConflictResolution::Overwrite => {
                        // Continue to write the file
                    }
                    ConflictResolution::Merge => {
                        // For now, treat merge as overwrite
                        // In a full implementation, this would merge file contents
                        conflicts.push(FileConflict {
                            path: file.path.clone(),
                            reason: "File already exists".to_string(),
                            resolution: "Merged (overwritten)".to_string(),
                        });
                    }
                }
            }

            // Write file
            fs::write(&file_path, &rendered_content).map_err(BoilerplateError::IoError)?;

            created_files.push(file.path.clone());
        }

        Ok(ScaffoldingResult {
            created_files,
            skipped_files,
            conflicts,
        })
    }

    /// Create a custom boilerplate from a template
    ///
    /// Generates a new boilerplate definition from a template directory.
    ///
    /// # Arguments
    /// * `source_dir` - Directory containing the template files
    /// * `boilerplate_id` - ID for the new boilerplate
    /// * `boilerplate_name` - Name for the new boilerplate
    /// * `language` - Programming language
    ///
    /// # Returns
    /// Created boilerplate or error
    ///
    /// # Requirements
    /// - Requirement 3.5: Support custom boilerplate creation
    pub fn create_custom(
        &self,
        source_dir: &Path,
        boilerplate_id: &str,
        boilerplate_name: &str,
        language: &str,
    ) -> Result<Boilerplate, BoilerplateError> {
        if !source_dir.exists() {
            return Err(BoilerplateError::InvalidStructure(format!(
                "Source directory not found: {}",
                source_dir.display()
            )));
        }

        let mut files = Vec::new();

        // Scan source directory for template files
        self.scan_directory(source_dir, source_dir, &mut files)?;

        Ok(Boilerplate {
            id: boilerplate_id.to_string(),
            name: boilerplate_name.to_string(),
            description: format!("Custom boilerplate for {}", language),
            language: language.to_string(),
            files,
            dependencies: Vec::new(),
            scripts: Vec::new(),
        })
    }

    /// Save a boilerplate to disk
    ///
    /// Writes the boilerplate metadata and files to a directory.
    ///
    /// # Arguments
    /// * `boilerplate` - Boilerplate to save
    /// * `target_dir` - Directory where to save the boilerplate
    ///
    /// # Returns
    /// Ok if successful, error otherwise
    pub fn save(
        &self,
        boilerplate: &Boilerplate,
        target_dir: &Path,
    ) -> Result<(), BoilerplateError> {
        // Validate boilerplate
        self.validate(boilerplate)?;

        // Create target directory
        fs::create_dir_all(target_dir).map_err(BoilerplateError::IoError)?;

        // Write metadata as YAML
        let metadata_path = target_dir.join("boilerplate.yaml");
        let yaml_content = serde_yaml::to_string(boilerplate)
            .map_err(|e| BoilerplateError::InvalidStructure(format!("YAML error: {}", e)))?;

        fs::write(&metadata_path, yaml_content).map_err(BoilerplateError::IoError)?;

        Ok(())
    }

    /// Evaluate a condition string with variables
    ///
    /// Simple condition evaluation for file inclusion.
    /// Supports basic syntax like "variable_name" or "!variable_name"
    ///
    /// # Arguments
    /// * `condition` - Condition string to evaluate
    /// * `variables` - Available variables
    ///
    /// # Returns
    /// True if condition is met, false otherwise
    fn evaluate_condition(&self, condition: &str, variables: &HashMap<String, String>) -> bool {
        let condition = condition.trim();

        // Handle negation
        if let Some(var_name) = condition.strip_prefix('!') {
            return !variables.contains_key(var_name.trim());
        }

        // Check if variable exists and is truthy
        variables
            .get(condition)
            .map(|v| !v.is_empty() && v != "false" && v != "0")
            .unwrap_or(false)
    }

    /// Render a template string with placeholder resolution
    ///
    /// # Arguments
    /// * `template` - Template string with placeholders
    /// * `resolver` - Placeholder resolver with values
    ///
    /// # Returns
    /// Rendered content or error
    fn render_template(
        &self,
        template: &str,
        resolver: &PlaceholderResolver,
    ) -> Result<String, BoilerplateError> {
        let mut result = template.to_string();

        // Find and replace all placeholders
        let re = regex::Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_-]*(?:_snake|-kebab|Camel)?)\}\}")
            .map_err(|e| BoilerplateError::InvalidStructure(format!("Regex error: {}", e)))?;

        for cap in re.captures_iter(template) {
            if let Some(placeholder_match) = cap.get(1) {
                let placeholder_content = placeholder_match.as_str();

                // Parse placeholder syntax to extract name and case transform
                let (name, case_transform) = self.parse_placeholder_syntax(placeholder_content)?;

                // Resolve the placeholder
                let resolved = resolver
                    .resolve(&name, case_transform)
                    .map_err(|e| BoilerplateError::InvalidStructure(e.to_string()))?;

                // Replace in result
                let placeholder_str = format!("{{{{{}}}}}", placeholder_content);
                result = result.replace(&placeholder_str, &resolved);
            }
        }

        Ok(result)
    }

    /// Parse placeholder syntax to extract name and case transform
    ///
    /// # Arguments
    /// * `content` - Placeholder content (e.g., "Name", "name_snake", "name-kebab")
    ///
    /// # Returns
    /// Tuple of (name, case_transform) or error
    fn parse_placeholder_syntax(
        &self,
        content: &str,
    ) -> Result<(String, CaseTransform), BoilerplateError> {
        let content = content.trim();

        // Determine case transform based on suffix
        if content.ends_with("_snake") {
            let name = content.trim_end_matches("_snake").to_string();
            Ok((name, CaseTransform::SnakeCase))
        } else if content.ends_with("-kebab") {
            let name = content.trim_end_matches("-kebab").to_string();
            Ok((name, CaseTransform::KebabCase))
        } else if content.ends_with("Camel") {
            let name = content.trim_end_matches("Camel").to_string();
            Ok((name, CaseTransform::CamelCase))
        } else if content.chars().all(|c| c.is_uppercase() || c == '_') && content.len() > 1 {
            // All uppercase = UPPERCASE transform
            Ok((content.to_string(), CaseTransform::UpperCase))
        } else if content.chars().next().is_some_and(|c| c.is_uppercase()) {
            // Starts with uppercase = PascalCase
            Ok((content.to_string(), CaseTransform::PascalCase))
        } else {
            // Default to lowercase
            Ok((content.to_string(), CaseTransform::LowerCase))
        }
    }

    /// Recursively scan a directory for template files
    ///
    /// # Arguments
    /// * `current_dir` - Current directory being scanned
    /// * `base_dir` - Base directory for relative path calculation
    /// * `files` - Accumulator for found files
    ///
    /// # Returns
    /// Ok if successful, error otherwise
    #[allow(clippy::only_used_in_recursion)]
    fn scan_directory(
        &self,
        current_dir: &Path,
        base_dir: &Path,
        files: &mut Vec<BoilerplateFile>,
    ) -> Result<(), BoilerplateError> {
        for entry in fs::read_dir(current_dir).map_err(BoilerplateError::IoError)? {
            let entry = entry.map_err(BoilerplateError::IoError)?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                self.scan_directory(&path, base_dir, files)?;
            } else if path.is_file() {
                // Read file content
                let content = fs::read_to_string(&path).map_err(BoilerplateError::IoError)?;

                // Calculate relative path
                let relative_path = path
                    .strip_prefix(base_dir)
                    .map_err(|_| {
                        BoilerplateError::InvalidStructure(
                            "Failed to calculate relative path".to_string(),
                        )
                    })?
                    .to_string_lossy()
                    .to_string();

                files.push(BoilerplateFile {
                    path: relative_path,
                    template: content,
                    condition: None,
                });
            }
        }

        Ok(())
    }
}

impl Default for BoilerplateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of scaffolding operation
#[derive(Debug, Clone)]
pub struct ScaffoldingResult {
    /// Files that were created
    pub created_files: Vec<String>,
    /// Files that were skipped due to conflicts
    pub skipped_files: Vec<String>,
    /// File conflicts that occurred
    pub conflicts: Vec<FileConflict>,
}

/// Information about a file conflict
#[derive(Debug, Clone)]
pub struct FileConflict {
    /// Path to the conflicting file
    pub path: String,
    /// Reason for the conflict
    pub reason: String,
    /// How the conflict was resolved
    pub resolution: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_boilerplate_manager() {
        let _manager = BoilerplateManager::new();
        // Manager is created successfully
    }

    #[test]
    fn test_validate_boilerplate_success() {
        let manager = BoilerplateManager::new();
        let boilerplate = Boilerplate {
            id: "test-bp".to_string(),
            name: "Test Boilerplate".to_string(),
            description: "A test boilerplate".to_string(),
            language: "rust".to_string(),
            files: vec![BoilerplateFile {
                path: "src/main.rs".to_string(),
                template: "fn main() {}".to_string(),
                condition: None,
            }],
            dependencies: vec![],
            scripts: vec![],
        };

        assert!(manager.validate(&boilerplate).is_ok());
    }

    #[test]
    fn test_validate_boilerplate_missing_id() {
        let manager = BoilerplateManager::new();
        let boilerplate = Boilerplate {
            id: "".to_string(),
            name: "Test Boilerplate".to_string(),
            description: "A test boilerplate".to_string(),
            language: "rust".to_string(),
            files: vec![],
            dependencies: vec![],
            scripts: vec![],
        };

        assert!(manager.validate(&boilerplate).is_err());
    }

    #[test]
    fn test_validate_boilerplate_missing_name() {
        let manager = BoilerplateManager::new();
        let boilerplate = Boilerplate {
            id: "test-bp".to_string(),
            name: "".to_string(),
            description: "A test boilerplate".to_string(),
            language: "rust".to_string(),
            files: vec![],
            dependencies: vec![],
            scripts: vec![],
        };

        assert!(manager.validate(&boilerplate).is_err());
    }

    #[test]
    fn test_validate_boilerplate_missing_language() {
        let manager = BoilerplateManager::new();
        let boilerplate = Boilerplate {
            id: "test-bp".to_string(),
            name: "Test Boilerplate".to_string(),
            description: "A test boilerplate".to_string(),
            language: "".to_string(),
            files: vec![],
            dependencies: vec![],
            scripts: vec![],
        };

        assert!(manager.validate(&boilerplate).is_err());
    }

    #[test]
    fn test_extract_placeholders() {
        let manager = BoilerplateManager::new();
        let boilerplate = Boilerplate {
            id: "test-bp".to_string(),
            name: "Test Boilerplate".to_string(),
            description: "A test boilerplate".to_string(),
            language: "rust".to_string(),
            files: vec![
                BoilerplateFile {
                    path: "src/main.rs".to_string(),
                    template: "pub struct {{Name}} {}".to_string(),
                    condition: None,
                },
                BoilerplateFile {
                    path: "Cargo.toml".to_string(),
                    template: "[package]\nname = \"{{name_snake}}\"".to_string(),
                    condition: None,
                },
            ],
            dependencies: vec![],
            scripts: vec![],
        };

        let placeholders = manager.extract_placeholders(&boilerplate).unwrap();
        assert!(placeholders.contains_key("Name"));
        assert!(placeholders.contains_key("name_snake"));
    }

    #[test]
    fn test_apply_boilerplate_skip_conflicts() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BoilerplateManager::new();

        // Create existing file
        let existing_file = temp_dir.path().join("src").join("main.rs");
        fs::create_dir_all(existing_file.parent().unwrap()).unwrap();
        fs::write(&existing_file, "// existing").unwrap();

        let boilerplate = Boilerplate {
            id: "test-bp".to_string(),
            name: "Test Boilerplate".to_string(),
            description: "A test boilerplate".to_string(),
            language: "rust".to_string(),
            files: vec![BoilerplateFile {
                path: "src/main.rs".to_string(),
                template: "fn main() {}".to_string(),
                condition: None,
            }],
            dependencies: vec![],
            scripts: vec![],
        };

        let variables = HashMap::new();
        let result = manager
            .apply(
                &boilerplate,
                temp_dir.path(),
                &variables,
                ConflictResolution::Skip,
            )
            .unwrap();

        assert_eq!(result.skipped_files.len(), 1);
        assert_eq!(result.created_files.len(), 0);

        // Verify existing file was not overwritten
        let content = fs::read_to_string(&existing_file).unwrap();
        assert_eq!(content, "// existing");
    }

    #[test]
    fn test_apply_boilerplate_overwrite_conflicts() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BoilerplateManager::new();

        // Create existing file
        let existing_file = temp_dir.path().join("src").join("main.rs");
        fs::create_dir_all(existing_file.parent().unwrap()).unwrap();
        fs::write(&existing_file, "// existing").unwrap();

        let boilerplate = Boilerplate {
            id: "test-bp".to_string(),
            name: "Test Boilerplate".to_string(),
            description: "A test boilerplate".to_string(),
            language: "rust".to_string(),
            files: vec![BoilerplateFile {
                path: "src/main.rs".to_string(),
                template: "fn main() {}".to_string(),
                condition: None,
            }],
            dependencies: vec![],
            scripts: vec![],
        };

        let variables = HashMap::new();
        let result = manager
            .apply(
                &boilerplate,
                temp_dir.path(),
                &variables,
                ConflictResolution::Overwrite,
            )
            .unwrap();

        assert_eq!(result.created_files.len(), 1);
        assert_eq!(result.skipped_files.len(), 0);

        // Verify file was overwritten
        let content = fs::read_to_string(&existing_file).unwrap();
        assert_eq!(content, "fn main() {}");
    }

    #[test]
    fn test_apply_boilerplate_with_variables() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BoilerplateManager::new();

        let boilerplate = Boilerplate {
            id: "test-bp".to_string(),
            name: "Test Boilerplate".to_string(),
            description: "A test boilerplate".to_string(),
            language: "rust".to_string(),
            files: vec![BoilerplateFile {
                path: "src/main.rs".to_string(),
                template: "pub struct {{Name}} {}".to_string(),
                condition: None,
            }],
            dependencies: vec![],
            scripts: vec![],
        };

        let mut variables = HashMap::new();
        variables.insert("Name".to_string(), "MyStruct".to_string());

        let result = manager
            .apply(
                &boilerplate,
                temp_dir.path(),
                &variables,
                ConflictResolution::Skip,
            )
            .unwrap();

        assert_eq!(result.created_files.len(), 1);

        let content = fs::read_to_string(temp_dir.path().join("src/main.rs")).unwrap();
        assert!(content.contains("MyStruct"));
    }

    #[test]
    fn test_apply_boilerplate_with_condition() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BoilerplateManager::new();

        let boilerplate = Boilerplate {
            id: "test-bp".to_string(),
            name: "Test Boilerplate".to_string(),
            description: "A test boilerplate".to_string(),
            language: "rust".to_string(),
            files: vec![
                BoilerplateFile {
                    path: "src/main.rs".to_string(),
                    template: "fn main() {}".to_string(),
                    condition: Some("include_main".to_string()),
                },
                BoilerplateFile {
                    path: "src/lib.rs".to_string(),
                    template: "pub fn lib() {}".to_string(),
                    condition: Some("include_lib".to_string()),
                },
            ],
            dependencies: vec![],
            scripts: vec![],
        };

        let mut variables = HashMap::new();
        variables.insert("include_main".to_string(), "true".to_string());

        let result = manager
            .apply(
                &boilerplate,
                temp_dir.path(),
                &variables,
                ConflictResolution::Skip,
            )
            .unwrap();

        assert_eq!(result.created_files.len(), 1);
        assert!(temp_dir.path().join("src/main.rs").exists());
        assert!(!temp_dir.path().join("src/lib.rs").exists());
    }

    #[test]
    fn test_create_custom_boilerplate() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BoilerplateManager::new();

        // Create source files
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
        fs::write(src_dir.join("lib.rs"), "pub fn lib() {}").unwrap();

        let boilerplate = manager
            .create_custom(temp_dir.path(), "custom-bp", "Custom Boilerplate", "rust")
            .unwrap();

        assert_eq!(boilerplate.id, "custom-bp");
        assert_eq!(boilerplate.name, "Custom Boilerplate");
        assert_eq!(boilerplate.language, "rust");
        assert_eq!(boilerplate.files.len(), 2);
    }

    #[test]
    fn test_save_boilerplate() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BoilerplateManager::new();

        let boilerplate = Boilerplate {
            id: "test-bp".to_string(),
            name: "Test Boilerplate".to_string(),
            description: "A test boilerplate".to_string(),
            language: "rust".to_string(),
            files: vec![BoilerplateFile {
                path: "src/main.rs".to_string(),
                template: "fn main() {}".to_string(),
                condition: None,
            }],
            dependencies: vec![],
            scripts: vec![],
        };

        let save_dir = temp_dir.path().join("saved-bp");
        manager.save(&boilerplate, &save_dir).unwrap();

        assert!(save_dir.join("boilerplate.yaml").exists());
    }

    #[test]
    fn test_evaluate_condition_true() {
        let manager = BoilerplateManager::new();
        let mut variables = HashMap::new();
        variables.insert("include_feature".to_string(), "true".to_string());

        assert!(manager.evaluate_condition("include_feature", &variables));
    }

    #[test]
    fn test_evaluate_condition_false() {
        let manager = BoilerplateManager::new();
        let mut variables = HashMap::new();
        variables.insert("include_feature".to_string(), "false".to_string());

        assert!(!manager.evaluate_condition("include_feature", &variables));
    }

    #[test]
    fn test_evaluate_condition_negation() {
        let manager = BoilerplateManager::new();
        let variables = HashMap::new();

        assert!(manager.evaluate_condition("!include_feature", &variables));
    }
}
