//! Node.js dependency parser for package.json

use crate::error::ResearchError;
use crate::models::Dependency;
use std::path::Path;
use tracing::debug;

/// Parses Node.js dependencies from package.json
#[derive(Debug)]
pub struct NodeJsParser;

impl NodeJsParser {
    /// Creates a new NodeJsParser
    pub fn new() -> Self {
        NodeJsParser
    }

    /// Parses dependencies from package.json
    pub fn parse(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let package_json_path = root.join("package.json");

        if !package_json_path.exists() {
            return Ok(Vec::new());
        }

        debug!("Parsing Node.js dependencies from {:?}", package_json_path);

        let content = std::fs::read_to_string(&package_json_path).map_err(|e| {
            ResearchError::DependencyParsingFailed {
                language: "Node.js".to_string(),
                path: Some(package_json_path.clone()),
                reason: format!("Failed to read package.json: {}", e),
            }
        })?;

        let package_json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| ResearchError::DependencyParsingFailed {
                language: "Node.js".to_string(),
                path: Some(package_json_path.clone()),
                reason: format!("Failed to parse package.json: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Parse regular dependencies
        if let Some(deps) = package_json.get("dependencies").and_then(|d| d.as_object()) {
            for (name, value) in deps {
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

        // Parse dev dependencies
        if let Some(deps) = package_json
            .get("devDependencies")
            .and_then(|d| d.as_object())
        {
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

        // Parse peer dependencies
        if let Some(deps) = package_json
            .get("peerDependencies")
            .and_then(|d| d.as_object())
        {
            for (name, value) in deps {
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

        Ok(dependencies)
    }

    /// Checks if package.json exists
    pub fn has_manifest(&self, root: &Path) -> bool {
        root.join("package.json").exists()
    }
}

impl Default for NodeJsParser {
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
    fn test_nodejs_parser_creation() {
        let parser = NodeJsParser::new();
        assert!(true);
    }

    #[test]
    fn test_nodejs_parser_no_manifest() {
        let parser = NodeJsParser::new();
        let temp_dir = TempDir::new().unwrap();
        let result = parser.parse(temp_dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_nodejs_parser_simple_dependencies() {
        let parser = NodeJsParser::new();
        let temp_dir = TempDir::new().unwrap();

        let package_json = r#"{
  "name": "test",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.0",
    "react": "^18.0.0"
  },
  "devDependencies": {
    "jest": "^29.0.0"
  }
}"#;

        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();

        let deps = parser.parse(temp_dir.path()).unwrap();
        assert_eq!(deps.len(), 3);

        // Check express
        let express = deps.iter().find(|d| d.name == "express").unwrap();
        assert_eq!(express.version, "^4.18.0");
        assert!(!express.is_dev);

        // Check jest
        let jest = deps.iter().find(|d| d.name == "jest").unwrap();
        assert_eq!(jest.version, "^29.0.0");
        assert!(jest.is_dev);
    }

    #[test]
    fn test_nodejs_parser_has_manifest() {
        let parser = NodeJsParser::new();
        let temp_dir = TempDir::new().unwrap();

        assert!(!parser.has_manifest(temp_dir.path()));

        fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        assert!(parser.has_manifest(temp_dir.path()));
    }
}
