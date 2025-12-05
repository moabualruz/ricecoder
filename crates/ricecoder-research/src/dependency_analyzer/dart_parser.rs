//! Dart/Flutter dependency parser for pubspec.yaml

use crate::error::ResearchError;
use crate::models::Dependency;
use std::path::Path;
use tracing::debug;

/// Parses Dart/Flutter dependencies from pubspec.yaml
#[derive(Debug)]
pub struct DartParser;

impl DartParser {
    /// Creates a new DartParser
    pub fn new() -> Self {
        DartParser
    }

    /// Parses dependencies from pubspec.yaml
    pub fn parse(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let pubspec_path = root.join("pubspec.yaml");

        if !pubspec_path.exists() {
            return Ok(Vec::new());
        }

        debug!("Parsing Dart/Flutter dependencies from {:?}", pubspec_path);

        let content = std::fs::read_to_string(&pubspec_path).map_err(|e| {
            ResearchError::DependencyParsingFailed {
                language: "Dart".to_string(),
                path: Some(pubspec_path.clone()),
                reason: format!("Failed to read pubspec.yaml: {}", e),
            }
        })?;

        let pubspec: serde_yaml::Value =
            serde_yaml::from_str(&content).map_err(|e| ResearchError::DependencyParsingFailed {
                language: "Dart".to_string(),
                path: Some(pubspec_path.clone()),
                reason: format!("Failed to parse pubspec.yaml: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Parse dependencies
        if let Some(deps) = pubspec.get("dependencies").and_then(|d| d.as_mapping()) {
            for (key, value) in deps {
                if let Some(name) = key.as_str() {
                    if name != "flutter" {
                        let version = if let Some(version_str) = value.as_str() {
                            version_str.to_string()
                        } else if let Some(mapping) = value.as_mapping() {
                            if let Some(version) =
                                mapping.get(serde_yaml::Value::String("version".to_string()))
                            {
                                version.as_str().unwrap_or("*").to_string()
                            } else {
                                "*".to_string()
                            }
                        } else {
                            "*".to_string()
                        };

                        dependencies.push(Dependency {
                            name: name.to_string(),
                            version: version.clone(),
                            constraints: Some(version),
                            is_dev: false,
                        });
                    }
                }
            }
        }

        // Parse dev_dependencies
        if let Some(deps) = pubspec.get("dev_dependencies").and_then(|d| d.as_mapping()) {
            for (key, value) in deps {
                if let Some(name) = key.as_str() {
                    let version = if let Some(version_str) = value.as_str() {
                        version_str.to_string()
                    } else if let Some(mapping) = value.as_mapping() {
                        if let Some(version) =
                            mapping.get(serde_yaml::Value::String("version".to_string()))
                        {
                            version.as_str().unwrap_or("*").to_string()
                        } else {
                            "*".to_string()
                        }
                    } else {
                        "*".to_string()
                    };

                    dependencies.push(Dependency {
                        name: name.to_string(),
                        version: version.clone(),
                        constraints: Some(version),
                        is_dev: true,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    /// Checks if pubspec.yaml exists
    pub fn has_manifest(&self, root: &Path) -> bool {
        root.join("pubspec.yaml").exists()
    }
}

impl Default for DartParser {
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
    fn test_dart_parser_creation() {
        let parser = DartParser::new();
        assert!(true);
    }

    #[test]
    fn test_dart_parser_no_manifest() {
        let parser = DartParser::new();
        let temp_dir = TempDir::new().unwrap();
        let result = parser.parse(temp_dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_dart_parser_simple_dependencies() {
        let parser = DartParser::new();
        let temp_dir = TempDir::new().unwrap();

        let pubspec = r#"
name: my_app
version: 1.0.0

dependencies:
  flutter:
    sdk: flutter
  provider: ^6.0.0
  http: ^0.13.0

dev_dependencies:
  flutter_test:
    sdk: flutter
  test: ^1.20.0
"#;

        fs::write(temp_dir.path().join("pubspec.yaml"), pubspec).unwrap();

        let deps = parser.parse(temp_dir.path()).unwrap();
        assert!(deps.len() >= 2);

        let provider = deps.iter().find(|d| d.name == "provider").unwrap();
        assert_eq!(provider.version, "^6.0.0");
        assert!(!provider.is_dev);
    }

    #[test]
    fn test_dart_parser_has_manifest() {
        let parser = DartParser::new();
        let temp_dir = TempDir::new().unwrap();

        assert!(!parser.has_manifest(temp_dir.path()));

        fs::write(temp_dir.path().join("pubspec.yaml"), "").unwrap();
        assert!(parser.has_manifest(temp_dir.path()));
    }
}
