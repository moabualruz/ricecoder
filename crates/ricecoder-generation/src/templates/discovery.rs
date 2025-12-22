//! Template and boilerplate discovery and metadata parsing
//!
//! Scans template and boilerplate directories, parses metadata, and returns available resources.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    models::{
        Boilerplate, BoilerplateDiscoveryResult, BoilerplateMetadata, BoilerplateSource, Template,
        TemplateMetadata,
    },
    templates::{
        error::{BoilerplateError, TemplateError},
        loader::TemplateLoader,
    },
};

/// Discovers available templates in project and global scopes
pub struct TemplateDiscovery {
    loader: TemplateLoader,
}

impl TemplateDiscovery {
    /// Create a new template discovery service
    pub fn new() -> Self {
        Self {
            loader: TemplateLoader::new(),
        }
    }

    /// Discover all available templates
    ///
    /// Searches in both project and global scopes, with project templates
    /// taking precedence over global templates with the same name.
    ///
    /// # Arguments
    /// * `project_root` - Root directory of the project
    ///
    /// # Returns
    /// Vector of discovered templates with metadata
    pub fn discover(&mut self, project_root: &Path) -> Result<DiscoveryResult, TemplateError> {
        let mut search_paths = Vec::new();
        let mut templates = Vec::new();

        // Search global templates
        let global_dir = self.get_global_templates_dir();
        if global_dir.exists() {
            search_paths.push(global_dir.clone());
            let global_templates = self.loader.load_global_templates()?;
            templates.extend(global_templates);
        }

        // Search project templates (override global ones)
        let project_dir = project_root.join(".ricecoder").join("templates");
        if project_dir.exists() {
            search_paths.push(project_dir.clone());
            let project_templates = self.loader.load_project_templates(project_root)?;

            // Create a map for deduplication
            let mut template_map: std::collections::HashMap<String, Template> =
                templates.into_iter().map(|t| (t.id.clone(), t)).collect();

            // Override with project templates
            for template in project_templates {
                template_map.insert(template.id.clone(), template);
            }

            templates = template_map.into_values().collect();
        }

        Ok(DiscoveryResult {
            templates,
            search_paths,
        })
    }

    /// Discover templates by language
    ///
    /// # Arguments
    /// * `project_root` - Root directory of the project
    /// * `language` - Programming language to filter by (e.g., "rs", "ts", "py")
    ///
    /// # Returns
    /// Vector of templates for the specified language
    pub fn discover_by_language(
        &mut self,
        project_root: &Path,
        language: &str,
    ) -> Result<Vec<Template>, TemplateError> {
        let result = self.discover(project_root)?;
        Ok(result
            .templates
            .into_iter()
            .filter(|t| t.language == language)
            .collect())
    }

    /// Discover templates by name pattern
    ///
    /// # Arguments
    /// * `project_root` - Root directory of the project
    /// * `pattern` - Name pattern to match (case-insensitive substring match)
    ///
    /// # Returns
    /// Vector of templates matching the pattern
    pub fn discover_by_name(
        &mut self,
        project_root: &Path,
        pattern: &str,
    ) -> Result<Vec<Template>, TemplateError> {
        let result = self.discover(project_root)?;
        let pattern_lower = pattern.to_lowercase();
        Ok(result
            .templates
            .into_iter()
            .filter(|t| t.name.to_lowercase().contains(&pattern_lower))
            .collect())
    }

    /// Parse template metadata from file
    ///
    /// Looks for metadata in template comments or a separate metadata file.
    ///
    /// # Arguments
    /// * `template_path` - Path to the template file
    ///
    /// # Returns
    /// Template metadata or error
    pub fn parse_metadata(&self, template_path: &Path) -> Result<TemplateMetadata, TemplateError> {
        let content = fs::read_to_string(template_path).map_err(TemplateError::IoError)?;

        // Extract metadata from template comments
        let mut metadata = TemplateMetadata {
            description: None,
            version: None,
            author: None,
        };

        // Parse first few lines for metadata comments
        for line in content.lines().take(10) {
            if line.contains("@description") {
                if let Some(desc) = line.split("@description").nth(1) {
                    metadata.description = Some(desc.trim().to_string());
                }
            } else if line.contains("@version") {
                if let Some(ver) = line.split("@version").nth(1) {
                    metadata.version = Some(ver.trim().to_string());
                }
            } else if line.contains("@author") {
                if let Some(auth) = line.split("@author").nth(1) {
                    metadata.author = Some(auth.trim().to_string());
                }
            }
        }

        Ok(metadata)
    }

    /// Validate template structure
    ///
    /// Checks that the template file exists and is readable.
    ///
    /// # Arguments
    /// * `template_path` - Path to the template file
    ///
    /// # Returns
    /// Ok if valid, error otherwise
    pub fn validate_template(&self, template_path: &Path) -> Result<(), TemplateError> {
        if !template_path.exists() {
            return Err(TemplateError::NotFound(format!(
                "Template not found: {}",
                template_path.display()
            )));
        }

        if !template_path.is_file() {
            return Err(TemplateError::ValidationFailed(format!(
                "Template is not a file: {}",
                template_path.display()
            )));
        }

        // Try to read the file
        fs::read_to_string(template_path).map_err(TemplateError::IoError)?;

        Ok(())
    }

    /// Get the global templates directory path
    fn get_global_templates_dir(&self) -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".ricecoder").join("templates")
        } else if let Ok(home) = std::env::var("USERPROFILE") {
            // Windows
            PathBuf::from(home).join(".ricecoder").join("templates")
        } else {
            PathBuf::from(".ricecoder/templates")
        }
    }
}

impl Default for TemplateDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of template discovery
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// Discovered templates
    pub templates: Vec<Template>,
    /// Paths that were searched
    pub search_paths: Vec<PathBuf>,
}

/// Discovers available boilerplates in project and global scopes
///
/// Implements precedence: project scope > global scope
/// When a boilerplate exists in both locations, the project-scoped version is used.
pub struct BoilerplateDiscovery;

impl BoilerplateDiscovery {
    /// Discover all available boilerplates
    ///
    /// Searches in both project and global scopes, with project boilerplates
    /// taking precedence over global boilerplates with the same name.
    ///
    /// # Arguments
    /// * `project_root` - Root directory of the project
    ///
    /// # Returns
    /// Result containing discovered boilerplates and search paths
    ///
    /// # Requirements
    /// - Requirement 4.1: Search `~/.ricecoder/boilerplates`
    /// - Requirement 4.2: Search `./.ricecoder/boilerplates`
    /// - Requirement 4.3: Apply precedence (project > global)
    pub fn discover(project_root: &Path) -> Result<BoilerplateDiscoveryResult, BoilerplateError> {
        let mut search_paths = Vec::new();
        let mut boilerplate_map: HashMap<String, BoilerplateMetadata> = HashMap::new();

        // Search global boilerplates first
        let global_dir = Self::get_global_boilerplates_dir();
        if global_dir.exists() {
            search_paths.push(global_dir.clone());
            let global_boilerplates = Self::scan_boilerplate_directory(&global_dir, true)?;
            for bp in global_boilerplates {
                boilerplate_map.insert(bp.id.clone(), bp);
            }
        }

        // Search project boilerplates (override global ones with same name)
        let project_dir = project_root.join(".ricecoder").join("boilerplates");
        if project_dir.exists() {
            search_paths.push(project_dir.clone());
            let project_boilerplates = Self::scan_boilerplate_directory(&project_dir, false)?;
            for bp in project_boilerplates {
                boilerplate_map.insert(bp.id.clone(), bp);
            }
        }

        let boilerplates: Vec<BoilerplateMetadata> = boilerplate_map.into_values().collect();

        Ok(BoilerplateDiscoveryResult {
            boilerplates,
            search_paths,
        })
    }

    /// Discover boilerplates by language
    ///
    /// # Arguments
    /// * `project_root` - Root directory of the project
    /// * `language` - Programming language to filter by (e.g., "rust", "typescript")
    ///
    /// # Returns
    /// Vector of boilerplates for the specified language
    pub fn discover_by_language(
        project_root: &Path,
        language: &str,
    ) -> Result<Vec<BoilerplateMetadata>, BoilerplateError> {
        let result = Self::discover(project_root)?;
        Ok(result
            .boilerplates
            .into_iter()
            .filter(|bp| bp.language.to_lowercase() == language.to_lowercase())
            .collect())
    }

    /// Discover boilerplates by name pattern
    ///
    /// # Arguments
    /// * `project_root` - Root directory of the project
    /// * `pattern` - Name pattern to match (case-insensitive substring match)
    ///
    /// # Returns
    /// Vector of boilerplates matching the pattern
    pub fn discover_by_name(
        project_root: &Path,
        pattern: &str,
    ) -> Result<Vec<BoilerplateMetadata>, BoilerplateError> {
        let result = Self::discover(project_root)?;
        let pattern_lower = pattern.to_lowercase();
        Ok(result
            .boilerplates
            .into_iter()
            .filter(|bp| bp.name.to_lowercase().contains(&pattern_lower))
            .collect())
    }

    /// Validate boilerplate structure
    ///
    /// Checks that the boilerplate directory contains required files and structure.
    ///
    /// # Arguments
    /// * `boilerplate_path` - Path to the boilerplate directory
    ///
    /// # Returns
    /// Ok if valid, error otherwise
    pub fn validate_boilerplate(boilerplate_path: &Path) -> Result<(), BoilerplateError> {
        if !boilerplate_path.exists() {
            return Err(BoilerplateError::NotFound(format!(
                "Boilerplate not found: {}",
                boilerplate_path.display()
            )));
        }

        if !boilerplate_path.is_dir() {
            return Err(BoilerplateError::InvalidStructure(format!(
                "Boilerplate is not a directory: {}",
                boilerplate_path.display()
            )));
        }

        // Check for boilerplate.yaml or boilerplate.json metadata file
        let yaml_path = boilerplate_path.join("boilerplate.yaml");
        let json_path = boilerplate_path.join("boilerplate.json");

        if !yaml_path.exists() && !json_path.exists() {
            return Err(BoilerplateError::InvalidStructure(format!(
                "Boilerplate missing metadata file (boilerplate.yaml or boilerplate.json): {}",
                boilerplate_path.display()
            )));
        }

        Ok(())
    }

    /// Parse boilerplate metadata from directory
    ///
    /// # Arguments
    /// * `boilerplate_path` - Path to the boilerplate directory
    ///
    /// # Returns
    /// Boilerplate metadata or error
    pub fn parse_metadata(boilerplate_path: &Path) -> Result<Boilerplate, BoilerplateError> {
        Self::validate_boilerplate(boilerplate_path)?;

        // Try to read boilerplate.yaml first, then boilerplate.json
        let yaml_path = boilerplate_path.join("boilerplate.yaml");
        let json_path = boilerplate_path.join("boilerplate.json");

        if yaml_path.exists() {
            let content = fs::read_to_string(&yaml_path).map_err(BoilerplateError::IoError)?;
            serde_yaml::from_str(&content)
                .map_err(|e| BoilerplateError::InvalidStructure(format!("Invalid YAML: {}", e)))
        } else if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(BoilerplateError::IoError)?;
            serde_json::from_str(&content)
                .map_err(|e| BoilerplateError::InvalidStructure(format!("Invalid JSON: {}", e)))
        } else {
            Err(BoilerplateError::InvalidStructure(
                "No boilerplate metadata file found".to_string(),
            ))
        }
    }

    /// Scan a boilerplate directory and return metadata for all boilerplates
    ///
    /// # Arguments
    /// * `directory` - Directory to scan
    /// * `is_global` - Whether this is a global directory
    ///
    /// # Returns
    /// Vector of boilerplate metadata
    fn scan_boilerplate_directory(
        directory: &Path,
        is_global: bool,
    ) -> Result<Vec<BoilerplateMetadata>, BoilerplateError> {
        let mut boilerplates = Vec::new();

        if !directory.exists() {
            return Ok(boilerplates);
        }

        for entry in fs::read_dir(directory).map_err(BoilerplateError::IoError)? {
            let entry = entry.map_err(BoilerplateError::IoError)?;
            let path = entry.path();

            if path.is_dir() {
                // Try to parse boilerplate metadata
                if let Ok(boilerplate) = Self::parse_metadata(&path) {
                    let source = if is_global {
                        BoilerplateSource::Global(path.clone())
                    } else {
                        BoilerplateSource::Project(path.clone())
                    };

                    let metadata = BoilerplateMetadata {
                        id: boilerplate.id,
                        name: boilerplate.name,
                        description: boilerplate.description,
                        language: boilerplate.language,
                        source,
                    };

                    boilerplates.push(metadata);
                }
            }
        }

        Ok(boilerplates)
    }

    /// Get the global boilerplates directory path
    fn get_global_boilerplates_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".ricecoder").join("boilerplates")
        } else if let Ok(home) = std::env::var("USERPROFILE") {
            // Windows
            PathBuf::from(home).join(".ricecoder").join("boilerplates")
        } else {
            PathBuf::from(".ricecoder/boilerplates")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_discover_templates() {
        let temp_dir = TempDir::new().unwrap();
        let templates_dir = temp_dir.path().join(".ricecoder").join("templates");
        fs::create_dir_all(&templates_dir).unwrap();

        fs::write(
            templates_dir.join("struct.rs.tmpl"),
            "pub struct {{Name}} {}",
        )
        .unwrap();
        fs::write(templates_dir.join("impl.rs.tmpl"), "impl {{Name}} {}").unwrap();

        let mut discovery = TemplateDiscovery::new();
        let result = discovery.discover(temp_dir.path()).unwrap();

        assert_eq!(result.templates.len(), 2);
        assert!(result.search_paths.iter().any(|p| p.ends_with("templates")));
    }

    #[test]
    fn test_discover_by_language() {
        let temp_dir = TempDir::new().unwrap();
        let templates_dir = temp_dir.path().join(".ricecoder").join("templates");
        fs::create_dir_all(&templates_dir).unwrap();

        fs::write(
            templates_dir.join("struct.rs.tmpl"),
            "pub struct {{Name}} {}",
        )
        .unwrap();
        fs::write(
            templates_dir.join("component.ts.tmpl"),
            "export const {{Name}} = () => {}",
        )
        .unwrap();

        let mut discovery = TemplateDiscovery::new();
        let rust_templates = discovery
            .discover_by_language(temp_dir.path(), "rs")
            .unwrap();

        assert_eq!(rust_templates.len(), 1);
        assert_eq!(rust_templates[0].language, "rs");
    }

    #[test]
    fn test_discover_by_name() {
        let temp_dir = TempDir::new().unwrap();
        let templates_dir = temp_dir.path().join(".ricecoder").join("templates");
        fs::create_dir_all(&templates_dir).unwrap();

        fs::write(
            templates_dir.join("struct.rs.tmpl"),
            "pub struct {{Name}} {}",
        )
        .unwrap();
        fs::write(templates_dir.join("impl.rs.tmpl"), "impl {{Name}} {}").unwrap();

        let mut discovery = TemplateDiscovery::new();
        let struct_templates = discovery
            .discover_by_name(temp_dir.path(), "struct")
            .unwrap();

        assert_eq!(struct_templates.len(), 1);
        assert_eq!(struct_templates[0].id, "struct");
    }

    #[test]
    fn test_validate_template() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("test.rs.tmpl");
        fs::write(&template_path, "pub struct {{Name}} {}").unwrap();

        let discovery = TemplateDiscovery::new();
        assert!(discovery.validate_template(&template_path).is_ok());
    }

    #[test]
    fn test_validate_nonexistent_template() {
        let discovery = TemplateDiscovery::new();
        let result = discovery.validate_template(Path::new("/nonexistent/template.rs.tmpl"));

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("test.rs.tmpl");

        let content = "// @description A test template\n// @version 1.0.0\n// @author Test Author\npub struct {{Name}} {}";
        fs::write(&template_path, content).unwrap();

        let discovery = TemplateDiscovery::new();
        let metadata = discovery.parse_metadata(&template_path).unwrap();

        assert!(metadata.description.is_some());
        assert!(metadata.version.is_some());
        assert!(metadata.author.is_some());
    }

    // Boilerplate discovery tests

    #[test]
    fn test_validate_boilerplate_missing_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let bp_dir = temp_dir.path().join("test-bp");
        fs::create_dir_all(&bp_dir).unwrap();

        let result = BoilerplateDiscovery::validate_boilerplate(&bp_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_boilerplate_with_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let bp_dir = temp_dir.path().join("test-bp");
        fs::create_dir_all(&bp_dir).unwrap();
        fs::write(bp_dir.join("boilerplate.yaml"), "id: test\nname: Test").unwrap();

        let result = BoilerplateDiscovery::validate_boilerplate(&bp_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_boilerplate_with_json() {
        let temp_dir = TempDir::new().unwrap();
        let bp_dir = temp_dir.path().join("test-bp");
        fs::create_dir_all(&bp_dir).unwrap();
        fs::write(
            bp_dir.join("boilerplate.json"),
            r#"{"id":"test","name":"Test"}"#,
        )
        .unwrap();

        let result = BoilerplateDiscovery::validate_boilerplate(&bp_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_discover_boilerplate_precedence() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join(".ricecoder").join("boilerplates");
        fs::create_dir_all(&project_dir).unwrap();

        // Create a project boilerplate
        let project_bp = project_dir.join("my-bp");
        fs::create_dir_all(&project_bp).unwrap();
        let project_metadata = r#"
id: my-bp
name: My Boilerplate
description: Project version
language: rust
files: []
dependencies: []
scripts: []
"#;
        fs::write(project_bp.join("boilerplate.yaml"), project_metadata).unwrap();

        let result = BoilerplateDiscovery::discover(temp_dir.path()).unwrap();

        // Should find the project boilerplate
        assert!(result.boilerplates.iter().any(|bp| bp.id == "my-bp"));

        // Verify it's marked as project source
        let bp = result
            .boilerplates
            .iter()
            .find(|bp| bp.id == "my-bp")
            .unwrap();
        assert!(matches!(bp.source, BoilerplateSource::Project(_)));
    }

    #[test]
    fn test_discover_boilerplate_by_language() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join(".ricecoder").join("boilerplates");
        fs::create_dir_all(&project_dir).unwrap();

        // Create Rust boilerplate
        let rust_bp = project_dir.join("rust-bp");
        fs::create_dir_all(&rust_bp).unwrap();
        let rust_metadata = r#"
id: rust-bp
name: Rust Boilerplate
description: Rust project
language: rust
files: []
dependencies: []
scripts: []
"#;
        fs::write(rust_bp.join("boilerplate.yaml"), rust_metadata).unwrap();

        // Create TypeScript boilerplate
        let ts_bp = project_dir.join("ts-bp");
        fs::create_dir_all(&ts_bp).unwrap();
        let ts_metadata = r#"
id: ts-bp
name: TypeScript Boilerplate
description: TypeScript project
language: typescript
files: []
dependencies: []
scripts: []
"#;
        fs::write(ts_bp.join("boilerplate.yaml"), ts_metadata).unwrap();

        let result = BoilerplateDiscovery::discover_by_language(temp_dir.path(), "rust").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "rust-bp");
    }

    #[test]
    fn test_discover_boilerplate_by_name() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join(".ricecoder").join("boilerplates");
        fs::create_dir_all(&project_dir).unwrap();

        // Create boilerplate
        let bp = project_dir.join("my-awesome-bp");
        fs::create_dir_all(&bp).unwrap();
        let metadata = r#"
id: my-awesome-bp
name: My Awesome Boilerplate
description: Test
language: rust
files: []
dependencies: []
scripts: []
"#;
        fs::write(bp.join("boilerplate.yaml"), metadata).unwrap();

        let result = BoilerplateDiscovery::discover_by_name(temp_dir.path(), "awesome").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "my-awesome-bp");
    }
}
