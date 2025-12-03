//! PHP dependency parser for composer.json

use crate::error::ResearchError;
use crate::models::Dependency;
use std::path::Path;
use tracing::debug;

/// Parses PHP dependencies from composer.json
#[derive(Debug)]
pub struct PhpParser;

impl PhpParser {
    /// Creates a new PhpParser
    pub fn new() -> Self {
        PhpParser
    }

    /// Parses dependencies from composer.json
    pub fn parse(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let composer_json_path = root.join("composer.json");

        if !composer_json_path.exists() {
            return Ok(Vec::new());
        }

        debug!("Parsing PHP dependencies from {:?}", composer_json_path);

        let content = std::fs::read_to_string(&composer_json_path)
            .map_err(|e| ResearchError::DependencyParsingFailed {
                language: "PHP".to_string(),
                path: Some(composer_json_path.clone()),
                reason: format!("Failed to read composer.json: {}", e),
            })?;

        let composer_json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| ResearchError::DependencyParsingFailed {
                language: "PHP".to_string(),
                path: Some(composer_json_path.clone()),
                reason: format!("Failed to parse composer.json: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Parse require dependencies
        if let Some(deps) = composer_json.get("require").and_then(|d| d.as_object()) {
            for (name, value) in deps {
                if name != "php" {
                    if let Some(version) = value.as_str() {
                        dependencies.push(Dependency {
                            name: name.clone(),
                            version: version.to_string(),
                            constraints: Some(version.to_string()),
                            is_dev: false,
                        });
                    }
                }
            }
        }

        // Parse require-dev dependencies
        if let Some(deps) = composer_json.get("require-dev").and_then(|d| d.as_object()) {
            for (name, value) in deps {
                if let Some(version) = value.as_str() {
                    dependencies.push(Dependency {
                        name: name.clone(),
                        version: version.to_string(),
                        constraints: Some(version.to_string()),
                        is_dev: true,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    /// Checks if composer.json exists
    pub fn has_manifest(&self, root: &Path) -> bool {
        root.join("composer.json").exists()
    }
}

impl Default for PhpParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_php_parser_creation() {
        let parser = PhpParser::new();
        assert!(true);
    }

    #[test]
    fn test_php_parser_no_manifest() {
        let parser = PhpParser::new();
        let temp_dir = TempDir::new().unwrap();
        let result = parser.parse(temp_dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_php_parser_simple_dependencies() {
        let parser = PhpParser::new();
        let temp_dir = TempDir::new().unwrap();

        let composer_json = r#"{
  "name": "test/project",
  "require": {
    "php": ">=7.4",
    "laravel/framework": "^9.0",
    "guzzlehttp/guzzle": "^7.0"
  },
  "require-dev": {
    "phpunit/phpunit": "^9.0"
  }
}"#;

        fs::write(temp_dir.path().join("composer.json"), composer_json).unwrap();

        let deps = parser.parse(temp_dir.path()).unwrap();
        assert_eq!(deps.len(), 3);

        let laravel = deps.iter().find(|d| d.name == "laravel/framework").unwrap();
        assert_eq!(laravel.version, "^9.0");
        assert!(!laravel.is_dev);

        let phpunit = deps.iter().find(|d| d.name == "phpunit/phpunit").unwrap();
        assert!(phpunit.is_dev);
    }

    #[test]
    fn test_php_parser_has_manifest() {
        let parser = PhpParser::new();
        let temp_dir = TempDir::new().unwrap();

        assert!(!parser.has_manifest(temp_dir.path()));

        fs::write(temp_dir.path().join("composer.json"), "{}").unwrap();
        assert!(parser.has_manifest(temp_dir.path()));
    }
}
