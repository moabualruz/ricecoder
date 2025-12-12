//! Project bootstrap and auto-detection for RiceCoder TUI
//!
//! This module provides automatic project type detection, configuration loading,
//! and integration initialization when RiceCoder starts in a project directory.

use crate::error::TuiResult;
use crate::code_editor_widget::Language;
// Project analysis moved to ricecoder-research crate
// use ricecoder_research::{ProjectAnalyzer, ProjectType};

// Stub types for TUI isolation
#[derive(Debug, Clone)]
pub struct ProjectAnalyzer;

#[derive(Debug, Clone)]
pub enum ProjectType {
    Rust,
    Python,
    TypeScript,
    Go,
    Java,
    Unknown,
}
use ricecoder_storage::ConfigLoader;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Project bootstrap configuration and state
#[derive(Debug, Clone)]
pub struct ProjectBootstrap {
    /// Current working directory
    pub working_directory: PathBuf,
    /// Detected project type
    pub project_type: Option<ProjectType>,
    /// Detected primary language
    pub primary_language: Option<Language>,
    /// Project-specific configuration
    pub project_config: HashMap<String, serde_json::Value>,
    /// Whether bootstrap has been completed
    pub bootstrapped: bool,
}

/// Project bootstrap result
#[derive(Debug, Clone)]
pub struct BootstrapResult {
    /// Detected project type
    pub project_type: ProjectType,
    /// Primary language
    pub primary_language: Language,
    /// Loaded configurations
    pub configurations: HashMap<String, serde_json::Value>,
    /// Initialized integrations
    pub integrations: Vec<String>,
}

impl ProjectBootstrap {
    /// Create a new project bootstrap instance
    pub fn new(working_directory: PathBuf) -> Self {
        Self {
            working_directory,
            project_type: None,
            primary_language: None,
            project_config: HashMap::new(),
            bootstrapped: false,
        }
    }

    /// Bootstrap the project by detecting type and loading configurations
    pub async fn bootstrap(&mut self) -> TuiResult<BootstrapResult> {
        if self.bootstrapped {
            return Err(crate::error::TuiError::config(
                "Project already bootstrapped".to_string()
            ));
        }

        // 1. Detect project type
        let project_type = self.detect_project_type()?;
        self.project_type = Some(project_type.clone());

        // 2. Detect primary language
        let primary_language = self.detect_primary_language()?;
        self.primary_language = Some(primary_language.clone());

        // 3. Load project configurations
        let configurations = self.load_project_configurations().await?;

        // 4. Initialize integrations
        let integrations = self.initialize_integrations(&project_type, &primary_language).await?;

        self.bootstrapped = true;

        Ok(BootstrapResult {
            project_type,
            primary_language,
            configurations,
            integrations,
        })
    }

    /// Detect the project type using the research analyzer
    fn detect_project_type(&self) -> TuiResult<ProjectType> {
        let analyzer = ProjectAnalyzer::new();

        match analyzer.detect_type(&self.working_directory) {
            Ok(project_type) => Ok(project_type),
            Err(_) => {
                // Fallback: try to detect from common files
                self.fallback_project_detection()
            }
        }
    }

    /// Fallback project detection when analyzer fails
    fn fallback_project_detection(&self) -> TuiResult<ProjectType> {
        // Check for common project files
        if self.working_directory.join("Cargo.toml").exists() {
            Ok(ProjectType::Library) // Could be Application, but Library is safer default
        } else if self.working_directory.join("package.json").exists() {
            Ok(ProjectType::Application)
        } else if self.working_directory.join("requirements.txt").exists()
               || self.working_directory.join("pyproject.toml").exists() {
            Ok(ProjectType::Application)
        } else if self.working_directory.join("go.mod").exists() {
            Ok(ProjectType::Application)
        } else {
            Ok(ProjectType::Unknown)
        }
    }

    /// Detect the primary programming language
    fn detect_primary_language(&self) -> TuiResult<Language> {
        // Check for language-specific files
        if self.working_directory.join("Cargo.toml").exists() {
            Ok(Language::Rust)
        } else if self.working_directory.join("package.json").exists() {
            Ok(Language::TypeScript) // Could be JavaScript, but TypeScript is more common
        } else if self.working_directory.join("requirements.txt").exists()
               || self.working_directory.join("pyproject.toml").exists() {
            Ok(Language::Python)
        } else if self.working_directory.join("go.mod").exists() {
            Ok(Language::Go)
        } else if self.working_directory.join("pom.xml").exists()
               || self.working_directory.join("build.gradle").exists() {
            Ok(Language::Java)
        } else if self.working_directory.join("composer.json").exists() {
            Ok(Language::Php)
        } else {
            Ok(Language::Unknown)
        }
    }

    /// Load project-specific configurations
    async fn load_project_configurations(&self) -> TuiResult<HashMap<String, serde_json::Value>> {
        let mut configs = HashMap::new();

        // Load language-specific configurations
        if let Some(language) = &self.primary_language {
            match language {
                Language::Rust => {
                    self.load_rust_config(&mut configs).await?;
                }
                Language::Python => {
                    self.load_python_config(&mut configs).await?;
                }
                Language::TypeScript => {
                    self.load_typescript_config(&mut configs).await?;
                }
                Language::Go => {
                    self.load_go_config(&mut configs).await?;
                }
                _ => {
                    // Generic configuration loading
                    self.load_generic_config(&mut configs).await?;
                }
            }
        }

        // Load project-specific ricecoder config
        self.load_ricecoder_project_config(&mut configs).await?;

        Ok(configs)
    }

    /// Load Rust-specific configuration
    async fn load_rust_config(&self, configs: &mut HashMap<String, serde_json::Value>) -> TuiResult<()> {
        // Load Cargo.toml for project information
        if let Ok(cargo_content) = tokio::fs::read_to_string(self.working_directory.join("Cargo.toml")).await {
            if let Ok(cargo_toml) = cargo_content.parse::<toml::Value>() {
                configs.insert("cargo".to_string(), serde_json::to_value(cargo_toml).unwrap_or_default());
            }
        }

        // Set Rust-specific defaults
        configs.insert("language".to_string(), serde_json::json!({
            "name": "rust",
            "lsp": "rust-analyzer",
            "formatter": "rustfmt",
            "test_runner": "cargo test"
        }));

        Ok(())
    }

    /// Load Python-specific configuration
    async fn load_python_config(&self, configs: &mut HashMap<String, serde_json::Value>) -> TuiResult<()> {
        // Load pyproject.toml or requirements.txt
        if let Ok(pyproject_content) = tokio::fs::read_to_string(self.working_directory.join("pyproject.toml")).await {
            if let Ok(pyproject) = pyproject_content.parse::<toml::Value>() {
                configs.insert("pyproject".to_string(), serde_json::to_value(pyproject).unwrap_or_default());
            }
        }

        if let Ok(req_content) = tokio::fs::read_to_string(self.working_directory.join("requirements.txt")).await {
            configs.insert("requirements".to_string(), serde_json::json!(req_content));
        }

        // Set Python-specific defaults
        configs.insert("language".to_string(), serde_json::json!({
            "name": "python",
            "lsp": "pylsp",
            "formatter": "black",
            "test_runner": "pytest"
        }));

        Ok(())
    }

    /// Load TypeScript-specific configuration
    async fn load_typescript_config(&self, configs: &mut HashMap<String, serde_json::Value>) -> TuiResult<()> {
        // Load package.json
        if let Ok(package_content) = tokio::fs::read_to_string(self.working_directory.join("package.json")).await {
            if let Ok(package_json) = serde_json::from_str(&package_content) {
                configs.insert("package".to_string(), package_json);
            }
        }

        // Load tsconfig.json
        if let Ok(tsconfig_content) = tokio::fs::read_to_string(self.working_directory.join("tsconfig.json")).await {
            if let Ok(tsconfig) = serde_json::from_str(&tsconfig_content) {
                configs.insert("tsconfig".to_string(), tsconfig);
            }
        }

        // Set TypeScript-specific defaults
        configs.insert("language".to_string(), serde_json::json!({
            "name": "typescript",
            "lsp": "typescript-language-server",
            "formatter": "prettier",
            "test_runner": "jest"
        }));

        Ok(())
    }

    /// Load Go-specific configuration
    async fn load_go_config(&self, configs: &mut HashMap<String, serde_json::Value>) -> TuiResult<()> {
        // Load go.mod
        if let Ok(gomod_content) = tokio::fs::read_to_string(self.working_directory.join("go.mod")).await {
            configs.insert("go_mod".to_string(), serde_json::json!(gomod_content));
        }

        // Set Go-specific defaults
        configs.insert("language".to_string(), serde_json::json!({
            "name": "go",
            "lsp": "gopls",
            "formatter": "gofmt",
            "test_runner": "go test"
        }));

        Ok(())
    }

    /// Load generic project configuration
    async fn load_generic_config(&self, configs: &mut HashMap<String, serde_json::Value>) -> TuiResult<()> {
        configs.insert("language".to_string(), serde_json::json!({
            "name": "unknown",
            "lsp": null,
            "formatter": null,
            "test_runner": null
        }));

        Ok(())
    }

    /// Load RiceCoder project-specific configuration
    async fn load_ricecoder_project_config(&self, configs: &mut HashMap<String, serde_json::Value>) -> TuiResult<()> {
        // Look for .ricecoder directory or config files
        let ricecoder_config = self.working_directory.join(".ricecoder");
        if ricecoder_config.exists() {
            // Load project-specific ricecoder config
            let config_path = ricecoder_config.join("config.yaml");
            if config_path.exists() {
                if let Ok(config_content) = tokio::fs::read_to_string(&config_path).await {
                    if let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&config_content) {
                        configs.insert("ricecoder_project".to_string(), serde_json::to_value(config).unwrap_or_default());
                    }
                }
            }
        }

        Ok(())
    }

    /// Initialize integrations based on project type and language
    async fn initialize_integrations(&self, _project_type: &ProjectType, language: &Language) -> TuiResult<Vec<String>> {
        let mut integrations = Vec::new();

        // Language-specific integrations
        match language {
            Language::Rust => {
                integrations.push("rust-analyzer".to_string());
                integrations.push("cargo".to_string());
                integrations.push("rustfmt".to_string());
            }
            Language::Python => {
                integrations.push("pylsp".to_string());
                integrations.push("black".to_string());
                integrations.push("pytest".to_string());
            }
            Language::TypeScript => {
                integrations.push("typescript-language-server".to_string());
                integrations.push("prettier".to_string());
                integrations.push("eslint".to_string());
            }
            Language::Go => {
                integrations.push("gopls".to_string());
                integrations.push("gofmt".to_string());
                integrations.push("go".to_string());
            }
            _ => {}
        }

        // VCS integration (always try to initialize)
        integrations.push("git".to_string());

        // File watching
        integrations.push("file_watcher".to_string());

        Ok(integrations)
    }

    /// Get project information for display
    pub fn get_project_info(&self) -> Option<ProjectInfo> {
        if !self.bootstrapped {
            return None;
        }

        Some(ProjectInfo {
            project_type: self.project_type.clone()?,
            primary_language: self.primary_language.clone()?,
            working_directory: self.working_directory.clone(),
            config_keys: self.project_config.keys().cloned().collect(),
        })
    }
}

/// Project information for display
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub project_type: ProjectType,
    pub primary_language: Language,
    pub working_directory: PathBuf,
    pub config_keys: Vec<String>,
}

impl ProjectInfo {
    /// Get display name for project type
    pub fn project_type_display(&self) -> &'static str {
        match self.project_type {
            ProjectType::Library => "Library",
            ProjectType::Application => "Application",
            ProjectType::Service => "Service",
            ProjectType::Monorepo => "Monorepo",
            ProjectType::Unknown => "Unknown",
        }
    }

    /// Get display name for language
    pub fn language_display(&self) -> &'static str {
        match self.primary_language {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Php => "PHP",
            Language::Ruby => "Ruby",
            Language::Swift => "Swift",
            Language::Scala => "Scala",
            Language::Shell => "Shell",
            Language::Kotlin => "Kotlin",
            Language::Yaml => "YAML",
            Language::Json => "JSON",
            Language::Markdown => "Markdown",
            Language::Sql => "SQL",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::PlainText => "Plain Text",
            Language::Unknown => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rust_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_path = temp_dir.path().join("Cargo.toml");
        tokio::fs::write(&cargo_path, r#"
[package]
name = "test"
version = "0.1.0"
        "#).await.unwrap();

        let bootstrap = ProjectBootstrap::new(temp_dir.path().to_path_buf());
        let result = bootstrap.bootstrap().await.unwrap();

        assert_eq!(result.primary_language, Language::Rust);
    }

    #[tokio::test]
    async fn test_python_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        let req_path = temp_dir.path().join("requirements.txt");
        tokio::fs::write(&req_path, "requests==2.25.1\n").await.unwrap();

        let bootstrap = ProjectBootstrap::new(temp_dir.path().to_path_buf());
        let result = bootstrap.bootstrap().await.unwrap();

        assert_eq!(result.primary_language, Language::Python);
    }

    #[tokio::test]
    async fn test_typescript_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("package.json");
        tokio::fs::write(&package_path, r#"{"name": "test", "version": "1.0.0"}"#).await.unwrap();

        let bootstrap = ProjectBootstrap::new(temp_dir.path().to_path_buf());
        let result = bootstrap.bootstrap().await.unwrap();

        assert_eq!(result.primary_language, Language::TypeScript);
    }
}
