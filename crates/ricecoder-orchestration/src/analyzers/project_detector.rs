//! Project detection and metadata extraction

use std::path::PathBuf;

use tracing::{debug, warn};

use crate::{
    error::{OrchestrationError, Result},
    models::{Project, ProjectStatus},
};

/// Detects project type and extracts metadata from project manifests
pub struct ProjectDetector;

impl ProjectDetector {
    /// Detects the type of a project from its manifest files
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the project directory
    ///
    /// # Returns
    ///
    /// The project type as a string, or None if not detected
    pub fn detect_project_type(path: &std::path::Path) -> Option<String> {
        // Check for Cargo.toml (Rust project)
        if path.join("Cargo.toml").exists() {
            return Some("rust".to_string());
        }

        // Check for package.json (Node.js project)
        if path.join("package.json").exists() {
            return Some("nodejs".to_string());
        }

        // Check for pyproject.toml (Python project)
        if path.join("pyproject.toml").exists() {
            return Some("python".to_string());
        }

        // Check for go.mod (Go project)
        if path.join("go.mod").exists() {
            return Some("go".to_string());
        }

        // Check for pom.xml (Java/Maven project)
        if path.join("pom.xml").exists() {
            return Some("java".to_string());
        }

        // Check for build.gradle (Gradle project)
        if path.join("build.gradle").exists() || path.join("build.gradle.kts").exists() {
            return Some("gradle".to_string());
        }

        None
    }

    /// Extracts project metadata from a project directory
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the project directory
    ///
    /// # Returns
    ///
    /// A Project struct with extracted metadata, or an error if extraction fails
    pub fn extract_metadata(path: &PathBuf) -> Result<Project> {
        let project_type = Self::detect_project_type(path).ok_or_else(|| {
            OrchestrationError::ProjectNotFound(format!("No project manifest found in {:?}", path))
        })?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OrchestrationError::ProjectNotFound(format!("Invalid project path: {:?}", path))
            })?;

        let version = Self::extract_version(path, &project_type);

        Ok(Project {
            path: path.clone(),
            name,
            project_type,
            version,
            status: ProjectStatus::Unknown,
        })
    }

    /// Extracts version from project manifest
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the project directory
    /// * `project_type` - The type of project
    ///
    /// # Returns
    ///
    /// The version string, or "0.1.0" if extraction fails
    fn extract_version(path: &std::path::Path, project_type: &str) -> String {
        match project_type {
            "rust" => Self::extract_rust_version(path),
            "nodejs" => Self::extract_nodejs_version(path),
            "python" => Self::extract_python_version(path),
            "go" => Self::extract_go_version(path),
            "java" => Self::extract_java_version(path),
            "gradle" => Self::extract_gradle_version(path),
            _ => "0.1.0".to_string(),
        }
    }

    /// Extracts version from Cargo.toml
    fn extract_rust_version(path: &std::path::Path) -> String {
        let cargo_toml = path.join("Cargo.toml");
        match std::fs::read_to_string(&cargo_toml) {
            Ok(content) => {
                for line in content.lines() {
                    if line.contains("version") && line.contains("=") {
                        if let Some(version_part) = line.split('=').nth(1) {
                            let version = version_part
                                .trim()
                                .trim_matches('"')
                                .trim_matches('\'')
                                .to_string();
                            if !version.is_empty() {
                                debug!("Extracted Rust version: {}", version);
                                return version;
                            }
                        }
                    }
                }
                "0.1.0".to_string()
            }
            Err(e) => {
                warn!("Failed to read Cargo.toml: {}", e);
                "0.1.0".to_string()
            }
        }
    }

    /// Extracts version from package.json
    fn extract_nodejs_version(path: &std::path::Path) -> String {
        let package_json = path.join("package.json");
        match std::fs::read_to_string(&package_json) {
            Ok(content) => {
                // Try JSON parsing first
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                        debug!("Extracted Node.js version: {}", version);
                        return version.to_string();
                    }
                }

                // Fallback to line-by-line parsing
                for line in content.lines() {
                    if line.contains("\"version\"") && line.contains(":") {
                        if let Some(version_part) = line.split(':').nth(1) {
                            let version = version_part
                                .trim()
                                .trim_matches(',')
                                .trim_matches('"')
                                .to_string();
                            if !version.is_empty()
                                && !version.contains('{')
                                && !version.contains('}')
                            {
                                debug!("Extracted Node.js version: {}", version);
                                return version;
                            }
                        }
                    }
                }
                "0.1.0".to_string()
            }
            Err(e) => {
                warn!("Failed to read package.json: {}", e);
                "0.1.0".to_string()
            }
        }
    }

    /// Extracts version from pyproject.toml
    fn extract_python_version(path: &std::path::Path) -> String {
        let pyproject_toml = path.join("pyproject.toml");
        match std::fs::read_to_string(&pyproject_toml) {
            Ok(content) => {
                for line in content.lines() {
                    if line.contains("version") && line.contains("=") {
                        if let Some(version_part) = line.split('=').nth(1) {
                            let version = version_part
                                .trim()
                                .trim_matches('"')
                                .trim_matches('\'')
                                .to_string();
                            if !version.is_empty() {
                                debug!("Extracted Python version: {}", version);
                                return version;
                            }
                        }
                    }
                }
                "0.1.0".to_string()
            }
            Err(e) => {
                warn!("Failed to read pyproject.toml: {}", e);
                "0.1.0".to_string()
            }
        }
    }

    /// Extracts version from go.mod
    fn extract_go_version(path: &std::path::Path) -> String {
        let go_mod = path.join("go.mod");
        match std::fs::read_to_string(&go_mod) {
            Ok(content) => {
                // Go modules don't have versions in go.mod, use module name
                for line in content.lines() {
                    if line.starts_with("module ") {
                        debug!("Found Go module: {}", line);
                        return "0.1.0".to_string();
                    }
                }
                "0.1.0".to_string()
            }
            Err(e) => {
                warn!("Failed to read go.mod: {}", e);
                "0.1.0".to_string()
            }
        }
    }

    /// Extracts version from pom.xml
    fn extract_java_version(path: &std::path::Path) -> String {
        let pom_xml = path.join("pom.xml");
        match std::fs::read_to_string(&pom_xml) {
            Ok(content) => {
                for line in content.lines() {
                    if line.contains("<version>") {
                        if let Some(version_part) = line.split("<version>").nth(1) {
                            if let Some(version) = version_part.split("</version>").next() {
                                let version = version.trim().to_string();
                                if !version.is_empty() {
                                    debug!("Extracted Java version: {}", version);
                                    return version;
                                }
                            }
                        }
                    }
                }
                "0.1.0".to_string()
            }
            Err(e) => {
                warn!("Failed to read pom.xml: {}", e);
                "0.1.0".to_string()
            }
        }
    }

    /// Extracts version from build.gradle or build.gradle.kts
    fn extract_gradle_version(path: &std::path::Path) -> String {
        let gradle_file = if path.join("build.gradle.kts").exists() {
            path.join("build.gradle.kts")
        } else {
            path.join("build.gradle")
        };

        match std::fs::read_to_string(&gradle_file) {
            Ok(content) => {
                for line in content.lines() {
                    if line.contains("version") && line.contains("=") {
                        if let Some(version_part) = line.split('=').nth(1) {
                            let version = version_part
                                .trim()
                                .trim_matches('"')
                                .trim_matches('\'')
                                .to_string();
                            if !version.is_empty() {
                                debug!("Extracted Gradle version: {}", version);
                                return version;
                            }
                        }
                    }
                }
                "0.1.0".to_string()
            }
            Err(e) => {
                warn!("Failed to read build.gradle: {}", e);
                "0.1.0".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_detect_rust_project() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(project_dir.join("Cargo.toml"), "[package]")
            .expect("failed to write Cargo.toml");

        let project_type = ProjectDetector::detect_project_type(&project_dir);
        assert_eq!(project_type, Some("rust".to_string()));
    }

    #[test]
    fn test_detect_nodejs_project() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(project_dir.join("package.json"), "{}")
            .expect("failed to write package.json");

        let project_type = ProjectDetector::detect_project_type(&project_dir);
        assert_eq!(project_type, Some("nodejs".to_string()));
    }

    #[test]
    fn test_detect_python_project() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(project_dir.join("pyproject.toml"), "[build-system]")
            .expect("failed to write pyproject.toml");

        let project_type = ProjectDetector::detect_project_type(&project_dir);
        assert_eq!(project_type, Some("python".to_string()));
    }

    #[test]
    fn test_detect_go_project() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(project_dir.join("go.mod"), "module example.com/test")
            .expect("failed to write go.mod");

        let project_type = ProjectDetector::detect_project_type(&project_dir);
        assert_eq!(project_type, Some("go".to_string()));
    }

    #[test]
    fn test_detect_java_project() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(project_dir.join("pom.xml"), "<project>").expect("failed to write pom.xml");

        let project_type = ProjectDetector::detect_project_type(&project_dir);
        assert_eq!(project_type, Some("java".to_string()));
    }

    #[test]
    fn test_detect_gradle_project() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(project_dir.join("build.gradle"), "plugins {}")
            .expect("failed to write build.gradle");

        let project_type = ProjectDetector::detect_project_type(&project_dir);
        assert_eq!(project_type, Some("gradle".to_string()));
    }

    #[test]
    fn test_detect_unknown_project() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();

        let project_type = ProjectDetector::detect_project_type(&project_dir);
        assert_eq!(project_type, None);
    }

    #[test]
    fn test_extract_metadata_rust() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.2.0\"\n",
        )
        .expect("failed to write Cargo.toml");

        let project =
            ProjectDetector::extract_metadata(&project_dir).expect("failed to extract metadata");

        assert_eq!(project.project_type, "rust");
        assert_eq!(project.version, "0.2.0");
    }

    #[test]
    fn test_extract_metadata_nodejs() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(
            project_dir.join("package.json"),
            r#"{"name": "test", "version": "1.0.0"}"#,
        )
        .expect("failed to write package.json");

        let project =
            ProjectDetector::extract_metadata(&project_dir).expect("failed to extract metadata");

        assert_eq!(project.project_type, "nodejs");
        assert_eq!(project.version, "1.0.0");
    }

    #[test]
    fn test_extract_metadata_python() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(
            project_dir.join("pyproject.toml"),
            "[project]\nname = \"test\"\nversion = \"2.1.0\"\n",
        )
        .expect("failed to write pyproject.toml");

        let project =
            ProjectDetector::extract_metadata(&project_dir).expect("failed to extract metadata");

        assert_eq!(project.project_type, "python");
        assert_eq!(project.version, "2.1.0");
    }

    #[test]
    fn test_extract_metadata_missing_manifest() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();

        let result = ProjectDetector::extract_metadata(&project_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_rust_version_with_quotes() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nversion = \"0.3.0\"\n",
        )
        .expect("failed to write Cargo.toml");

        let version = ProjectDetector::extract_rust_version(&project_dir);
        assert_eq!(version, "0.3.0");
    }

    #[test]
    fn test_extract_version_missing_field() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .expect("failed to write Cargo.toml");

        let version = ProjectDetector::extract_rust_version(&project_dir);
        assert_eq!(version, "0.1.0");
    }

    #[test]
    fn test_extract_java_version() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(
            project_dir.join("pom.xml"),
            "<project><version>1.2.3</version></project>",
        )
        .expect("failed to write pom.xml");

        let version = ProjectDetector::extract_java_version(&project_dir);
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn test_extract_gradle_version() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(
            project_dir.join("build.gradle"),
            "plugins {}\nversion = \"3.0.0\"\n",
        )
        .expect("failed to write build.gradle");

        let version = ProjectDetector::extract_gradle_version(&project_dir);
        assert_eq!(version, "3.0.0");
    }
}
