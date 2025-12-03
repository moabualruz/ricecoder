//! Python dependency parser for pyproject.toml and requirements.txt

use crate::error::ResearchError;
use crate::models::Dependency;
use std::path::Path;
use tracing::debug;

/// Parses Python dependencies from pyproject.toml and requirements.txt
#[derive(Debug)]
pub struct PythonParser;

impl PythonParser {
    /// Creates a new PythonParser
    pub fn new() -> Self {
        PythonParser
    }

    /// Parses dependencies from pyproject.toml or requirements.txt
    pub fn parse(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let mut dependencies = Vec::new();

        // Try pyproject.toml first
        let pyproject_path = root.join("pyproject.toml");
        if pyproject_path.exists() {
            debug!("Parsing Python dependencies from {:?}", pyproject_path);
            if let Ok(mut deps) = self.parse_pyproject(&pyproject_path) {
                dependencies.append(&mut deps);
            }
        }

        // Try requirements.txt
        let requirements_path = root.join("requirements.txt");
        if requirements_path.exists() {
            debug!("Parsing Python dependencies from {:?}", requirements_path);
            if let Ok(mut deps) = self.parse_requirements(&requirements_path) {
                dependencies.append(&mut deps);
            }
        }

        Ok(dependencies)
    }

    /// Parses dependencies from pyproject.toml
    fn parse_pyproject(&self, path: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ResearchError::DependencyParsingFailed {
                language: "Python".to_string(),
                path: Some(path.to_path_buf()),
                reason: format!("Failed to read pyproject.toml: {}", e),
            })?;

        let pyproject: toml::Value = toml::from_str(&content)
            .map_err(|e| ResearchError::DependencyParsingFailed {
                language: "Python".to_string(),
                path: Some(path.to_path_buf()),
                reason: format!("Failed to parse pyproject.toml: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Parse project dependencies
        if let Some(project) = pyproject.get("project") {
            if let Some(deps) = project.get("dependencies").and_then(|d| d.as_array()) {
                for dep_str in deps {
                    if let Some(dep_str) = dep_str.as_str() {
                        if let Some(dep) = self.parse_requirement_string(dep_str, false) {
                            dependencies.push(dep);
                        }
                    }
                }
            }

            // Parse optional dependencies
            if let Some(optional) = project.get("optional-dependencies").and_then(|o| o.as_table()) {
                for (_group, deps) in optional {
                    if let Some(deps_array) = deps.as_array() {
                        for dep_str in deps_array {
                            if let Some(dep_str) = dep_str.as_str() {
                                if let Some(dep) = self.parse_requirement_string(dep_str, false) {
                                    dependencies.push(dep);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Parse poetry dependencies
        if let Some(tool) = pyproject.get("tool") {
            if let Some(poetry) = tool.get("poetry") {
                if let Some(deps) = poetry.get("dependencies").and_then(|d| d.as_table()) {
                    for (name, value) in deps {
                        if name != "python" {
                            if let Some(version) = value.as_str() {
                                dependencies.push(Dependency {
                                    name: name.clone(),
                                    version: version.to_string(),
                                    constraints: Some(version.to_string()),
                                    is_dev: false,
                                });
                            } else if let Some(table) = value.as_table() {
                                if let Some(version) = table.get("version").and_then(|v| v.as_str()) {
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
                }

                if let Some(deps) = poetry.get("dev-dependencies").and_then(|d| d.as_table()) {
                    for (name, value) in deps {
                        if let Some(version) = value.as_str() {
                            dependencies.push(Dependency {
                                name: name.clone(),
                                version: version.to_string(),
                                constraints: Some(version.to_string()),
                                is_dev: true,
                            });
                        } else if let Some(table) = value.as_table() {
                            if let Some(version) = table.get("version").and_then(|v| v.as_str()) {
                                dependencies.push(Dependency {
                                    name: name.clone(),
                                    version: version.to_string(),
                                    constraints: Some(version.to_string()),
                                    is_dev: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Parses dependencies from requirements.txt
    fn parse_requirements(&self, path: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ResearchError::DependencyParsingFailed {
                language: "Python".to_string(),
                path: Some(path.to_path_buf()),
                reason: format!("Failed to read requirements.txt: {}", e),
            })?;

        let mut dependencies = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(dep) = self.parse_requirement_string(line, false) {
                dependencies.push(dep);
            }
        }

        Ok(dependencies)
    }

    /// Parses a single requirement string (e.g., "package>=1.0,<2.0")
    fn parse_requirement_string(&self, req_str: &str, is_dev: bool) -> Option<Dependency> {
        let req_str = req_str.trim();

        // Handle extras syntax: package[extra]>=1.0
        let req_str = if let Some(bracket_pos) = req_str.find('[') {
            &req_str[..bracket_pos]
        } else {
            req_str
        };

        // Split on operators
        let operators = [">=", "<=", "==", "!=", "~=", ">", "<"];
        let mut name = req_str;
        let mut version = String::new();
        let mut constraints = None;

        for op in &operators {
            if let Some(pos) = req_str.find(op) {
                name = &req_str[..pos];
                version = req_str[pos..].to_string();
                constraints = Some(version.clone());
                break;
            }
        }

        if name.is_empty() {
            return None;
        }

        if version.is_empty() {
            version = "*".to_string();
        }

        Some(Dependency {
            name: name.to_string(),
            version,
            constraints,
            is_dev,
        })
    }

    /// Checks if Python manifest files exist
    pub fn has_manifest(&self, root: &Path) -> bool {
        root.join("pyproject.toml").exists() || root.join("requirements.txt").exists()
    }
}

impl Default for PythonParser {
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
    fn test_python_parser_creation() {
        let parser = PythonParser::new();
        assert!(true);
    }

    #[test]
    fn test_python_parser_no_manifest() {
        let parser = PythonParser::new();
        let temp_dir = TempDir::new().unwrap();
        let result = parser.parse(temp_dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_python_parser_requirements_txt() {
        let parser = PythonParser::new();
        let temp_dir = TempDir::new().unwrap();

        let requirements = r#"
requests>=2.28.0
django==4.1.0
pytest>=7.0
# This is a comment
numpy>=1.20,<2.0
"#;

        fs::write(temp_dir.path().join("requirements.txt"), requirements).unwrap();

        let deps = parser.parse(temp_dir.path()).unwrap();
        assert_eq!(deps.len(), 4);

        let requests = deps.iter().find(|d| d.name == "requests").unwrap();
        assert_eq!(requests.version, ">=2.28.0");
    }

    #[test]
    fn test_python_parser_pyproject_toml() {
        let parser = PythonParser::new();
        let temp_dir = TempDir::new().unwrap();

        let pyproject = r#"
[project]
name = "test"
dependencies = [
    "requests>=2.28.0",
    "django==4.1.0"
]

[project.optional-dependencies]
dev = ["pytest>=7.0"]
"#;

        fs::write(temp_dir.path().join("pyproject.toml"), pyproject).unwrap();

        let deps = parser.parse(temp_dir.path()).unwrap();
        assert!(deps.len() >= 2);

        let requests = deps.iter().find(|d| d.name == "requests").unwrap();
        assert_eq!(requests.version, ">=2.28.0");
    }

    #[test]
    fn test_python_parser_has_manifest() {
        let parser = PythonParser::new();
        let temp_dir = TempDir::new().unwrap();

        assert!(!parser.has_manifest(temp_dir.path()));

        fs::write(temp_dir.path().join("requirements.txt"), "").unwrap();
        assert!(parser.has_manifest(temp_dir.path()));
    }
}
