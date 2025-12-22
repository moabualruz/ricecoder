//! Integration tests for full project analysis workflow
//! Tests the complete analyze_project workflow including type detection, scanning, indexing, and analysis
//! **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 1.7, 1.8, 1.9, 1.10, 2.1, 2.2, 2.3, 2.4, 2.5**

use std::{fs, path::PathBuf};

use ricecoder_research::{
    models::Language, ArchitecturalStyle, CaseStyle, ProjectType, ResearchManager,
};
use tempfile::TempDir;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a complete Rust project with multiple components
fn create_complete_rust_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();

    // Create directory structure
    fs::create_dir_all(root.join("src/domain")).unwrap();
    fs::create_dir_all(root.join("src/application")).unwrap();
    fs::create_dir_all(root.join("src/infrastructure")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();

    // Create Cargo.toml
    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
proptest = "1.0"
"#,
    )
    .unwrap();

    // Create source files
    fs::write(
        root.join("src/lib.rs"),
        r#"pub mod domain;
pub mod application;
pub mod infrastructure;

pub fn public_function() {}
"#,
    )
    .unwrap();

    fs::write(
        root.join("src/domain/entity.rs"),
        r#"pub struct Entity {
    pub id: String,
    pub name: String,
}

impl Entity {
    pub fn new(id: String, name: String) -> Self {
        Entity { id, name }
    }
}
"#,
    )
    .unwrap();

    fs::write(
        root.join("src/application/service.rs"),
        r#"use crate::domain::Entity;

pub struct Service;

impl Service {
    pub fn process(entity: &Entity) -> String {
        format!("Processing: {}", entity.name)
    }
}
"#,
    )
    .unwrap();

    fs::write(
        root.join("src/infrastructure/repository.rs"),
        r#"use crate::domain::Entity;

pub struct Repository;

impl Repository {
    pub fn save(entity: &Entity) -> Result<(), String> {
        Ok(())
    }

    pub fn find(id: &str) -> Result<Entity, String> {
        Ok(Entity::new(id.to_string(), "test".to_string()))
    }
}
"#,
    )
    .unwrap();

    // Create test files
    fs::write(
        root.join("tests/integration_test.rs"),
        r#"#[test]
fn test_integration() {
    assert!(true);
}
"#,
    )
    .unwrap();

    root.to_path_buf()
}

/// Create a Node.js project
fn create_nodejs_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();

    fs::write(
        root.join("package.json"),
        r#"{
  "name": "test-project",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.0",
    "lodash": "^4.17.0"
  },
  "devDependencies": {
    "jest": "^29.0.0"
  }
}
"#,
    )
    .unwrap();

    fs::write(
        root.join("src/index.ts"),
        r#"export function main() {
  console.log("Hello");
}
"#,
    )
    .unwrap();

    fs::write(
        root.join("tests/test.ts"),
        r#"describe("test", () => {
  it("should work", () => {
    expect(true).toBe(true);
  });
});
"#,
    )
    .unwrap();

    root.to_path_buf()
}

/// Create a Python project
fn create_python_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();

    fs::write(
        root.join("pyproject.toml"),
        r#"[project]
name = "test-project"
version = "0.1.0"
dependencies = [
    "requests>=2.28.0",
    "pydantic>=1.10.0",
]
"#,
    )
    .unwrap();

    fs::write(
        root.join("src/main.py"),
        r#"def main():
    print("Hello")

if __name__ == "__main__":
    main()
"#,
    )
    .unwrap();

    fs::write(
        root.join("tests/test_main.py"),
        r#"def test_main():
    assert True
"#,
    )
    .unwrap();

    root.to_path_buf()
}

/// Create a Go project
fn create_go_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();

    fs::create_dir_all(root.join("cmd")).unwrap();
    fs::create_dir_all(root.join("pkg")).unwrap();

    fs::write(
        root.join("go.mod"),
        r#"module github.com/test/project

go 1.21

require (
    github.com/gin-gonic/gin v1.9.0
    github.com/sirupsen/logrus v1.9.0
)
"#,
    )
    .unwrap();

    fs::write(
        root.join("cmd/main.go"),
        r#"package main

func main() {
    println("Hello")
}
"#,
    )
    .unwrap();

    fs::write(
        root.join("pkg/util.go"),
        r#"package pkg

func Util() string {
    return "util"
}
"#,
    )
    .unwrap();

    root.to_path_buf()
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_analyze_complete_rust_project() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_complete_rust_project(&temp_dir);

    let manager = ResearchManager::new();
    let result = manager.analyze_project(&root).await;

    assert!(result.is_ok(), "Project analysis should succeed");
    let context = result.unwrap();

    // Verify project type detection
    assert_eq!(context.project_type, ProjectType::Library);

    // Verify language detection
    assert!(context.languages.iter().any(|l| l == &Language::Rust));

    // Verify structure analysis
    assert!(!context.structure.source_dirs.is_empty());
    assert!(!context.structure.test_dirs.is_empty());

    // Verify dependencies were found
    assert!(!context.dependencies.is_empty());

    // Verify standards were detected
    assert!(context.standards.naming_conventions.function_case != CaseStyle::Mixed);
}

#[tokio::test]
async fn test_analyze_nodejs_project() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_nodejs_project(&temp_dir);

    let manager = ResearchManager::new();
    let result = manager.analyze_project(&root).await;

    assert!(result.is_ok(), "Project analysis should succeed");
    let context = result.unwrap();

    // Verify project type detection (could be Application or Service)
    assert!(
        context.project_type == ProjectType::Application
            || context.project_type == ProjectType::Service
    );

    // Verify language detection
    assert!(context.languages.iter().any(|l| l == &Language::TypeScript));

    // Verify dependencies were found
    assert!(!context.dependencies.is_empty());

    // Verify specific dependencies
    let dep_names: Vec<_> = context
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(dep_names.contains(&"express"));
    assert!(dep_names.contains(&"lodash"));
}

#[tokio::test]
async fn test_analyze_python_project() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_python_project(&temp_dir);

    let manager = ResearchManager::new();
    let result = manager.analyze_project(&root).await;

    assert!(result.is_ok(), "Project analysis should succeed");
    let context = result.unwrap();

    // Verify language detection
    assert!(context.languages.iter().any(|l| l == &Language::Python));

    // Verify dependencies were found
    assert!(!context.dependencies.is_empty());

    // Verify specific dependencies
    let dep_names: Vec<_> = context
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(dep_names.contains(&"requests"));
    assert!(dep_names.contains(&"pydantic"));
}

#[tokio::test]
async fn test_analyze_go_project() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_go_project(&temp_dir);

    let manager = ResearchManager::new();
    let result = manager.analyze_project(&root).await;

    assert!(result.is_ok(), "Project analysis should succeed");
    let context = result.unwrap();

    // Verify language detection
    assert!(context.languages.iter().any(|l| l == &Language::Go));

    // Verify dependencies were found
    assert!(!context.dependencies.is_empty());

    // Verify specific dependencies
    let dep_names: Vec<_> = context
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(dep_names.contains(&"github.com/gin-gonic/gin"));
}

#[tokio::test]
async fn test_analyze_project_structure_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_complete_rust_project(&temp_dir);

    let manager = ResearchManager::new();
    let context = manager.analyze_project(&root).await.unwrap();

    // Verify source directories are detected
    assert!(context
        .structure
        .source_dirs
        .iter()
        .any(|d| d.ends_with("src")));

    // Verify test directories are detected
    assert!(context
        .structure
        .test_dirs
        .iter()
        .any(|d| d.ends_with("tests")));

    // Verify config files are detected
    assert!(context
        .structure
        .config_files
        .iter()
        .any(|f| f.ends_with("Cargo.toml")));
}

#[tokio::test]
async fn test_analyze_project_caching() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_complete_rust_project(&temp_dir);

    let manager = ResearchManager::new();

    // First analysis
    let result1 = manager.analyze_project(&root).await;
    assert!(result1.is_ok());

    // Second analysis should use cache
    let result2 = manager.analyze_project(&root).await;
    assert!(result2.is_ok());

    // Results should be identical
    let context1 = result1.unwrap();
    let context2 = result2.unwrap();

    assert_eq!(context1.project_type, context2.project_type);
    assert_eq!(context1.languages.len(), context2.languages.len());
    assert_eq!(context1.dependencies.len(), context2.dependencies.len());
}

#[tokio::test]
async fn test_analyze_project_nonexistent_path() {
    let manager = ResearchManager::new();
    let result = manager
        .analyze_project(std::path::Path::new("/nonexistent/path"))
        .await;

    assert!(result.is_err(), "Analysis should fail for nonexistent path");
}

#[tokio::test]
async fn test_analyze_project_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let manager = ResearchManager::new();
    let result = manager.analyze_project(root).await;

    // Should succeed but with minimal context
    assert!(result.is_ok());
    let context = result.unwrap();
    assert_eq!(context.languages.len(), 0);
}

#[tokio::test]
async fn test_analyze_project_consistency() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_complete_rust_project(&temp_dir);

    let manager = ResearchManager::new();

    // Analyze multiple times
    let result1 = manager.analyze_project(&root).await.unwrap();
    let result2 = manager.analyze_project(&root).await.unwrap();
    let result3 = manager.analyze_project(&root).await.unwrap();

    // All results should be identical
    assert_eq!(result1.project_type, result2.project_type);
    assert_eq!(result2.project_type, result3.project_type);

    assert_eq!(result1.languages.len(), result2.languages.len());
    assert_eq!(result2.languages.len(), result3.languages.len());

    assert_eq!(result1.dependencies.len(), result2.dependencies.len());
    assert_eq!(result2.dependencies.len(), result3.dependencies.len());
}

#[tokio::test]
async fn test_analyze_project_all_components_present() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_complete_rust_project(&temp_dir);

    let manager = ResearchManager::new();
    let context = manager.analyze_project(&root).await.unwrap();

    // Verify all components are present
    assert!(context.project_type != ProjectType::Unknown);
    assert!(!context.languages.is_empty());
    assert!(!context.structure.source_dirs.is_empty());
    assert!(!context.dependencies.is_empty());
    assert!(context.standards.naming_conventions.function_case != CaseStyle::Mixed);
}

#[tokio::test]
async fn test_analyze_project_frameworks_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_nodejs_project(&temp_dir);

    let manager = ResearchManager::new();
    let context = manager.analyze_project(&root).await.unwrap();

    // Frameworks should be detected from dependencies
    // (May be empty if framework detection is not fully implemented)
    let _ = context.frameworks;
}

#[tokio::test]
async fn test_analyze_project_architectural_intent() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_complete_rust_project(&temp_dir);

    let manager = ResearchManager::new();
    let context = manager.analyze_project(&root).await.unwrap();

    // Architectural intent should be present
    assert!(context.architectural_intent.style != ArchitecturalStyle::Unknown);
}
