//! Rust dependency parser for Cargo.toml

use crate::error::ResearchError;
use crate::models::Dependency;
use std::path::Path;
use tracing::debug;

/// Parses Rust dependencies from Cargo.toml
#[derive(Debug)]
pub struct RustParser;

impl RustParser {
    /// Creates a new RustParser
    pub fn new() -> Self {
        RustParser
    }

    /// Parses dependencies from Cargo.toml
    pub fn parse(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let cargo_toml_path = root.join("Cargo.toml");

        if !cargo_toml_path.exists() {
            return Ok(Vec::new());
        }

        debug!("Parsing Rust dependencies from {:?}", cargo_toml_path);

        let content = std::fs::read_to_string(&cargo_toml_path).map_err(|e| {
            ResearchError::DependencyParsingFailed {
                language: "Rust".to_string(),
                path: Some(cargo_toml_path.clone()),
                reason: format!("Failed to read Cargo.toml: {}", e),
            }
        })?;

        let cargo_toml: toml::Value =
            toml::from_str(&content).map_err(|e| ResearchError::DependencyParsingFailed {
                language: "Rust".to_string(),
                path: Some(cargo_toml_path.clone()),
                reason: format!("Failed to parse Cargo.toml: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Parse regular dependencies
        if let Some(deps) = cargo_toml.get("dependencies").and_then(|d| d.as_table()) {
            for (name, value) in deps {
                if let Some(dep) = self.parse_dependency(name, value, false) {
                    dependencies.push(dep);
                }
            }
        }

        // Parse dev dependencies
        if let Some(deps) = cargo_toml
            .get("dev-dependencies")
            .and_then(|d| d.as_table())
        {
            for (name, value) in deps {
                if let Some(dep) = self.parse_dependency(name, value, true) {
                    dependencies.push(dep);
                }
            }
        }

        // Parse build dependencies
        if let Some(deps) = cargo_toml
            .get("build-dependencies")
            .and_then(|d| d.as_table())
        {
            for (name, value) in deps {
                if let Some(dep) = self.parse_dependency(name, value, false) {
                    dependencies.push(dep);
                }
            }
        }

        // Parse workspace dependencies if this is a workspace
        if let Some(workspace) = cargo_toml.get("workspace") {
            if let Some(deps) = workspace.get("dependencies").and_then(|d| d.as_table()) {
                for (name, value) in deps {
                    if let Some(dep) = self.parse_dependency(name, value, false) {
                        dependencies.push(dep);
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Checks if Cargo.toml exists
    pub fn has_manifest(&self, root: &Path) -> bool {
        root.join("Cargo.toml").exists()
    }

    /// Parses a single dependency entry
    fn parse_dependency(
        &self,
        name: &str,
        value: &toml::Value,
        is_dev: bool,
    ) -> Option<Dependency> {
        let (version, constraints) = if let Some(version_str) = value.as_str() {
            // Simple version string
            (version_str.to_string(), Some(version_str.to_string()))
        } else if let Some(table) = value.as_table() {
            // Complex dependency specification
            if let Some(version) = table.get("version").and_then(|v| v.as_str()) {
                (version.to_string(), Some(version.to_string()))
            } else if let Some(path) = table.get("path").and_then(|p| p.as_str()) {
                // Path dependency
                ("path".to_string(), Some(format!("path: {}", path)))
            } else if let Some(git) = table.get("git").and_then(|g| g.as_str()) {
                // Git dependency
                ("git".to_string(), Some(format!("git: {}", git)))
            } else {
                return None;
            }
        } else {
            return None;
        };

        Some(Dependency {
            name: name.to_string(),
            version,
            constraints,
            is_dev,
        })
    }
}

impl Default for RustParser {
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
    fn test_rust_parser_creation() {
        let parser = RustParser::new();
        assert!(true);
    }

    #[test]
    fn test_rust_parser_no_manifest() {
        let parser = RustParser::new();
        let temp_dir = TempDir::new().unwrap();
        let result = parser.parse(temp_dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_rust_parser_simple_dependencies() {
        let parser = RustParser::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
proptest = "1.0"
"#;

        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let deps = parser.parse(temp_dir.path()).unwrap();
        assert_eq!(deps.len(), 3);

        // Check serde
        let serde = deps.iter().find(|d| d.name == "serde").unwrap();
        assert_eq!(serde.version, "1.0");
        assert!(!serde.is_dev);

        // Check tokio
        let tokio = deps.iter().find(|d| d.name == "tokio").unwrap();
        assert_eq!(tokio.version, "1.0");
        assert!(!tokio.is_dev);

        // Check proptest
        let proptest = deps.iter().find(|d| d.name == "proptest").unwrap();
        assert_eq!(proptest.version, "1.0");
        assert!(proptest.is_dev);
    }

    #[test]
    fn test_rust_parser_has_manifest() {
        let parser = RustParser::new();
        let temp_dir = TempDir::new().unwrap();

        assert!(!parser.has_manifest(temp_dir.path()));

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        assert!(parser.has_manifest(temp_dir.path()));
    }
}
