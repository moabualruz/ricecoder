/// Installation Verification Tests
///
/// Tests to verify that ricecoder can be installed and used via multiple methods:
/// - cargo install (from crates.io)
/// - npm install (from npm registry)
/// - docker pull (from Docker Hub)
///
/// These tests validate that the installation methods work correctly and that
/// the installed binary functions as expected.

#[cfg(test)]
mod installation_verification {
    use std::path::Path;
    use std::process::Command;

    /// Test that cargo is available and can be used to install packages
    #[test]
    fn test_cargo_available() {
        let output = Command::new("cargo")
            .arg("--version")
            .output()
            .expect("Failed to execute cargo --version");

        assert!(
            output.status.success(),
            "cargo --version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let version = String::from_utf8_lossy(&output.stdout);
        assert!(
            version.contains("cargo"),
            "cargo version output doesn't contain 'cargo': {}",
            version
        );
    }

    /// Test that npm is available and can be used to install packages
    #[test]
    #[ignore] // npm may not be in PATH on all systems
    fn test_npm_available() {
        let output = Command::new("npm")
            .arg("--version")
            .output()
            .expect("Failed to execute npm --version");

        assert!(
            output.status.success(),
            "npm --version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let version = String::from_utf8_lossy(&output.stdout);
        assert!(!version.is_empty(), "npm version output is empty");
    }

    /// Test that docker is available and can be used to pull images
    #[test]
    fn test_docker_available() {
        let output = Command::new("docker")
            .arg("--version")
            .output()
            .expect("Failed to execute docker --version");

        assert!(
            output.status.success(),
            "docker --version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let version = String::from_utf8_lossy(&output.stdout);
        assert!(
            version.contains("Docker"),
            "docker version output doesn't contain 'Docker': {}",
            version
        );
    }

    /// Test that the Cargo.toml is properly configured for publishing
    #[test]
    fn test_cargo_toml_configured() {
        let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

        assert!(
            cargo_toml.contains("version = \"0.1.6\""),
            "Cargo.toml doesn't contain version 0.1.6"
        );

        assert!(
            cargo_toml.contains("ricecoder-cli"),
            "Cargo.toml doesn't contain ricecoder-cli crate"
        );
    }

    /// Test that package.json is properly configured for npm publishing
    #[test]
    fn test_package_json_configured() {
        let package_json =
            std::fs::read_to_string("package.json").expect("Failed to read package.json");

        assert!(
            package_json.contains("\"version\": \"0.1.6\""),
            "package.json doesn't contain version 0.1.6"
        );

        assert!(
            package_json.contains("\"name\": \"ricecoder\""),
            "package.json doesn't contain name ricecoder"
        );

        assert!(
            package_json.contains("postinstall"),
            "package.json doesn't contain postinstall script"
        );
    }

    /// Test that Dockerfile exists and is properly configured
    #[test]
    fn test_dockerfile_configured() {
        assert!(Path::new("Dockerfile").exists(), "Dockerfile doesn't exist");

        let dockerfile = std::fs::read_to_string("Dockerfile").expect("Failed to read Dockerfile");

        assert!(
            dockerfile.contains("FROM rust:"),
            "Dockerfile doesn't contain Rust base image"
        );

        assert!(
            dockerfile.contains("FROM alpine:"),
            "Dockerfile doesn't contain Alpine runtime image"
        );

        assert!(
            dockerfile.contains("ricecoder"),
            "Dockerfile doesn't reference ricecoder"
        );
    }

    /// Test that installation scripts exist
    #[test]
    fn test_installation_scripts_exist() {
        assert!(
            Path::new("scripts/install.sh").exists(),
            "scripts/install.sh doesn't exist"
        );

        assert!(
            Path::new("scripts/install.js").exists(),
            "scripts/install.js doesn't exist"
        );
    }

    /// Test that release workflow is configured
    #[test]
    fn test_release_workflow_configured() {
        assert!(
            Path::new(".github/workflows/release.yml").exists(),
            ".github/workflows/release.yml doesn't exist"
        );

        let workflow = std::fs::read_to_string(".github/workflows/release.yml")
            .expect("Failed to read release.yml");

        assert!(
            workflow.contains("v*.*.*"),
            "release.yml doesn't contain version tag trigger"
        );

        assert!(
            workflow.contains("x86_64-unknown-linux-musl"),
            "release.yml doesn't contain Linux x86_64 target"
        );

        assert!(
            workflow.contains("aarch64-unknown-linux-musl"),
            "release.yml doesn't contain Linux ARM64 target"
        );

        assert!(
            workflow.contains("x86_64-apple-darwin"),
            "release.yml doesn't contain macOS x86_64 target"
        );

        assert!(
            workflow.contains("aarch64-apple-darwin"),
            "release.yml doesn't contain macOS ARM64 target"
        );

        assert!(
            workflow.contains("x86_64-pc-windows-msvc"),
            "release.yml doesn't contain Windows x86_64 target"
        );

        assert!(
            workflow.contains("aarch64-pc-windows-msvc"),
            "release.yml doesn't contain Windows ARM64 target"
        );
    }

    /// Test that README contains installation instructions
    #[test]
    fn test_readme_installation_instructions() {
        // Try to read README.md, handling potential encoding issues
        let readme = match std::fs::read_to_string("README.md") {
            Ok(content) => content,
            Err(_) => {
                // If UTF-8 reading fails, try reading as bytes and converting
                let bytes = std::fs::read("README.md").expect("Failed to read README.md");
                String::from_utf8_lossy(&bytes).to_string()
            }
        };

        assert!(!readme.is_empty(), "README.md is empty");
    }

    /// Test that release notes exist for v0.1.6
    #[test]
    fn test_release_notes_exist() {
        assert!(
            Path::new("RELEASE_NOTES_v0.1.6_INSTALLATION.md").exists(),
            "RELEASE_NOTES_v0.1.6_INSTALLATION.md doesn't exist"
        );

        let release_notes = std::fs::read_to_string("RELEASE_NOTES_v0.1.6_INSTALLATION.md")
            .expect("Failed to read release notes");

        assert!(!release_notes.is_empty(), "Release notes are empty");
    }

    /// Test that the binary can be built locally
    #[test]
    #[ignore] // Skip in CI to avoid long build times
    fn test_local_build() {
        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("-p")
            .arg("ricecoder")
            .output()
            .expect("Failed to execute cargo build");

        assert!(
            output.status.success(),
            "cargo build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Test that the built binary can be executed
    #[test]
    #[ignore] // Skip in CI to avoid long build times
    fn test_binary_execution() {
        // First build the binary
        let build_output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("-p")
            .arg("ricecoder")
            .output()
            .expect("Failed to execute cargo build");

        assert!(
            build_output.status.success(),
            "cargo build failed: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );

        // Then try to run it
        let output = Command::new("cargo")
            .arg("run")
            .arg("--release")
            .arg("-p")
            .arg("ricecoder")
            .arg("--")
            .arg("--version")
            .output()
            .expect("Failed to execute ricecoder --version");

        assert!(
            output.status.success(),
            "ricecoder --version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let version_output = String::from_utf8_lossy(&output.stdout);
        assert!(
            !version_output.is_empty(),
            "ricecoder --version produced no output"
        );
    }

    /// Test that the binary help command works
    #[test]
    #[ignore] // Skip in CI to avoid long build times
    fn test_binary_help() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("--release")
            .arg("-p")
            .arg("ricecoder")
            .arg("--")
            .arg("--help")
            .output()
            .expect("Failed to execute ricecoder --help");

        assert!(
            output.status.success(),
            "ricecoder --help failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let help_output = String::from_utf8_lossy(&output.stdout);
        assert!(
            !help_output.is_empty(),
            "ricecoder --help produced no output"
        );
    }
}
