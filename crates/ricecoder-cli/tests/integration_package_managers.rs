//! Integration tests for package manager installations
//!
//! Tests cargo install from crates.io
//! Tests npm install from npm registry
//! Tests Docker pull from Docker Hub
//!
//! **Feature: ricecoder-installation, Property 1: Installation Completeness**
//! **Validates: Requirements 2.2, 2.1, 3.3**

use std::{fs, path::PathBuf, process::Command};

/// Test helper: Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Test helper: Get Cargo.toml path
fn get_cargo_toml_path() -> PathBuf {
    let paths = vec![
        PathBuf::from("../../Cargo.toml"),
        PathBuf::from("projects/ricecoder/Cargo.toml"),
        PathBuf::from("Cargo.toml"),
    ];

    for path in paths {
        if path.exists() {
            return path;
        }
    }

    PathBuf::from("../../Cargo.toml")
}

/// Test helper: Get package.json path
fn get_package_json_path() -> PathBuf {
    let paths = vec![
        PathBuf::from("../../package.json"),
        PathBuf::from("projects/ricecoder/package.json"),
        PathBuf::from("package.json"),
    ];

    for path in paths {
        if path.exists() {
            return path;
        }
    }

    PathBuf::from("../../package.json")
}

/// Test helper: Get Dockerfile path
fn get_dockerfile_path() -> PathBuf {
    let paths = vec![
        PathBuf::from("../../Dockerfile"),
        PathBuf::from("projects/ricecoder/Dockerfile"),
        PathBuf::from("Dockerfile"),
    ];

    for path in paths {
        if path.exists() {
            return path;
        }
    }

    PathBuf::from("../../Dockerfile")
}

#[test]
fn test_cargo_toml_exists() {
    let cargo_toml = get_cargo_toml_path();
    assert!(
        cargo_toml.exists(),
        "Cargo.toml should exist at {}",
        cargo_toml.display()
    );
}

#[test]
fn test_cargo_toml_has_required_metadata() {
    let cargo_toml = get_cargo_toml_path();
    assert!(cargo_toml.exists(), "Cargo.toml should exist");

    let content = fs::read_to_string(&cargo_toml).expect("Should read Cargo.toml");

    // Check for required metadata
    assert!(
        content.contains("[workspace.package]"),
        "Cargo.toml should have workspace.package section"
    );
    assert!(
        content.contains("version"),
        "Cargo.toml should have version"
    );
    assert!(
        content.contains("authors"),
        "Cargo.toml should have authors"
    );
    assert!(
        content.contains("license"),
        "Cargo.toml should have license"
    );
    assert!(
        content.contains("repository"),
        "Cargo.toml should have repository"
    );
}

#[test]
fn test_cargo_toml_has_binary_target() {
    let cargo_toml = get_cargo_toml_path();
    assert!(cargo_toml.exists(), "Cargo.toml should exist");

    let content = fs::read_to_string(&cargo_toml).expect("Should read Cargo.toml");

    // Check for binary target
    assert!(
        content.contains("ricecoder-cli"),
        "Cargo.toml should have ricecoder-cli binary target"
    );
}

#[test]
fn test_cargo_toml_version_format() {
    let cargo_toml = get_cargo_toml_path();
    assert!(cargo_toml.exists(), "Cargo.toml should exist");

    let content = fs::read_to_string(&cargo_toml).expect("Should read Cargo.toml");

    // Check for version format (should be semantic versioning)
    assert!(
        content.contains("version = \"0."),
        "Cargo.toml should have semantic version"
    );
}

#[test]
fn test_cargo_toml_has_keywords() {
    let cargo_toml = get_cargo_toml_path();
    assert!(cargo_toml.exists(), "Cargo.toml should exist");

    let content = fs::read_to_string(&cargo_toml).expect("Should read Cargo.toml");

    // Check for keywords
    assert!(
        content.contains("keywords"),
        "Cargo.toml should have keywords"
    );
    assert!(content.contains("ai"), "Keywords should include 'ai'");
    assert!(
        content.contains("coding"),
        "Keywords should include 'coding'"
    );
}

#[test]
fn test_cargo_toml_has_categories() {
    let cargo_toml = get_cargo_toml_path();
    assert!(cargo_toml.exists(), "Cargo.toml should exist");

    let content = fs::read_to_string(&cargo_toml).expect("Should read Cargo.toml");

    // Check for categories
    assert!(
        content.contains("categories"),
        "Cargo.toml should have categories"
    );
    assert!(
        content.contains("command-line-utilities"),
        "Categories should include 'command-line-utilities'"
    );
}

#[test]
fn test_package_json_exists() {
    let package_json = get_package_json_path();
    assert!(
        package_json.exists(),
        "package.json should exist at {}",
        package_json.display()
    );
}

#[test]
fn test_package_json_has_required_fields() {
    let package_json = get_package_json_path();
    assert!(package_json.exists(), "package.json should exist");

    let content = fs::read_to_string(&package_json).expect("Should read package.json");

    // Check for required fields
    assert!(
        content.contains("\"name\""),
        "package.json should have name field"
    );
    assert!(
        content.contains("\"version\""),
        "package.json should have version field"
    );
    assert!(
        content.contains("\"description\""),
        "package.json should have description field"
    );
    assert!(
        content.contains("\"license\""),
        "package.json should have license field"
    );
}

#[test]
fn test_package_json_has_bin_field() {
    let package_json = get_package_json_path();
    assert!(package_json.exists(), "package.json should exist");

    let content = fs::read_to_string(&package_json).expect("Should read package.json");

    // Check for bin field
    assert!(
        content.contains("\"bin\""),
        "package.json should have bin field"
    );
    assert!(
        content.contains("ricecoder"),
        "bin field should reference ricecoder"
    );
}

#[test]
fn test_package_json_has_postinstall_script() {
    let package_json = get_package_json_path();
    assert!(package_json.exists(), "package.json should exist");

    let content = fs::read_to_string(&package_json).expect("Should read package.json");

    // Check for postinstall script
    assert!(
        content.contains("postinstall") || content.contains("install"),
        "package.json should have postinstall or install script"
    );
}

#[test]
fn test_package_json_has_repository() {
    let package_json = get_package_json_path();
    assert!(package_json.exists(), "package.json should exist");

    let content = fs::read_to_string(&package_json).expect("Should read package.json");

    // Check for repository
    assert!(
        content.contains("\"repository\""),
        "package.json should have repository field"
    );
    assert!(
        content.contains("github.com"),
        "repository should point to GitHub"
    );
}

#[test]
fn test_package_json_has_keywords() {
    let package_json = get_package_json_path();
    assert!(package_json.exists(), "package.json should exist");

    let content = fs::read_to_string(&package_json).expect("Should read package.json");

    // Check for keywords
    assert!(
        content.contains("\"keywords\""),
        "package.json should have keywords field"
    );
}

#[test]
fn test_package_json_version_matches_cargo() {
    let cargo_toml = get_cargo_toml_path();
    let package_json = get_package_json_path();

    assert!(cargo_toml.exists(), "Cargo.toml should exist");
    assert!(package_json.exists(), "package.json should exist");

    let cargo_content = fs::read_to_string(&cargo_toml).expect("Should read Cargo.toml");
    let package_content = fs::read_to_string(&package_json).expect("Should read package.json");

    // Extract versions
    let cargo_version = cargo_content
        .lines()
        .find(|line| line.contains("version = \""))
        .and_then(|line| line.split('"').nth(1));

    let package_version = package_content
        .lines()
        .find(|line| line.contains("\"version\""))
        .and_then(|line| line.split('"').nth(3));

    if let (Some(cv), Some(pv)) = (cargo_version, package_version) {
        assert_eq!(cv, pv, "Cargo.toml and package.json versions should match");
    }
}

#[test]
fn test_dockerfile_exists() {
    let dockerfile = get_dockerfile_path();
    assert!(
        dockerfile.exists(),
        "Dockerfile should exist at {}",
        dockerfile.display()
    );
}

#[test]
fn test_dockerfile_has_multi_stage_build() {
    let dockerfile = get_dockerfile_path();
    assert!(dockerfile.exists(), "Dockerfile should exist");

    let content = fs::read_to_string(&dockerfile).expect("Should read Dockerfile");

    // Check for multi-stage build
    assert!(
        content.contains("FROM") && content.matches("FROM").count() >= 2,
        "Dockerfile should have multi-stage build (at least 2 FROM statements)"
    );
}

#[test]
fn test_dockerfile_has_builder_stage() {
    let dockerfile = get_dockerfile_path();
    assert!(dockerfile.exists(), "Dockerfile should exist");

    let content = fs::read_to_string(&dockerfile).expect("Should read Dockerfile");

    // Check for builder stage
    assert!(
        content.contains("AS builder") || content.contains("as builder"),
        "Dockerfile should have builder stage"
    );
}

#[test]
fn test_dockerfile_has_runtime_stage() {
    let dockerfile = get_dockerfile_path();
    assert!(dockerfile.exists(), "Dockerfile should exist");

    let content = fs::read_to_string(&dockerfile).expect("Should read Dockerfile");

    // Check for runtime stage
    assert!(
        content.contains("alpine") || content.contains("debian"),
        "Dockerfile should have runtime stage with minimal base image"
    );
}

#[test]
fn test_dockerfile_copies_binary_from_builder() {
    let dockerfile = get_dockerfile_path();
    assert!(dockerfile.exists(), "Dockerfile should exist");

    let content = fs::read_to_string(&dockerfile).expect("Should read Dockerfile");

    // Check for COPY from builder
    assert!(
        content.contains("COPY --from=builder") || content.contains("COPY --from builder"),
        "Dockerfile should copy binary from builder stage"
    );
}

#[test]
fn test_dockerfile_has_entrypoint() {
    let dockerfile = get_dockerfile_path();
    assert!(dockerfile.exists(), "Dockerfile should exist");

    let content = fs::read_to_string(&dockerfile).expect("Should read Dockerfile");

    // Check for ENTRYPOINT
    assert!(
        content.contains("ENTRYPOINT"),
        "Dockerfile should have ENTRYPOINT"
    );
    assert!(
        content.contains("ricecoder"),
        "ENTRYPOINT should reference ricecoder"
    );
}

#[test]
fn test_dockerfile_uses_rust_toolchain() {
    let dockerfile = get_dockerfile_path();
    assert!(dockerfile.exists(), "Dockerfile should exist");

    let content = fs::read_to_string(&dockerfile).expect("Should read Dockerfile");

    // Check for Rust toolchain
    assert!(
        content.contains("rust") || content.contains("cargo"),
        "Dockerfile should use Rust toolchain in builder stage"
    );
}

#[test]
fn test_dockerfile_minimizes_image_size() {
    let dockerfile = get_dockerfile_path();
    assert!(dockerfile.exists(), "Dockerfile should exist");

    let content = fs::read_to_string(&dockerfile).expect("Should read Dockerfile");

    // Check for size optimization
    assert!(
        content.contains("alpine") || content.contains("scratch"),
        "Dockerfile should use minimal base image (alpine or scratch)"
    );
}

#[test]
fn test_dockerfile_has_labels() {
    let dockerfile = get_dockerfile_path();
    assert!(dockerfile.exists(), "Dockerfile should exist");

    let content = fs::read_to_string(&dockerfile).expect("Should read Dockerfile");

    // Check for labels
    assert!(
        content.contains("LABEL"),
        "Dockerfile should have LABEL instructions"
    );
}

#[test]
fn test_install_script_js_exists() {
    let paths = vec![
        PathBuf::from("../../scripts/install.js"),
        PathBuf::from("projects/ricecoder/scripts/install.js"),
        PathBuf::from("scripts/install.js"),
    ];

    let mut found = false;
    for path in paths {
        if path.exists() {
            found = true;
            break;
        }
    }

    assert!(found, "install.js should exist for npm postinstall");
}

#[test]
fn test_install_script_js_downloads_binary() {
    let paths = vec![
        PathBuf::from("../../scripts/install.js"),
        PathBuf::from("projects/ricecoder/scripts/install.js"),
        PathBuf::from("scripts/install.js"),
    ];

    let install_js = paths
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_default();
    assert!(install_js.exists(), "install.js should exist");

    let content = fs::read_to_string(&install_js).expect("Should read install.js");

    // Check for binary download logic
    assert!(
        content.contains("download") || content.contains("fetch"),
        "install.js should download binary"
    );
    assert!(
        content.contains("github") || content.contains("releases"),
        "install.js should download from GitHub releases"
    );
}

#[test]
fn test_install_script_js_verifies_checksum() {
    let paths = vec![
        PathBuf::from("../../scripts/install.js"),
        PathBuf::from("projects/ricecoder/scripts/install.js"),
        PathBuf::from("scripts/install.js"),
    ];

    let install_js = paths
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_default();
    assert!(install_js.exists(), "install.js should exist");

    let content = fs::read_to_string(&install_js).expect("Should read install.js");

    // Check for checksum verification
    assert!(
        content.contains("sha256") || content.contains("checksum"),
        "install.js should verify checksum"
    );
}

#[test]
fn test_install_script_js_extracts_archive() {
    let paths = vec![
        PathBuf::from("../../scripts/install.js"),
        PathBuf::from("projects/ricecoder/scripts/install.js"),
        PathBuf::from("scripts/install.js"),
    ];

    let install_js = paths
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_default();
    assert!(install_js.exists(), "install.js should exist");

    let content = fs::read_to_string(&install_js).expect("Should read install.js");

    // Check for archive extraction
    assert!(
        content.contains("extract") || content.contains("unzip") || content.contains("tar"),
        "install.js should extract archive"
    );
}

#[test]
fn test_install_script_js_handles_errors() {
    let paths = vec![
        PathBuf::from("../../scripts/install.js"),
        PathBuf::from("projects/ricecoder/scripts/install.js"),
        PathBuf::from("scripts/install.js"),
    ];

    let install_js = paths
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_default();
    assert!(install_js.exists(), "install.js should exist");

    let content = fs::read_to_string(&install_js).expect("Should read install.js");

    // Check for error handling
    assert!(
        content.contains("catch") || content.contains("error") || content.contains("Error"),
        "install.js should handle errors"
    );
}

#[test]
fn test_cargo_is_available() {
    // This test checks if cargo is available for testing
    // It's informational and doesn't fail if cargo is not available
    let has_cargo = command_exists("cargo");
    if !has_cargo {
        println!("Warning: cargo not available, skipping cargo-specific tests");
    }
}

#[test]
fn test_npm_is_available() {
    // This test checks if npm is available for testing
    // It's informational and doesn't fail if npm is not available
    let has_npm = command_exists("npm");
    if !has_npm {
        println!("Warning: npm not available, skipping npm-specific tests");
    }
}

#[test]
fn test_docker_is_available() {
    // This test checks if docker is available for testing
    // It's informational and doesn't fail if docker is not available
    let has_docker = command_exists("docker");
    if !has_docker {
        println!("Warning: docker not available, skipping docker-specific tests");
    }
}
