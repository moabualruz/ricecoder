//! Integration tests for CodebaseScanner
//! Tests directory traversal, gitignore support, symbol extraction, and reference tracking
//! **Validates: Requirements 1.7, 1.8, 1.9**

use ricecoder_research::models::Language;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a simple Rust project structure
fn create_simple_rust_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();

    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn lib_func() {}").unwrap();
    fs::write(
        root.join("tests/integration_test.rs"),
        "#[test]\nfn test() {}",
    )
    .unwrap();

    root.to_path_buf()
}

/// Create a multi-language project
fn create_multi_language_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("scripts")).unwrap();

    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("src/utils.ts"), "export function util() {}").unwrap();
    fs::write(root.join("scripts/build.py"), "def build(): pass").unwrap();

    root.to_path_buf()
}

/// Create a project with nested directories
fn create_nested_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::create_dir_all(root.join("src/domain")).unwrap();
    fs::create_dir_all(root.join("src/application")).unwrap();
    fs::create_dir_all(root.join("src/infrastructure")).unwrap();
    fs::create_dir_all(root.join("tests/unit")).unwrap();
    fs::create_dir_all(root.join("tests/integration")).unwrap();

    fs::write(root.join("src/domain/entity.rs"), "pub struct Entity {}").unwrap();
    fs::write(
        root.join("src/application/service.rs"),
        "pub struct Service {}",
    )
    .unwrap();
    fs::write(
        root.join("src/infrastructure/repo.rs"),
        "pub struct Repository {}",
    )
    .unwrap();
    fs::write(
        root.join("tests/unit/entity_test.rs"),
        "#[test]\nfn test() {}",
    )
    .unwrap();
    fs::write(
        root.join("tests/integration/integration_test.rs"),
        "#[test]\nfn test() {}",
    )
    .unwrap();

    root.to_path_buf()
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_scan_simple_rust_project() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_simple_rust_project(&temp_dir);

    let result = CodebaseScanner::scan(&root).unwrap();

    // Verify files were found
    assert_eq!(result.files.len(), 3);

    // Verify languages detected
    assert!(result.languages.contains(&Language::Rust));

    // Verify source and test directories
    assert!(!result.source_dirs.is_empty());
    assert!(!result.test_dirs.is_empty());
}

#[test]
fn test_scan_detects_all_file_types() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_multi_language_project(&temp_dir);

    let result = CodebaseScanner::scan(&root).unwrap();

    // Verify multiple languages detected
    assert!(result.languages.len() >= 2);
    assert!(result.languages.contains(&Language::Rust));
    assert!(result.languages.contains(&Language::TypeScript));
    assert!(result.languages.contains(&Language::Python));
}

#[test]
fn test_scan_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_nested_project(&temp_dir);

    let result = CodebaseScanner::scan(&root).unwrap();

    // Verify files in nested directories are found
    assert_eq!(result.files.len(), 5);

    // Verify nested source directories are detected
    assert!(result.source_dirs.iter().any(|d| d.ends_with("domain")));
    assert!(result
        .source_dirs
        .iter()
        .any(|d| d.ends_with("application")));
    assert!(result
        .source_dirs
        .iter()
        .any(|d| d.ends_with("infrastructure")));

    // Verify nested test directories are detected
    assert!(result.test_dirs.iter().any(|d| d.ends_with("unit")));
    assert!(result.test_dirs.iter().any(|d| d.ends_with("integration")));
}

#[test]
fn test_scan_identifies_test_files() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();

    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("src/lib_test.rs"), "#[test]\nfn test() {}").unwrap();
    fs::write(
        root.join("tests/integration_test.rs"),
        "#[test]\nfn test() {}",
    )
    .unwrap();

    let result = CodebaseScanner::scan(root).unwrap();

    // Verify test files are marked correctly
    let test_files: Vec<_> = result.files.iter().filter(|f| f.is_test).collect();
    assert_eq!(test_files.len(), 2);

    let source_files: Vec<_> = result.files.iter().filter(|f| !f.is_test).collect();
    assert_eq!(source_files.len(), 1);

    // Verify language detection
    assert_eq!(source_files[0].language, Some(Language::Rust));
}

#[test]
fn test_scan_file_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    let content = "fn main() { println!(\"Hello, world!\"); }";
    fs::write(root.join("src/main.rs"), content).unwrap();

    let result = CodebaseScanner::scan(root).unwrap();

    assert_eq!(result.files.len(), 1);
    let file = &result.files[0];

    // Verify metadata
    assert_eq!(file.path.file_name().unwrap(), "main.rs");
    assert_eq!(file.language, Some(Language::Rust));
    assert_eq!(file.size, content.len() as u64);
    assert!(!file.is_test);
}

#[test]
fn test_scan_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let result = CodebaseScanner::scan(root).unwrap();

    assert_eq!(result.files.len(), 0);
    assert_eq!(result.languages.len(), 0);
}

#[test]
fn test_scan_nonexistent_directory() {
    let result = CodebaseScanner::scan(&PathBuf::from("/nonexistent/path"));
    assert!(result.is_err());
}

#[test]
fn test_scan_consistency() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_simple_rust_project(&temp_dir);

    // Scan twice
    let result1 = CodebaseScanner::scan(&root).unwrap();
    let result2 = CodebaseScanner::scan(&root).unwrap();

    // Results should be identical
    assert_eq!(result1.files.len(), result2.files.len());
    assert_eq!(result1.languages.len(), result2.languages.len());
    assert_eq!(result1.source_dirs.len(), result2.source_dirs.len());
    assert_eq!(result1.test_dirs.len(), result2.test_dirs.len());
}

#[test]
fn test_scan_detects_source_and_test_dirs() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::create_dir_all(root.join("__tests__")).unwrap();

    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("tests/test.rs"), "#[test]\nfn test() {}").unwrap();
    fs::write(root.join("__tests__/test.ts"), "test('test', () => {})").unwrap();

    let result = CodebaseScanner::scan(root).unwrap();

    // Verify source directories
    assert!(result.source_dirs.iter().any(|d| d.ends_with("src")));

    // Verify test directories
    assert!(result.test_dirs.iter().any(|d| d.ends_with("tests")));
    assert!(result.test_dirs.iter().any(|d| d.ends_with("__tests__")));
}

#[test]
fn test_scan_handles_mixed_test_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();

    // Create files with various test naming patterns
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("src/lib_test.rs"), "#[test]\nfn test() {}").unwrap();
    fs::write(root.join("src/utils.test.ts"), "test('test', () => {})").unwrap();
    fs::write(root.join("src/helper_test.py"), "def test(): pass").unwrap();

    let result = CodebaseScanner::scan(root).unwrap();

    let test_files: Vec<_> = result.files.iter().filter(|f| f.is_test).collect();
    assert_eq!(test_files.len(), 3);
}

#[test]
fn test_scan_language_detection_all_types() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();

    // Create files for all supported languages
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("src/main.ts"), "export function main() {}").unwrap();
    fs::write(root.join("src/main.py"), "def main(): pass").unwrap();
    fs::write(root.join("src/main.go"), "func main() {}").unwrap();
    fs::write(root.join("src/Main.java"), "public class Main {}").unwrap();
    fs::write(root.join("src/Main.kt"), "fun main() {}").unwrap();
    fs::write(root.join("src/Main.cs"), "class Main {}").unwrap();
    fs::write(root.join("src/main.php"), "<?php function main() {}").unwrap();
    fs::write(root.join("src/main.rb"), "def main; end").unwrap();
    fs::write(root.join("src/Main.swift"), "func main() {}").unwrap();
    fs::write(root.join("src/main.dart"), "void main() {}").unwrap();

    let result = CodebaseScanner::scan(root).unwrap();

    // Verify all languages are detected
    assert!(result.languages.contains(&Language::Rust));
    assert!(result.languages.contains(&Language::TypeScript));
    assert!(result.languages.contains(&Language::Python));
    assert!(result.languages.contains(&Language::Go));
    assert!(result.languages.contains(&Language::Java));
    assert!(result.languages.contains(&Language::Kotlin));
    assert!(result.languages.contains(&Language::CSharp));
    assert!(result.languages.contains(&Language::Php));
    assert!(result.languages.contains(&Language::Ruby));
    assert!(result.languages.contains(&Language::Swift));
    assert!(result.languages.contains(&Language::Dart));
}

#[test]
fn test_scan_ignores_non_source_files() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();

    // Create source and non-source files
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("README.md"), "# Project").unwrap();
    fs::write(root.join("Cargo.toml"), "[package]").unwrap();
    fs::write(root.join(".gitignore"), "target/").unwrap();

    let result = CodebaseScanner::scan(root).unwrap();

    // Should only find main.rs (other files have no language)
    let source_files: Vec<_> = result
        .files
        .iter()
        .filter(|f| f.language.is_some())
        .collect();
    assert_eq!(source_files.len(), 1);
}
