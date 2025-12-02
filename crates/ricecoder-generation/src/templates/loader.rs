//! Template loading from files and directories
//!
//! Loads templates from `.tmpl` files in global and project-specific locations.
//! Supports template inheritance and includes.

use crate::models::{Template, TemplateMetadata};
use crate::templates::error::TemplateError;
use crate::templates::parser::TemplateParser;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Loads templates from files and directories
pub struct TemplateLoader {
    /// Cache of loaded templates
    cache: HashMap<String, Template>,
}

impl TemplateLoader {
    /// Create a new template loader
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Load a template from a file
    ///
    /// # Arguments
    /// * `path` - Path to the template file (.tmpl extension)
    ///
    /// # Returns
    /// Loaded template or error
    pub fn load_from_file(&mut self, path: &Path) -> Result<Template, TemplateError> {
        // Check cache first
        if let Some(cached) = self.cache.get(path.to_string_lossy().as_ref()) {
            return Ok(cached.clone());
        }

        // Read file
        let content = fs::read_to_string(path)
            .map_err(TemplateError::IoError)?;

        // Parse template to validate syntax
        let parsed = TemplateParser::parse(&content)?;

        // Extract template ID from filename (without .tmpl and language extension)
        let id = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| {
                // Remove .tmpl extension
                let name = name.strip_suffix(".tmpl").unwrap_or(name);
                // Remove language extension (e.g., .rs, .ts, .py)
                if let Some(dot_pos) = name.rfind('.') {
                    &name[..dot_pos]
                } else {
                    name
                }
            })
            .unwrap_or("unknown")
            .to_string();

        // Create template
        let template = Template {
            id: id.clone(),
            name: id,
            language: self.detect_language(path),
            content,
            placeholders: parsed.placeholders,
            metadata: TemplateMetadata {
                description: None,
                version: None,
                author: None,
            },
        };

        // Cache the template
        self.cache.insert(path.to_string_lossy().to_string(), template.clone());

        Ok(template)
    }

    /// Load all templates from a directory
    ///
    /// # Arguments
    /// * `dir` - Directory containing .tmpl files
    ///
    /// # Returns
    /// Vector of loaded templates or error
    pub fn load_from_directory(&mut self, dir: &Path) -> Result<Vec<Template>, TemplateError> {
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut templates = Vec::new();

        // Recursively scan directory for .tmpl files
        self.scan_directory(dir, &mut templates)?;

        Ok(templates)
    }

    /// Scan directory recursively for .tmpl files
    fn scan_directory(&mut self, dir: &Path, templates: &mut Vec<Template>) -> Result<(), TemplateError> {
        let entries = fs::read_dir(dir)
            .map_err(TemplateError::IoError)?;

        for entry in entries {
            let entry = entry.map_err(TemplateError::IoError)?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                self.scan_directory(&path, templates)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("tmpl") {
                // Load template file
                match self.load_from_file(&path) {
                    Ok(template) => templates.push(template),
                    Err(e) => {
                        // Log error but continue scanning
                        eprintln!("Failed to load template {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load templates from global location (~/.ricecoder/templates/)
    ///
    /// # Returns
    /// Vector of loaded templates or error
    pub fn load_global_templates(&mut self) -> Result<Vec<Template>, TemplateError> {
        let global_dir = self.get_global_templates_dir();
        self.load_from_directory(&global_dir)
    }

    /// Load templates from project location (./.ricecoder/templates/)
    ///
    /// # Arguments
    /// * `project_root` - Root directory of the project
    ///
    /// # Returns
    /// Vector of loaded templates or error
    pub fn load_project_templates(&mut self, project_root: &Path) -> Result<Vec<Template>, TemplateError> {
        let project_dir = project_root.join(".ricecoder").join("templates");
        self.load_from_directory(&project_dir)
    }

    /// Load templates from both global and project locations
    ///
    /// Project templates take precedence over global templates with the same name.
    ///
    /// # Arguments
    /// * `project_root` - Root directory of the project
    ///
    /// # Returns
    /// Vector of loaded templates (project templates override global ones)
    pub fn load_all_templates(&mut self, project_root: &Path) -> Result<Vec<Template>, TemplateError> {
        // Load global templates first
        let templates = self.load_global_templates()?;

        // Load project templates
        let project_templates = self.load_project_templates(project_root)?;

        // Create a map of templates by ID for deduplication
        let mut template_map: HashMap<String, Template> = templates
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect();

        // Add/override with project templates
        for template in project_templates {
            template_map.insert(template.id.clone(), template);
        }

        Ok(template_map.into_values().collect())
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

    /// Detect programming language from file extension
    fn detect_language(&self, path: &Path) -> String {
        path.extension()
            .and_then(|_ext| {
                // Get the extension before .tmpl
                let parts: Vec<&str> = path
                    .file_name()?
                    .to_str()?
                    .split('.')
                    .collect();
                
                if parts.len() >= 2 {
                    Some(parts[parts.len() - 2].to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Clear the template cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            cached_templates: self.cache.len(),
        }
    }
}

impl Default for TemplateLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the template cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached templates
    pub cached_templates: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("test.rs.tmpl");
        
        let content = "pub struct {{Name}} {\n    pub field: String,\n}";
        fs::write(&template_path, content).unwrap();

        let mut loader = TemplateLoader::new();
        let template = loader.load_from_file(&template_path).unwrap();

        assert_eq!(template.id, "test");
        assert_eq!(template.language, "rs");
        assert_eq!(template.content, content);
    }

    #[test]
    fn test_load_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create multiple template files
        fs::write(temp_dir.path().join("struct.rs.tmpl"), "pub struct {{Name}} {}").unwrap();
        fs::write(temp_dir.path().join("impl.rs.tmpl"), "impl {{Name}} {}").unwrap();

        let mut loader = TemplateLoader::new();
        let templates = loader.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(templates.len(), 2);
        assert!(templates.iter().any(|t| t.id == "struct"));
        assert!(templates.iter().any(|t| t.id == "impl"));
    }

    #[test]
    fn test_load_nonexistent_directory() {
        let mut loader = TemplateLoader::new();
        let templates = loader.load_from_directory(Path::new("/nonexistent/path")).unwrap();
        
        assert_eq!(templates.len(), 0);
    }

    #[test]
    fn test_detect_language() {
        let loader = TemplateLoader::new();
        
        assert_eq!(loader.detect_language(Path::new("test.rs.tmpl")), "rs");
        assert_eq!(loader.detect_language(Path::new("test.ts.tmpl")), "ts");
        assert_eq!(loader.detect_language(Path::new("test.py.tmpl")), "py");
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("test.rs.tmpl");
        fs::write(&template_path, "pub struct {{Name}} {}").unwrap();

        let mut loader = TemplateLoader::new();
        loader.load_from_file(&template_path).unwrap();

        let stats = loader.cache_stats();
        assert_eq!(stats.cached_templates, 1);
    }

    #[test]
    fn test_clear_cache() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("test.rs.tmpl");
        fs::write(&template_path, "pub struct {{Name}} {}").unwrap();

        let mut loader = TemplateLoader::new();
        loader.load_from_file(&template_path).unwrap();
        assert_eq!(loader.cache_stats().cached_templates, 1);

        loader.clear_cache();
        assert_eq!(loader.cache_stats().cached_templates, 0);
    }
}
