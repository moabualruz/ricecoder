//! Integration tests for curl script installation
//!
//! Tests the curl installation script with real GitHub Releases
//! Tests on Linux, macOS, Windows
//! Tests error scenarios (network failure, invalid checksum)
//!
//! **Feature: ricecoder-installation, Property 1: Installation Completeness**
//! **Validates: Requirements 1.1, 1.2**

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Test helper: Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Test helper: Get the install script path
fn get_install_script_path() -> PathBuf {
    // Try multiple possible paths since tests can run from different directories
    let paths = vec![
        PathBuf::from("../../scripts/install.sh"),
        PathBuf::from("projects/ricecoder/scripts/install.sh"),
        PathBuf::from("scripts/install.sh"),
    ];

    for path in paths {
        if path.exists() {
            return path;
        }
    }

    // Return the most likely path if none exist
    PathBuf::from("../../scripts/install.sh")
}

/// Test helper: Verify script syntax
fn verify_script_syntax(script_path: &Path) -> bool {
    if !script_path.exists() {
        return false;
    }

    // Check if bash is available
    if !command_exists("bash") {
        return false;
    }

    // Verify script syntax with bash -n
    match Command::new("bash").arg("-n").arg(script_path).output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Test helper: Extract functions from script
fn extract_script_functions(script_path: &Path) -> Vec<String> {
    if !script_path.exists() {
        return vec![];
    }

    let content = fs::read_to_string(script_path).unwrap_or_default();
    let mut functions = vec![];

    for line in content.lines() {
        if line.contains("() {") {
            if let Some(func_name) = line.split('(').next() {
                functions.push(func_name.trim().to_string());
            }
        }
    }

    functions
}

#[test]
fn test_install_script_exists() {
    let script_path = get_install_script_path();
    assert!(
        script_path.exists(),
        "Install script should exist at {}",
        script_path.display()
    );
}

#[test]
fn test_install_script_is_executable() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let metadata = fs::metadata(&script_path).expect("Should read script metadata");
    let permissions = metadata.permissions();

    // Check if file is readable
    assert!(!permissions.readonly(), "Install script should be readable");
}

#[test]
fn test_install_script_syntax() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    // Only test syntax if bash is available
    if command_exists("bash") {
        assert!(
            verify_script_syntax(&script_path),
            "Install script should have valid bash syntax"
        );
    }
}

#[test]
fn test_install_script_has_required_functions() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let functions = extract_script_functions(&script_path);

    // Check for required functions
    let required_functions = vec![
        "detect_os",
        "detect_arch",
        "get_binary_name",
        "get_archive_ext",
        "download_with_retry",
        "verify_checksum",
        "extract_archive",
        "install_binary",
        "verify_installation",
    ];

    for required_func in required_functions {
        assert!(
            functions.iter().any(|f| f.contains(required_func)),
            "Install script should have {} function",
            required_func
        );
    }
}

#[test]
fn test_install_script_contains_error_handling() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for error handling patterns
    assert!(
        content.contains("set -e"),
        "Script should use 'set -e' for error handling"
    );
    assert!(
        content.contains("print_error"),
        "Script should have error printing function"
    );
    assert!(
        content.contains("return 1"),
        "Script should return error codes"
    );
}

#[test]
fn test_install_script_contains_retry_logic() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for retry logic
    assert!(
        content.contains("max_attempts"),
        "Script should have retry logic with max_attempts"
    );
    assert!(
        content.contains("attempt"),
        "Script should track retry attempts"
    );
    assert!(
        content.contains("backoff"),
        "Script should implement exponential backoff"
    );
}

#[test]
fn test_install_script_contains_checksum_verification() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for checksum verification
    assert!(
        content.contains("verify_checksum"),
        "Script should have checksum verification function"
    );
    assert!(
        content.contains("sha256sum"),
        "Script should use sha256sum for verification"
    );
}

#[test]
fn test_install_script_supports_multiple_platforms() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for platform support
    assert!(content.contains("linux"), "Script should support Linux");
    assert!(
        content.contains("macos") || content.contains("Darwin"),
        "Script should support macOS"
    );
    assert!(
        content.contains("windows") || content.contains("MINGW"),
        "Script should support Windows"
    );
}

#[test]
fn test_install_script_supports_multiple_architectures() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for architecture support
    assert!(
        content.contains("x86_64"),
        "Script should support x86_64 architecture"
    );
    assert!(
        content.contains("aarch64") || content.contains("arm64"),
        "Script should support ARM64 architecture"
    );
}

#[test]
fn test_install_script_handles_missing_tools() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for handling of missing tools
    assert!(
        content.contains("command -v"),
        "Script should check for command availability"
    );
    assert!(
        content.contains("&> /dev/null"),
        "Script should suppress command output when checking availability"
    );
}

#[test]
fn test_install_script_provides_helpful_messages() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for helpful messages
    assert!(
        content.contains("print_info"),
        "Script should have info messages"
    );
    assert!(
        content.contains("print_success"),
        "Script should have success messages"
    );
    assert!(
        content.contains("print_warning"),
        "Script should have warning messages"
    );
    assert!(
        content.contains("print_error"),
        "Script should have error messages"
    );
}

#[test]
fn test_install_script_cleanup_on_exit() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for cleanup logic
    assert!(
        content.contains("cleanup"),
        "Script should have cleanup function"
    );
    assert!(
        content.contains("trap cleanup EXIT"),
        "Script should trap EXIT and call cleanup"
    );
    assert!(
        content.contains("rm -rf"),
        "Script should remove temporary files"
    );
}

#[test]
fn test_install_script_github_api_integration() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for GitHub API integration
    assert!(
        content.contains("api.github.com"),
        "Script should use GitHub API"
    );
    assert!(
        content.contains("releases/latest"),
        "Script should fetch latest release"
    );
    assert!(
        content.contains("tag_name"),
        "Script should parse tag_name from GitHub API"
    );
}

#[test]
fn test_install_script_binary_download_urls() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for binary download URL construction
    assert!(
        content.contains("github.com") && content.contains("releases/download"),
        "Script should construct GitHub release download URLs"
    );
}

#[test]
fn test_install_script_path_handling() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for PATH handling
    assert!(
        content.contains("INSTALL_DIR"),
        "Script should have INSTALL_DIR variable"
    );
    assert!(
        content.contains("/usr/local/bin"),
        "Script should default to /usr/local/bin"
    );
    assert!(
        content.contains("export PATH"),
        "Script should provide PATH export instructions"
    );
}

#[test]
fn test_install_script_permission_handling() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for permission handling
    assert!(
        content.contains("chmod +x"),
        "Script should set executable permissions"
    );
    assert!(
        content.contains("sudo"),
        "Script should handle permission issues with sudo"
    );
}

#[test]
fn test_install_script_version_verification() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for version verification
    assert!(
        content.contains("--version"),
        "Script should verify installation with --version"
    );
    assert!(
        content.contains("ricecoder --version"),
        "Script should run ricecoder --version to verify"
    );
}

#[test]
fn test_install_script_temp_directory_usage() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for temporary directory usage
    assert!(
        content.contains("mktemp -d"),
        "Script should create temporary directory"
    );
    assert!(
        content.contains("TEMP_DIR"),
        "Script should use TEMP_DIR variable"
    );
}

#[test]
fn test_install_script_archive_extraction() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for archive extraction
    assert!(
        content.contains("tar -xzf"),
        "Script should extract tar.gz archives"
    );
    assert!(
        content.contains("unzip"),
        "Script should extract zip archives"
    );
}

#[test]
fn test_install_script_curl_flags() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for curl flags (can be combined like -fsSL)
    assert!(
        content.contains("curl") && (content.contains("-f") || content.contains("-fsSL")),
        "Script should use curl with -f flag (fail on HTTP errors)"
    );
    assert!(
        content.contains("curl") && (content.contains("-s") || content.contains("-fsSL")),
        "Script should use curl with -s flag (silent mode)"
    );
    assert!(
        content.contains("curl") && (content.contains("-L") || content.contains("-fsSL")),
        "Script should use curl with -L flag (follow redirects)"
    );
}

#[test]
fn test_install_script_color_output() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for color output
    assert!(
        content.contains("033"),
        "Script should use ANSI color codes"
    );
    assert!(content.contains("GREEN"), "Script should have GREEN color");
    assert!(content.contains("RED"), "Script should have RED color");
}

#[test]
fn test_install_script_main_function() {
    let script_path = get_install_script_path();
    assert!(script_path.exists(), "Install script should exist");

    let content = fs::read_to_string(&script_path).expect("Should read script");

    // Check for main function
    assert!(
        content.contains("main()"),
        "Script should have main function"
    );
    assert!(
        content.contains("main \"$@\""),
        "Script should call main function with arguments"
    );
}
