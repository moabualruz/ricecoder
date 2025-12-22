//! Project analyzer for detecting project type and structure

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::{
    error::ResearchError,
    models::{Framework, Language, ProjectStructure, ProjectType},
};

/// Analyzes project structure and metadata to understand project type and organization
#[derive(Debug)]
pub struct ProjectAnalyzer;

impl ProjectAnalyzer {
    /// Create a new ProjectAnalyzer
    pub fn new() -> Self {
        ProjectAnalyzer
    }

    /// Detect the type of project at the given path
    ///
    /// Analyzes the project structure to determine if it's a library, application,
    /// service, monorepo, or unknown type.
    ///
    /// # Arguments
    ///
    /// * `root` - Root path of the project
    ///
    /// # Returns
    ///
    /// The detected `ProjectType`, or a `ResearchError`
    pub fn detect_type(&self, root: &Path) -> Result<ProjectType, ResearchError> {
        if !root.exists() {
            return Err(ResearchError::ProjectNotFound {
                path: root.to_path_buf(),
                reason: "Cannot detect project type: directory does not exist".to_string(),
            });
        }

        // Detect languages first to understand project type
        let languages = self.detect_languages(root)?;

        // Check for monorepo patterns
        if self.is_monorepo(root, &languages)? {
            return Ok(ProjectType::Monorepo);
        }

        // Check for library vs application
        if self.is_library(root, &languages)? {
            Ok(ProjectType::Library)
        } else if self.is_service(root, &languages)? {
            Ok(ProjectType::Service)
        } else {
            Ok(ProjectType::Application)
        }
    }

    /// Analyze the structure of a project
    ///
    /// Identifies source directories, test directories, configuration files,
    /// and entry points.
    ///
    /// # Arguments
    ///
    /// * `root` - Root path of the project
    ///
    /// # Returns
    ///
    /// A `ProjectStructure` containing the analysis, or a `ResearchError`
    pub fn analyze_structure(&self, root: &Path) -> Result<ProjectStructure, ResearchError> {
        if !root.exists() {
            return Err(ResearchError::ProjectNotFound {
                path: root.to_path_buf(),
                reason: "Cannot analyze structure: directory does not exist".to_string(),
            });
        }

        let source_dirs = self.find_source_directories(root)?;
        let test_dirs = self.find_test_directories(root)?;
        let config_files = self.find_config_files(root)?;
        let entry_points = self.find_entry_points(root)?;

        Ok(ProjectStructure {
            root: root.to_path_buf(),
            source_dirs,
            test_dirs,
            config_files,
            entry_points,
        })
    }

    /// Identify frameworks and libraries used in the project
    ///
    /// # Arguments
    ///
    /// * `root` - Root path of the project
    ///
    /// # Returns
    ///
    /// A vector of detected `Framework`s, or a `ResearchError`
    pub fn identify_frameworks(&self, root: &Path) -> Result<Vec<Framework>, ResearchError> {
        if !root.exists() {
            return Err(ResearchError::ProjectNotFound {
                path: root.to_path_buf(),
                reason: "Cannot identify frameworks: directory does not exist".to_string(),
            });
        }

        let mut frameworks = Vec::new();

        // Check for common frameworks based on manifest files
        if let Ok(cargo_toml) = std::fs::read_to_string(root.join("Cargo.toml")) {
            // Parse Cargo.toml for dependencies
            if cargo_toml.contains("tokio") {
                frameworks.push(Framework {
                    name: "tokio".to_string(),
                    version: self.extract_version(&cargo_toml, "tokio"),
                });
            }
            if cargo_toml.contains("serde") {
                frameworks.push(Framework {
                    name: "serde".to_string(),
                    version: self.extract_version(&cargo_toml, "serde"),
                });
            }
            if cargo_toml.contains("actix") {
                frameworks.push(Framework {
                    name: "actix".to_string(),
                    version: self.extract_version(&cargo_toml, "actix"),
                });
            }
            if cargo_toml.contains("axum") {
                frameworks.push(Framework {
                    name: "axum".to_string(),
                    version: self.extract_version(&cargo_toml, "axum"),
                });
            }
        }

        if let Ok(package_json) = std::fs::read_to_string(root.join("package.json")) {
            // Parse package.json for dependencies
            if package_json.contains("\"react\"") {
                frameworks.push(Framework {
                    name: "react".to_string(),
                    version: self.extract_json_version(&package_json, "react"),
                });
            }
            if package_json.contains("\"express\"") {
                frameworks.push(Framework {
                    name: "express".to_string(),
                    version: self.extract_json_version(&package_json, "express"),
                });
            }
            if package_json.contains("\"next\"") {
                frameworks.push(Framework {
                    name: "next".to_string(),
                    version: self.extract_json_version(&package_json, "next"),
                });
            }
        }

        Ok(frameworks)
    }

    // ========================================================================
    // Private helper methods
    // ========================================================================

    /// Detect programming languages used in the project
    fn detect_languages(&self, root: &Path) -> Result<Vec<Language>, ResearchError> {
        let mut languages = Vec::new();

        // Check for Rust
        if root.join("Cargo.toml").exists() {
            languages.push(Language::Rust);
        }

        // Check for Node.js
        if root.join("package.json").exists() {
            languages.push(Language::TypeScript);
        }

        // Check for Python
        if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
            languages.push(Language::Python);
        }

        // Check for Go
        if root.join("go.mod").exists() {
            languages.push(Language::Go);
        }

        // Check for Java
        if root.join("pom.xml").exists() || root.join("build.gradle").exists() {
            languages.push(Language::Java);
        }

        // Check for Kotlin
        if root.join("build.gradle.kts").exists() {
            languages.push(Language::Kotlin);
        }

        // Check for .NET
        if self.has_csproj_files(root)? || root.join("packages.config").exists() {
            languages.push(Language::CSharp);
        }

        // Check for PHP
        if root.join("composer.json").exists() {
            languages.push(Language::Php);
        }

        // Check for Ruby
        if root.join("Gemfile").exists() {
            languages.push(Language::Ruby);
        }

        // Check for Swift
        if root.join("Package.swift").exists() {
            languages.push(Language::Swift);
        }

        // Check for Dart
        if root.join("pubspec.yaml").exists() {
            languages.push(Language::Dart);
        }

        Ok(languages)
    }

    /// Check if the project is a monorepo
    fn is_monorepo(&self, root: &Path, languages: &[Language]) -> Result<bool, ResearchError> {
        // Rust workspace
        if languages.contains(&Language::Rust) {
            if let Ok(cargo_toml) = std::fs::read_to_string(root.join("Cargo.toml")) {
                if cargo_toml.contains("[workspace]") {
                    return Ok(true);
                }
            }
        }

        // Node.js monorepo (lerna, yarn workspaces, npm workspaces)
        if languages.contains(&Language::TypeScript) {
            if let Ok(package_json) = std::fs::read_to_string(root.join("package.json")) {
                if package_json.contains("\"workspaces\"") {
                    return Ok(true);
                }
            }
            if root.join("lerna.json").exists() {
                return Ok(true);
            }
        }

        // Check for multiple independent projects in subdirectories
        let mut project_count = 0;
        for entry in WalkDir::new(root)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.join("Cargo.toml").exists()
                || path.join("package.json").exists()
                || path.join("pyproject.toml").exists()
            {
                project_count += 1;
            }
        }

        Ok(project_count > 1)
    }

    /// Check if the project is a library
    fn is_library(&self, root: &Path, languages: &[Language]) -> Result<bool, ResearchError> {
        // Rust library
        if languages.contains(&Language::Rust) {
            if let Ok(cargo_toml) = std::fs::read_to_string(root.join("Cargo.toml")) {
                // Check for [lib] section
                if cargo_toml.contains("[lib]") {
                    return Ok(true);
                }
                // Check if there's no [[bin]] section
                if !cargo_toml.contains("[[bin]]") && root.join("src/lib.rs").exists() {
                    return Ok(true);
                }
            }
        }

        // Node.js library (has "main" or "exports" in package.json)
        if languages.contains(&Language::TypeScript) {
            if let Ok(package_json) = std::fs::read_to_string(root.join("package.json")) {
                if (package_json.contains("\"main\"") || package_json.contains("\"exports\""))
                    && !package_json.contains("\"bin\"")
                {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Check if the project is a service/microservice
    fn is_service(&self, root: &Path, _languages: &[Language]) -> Result<bool, ResearchError> {
        // Check for common service indicators
        if let Ok(cargo_toml) = std::fs::read_to_string(root.join("Cargo.toml")) {
            // Web frameworks indicate a service
            if cargo_toml.contains("actix")
                || cargo_toml.contains("axum")
                || cargo_toml.contains("rocket")
            {
                return Ok(true);
            }
        }

        if let Ok(package_json) = std::fs::read_to_string(root.join("package.json")) {
            // Express, Fastify, etc. indicate a service
            if package_json.contains("\"express\"")
                || package_json.contains("\"fastify\"")
                || package_json.contains("\"koa\"")
            {
                return Ok(true);
            }
        }

        // Check for Dockerfile (common in services)
        if root.join("Dockerfile").exists() {
            return Ok(true);
        }

        Ok(false)
    }

    /// Find source directories in the project
    fn find_source_directories(&self, root: &Path) -> Result<Vec<PathBuf>, ResearchError> {
        let mut source_dirs = Vec::new();

        // Common source directory patterns
        let common_patterns = vec!["src", "lib", "app", "source", "code"];

        for pattern in common_patterns {
            let path = root.join(pattern);
            if path.exists() && path.is_dir() {
                source_dirs.push(path);
            }
        }

        // Language-specific patterns
        if (root.join("src/main.rs").exists() || root.join("src/lib.rs").exists())
            && !source_dirs.contains(&root.join("src"))
        {
            source_dirs.push(root.join("src"));
        }

        Ok(source_dirs)
    }

    /// Find test directories in the project
    fn find_test_directories(&self, root: &Path) -> Result<Vec<PathBuf>, ResearchError> {
        let mut test_dirs = Vec::new();

        // Common test directory patterns
        let common_patterns = vec!["tests", "test", "__tests__", "spec", "specs"];

        for pattern in common_patterns {
            let path = root.join(pattern);
            if path.exists() && path.is_dir() {
                test_dirs.push(path);
            }
        }

        Ok(test_dirs)
    }

    /// Find configuration files in the project
    fn find_config_files(&self, root: &Path) -> Result<Vec<PathBuf>, ResearchError> {
        let mut config_files = Vec::new();

        // Common configuration files
        let config_patterns = vec![
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "go.mod",
            "pom.xml",
            "build.gradle",
            "build.gradle.kts",
            ".csproj",
            "composer.json",
            "Gemfile",
            "Package.swift",
            "pubspec.yaml",
            "Dockerfile",
            ".env",
            ".env.example",
            "tsconfig.json",
            "jest.config.js",
            "webpack.config.js",
        ];

        for pattern in config_patterns {
            let path = root.join(pattern);
            if path.exists() && path.is_file() {
                config_files.push(path);
            }
        }

        Ok(config_files)
    }

    /// Find entry points in the project
    fn find_entry_points(&self, root: &Path) -> Result<Vec<PathBuf>, ResearchError> {
        let mut entry_points = Vec::new();

        // Rust entry points
        if root.join("src/main.rs").exists() {
            entry_points.push(root.join("src/main.rs"));
        }

        // Node.js entry points
        if let Ok(package_json) = std::fs::read_to_string(root.join("package.json")) {
            if let Some(main_start) = package_json.find("\"main\"") {
                if let Some(colon_pos) = package_json[main_start..].find(':') {
                    if let Some(quote_start) = package_json[main_start + colon_pos..].find('"') {
                        if let Some(quote_end) =
                            package_json[main_start + colon_pos + quote_start + 1..].find('"')
                        {
                            let main_file = &package_json[main_start + colon_pos + quote_start + 1
                                ..main_start + colon_pos + quote_start + 1 + quote_end];
                            let path = root.join(main_file);
                            if path.exists() {
                                entry_points.push(path);
                            }
                        }
                    }
                }
            }
        }

        // Python entry points
        if root.join("main.py").exists() {
            entry_points.push(root.join("main.py"));
        }
        if root.join("__main__.py").exists() {
            entry_points.push(root.join("__main__.py"));
        }

        // Go entry points
        if root.join("main.go").exists() {
            entry_points.push(root.join("main.go"));
        }

        Ok(entry_points)
    }

    /// Check if project has .csproj files
    fn has_csproj_files(&self, root: &Path) -> Result<bool, ResearchError> {
        for entry in WalkDir::new(root)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().is_some_and(|ext| ext == "csproj") {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Extract version from Cargo.toml
    fn extract_version(&self, content: &str, package: &str) -> Option<String> {
        // Simple regex-free version extraction
        let search_str = format!("{} =", package);
        if let Some(pos) = content.find(&search_str) {
            let after = &content[pos + search_str.len()..];
            if let Some(quote_pos) = after.find('"') {
                if let Some(end_quote) = after[quote_pos + 1..].find('"') {
                    let version = &after[quote_pos + 1..quote_pos + 1 + end_quote];
                    return Some(version.to_string());
                }
            }
        }
        None
    }

    /// Extract version from package.json
    fn extract_json_version(&self, content: &str, package: &str) -> Option<String> {
        let search_str = format!("\"{}\":", package);
        if let Some(pos) = content.find(&search_str) {
            let after = &content[pos + search_str.len()..];
            if let Some(quote_pos) = after.find('"') {
                if let Some(end_quote) = after[quote_pos + 1..].find('"') {
                    let version = &after[quote_pos + 1..quote_pos + 1 + end_quote];
                    return Some(version.to_string());
                }
            }
        }
        None
    }
}

impl Default for ProjectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_project_analyzer_creation() {
        let analyzer = ProjectAnalyzer::new();
        assert_eq!(std::mem::size_of_val(&analyzer), 0);
    }

    #[test]
    fn test_project_analyzer_default() {
        let analyzer = ProjectAnalyzer::default();
        assert_eq!(std::mem::size_of_val(&analyzer), 0);
    }

    #[test]
    fn test_detect_type_nonexistent_path() {
        let analyzer = ProjectAnalyzer::new();
        let result = analyzer.detect_type(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_structure_nonexistent_path() {
        let analyzer = ProjectAnalyzer::new();
        let result = analyzer.analyze_structure(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_identify_frameworks_nonexistent_path() {
        let analyzer = ProjectAnalyzer::new();
        let result = analyzer.identify_frameworks(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_rust_project() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, "[package]\nname = \"test\"\n").unwrap();

        let analyzer = ProjectAnalyzer::new();
        let result = analyzer.detect_type(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_source_directories() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("src")).unwrap();
        std::fs::create_dir(temp_dir.path().join("lib")).unwrap();

        let analyzer = ProjectAnalyzer::new();
        let result = analyzer.find_source_directories(temp_dir.path()).unwrap();
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_find_test_directories() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("tests")).unwrap();
        std::fs::create_dir(temp_dir.path().join("test")).unwrap();

        let analyzer = ProjectAnalyzer::new();
        let result = analyzer.find_test_directories(temp_dir.path()).unwrap();
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_find_config_files() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Cargo.toml"), "").unwrap();
        std::fs::write(temp_dir.path().join("package.json"), "").unwrap();

        let analyzer = ProjectAnalyzer::new();
        let result = analyzer.find_config_files(temp_dir.path()).unwrap();
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_find_entry_points() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("src")).unwrap();
        std::fs::write(temp_dir.path().join("src/main.rs"), "").unwrap();

        let analyzer = ProjectAnalyzer::new();
        let result = analyzer.find_entry_points(temp_dir.path()).unwrap();
        assert!(!result.is_empty());
    }
}
