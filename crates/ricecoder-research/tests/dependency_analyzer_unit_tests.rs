//! Unit tests for DependencyAnalyzer
//!
//! Tests for all supported languages: Rust, Node.js, Python, Go, Java, Kotlin, .NET, PHP, Ruby, Swift, Dart/Flutter

use ricecoder_research::DependencyAnalyzer;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Rust Dependency Tests
// ============================================================================

#[test]
fn test_rust_simple_dependencies() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    let cargo_toml = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
proptest = "1.0"
"#;

    fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert_eq!(deps.len(), 3);

    let serde = deps.iter().find(|d| d.name == "serde").unwrap();
    assert_eq!(serde.version, "1.0");
    assert!(!serde.is_dev);

    let proptest = deps.iter().find(|d| d.name == "proptest").unwrap();
    assert!(proptest.is_dev);
}

#[test]
fn test_rust_workspace_dependencies() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    let cargo_toml = r#"
[workspace]
members = ["crate1", "crate2"]

[workspace.dependencies]
serde = "1.0"
tokio = "1.0"
"#;

    fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert!(deps.len() >= 2);

    let serde = deps.iter().find(|d| d.name == "serde");
    assert!(serde.is_some());
}

// ============================================================================
// Node.js Dependency Tests
// ============================================================================

#[test]
fn test_nodejs_dependencies() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    let package_json = r#"{
  "name": "test",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.0",
    "react": "^18.0.0"
  },
  "devDependencies": {
    "jest": "^29.0.0"
  },
  "peerDependencies": {
    "react-dom": "^18.0.0"
  }
}"#;

    fs::write(temp_dir.path().join("package.json"), package_json).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert_eq!(deps.len(), 4);

    let express = deps.iter().find(|d| d.name == "express").unwrap();
    assert_eq!(express.version, "^4.18.0");
    assert!(!express.is_dev);

    let jest = deps.iter().find(|d| d.name == "jest").unwrap();
    assert!(jest.is_dev);
}

// ============================================================================
// Python Dependency Tests
// ============================================================================

#[test]
fn test_python_requirements_txt() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    let requirements = r#"
requests>=2.28.0
django==4.1.0
pytest>=7.0
numpy>=1.20,<2.0
"#;

    fs::write(temp_dir.path().join("requirements.txt"), requirements).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert_eq!(deps.len(), 4);

    let requests = deps.iter().find(|d| d.name == "requests").unwrap();
    assert_eq!(requests.version, ">=2.28.0");
}

#[test]
fn test_python_pyproject_toml() {
    let analyzer = DependencyAnalyzer::new();
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

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert!(deps.len() >= 2);

    let requests = deps.iter().find(|d| d.name == "requests");
    assert!(requests.is_some());
}

// ============================================================================
// Go Dependency Tests
// ============================================================================

#[test]
fn test_go_dependencies() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    let go_mod = r#"module github.com/user/project

go 1.21

require (
    github.com/gorilla/mux v1.8.0
    github.com/lib/pq v1.10.9
)
"#;

    fs::write(temp_dir.path().join("go.mod"), go_mod).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert_eq!(deps.len(), 2);

    let mux = deps.iter().find(|d| d.name == "github.com/gorilla/mux").unwrap();
    assert_eq!(mux.version, "v1.8.0");
}

// ============================================================================
// Java Dependency Tests
// ============================================================================

#[test]
fn test_java_pom_xml() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    let pom = r#"<?xml version="1.0"?>
<project>
    <dependencies>
        <dependency>
            <groupId>junit</groupId>
            <artifactId>junit</artifactId>
            <version>4.13.2</version>
            <scope>test</scope>
        </dependency>
        <dependency>
            <groupId>org.springframework</groupId>
            <artifactId>spring-core</artifactId>
            <version>5.3.0</version>
        </dependency>
    </dependencies>
</project>"#;

    fs::write(temp_dir.path().join("pom.xml"), pom).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert_eq!(deps.len(), 2);

    let junit = deps.iter().find(|d| d.name.contains("junit")).unwrap();
    assert!(junit.is_dev);

    let spring = deps.iter().find(|d| d.name.contains("spring")).unwrap();
    assert!(!spring.is_dev);
}

// ============================================================================
// Kotlin Dependency Tests
// ============================================================================

#[test]
fn test_kotlin_gradle_kts() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    let gradle_kts = r#"
dependencies {
    implementation("org.jetbrains.kotlin:kotlin-stdlib:1.9.0")
    testImplementation("junit:junit:4.13.2")
}
"#;

    fs::write(temp_dir.path().join("build.gradle.kts"), gradle_kts).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert_eq!(deps.len(), 2);
}

// ============================================================================
// .NET Dependency Tests
// ============================================================================

#[test]
fn test_dotnet_csproj() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    let csproj = r#"<?xml version="1.0"?>
<Project Sdk="Microsoft.NET.Sdk">
  <ItemGroup>
    <PackageReference Include="Newtonsoft.Json" Version="13.0.1" />
    <PackageReference Include="System.Net.Http" Version="4.3.4" />
  </ItemGroup>
</Project>"#;

    fs::write(temp_dir.path().join("test.csproj"), csproj).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert_eq!(deps.len(), 2);

    let newtonsoft = deps.iter().find(|d| d.name == "Newtonsoft.Json").unwrap();
    assert_eq!(newtonsoft.version, "13.0.1");
}

// ============================================================================
// PHP Dependency Tests
// ============================================================================

#[test]
fn test_php_composer_json() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    let composer_json = r#"{
  "name": "test/project",
  "require": {
    "php": ">=7.4",
    "laravel/framework": "^9.0",
    "guzzlehttp/guzzle": "^7.0"
  },
  "require-dev": {
    "phpunit/phpunit": "^9.0"
  }
}"#;

    fs::write(temp_dir.path().join("composer.json"), composer_json).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert_eq!(deps.len(), 3);

    let laravel = deps.iter().find(|d| d.name == "laravel/framework").unwrap();
    assert_eq!(laravel.version, "^9.0");
    assert!(!laravel.is_dev);

    let phpunit = deps.iter().find(|d| d.name == "phpunit/phpunit").unwrap();
    assert!(phpunit.is_dev);
}

// ============================================================================
// Ruby Dependency Tests
// ============================================================================

#[test]
fn test_ruby_gemfile() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    let gemfile = r#"
source 'https://rubygems.org'

gem 'rails', '~> 7.0'
gem 'pg', '~> 1.1'
gem 'puma', '~> 5.0'
"#;

    fs::write(temp_dir.path().join("Gemfile"), gemfile).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert!(deps.len() >= 3);

    let rails = deps.iter().find(|d| d.name == "rails").unwrap();
    assert_eq!(rails.version, "~> 7.0");
}

// ============================================================================
// Swift Dependency Tests
// ============================================================================

#[test]
fn test_swift_package_swift() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    let package_swift = r#"
// swift-tools-version:5.5
import PackageDescription

let package = Package(
    name: "MyPackage",
    dependencies: [
        .package(url: "https://github.com/apple/swift-nio.git", from: "2.0.0"),
        .package(url: "https://github.com/vapor/vapor.git", .upToNextMajor(from: "4.0.0"))
    ]
)
"#;

    fs::write(temp_dir.path().join("Package.swift"), package_swift).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert_eq!(deps.len(), 2);

    let nio = deps.iter().find(|d| d.name.contains("nio")).unwrap();
    assert_eq!(nio.version, "2.0.0");
}

// ============================================================================
// Dart/Flutter Dependency Tests
// ============================================================================

#[test]
fn test_dart_pubspec_yaml() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    let pubspec = r#"
name: my_app
version: 1.0.0

dependencies:
  flutter:
    sdk: flutter
  provider: ^6.0.0
  http: ^0.13.0

dev_dependencies:
  flutter_test:
    sdk: flutter
  test: ^1.20.0
"#;

    fs::write(temp_dir.path().join("pubspec.yaml"), pubspec).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert!(deps.len() >= 2);

    let provider = deps.iter().find(|d| d.name == "provider").unwrap();
    assert_eq!(provider.version, "^6.0.0");
    assert!(!provider.is_dev);
}

// ============================================================================
// Multi-Language Project Tests
// ============================================================================

#[test]
fn test_multi_language_project() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    // Create Rust manifest
    let cargo_toml = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#;
    fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

    // Create Node.js manifest
    let package_json = r#"{
  "name": "test",
  "dependencies": {
    "express": "^4.18.0"
  }
}"#;
    fs::write(temp_dir.path().join("package.json"), package_json).unwrap();

    let deps = analyzer.analyze(temp_dir.path()).unwrap();
    assert_eq!(deps.len(), 2);

    let serde = deps.iter().find(|d| d.name == "serde");
    assert!(serde.is_some());

    let express = deps.iter().find(|d| d.name == "express");
    assert!(express.is_some());
}

// ============================================================================
// Language Detection Tests
// ============================================================================

#[test]
fn test_detect_languages_rust() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    let languages = analyzer.detect_languages(temp_dir.path()).unwrap();
    assert!(languages.iter().any(|l| format!("{:?}", l).contains("Rust")));
}

#[test]
fn test_detect_languages_nodejs() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("package.json"), "{}").unwrap();

    let languages = analyzer.detect_languages(temp_dir.path()).unwrap();
    assert!(languages.iter().any(|l| format!("{:?}", l).contains("TypeScript")));
}

#[test]
fn test_detect_languages_multiple() {
    let analyzer = DependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
    fs::write(temp_dir.path().join("go.mod"), "module test").unwrap();

    let languages = analyzer.detect_languages(temp_dir.path()).unwrap();
    assert!(languages.len() >= 3);
}

// ============================================================================
// Version Conflict Tests
// ============================================================================

#[test]
fn test_version_conflict_detection() {
    let analyzer = DependencyAnalyzer::new();

    let deps = vec![
        ricecoder_research::models::Dependency {
            name: "serde".to_string(),
            version: "1.0.0".to_string(),
            constraints: None,
            is_dev: false,
        },
        ricecoder_research::models::Dependency {
            name: "serde".to_string(),
            version: "2.0.0".to_string(),
            constraints: None,
            is_dev: false,
        },
    ];

    let conflicts = analyzer.analyze_version_conflicts(&deps);
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].dependency_name, "serde");
    assert_eq!(conflicts[0].versions.len(), 2);
}

#[test]
fn test_no_version_conflicts() {
    let analyzer = DependencyAnalyzer::new();

    let deps = vec![
        ricecoder_research::models::Dependency {
            name: "serde".to_string(),
            version: "1.0.0".to_string(),
            constraints: None,
            is_dev: false,
        },
        ricecoder_research::models::Dependency {
            name: "tokio".to_string(),
            version: "1.0.0".to_string(),
            constraints: None,
            is_dev: false,
        },
    ];

    let conflicts = analyzer.analyze_version_conflicts(&deps);
    assert!(conflicts.is_empty());
}
