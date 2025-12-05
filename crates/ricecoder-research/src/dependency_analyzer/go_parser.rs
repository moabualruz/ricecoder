//! Go dependency parser for go.mod

use crate::error::ResearchError;
use crate::models::Dependency;
use std::path::Path;
use tracing::debug;

/// Parses Go dependencies from go.mod
#[derive(Debug)]
pub struct GoParser;

impl GoParser {
    /// Creates a new GoParser
    pub fn new() -> Self {
        GoParser
    }

    /// Parses dependencies from go.mod
    pub fn parse(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let go_mod_path = root.join("go.mod");

        if !go_mod_path.exists() {
            return Ok(Vec::new());
        }

        debug!("Parsing Go dependencies from {:?}", go_mod_path);

        let content = std::fs::read_to_string(&go_mod_path).map_err(|e| {
            ResearchError::DependencyParsingFailed {
                language: "Go".to_string(),
                path: Some(go_mod_path.clone()),
                reason: format!("Failed to read go.mod: {}", e),
            }
        })?;

        let mut dependencies = Vec::new();
        let mut in_require = false;

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // Check for require block
            if line.starts_with("require") {
                in_require = true;
                if line == "require" {
                    continue;
                }
                // Handle inline require: require (...)
                if line.starts_with("require (") {
                    continue;
                }
            }

            // Check for end of require block
            if in_require && line == ")" {
                in_require = false;
                continue;
            }

            // Parse require block entries
            if in_require || (line.starts_with("require ") && !line.contains("(")) {
                let line = if let Some(stripped) = line.strip_prefix("require ") {
                    stripped
                } else {
                    line
                };

                if let Some(dep) = self.parse_require_line(line) {
                    dependencies.push(dep);
                }
            }
        }

        Ok(dependencies)
    }

    /// Parses a single require line (e.g., "github.com/user/repo v1.2.3")
    fn parse_require_line(&self, line: &str) -> Option<Dependency> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 2 {
            return None;
        }

        let name = parts[0];
        let version = parts[1];

        // Skip indirect marker if present
        let version = if version == "indirect" && parts.len() > 2 {
            parts[2]
        } else {
            version
        };

        Some(Dependency {
            name: name.to_string(),
            version: version.to_string(),
            constraints: Some(version.to_string()),
            is_dev: false,
        })
    }

    /// Checks if go.mod exists
    pub fn has_manifest(&self, root: &Path) -> bool {
        root.join("go.mod").exists()
    }
}

impl Default for GoParser {
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
    fn test_go_parser_creation() {
        let parser = GoParser::new();
        assert!(true);
    }

    #[test]
    fn test_go_parser_no_manifest() {
        let parser = GoParser::new();
        let temp_dir = TempDir::new().unwrap();
        let result = parser.parse(temp_dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_go_parser_simple_dependencies() {
        let parser = GoParser::new();
        let temp_dir = TempDir::new().unwrap();

        let go_mod = r#"module github.com/user/project

go 1.21

require (
    github.com/gorilla/mux v1.8.0
    github.com/lib/pq v1.10.9
)
"#;

        fs::write(temp_dir.path().join("go.mod"), go_mod).unwrap();

        let deps = parser.parse(temp_dir.path()).unwrap();
        assert_eq!(deps.len(), 2);

        let mux = deps
            .iter()
            .find(|d| d.name == "github.com/gorilla/mux")
            .unwrap();
        assert_eq!(mux.version, "v1.8.0");
    }

    #[test]
    fn test_go_parser_has_manifest() {
        let parser = GoParser::new();
        let temp_dir = TempDir::new().unwrap();

        assert!(!parser.has_manifest(temp_dir.path()));

        fs::write(temp_dir.path().join("go.mod"), "module test").unwrap();
        assert!(parser.has_manifest(temp_dir.path()));
    }
}
