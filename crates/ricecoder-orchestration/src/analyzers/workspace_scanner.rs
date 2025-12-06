//! Workspace scanning and project discovery

use crate::error::Result;
use crate::models::{Project, ProjectStatus};
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Scans a workspace to discover all projects and their metadata
///
/// The WorkspaceScanner is responsible for discovering all projects in a workspace,
/// extracting their metadata, and building the initial project list. It uses
/// `ricecoder_storage::PathResolver` for all path operations.
pub struct WorkspaceScanner {
    workspace_root: PathBuf,
}

impl WorkspaceScanner {
    /// Creates a new WorkspaceScanner for the given workspace root
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - The root path of the workspace
    ///
    /// # Returns
    ///
    /// A new WorkspaceScanner instance
    pub fn new(workspace_root: PathBuf) -> Self {
        debug!("Creating WorkspaceScanner for workspace: {:?}", workspace_root);
        Self { workspace_root }
    }

    /// Scans the workspace and discovers all projects
    ///
    /// This method scans the workspace root for all projects and extracts
    /// their metadata (name, path, type). It handles missing or malformed
    /// project files gracefully.
    ///
    /// # Returns
    ///
    /// A vector of discovered projects
    pub async fn scan_workspace(&self) -> Result<Vec<Project>> {
        info!("Scanning workspace: {:?}", self.workspace_root);

        let mut projects = Vec::new();

        // Check if workspace root exists
        if !self.workspace_root.exists() {
            debug!("Workspace root does not exist: {:?}", self.workspace_root);
            return Ok(projects);
        }

        // Scan for projects in standard locations
        let projects_dir = self.workspace_root.join("projects");
        if projects_dir.exists() {
            debug!("Scanning projects directory: {:?}", projects_dir);
            projects.extend(self.scan_directory(&projects_dir).await?);
        }

        // Scan for crates in standard locations
        let crates_dir = self.workspace_root.join("crates");
        if crates_dir.exists() {
            debug!("Scanning crates directory: {:?}", crates_dir);
            projects.extend(self.scan_directory(&crates_dir).await?);
        }

        info!("Discovered {} projects", projects.len());
        Ok(projects)
    }

    /// Scans a directory for projects
    ///
    /// # Arguments
    ///
    /// * `dir` - The directory to scan
    ///
    /// # Returns
    ///
    /// A vector of projects found in the directory
    async fn scan_directory(&self, dir: &PathBuf) -> Result<Vec<Project>> {
        let mut projects = Vec::new();

        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_dir() {
                        // Check if this directory is a project
                        if let Some(project) = self.detect_project(&path).await {
                            projects.push(project);
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Error reading directory {:?}: {}", dir, e);
            }
        }

        Ok(projects)
    }

    /// Detects if a directory is a project and extracts its metadata
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// A Project if the directory is a valid project, None otherwise
    async fn detect_project(&self, path: &std::path::Path) -> Option<Project> {
        // Check for Cargo.toml (Rust project)
        let cargo_toml = path.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    debug!("Detected Rust project: {}", name_str);
                    let version = self.extract_rust_version(&cargo_toml).await;
                    return Some(Project {
                        path: path.to_path_buf(),
                        name: name_str.to_string(),
                        project_type: "rust".to_string(),
                        version,
                        status: ProjectStatus::Unknown,
                    });
                }
            }
        }

        // Check for package.json (Node.js project)
        let package_json = path.join("package.json");
        if package_json.exists() {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    debug!("Detected Node.js project: {}", name_str);
                    let version = self.extract_nodejs_version(&package_json).await;
                    return Some(Project {
                        path: path.to_path_buf(),
                        name: name_str.to_string(),
                        project_type: "nodejs".to_string(),
                        version,
                        status: ProjectStatus::Unknown,
                    });
                }
            }
        }

        // Check for pyproject.toml (Python project)
        let pyproject_toml = path.join("pyproject.toml");
        if pyproject_toml.exists() {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    debug!("Detected Python project: {}", name_str);
                    let version = self.extract_python_version(&pyproject_toml).await;
                    return Some(Project {
                        path: path.to_path_buf(),
                        name: name_str.to_string(),
                        project_type: "python".to_string(),
                        version,
                        status: ProjectStatus::Unknown,
                    });
                }
            }
        }

        None
    }

    /// Extracts version from Cargo.toml
    async fn extract_rust_version(&self, cargo_toml: &PathBuf) -> String {
        match std::fs::read_to_string(cargo_toml) {
            Ok(content) => {
                // Simple version extraction from Cargo.toml
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
    async fn extract_nodejs_version(&self, package_json: &PathBuf) -> String {
        match std::fs::read_to_string(package_json) {
            Ok(content) => {
                // Simple version extraction from package.json
                // Look for "version": "X.Y.Z" pattern
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
                            if !version.is_empty() && !version.contains('{') && !version.contains('}') {
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
    async fn extract_python_version(&self, pyproject_toml: &PathBuf) -> String {
        match std::fs::read_to_string(pyproject_toml) {
            Ok(content) => {
                // Simple version extraction from pyproject.toml
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_workspace_scanner_creation() {
        let root = PathBuf::from("/test/workspace");
        let scanner = WorkspaceScanner::new(root.clone());

        assert_eq!(scanner.workspace_root, root);
    }

    #[tokio::test]
    async fn test_scan_nonexistent_workspace() {
        let root = PathBuf::from("/nonexistent/workspace");
        let scanner = WorkspaceScanner::new(root);

        let projects = scanner.scan_workspace().await.expect("scan failed");
        assert_eq!(projects.len(), 0);
    }

    #[tokio::test]
    async fn test_scan_empty_workspace() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let root = temp_dir.path().to_path_buf();

        let scanner = WorkspaceScanner::new(root);
        let projects = scanner.scan_workspace().await.expect("scan failed");

        assert_eq!(projects.len(), 0);
    }

    #[tokio::test]
    async fn test_detect_rust_project() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_dir).expect("failed to create project dir");

        // Create Cargo.toml
        std::fs::write(project_dir.join("Cargo.toml"), "[package]\nname = \"test\"")
            .expect("failed to write Cargo.toml");

        let scanner = WorkspaceScanner::new(temp_dir.path().to_path_buf());
        let project = scanner.detect_project(&project_dir).await;

        assert!(project.is_some());
        let proj = project.unwrap();
        assert_eq!(proj.name, "test-project");
        assert_eq!(proj.project_type, "rust");
    }

    #[tokio::test]
    async fn test_detect_nodejs_project() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().join("node-project");
        std::fs::create_dir(&project_dir).expect("failed to create project dir");

        // Create package.json
        std::fs::write(project_dir.join("package.json"), "{}")
            .expect("failed to write package.json");

        let scanner = WorkspaceScanner::new(temp_dir.path().to_path_buf());
        let project = scanner.detect_project(&project_dir).await;

        assert!(project.is_some());
        let proj = project.unwrap();
        assert_eq!(proj.name, "node-project");
        assert_eq!(proj.project_type, "nodejs");
    }

    #[tokio::test]
    async fn test_detect_python_project() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let project_dir = temp_dir.path().join("python-project");
        std::fs::create_dir(&project_dir).expect("failed to create project dir");

        // Create pyproject.toml
        std::fs::write(project_dir.join("pyproject.toml"), "[build-system]")
            .expect("failed to write pyproject.toml");

        let scanner = WorkspaceScanner::new(temp_dir.path().to_path_buf());
        let project = scanner.detect_project(&project_dir).await;

        assert!(project.is_some());
        let proj = project.unwrap();
        assert_eq!(proj.name, "python-project");
        assert_eq!(proj.project_type, "python");
    }

    #[tokio::test]
    async fn test_scan_workspace_with_projects() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let projects_dir = temp_dir.path().join("projects");
        std::fs::create_dir(&projects_dir).expect("failed to create projects dir");

        // Create a Rust project
        let rust_project = projects_dir.join("rust-project");
        std::fs::create_dir(&rust_project).expect("failed to create rust project");
        std::fs::write(rust_project.join("Cargo.toml"), "[package]\nname = \"test\"")
            .expect("failed to write Cargo.toml");

        let scanner = WorkspaceScanner::new(temp_dir.path().to_path_buf());
        let projects = scanner.scan_workspace().await.expect("scan failed");

        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "rust-project");
        assert_eq!(projects[0].project_type, "rust");
    }

    #[tokio::test]
    async fn test_extract_rust_version() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            "[package]\nname = \"test\"\nversion = \"0.2.5\"\n",
        )
        .expect("failed to write Cargo.toml");

        let scanner = WorkspaceScanner::new(temp_dir.path().to_path_buf());
        let version = scanner.extract_rust_version(&cargo_toml).await;

        assert_eq!(version, "0.2.5");
    }

    #[tokio::test]
    async fn test_extract_nodejs_version() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let package_json = temp_dir.path().join("package.json");
        std::fs::write(
            &package_json,
            r#"{"name": "test", "version": "1.0.0"}"#,
        )
        .expect("failed to write package.json");

        let scanner = WorkspaceScanner::new(temp_dir.path().to_path_buf());
        let version = scanner.extract_nodejs_version(&package_json).await;

        assert_eq!(version, "1.0.0");
    }

    #[tokio::test]
    async fn test_extract_python_version() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let pyproject_toml = temp_dir.path().join("pyproject.toml");
        std::fs::write(
            &pyproject_toml,
            "[project]\nname = \"test\"\nversion = \"2.1.0\"\n",
        )
        .expect("failed to write pyproject.toml");

        let scanner = WorkspaceScanner::new(temp_dir.path().to_path_buf());
        let version = scanner.extract_python_version(&pyproject_toml).await;

        assert_eq!(version, "2.1.0");
    }

    #[tokio::test]
    async fn test_extract_rust_version_with_quotes() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            "[package]\nname = \"test\"\nversion = \"0.3.0\"\n",
        )
        .expect("failed to write Cargo.toml");

        let scanner = WorkspaceScanner::new(temp_dir.path().to_path_buf());
        let version = scanner.extract_rust_version(&cargo_toml).await;

        assert_eq!(version, "0.3.0");
    }

    #[tokio::test]
    async fn test_extract_version_from_malformed_file() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, "[package]\nname = \"test\"\n")
            .expect("failed to write Cargo.toml");

        let scanner = WorkspaceScanner::new(temp_dir.path().to_path_buf());
        let version = scanner.extract_rust_version(&cargo_toml).await;

        // Should return default version when version field is missing
        assert_eq!(version, "0.1.0");
    }

    #[tokio::test]
    async fn test_scan_workspace_with_multiple_projects() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let projects_dir = temp_dir.path().join("projects");
        std::fs::create_dir(&projects_dir).expect("failed to create projects dir");

        // Create multiple projects
        for i in 0..3 {
            let project_dir = projects_dir.join(format!("project-{}", i));
            std::fs::create_dir(&project_dir).expect("failed to create project dir");
            std::fs::write(
                project_dir.join("Cargo.toml"),
                format!("[package]\nname = \"project-{}\"\nversion = \"0.{}.0\"\n", i, i),
            )
            .expect("failed to write Cargo.toml");
        }

        let scanner = WorkspaceScanner::new(temp_dir.path().to_path_buf());
        let projects = scanner.scan_workspace().await.expect("scan failed");

        assert_eq!(projects.len(), 3);
        for (i, project) in projects.iter().enumerate() {
            assert_eq!(project.name, format!("project-{}", i));
            assert_eq!(project.project_type, "rust");
        }
    }

    #[tokio::test]
    async fn test_scan_workspace_with_mixed_projects() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let projects_dir = temp_dir.path().join("projects");
        std::fs::create_dir(&projects_dir).expect("failed to create projects dir");

        // Create a Rust project
        let rust_project = projects_dir.join("rust-project");
        std::fs::create_dir(&rust_project).expect("failed to create rust project");
        std::fs::write(rust_project.join("Cargo.toml"), "[package]\nname = \"test\"")
            .expect("failed to write Cargo.toml");

        // Create a Node.js project
        let node_project = projects_dir.join("node-project");
        std::fs::create_dir(&node_project).expect("failed to create node project");
        std::fs::write(node_project.join("package.json"), "{}")
            .expect("failed to write package.json");

        // Create a Python project
        let python_project = projects_dir.join("python-project");
        std::fs::create_dir(&python_project).expect("failed to create python project");
        std::fs::write(python_project.join("pyproject.toml"), "[build-system]")
            .expect("failed to write pyproject.toml");

        let scanner = WorkspaceScanner::new(temp_dir.path().to_path_buf());
        let projects = scanner.scan_workspace().await.expect("scan failed");

        assert_eq!(projects.len(), 3);

        let project_types: Vec<_> = projects.iter().map(|p| p.project_type.as_str()).collect();
        assert!(project_types.contains(&"rust"));
        assert!(project_types.contains(&"nodejs"));
        assert!(project_types.contains(&"python"));
    }
}
