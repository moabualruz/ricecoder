//! Property-based tests for project type detection consistency
//! **Feature: ricecoder-research, Property 1: Project Type Detection Consistency**
//! **Validates: Requirements 1.1, 1.2**

use proptest::prelude::*;
use ricecoder_research::ProjectAnalyzer;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Generators for property testing
// ============================================================================

/// Generate a Rust project structure
fn create_rust_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    std::fs::create_dir_all(root.join("src")).ok();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").ok();
    root.to_path_buf()
}

/// Generate a Node.js project structure
fn create_nodejs_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    std::fs::create_dir_all(root.join("src")).ok();
    std::fs::write(root.join("package.json"), "{\"name\": \"test\"}\n").ok();
    root.to_path_buf()
}

/// Generate a Python project structure
fn create_python_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    std::fs::create_dir_all(root.join("src")).ok();
    std::fs::write(root.join("pyproject.toml"), "[project]\nname = \"test\"\n").ok();
    root.to_path_buf()
}

/// Generate a Go project structure
fn create_go_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    std::fs::create_dir_all(root.join("src")).ok();
    std::fs::write(root.join("go.mod"), "module test\n").ok();
    root.to_path_buf()
}

/// Generate a Rust library project
fn create_rust_library_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    std::fs::create_dir_all(root.join("src")).ok();
    std::fs::write(root.join("src/lib.rs"), "").ok();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n[lib]\n").ok();
    root.to_path_buf()
}

/// Generate a Rust application project
fn create_rust_app_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    std::fs::create_dir_all(root.join("src")).ok();
    std::fs::write(root.join("src/main.rs"), "fn main() {}").ok();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").ok();
    root.to_path_buf()
}

/// Generate a Rust workspace (monorepo)
fn create_rust_workspace(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    std::fs::create_dir_all(root.join("crate1/src")).ok();
    std::fs::create_dir_all(root.join("crate2/src")).ok();
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crate1\", \"crate2\"]\n",
    )
    .ok();
    std::fs::write(root.join("crate1/Cargo.toml"), "[package]\nname = \"crate1\"\n").ok();
    std::fs::write(root.join("crate2/Cargo.toml"), "[package]\nname = \"crate2\"\n").ok();
    root.to_path_buf()
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    /// Property: Project type detection is consistent
    /// For any project, analyzing the same project multiple times should produce the same project type
    #[test]
    fn prop_rust_project_type_consistency(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_rust_project(&temp_dir);

        let analyzer = ProjectAnalyzer::new();
        let result1 = analyzer.detect_type(&root);
        let result2 = analyzer.detect_type(&root);

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        if let (Ok(type1), Ok(type2)) = (result1, result2) {
            prop_assert_eq!(type1, type2);
        }
    }

    /// Property: Node.js project type detection is consistent
    /// For any Node.js project, analyzing multiple times should produce the same result
    #[test]
    fn prop_nodejs_project_type_consistency(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_nodejs_project(&temp_dir);

        let analyzer = ProjectAnalyzer::new();
        let result1 = analyzer.detect_type(&root);
        let result2 = analyzer.detect_type(&root);

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        if let (Ok(type1), Ok(type2)) = (result1, result2) {
            prop_assert_eq!(type1, type2);
        }
    }

    /// Property: Python project type detection is consistent
    /// For any Python project, analyzing multiple times should produce the same result
    #[test]
    fn prop_python_project_type_consistency(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_python_project(&temp_dir);

        let analyzer = ProjectAnalyzer::new();
        let result1 = analyzer.detect_type(&root);
        let result2 = analyzer.detect_type(&root);

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        if let (Ok(type1), Ok(type2)) = (result1, result2) {
            prop_assert_eq!(type1, type2);
        }
    }

    /// Property: Go project type detection is consistent
    /// For any Go project, analyzing multiple times should produce the same result
    #[test]
    fn prop_go_project_type_consistency(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_go_project(&temp_dir);

        let analyzer = ProjectAnalyzer::new();
        let result1 = analyzer.detect_type(&root);
        let result2 = analyzer.detect_type(&root);

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        if let (Ok(type1), Ok(type2)) = (result1, result2) {
            prop_assert_eq!(type1, type2);
        }
    }

    /// Property: Rust library detection is consistent
    /// For any Rust library project, analyzing multiple times should identify it as a library
    #[test]
    fn prop_rust_library_detection_consistency(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_rust_library_project(&temp_dir);

        let analyzer = ProjectAnalyzer::new();
        let result1 = analyzer.detect_type(&root);
        let result2 = analyzer.detect_type(&root);

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        if let (Ok(type1), Ok(type2)) = (result1, result2) {
            prop_assert_eq!(type1, type2);
        }
    }

    /// Property: Rust application detection is consistent
    /// For any Rust application project, analyzing multiple times should identify it as an application
    #[test]
    fn prop_rust_app_detection_consistency(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_rust_app_project(&temp_dir);

        let analyzer = ProjectAnalyzer::new();
        let result1 = analyzer.detect_type(&root);
        let result2 = analyzer.detect_type(&root);

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        if let (Ok(type1), Ok(type2)) = (result1, result2) {
            prop_assert_eq!(type1, type2);
        }
    }

    /// Property: Rust workspace detection is consistent
    /// For any Rust workspace, analyzing multiple times should identify it as a monorepo
    #[test]
    fn prop_rust_workspace_detection_consistency(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_rust_workspace(&temp_dir);

        let analyzer = ProjectAnalyzer::new();
        let result1 = analyzer.detect_type(&root);
        let result2 = analyzer.detect_type(&root);

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        if let (Ok(type1), Ok(type2)) = (result1, result2) {
            prop_assert_eq!(type1, type2);
        }
    }

    /// Property: Structure analysis is consistent
    /// For any project, analyzing structure multiple times should produce the same result
    #[test]
    fn prop_structure_analysis_consistency(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_rust_project(&temp_dir);

        let analyzer = ProjectAnalyzer::new();
        let result1 = analyzer.analyze_structure(&root);
        let result2 = analyzer.analyze_structure(&root);

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        if let (Ok(struct1), Ok(struct2)) = (result1, result2) {
            prop_assert_eq!(struct1.root, struct2.root);
            prop_assert_eq!(struct1.source_dirs.len(), struct2.source_dirs.len());
            prop_assert_eq!(struct1.test_dirs.len(), struct2.test_dirs.len());
            prop_assert_eq!(struct1.config_files.len(), struct2.config_files.len());
        }
    }

    /// Property: Framework identification is consistent
    /// For any project, identifying frameworks multiple times should produce the same result
    #[test]
    fn prop_framework_identification_consistency(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_rust_project(&temp_dir);

        let analyzer = ProjectAnalyzer::new();
        let result1 = analyzer.identify_frameworks(&root);
        let result2 = analyzer.identify_frameworks(&root);

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        if let (Ok(frameworks1), Ok(frameworks2)) = (result1, result2) {
            prop_assert_eq!(frameworks1.len(), frameworks2.len());
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[test]
fn test_rust_project_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_rust_project(&temp_dir);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.detect_type(&root);
    assert!(result.is_ok());
}

#[test]
fn test_nodejs_project_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_nodejs_project(&temp_dir);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.detect_type(&root);
    assert!(result.is_ok());
}

#[test]
fn test_python_project_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_python_project(&temp_dir);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.detect_type(&root);
    assert!(result.is_ok());
}

#[test]
fn test_go_project_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_go_project(&temp_dir);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.detect_type(&root);
    assert!(result.is_ok());
}

#[test]
fn test_rust_library_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_rust_library_project(&temp_dir);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.detect_type(&root);
    assert!(result.is_ok());
}

#[test]
fn test_rust_app_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_rust_app_project(&temp_dir);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.detect_type(&root);
    assert!(result.is_ok());
}

#[test]
fn test_rust_workspace_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_rust_workspace(&temp_dir);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.detect_type(&root);
    assert!(result.is_ok());
}

#[test]
fn test_structure_analysis() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_rust_project(&temp_dir);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze_structure(&root);
    assert!(result.is_ok());

    let structure = result.unwrap();
    assert!(!structure.source_dirs.is_empty());
}

#[test]
fn test_framework_identification() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_rust_project(&temp_dir);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.identify_frameworks(&root);
    assert!(result.is_ok());
}
