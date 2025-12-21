use std::path::PathBuf;
/// Binary Execution Tests
///
/// Tests that verify ricecoder binary executes correctly on all platforms.
/// These tests validate:
/// - Binary exists and is executable
/// - --version command works
/// - --help command works
/// - Binary startup time is acceptable
/// - Binary memory usage is reasonable
///
/// NOTE: These tests are designed to run after cross-compilation for all targets.
/// They will be skipped if the binary for the current platform is not available.
/// To run these tests, build the binary first:
///   cargo build --release --target <target>
use std::process::Command;

/// Get the binary path for the current platform
fn get_binary_path() -> PathBuf {
    // Get the workspace root by going up from the crate directory
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to determine workspace root");

    let target_dir = workspace_root.join("target");

    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "x86_64")]
    {
        target_dir.join("x86_64-unknown-linux-musl/release/ricecoder")
    }

    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "aarch64")]
    {
        target_dir.join("aarch64-unknown-linux-musl/release/ricecoder")
    }

    #[cfg(target_os = "macos")]
    #[cfg(target_arch = "x86_64")]
    {
        target_dir.join("x86_64-apple-darwin/release/ricecoder")
    }

    #[cfg(target_os = "macos")]
    #[cfg(target_arch = "aarch64")]
    {
        target_dir.join("aarch64-apple-darwin/release/ricecoder")
    }

    #[cfg(target_os = "windows")]
    #[cfg(target_arch = "x86_64")]
    {
        target_dir.join("x86_64-pc-windows-msvc/release/ricecoder.exe")
    }

    #[cfg(target_os = "windows")]
    #[cfg(target_arch = "aarch64")]
    {
        target_dir.join("aarch64-pc-windows-msvc/release/ricecoder.exe")
    }

    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "aarch64"),
    )))]
    {
        panic!("Unsupported platform/architecture combination");
    }
}

/// Test that binary exists
#[test]
fn test_binary_exists() {
    let binary_path = get_binary_path();
    if !binary_path.exists() {
        eprintln!("Binary not found at {:?}. Skipping test.", binary_path);
        eprintln!("Make sure to build with: cargo build --release --target <target>");
        return;
    }
    assert!(binary_path.exists());
}

/// Test that binary is executable
#[test]
fn test_binary_is_executable() {
    let binary_path = get_binary_path();

    if !binary_path.exists() {
        eprintln!("Binary not found at {:?}. Skipping test.", binary_path);
        return;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&binary_path).expect("Failed to get binary metadata");
        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Check if executable bit is set (0o111)
        assert!(
            mode & 0o111 != 0,
            "Binary is not executable. Run: chmod +x {:?}",
            binary_path
        );
    }

    #[cfg(not(unix))]
    {
        // On Windows, just check that the file exists
        assert!(binary_path.exists(), "Binary must exist");
    }
}

/// Test --version command
#[test]
fn test_version_command() {
    let binary_path = get_binary_path();

    if !binary_path.exists() {
        eprintln!("Binary not found at {:?}. Skipping test.", binary_path);
        return;
    }

    let output = Command::new(&binary_path)
        .arg("--version")
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "Binary --version command failed with exit code: {}",
        output.status.code().unwrap_or(-1)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ricecoder"),
        "Version output doesn't contain 'ricecoder': {}",
        stdout
    );
}

/// Test --help command
#[test]
fn test_help_command() {
    let binary_path = get_binary_path();

    if !binary_path.exists() {
        eprintln!("Binary not found at {:?}. Skipping test.", binary_path);
        return;
    }

    let output = Command::new(&binary_path)
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "Binary --help command failed with exit code: {}",
        output.status.code().unwrap_or(-1)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Help output is empty");

    // Check for common help keywords
    let has_help_content = stdout.contains("Usage")
        || stdout.contains("Commands")
        || stdout.contains("Options")
        || stdout.contains("ricecoder");

    assert!(
        has_help_content,
        "Help output doesn't contain expected content: {}",
        stdout
    );
}

/// Test that binary exits with error on invalid command
#[test]
fn test_invalid_command_error() {
    let binary_path = get_binary_path();

    if !binary_path.exists() {
        eprintln!("Binary not found at {:?}. Skipping test.", binary_path);
        return;
    }

    let output = Command::new(&binary_path)
        .arg("invalid-command-that-does-not-exist")
        .output()
        .expect("Failed to execute binary");

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Binary should fail on invalid command"
    );
}

/// Test binary startup time (should be < 2 seconds)
#[test]
fn test_binary_startup_time() {
    let binary_path = get_binary_path();

    if !binary_path.exists() {
        eprintln!("Binary not found at {:?}. Skipping test.", binary_path);
        return;
    }

    let start = std::time::Instant::now();

    let _output = Command::new(&binary_path)
        .arg("--version")
        .output()
        .expect("Failed to execute binary");

    let elapsed = start.elapsed();

    // Binary should start and execute --version in less than 2 seconds
    assert!(
        elapsed.as_secs() < 2,
        "Binary startup time is too slow: {:?}",
        elapsed
    );
}

/// Test binary file size (should be reasonable)
#[test]
fn test_binary_file_size() {
    let binary_path = get_binary_path();

    if !binary_path.exists() {
        eprintln!("Binary not found at {:?}. Skipping test.", binary_path);
        return;
    }

    let metadata = std::fs::metadata(&binary_path).expect("Failed to get binary metadata");

    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

    // Binary should be less than 500MB (reasonable for a Rust binary)
    assert!(
        size_mb < 500.0,
        "Binary size is too large: {:.2} MB",
        size_mb
    );

    // Binary should be at least 1MB (should contain actual code)
    assert!(size_mb > 1.0, "Binary size is too small: {:.2} MB", size_mb);
}

/// Property Test: Binary execution completeness
/// For any binary on any platform, it SHALL execute correctly
///
/// **Feature: ricecoder-installation, Property 4: Cross-Platform Execution**
/// **Validates: Requirements 6.6, 6.7**
#[test]
fn property_test_binary_execution_completeness() {
    let binary_path = get_binary_path();

    if !binary_path.exists() {
        eprintln!("Binary not found at {:?}. Skipping test.", binary_path);
        return;
    }

    // Test 1: Binary exists
    assert!(binary_path.exists(), "Binary must exist");

    // Test 2: Binary is executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&binary_path).expect("Failed to get metadata");
        let mode = metadata.permissions().mode();
        assert!(mode & 0o111 != 0, "Binary must be executable");
    }

    // Test 3: Binary executes successfully
    let output = Command::new(&binary_path)
        .arg("--version")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success(), "Binary must execute successfully");

    // Test 4: Binary produces expected output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Binary must produce output");
    assert!(
        stdout.contains("ricecoder"),
        "Output must contain 'ricecoder'"
    );
}

/// Property Test: Static linking verification (Linux only)
/// For any Linux binary, it SHALL be statically linked
///
/// **Feature: ricecoder-installation, Property 5: Static Linking Verification**
/// **Validates: Requirements 6.3**
#[test]
#[cfg(target_os = "linux")]
fn property_test_static_linking_verification() {
    let binary_path = get_binary_path();

    if !binary_path.exists() {
        eprintln!("Binary not found at {:?}. Skipping test.", binary_path);
        return;
    }

    // Check if binary is statically linked using 'file' command
    let output = Command::new("file")
        .arg(&binary_path)
        .output()
        .expect("Failed to run 'file' command");

    let file_info = String::from_utf8_lossy(&output.stdout);

    // For statically linked binaries, 'file' output should contain "statically linked"
    // or should NOT contain "dynamically linked"
    let is_static =
        file_info.contains("statically linked") || !file_info.contains("dynamically linked");

    assert!(
        is_static,
        "Linux binary must be statically linked. File info: {}",
        file_info
    );
}

/// Property Test: Binary consistency across runs
/// For any binary, running it multiple times SHALL produce consistent output
///
/// **Feature: ricecoder-installation, Property 1: Installation Completeness**
/// **Validates: Requirements 1.2, 2.4, 3.3, 4.2**
#[test]
fn property_test_binary_consistency() {
    let binary_path = get_binary_path();

    if !binary_path.exists() {
        eprintln!("Binary not found at {:?}. Skipping test.", binary_path);
        return;
    }

    // Run binary multiple times
    let mut outputs = Vec::new();
    for _ in 0..3 {
        let output = Command::new(&binary_path)
            .arg("--version")
            .output()
            .expect("Failed to execute binary");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        outputs.push(stdout);
    }

    // All outputs should be identical
    for i in 1..outputs.len() {
        assert_eq!(
            outputs[0], outputs[i],
            "Binary output must be consistent across runs"
        );
    }
}
