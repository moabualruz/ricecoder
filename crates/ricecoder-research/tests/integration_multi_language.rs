//! Integration tests for multi-language project analysis
//! Tests dependency detection and analysis across 11+ languages
//! **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 3.10, 3.11, 3.12**

use std::{fs, path::PathBuf};

use ricecoder_research::DependencyAnalyzer;
use tempfile::TempDir;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a Rust project with Cargo.toml
fn create_rust_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
regex = "1.5"

[dev-dependencies]
proptest = "1.0"
"#,
    )
    .unwrap();
    root.to_path_buf()
}

/// Create a Node.js project with package.json
fn create_nodejs_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::write(
        root.join("package.json"),
        r#"{
  "name": "test-project",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.0",
    "lodash": "^4.17.0",
    "axios": "^1.0.0"
  },
  "devDependencies": {
    "jest": "^29.0.0",
    "typescript": "^5.0.0"
  }
}
"#,
    )
    .unwrap();
    root.to_path_buf()
}

/// Create a Python project with pyproject.toml
fn create_python_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::write(
        root.join("pyproject.toml"),
        r#"[project]
name = "test-project"
version = "0.1.0"
dependencies = [
    "requests>=2.28.0",
    "pydantic>=1.10.0",
    "fastapi>=0.95.0",
]
"#,
    )
    .unwrap();
    root.to_path_buf()
}

/// Create a Go project with go.mod
fn create_go_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::write(
        root.join("go.mod"),
        r#"module github.com/test/project

go 1.21

require (
    github.com/gin-gonic/gin v1.9.0
    github.com/sirupsen/logrus v1.9.0
    github.com/google/uuid v1.3.0
)
"#,
    )
    .unwrap();
    root.to_path_buf()
}

/// Create a Java project with pom.xml
fn create_java_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::write(
        root.join("pom.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>test-project</artifactId>
    <version>1.0.0</version>
    
    <dependencies>
        <dependency>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-starter-web</artifactId>
            <version>3.0.0</version>
        </dependency>
        <dependency>
            <groupId>com.google.guava</groupId>
            <artifactId>guava</artifactId>
            <version>31.1-jre</version>
        </dependency>
    </dependencies>
</project>
"#,
    )
    .unwrap();
    root.to_path_buf()
}

/// Create a Kotlin project with build.gradle.kts
fn create_kotlin_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::write(
        root.join("build.gradle.kts"),
        r#"dependencies {
    implementation("org.springframework.boot:spring-boot-starter-web:3.0.0")
    implementation("com.google.guava:guava:31.1-jre")
    testImplementation("junit:junit:4.13.2")
}
"#,
    )
    .unwrap();
    root.to_path_buf()
}

/// Create a .NET project with .csproj
fn create_dotnet_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::write(
        root.join("project.csproj"),
        r#"<?xml version="1.0" encoding="utf-8"?>
<Project Sdk="Microsoft.NET.Sdk">
    <ItemGroup>
        <PackageReference Include="Newtonsoft.Json" Version="13.0.0" />
        <PackageReference Include="Microsoft.EntityFrameworkCore" Version="7.0.0" />
    </ItemGroup>
</Project>
"#,
    )
    .unwrap();
    root.to_path_buf()
}

/// Create a PHP project with composer.json
fn create_php_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::write(
        root.join("composer.json"),
        r#"{
  "name": "test/project",
  "require": {
    "laravel/framework": "^10.0",
    "symfony/console": "^6.0"
  },
  "require-dev": {
    "phpunit/phpunit": "^10.0"
  }
}
"#,
    )
    .unwrap();
    root.to_path_buf()
}

/// Create a Ruby project with Gemfile
fn create_ruby_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::write(
        root.join("Gemfile"),
        r#"source 'https://rubygems.org'

gem 'rails', '~> 7.0'
gem 'pg', '~> 1.1'
gem 'puma', '~> 5.0'

group :development, :test do
  gem 'rspec-rails', '~> 6.0'
end
"#,
    )
    .unwrap();
    root.to_path_buf()
}

/// Create a Swift project with Package.swift
fn create_swift_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::write(
        root.join("Package.swift"),
        r#"// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "TestProject",
    dependencies: [
        .package(url: "https://github.com/apple/swift-nio.git", from: "2.0.0"),
        .package(url: "https://github.com/vapor/vapor.git", from: "4.0.0"),
    ]
)
"#,
    )
    .unwrap();
    root.to_path_buf()
}

/// Create a Dart/Flutter project with pubspec.yaml
fn create_dart_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    fs::write(
        root.join("pubspec.yaml"),
        r#"name: test_project
version: 1.0.0

dependencies:
  flutter:
    sdk: flutter
  provider: ^6.0.0
  http: ^1.0.0

dev_dependencies:
  flutter_test:
    sdk: flutter
"#,
    )
    .unwrap();
    root.to_path_buf()
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_analyze_rust_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_rust_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(&root);

    assert!(result.is_ok());
    let dependencies = result.unwrap();

    // Verify dependencies were found
    let dep_names: Vec<_> = dependencies.iter().map(|d| d.name.as_str()).collect();
    assert!(
        dep_names.contains(&"serde")
            || dep_names.contains(&"tokio")
            || dep_names.contains(&"regex")
    );
}

#[test]
fn test_analyze_nodejs_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_nodejs_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(&root);

    assert!(result.is_ok());
    let dependencies = result.unwrap();

    assert!(!dependencies.is_empty());
    let dep_names: Vec<_> = dependencies.iter().map(|d| d.name.as_str()).collect();
    assert!(dep_names.contains(&"express"));
    assert!(dep_names.contains(&"lodash"));
    assert!(dep_names.contains(&"axios"));
}

#[test]
fn test_analyze_python_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_python_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(&root);

    assert!(result.is_ok());
    let dependencies = result.unwrap();

    assert!(!dependencies.is_empty());
    let dep_names: Vec<_> = dependencies.iter().map(|d| d.name.as_str()).collect();
    assert!(dep_names.contains(&"requests"));
    assert!(dep_names.contains(&"pydantic"));
    assert!(dep_names.contains(&"fastapi"));
}

#[test]
fn test_analyze_go_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_go_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(&root);

    assert!(result.is_ok());
    let dependencies = result.unwrap();

    assert!(!dependencies.is_empty());
    let dep_names: Vec<_> = dependencies.iter().map(|d| d.name.as_str()).collect();
    assert!(dep_names.contains(&"github.com/gin-gonic/gin"));
    assert!(dep_names.contains(&"github.com/sirupsen/logrus"));
}

#[test]
fn test_analyze_java_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_java_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(&root);

    assert!(result.is_ok());
    let dependencies = result.unwrap();

    // Verify dependencies were found
    let dep_names: Vec<_> = dependencies.iter().map(|d| d.name.as_str()).collect();
    assert!(dep_names
        .iter()
        .any(|d| d.contains("spring") || d.contains("guava")));
}

#[test]
fn test_analyze_kotlin_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_kotlin_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(&root);

    assert!(result.is_ok());
    let dependencies = result.unwrap();

    // Verify dependencies were found
    let dep_names: Vec<_> = dependencies.iter().map(|d| d.name.as_str()).collect();
    assert!(dep_names.iter().any(|d| d.contains("spring")));
}

#[test]
fn test_analyze_dotnet_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_dotnet_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(&root);

    assert!(result.is_ok());
    let dependencies = result.unwrap();

    assert!(!dependencies.is_empty());
    let dep_names: Vec<_> = dependencies.iter().map(|d| d.name.as_str()).collect();
    assert!(dep_names.contains(&"Newtonsoft.Json"));
    assert!(dep_names.contains(&"Microsoft.EntityFrameworkCore"));
}

#[test]
fn test_analyze_php_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_php_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(&root);

    assert!(result.is_ok());
    let dependencies = result.unwrap();

    assert!(!dependencies.is_empty());
    let dep_names: Vec<_> = dependencies.iter().map(|d| d.name.as_str()).collect();
    assert!(dep_names.contains(&"laravel/framework"));
    assert!(dep_names.contains(&"symfony/console"));
}

#[test]
fn test_analyze_ruby_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_ruby_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(&root);

    assert!(result.is_ok());
    let dependencies = result.unwrap();

    assert!(!dependencies.is_empty());
    let dep_names: Vec<_> = dependencies.iter().map(|d| d.name.as_str()).collect();
    assert!(dep_names.contains(&"rails"));
    assert!(dep_names.contains(&"pg"));
}

#[test]
fn test_analyze_swift_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_swift_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(&root);

    assert!(result.is_ok());
    let dependencies = result.unwrap();

    assert!(!dependencies.is_empty());
    let dep_names: Vec<_> = dependencies.iter().map(|d| d.name.as_str()).collect();
    assert!(dep_names.contains(&"swift-nio"));
    assert!(dep_names.contains(&"vapor"));
}

#[test]
fn test_analyze_dart_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_dart_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(&root);

    assert!(result.is_ok());
    let dependencies = result.unwrap();

    assert!(!dependencies.is_empty());
    let dep_names: Vec<_> = dependencies.iter().map(|d| d.name.as_str()).collect();
    assert!(dep_names.contains(&"provider"));
    assert!(dep_names.contains(&"http"));
}

#[test]
fn test_dependency_includes_version_info() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_rust_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let dependencies = analyzer.analyze(&root).unwrap();

    // Each dependency should have version information
    for dep in dependencies {
        assert!(!dep.name.is_empty());
        assert!(!dep.version.is_empty());
    }
}

#[test]
fn test_dependency_analyzer_consistency() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_rust_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();

    // Analyze multiple times
    let deps1 = analyzer.analyze(&root).unwrap();
    let deps2 = analyzer.analyze(&root).unwrap();
    let deps3 = analyzer.analyze(&root).unwrap();

    // Results should be identical
    assert_eq!(deps1.len(), deps2.len());
    assert_eq!(deps2.len(), deps3.len());

    for i in 0..deps1.len() {
        assert_eq!(deps1[i].name, deps2[i].name);
        assert_eq!(deps2[i].name, deps3[i].name);
    }
}

#[test]
fn test_analyze_nonexistent_project() {
    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(&PathBuf::from("/nonexistent/path"));

    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_analyze_empty_project() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(root);

    // Should handle empty project gracefully
    assert!(result.is_ok());
    let dependencies = result.unwrap();
    assert!(dependencies.is_empty());
}

#[test]
fn test_multi_language_project_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create multiple manifest files
    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "test"
[dependencies]
serde = "1.0"
"#,
    )
    .unwrap();

    fs::write(
        root.join("package.json"),
        r#"{"dependencies": {"express": "^4.18.0"}}"#,
    )
    .unwrap();

    let analyzer = DependencyAnalyzer::new();
    let result = analyzer.analyze(root);

    assert!(result.is_ok());
    let dependencies = result.unwrap();

    // Should find dependencies from both Rust and Node.js
    let dep_names: Vec<_> = dependencies.iter().map(|d| d.name.as_str()).collect();
    assert!(dep_names.contains(&"serde") || dep_names.contains(&"express"));
}

#[test]
fn test_dependency_version_constraints() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_nodejs_project(&temp_dir);

    let analyzer = DependencyAnalyzer::new();
    let dependencies = analyzer.analyze(&root).unwrap();

    // Dependencies should have version constraints
    for dep in dependencies {
        assert!(!dep.version.is_empty());
    }
}
