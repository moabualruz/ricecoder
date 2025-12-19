//! Integration tests for ricegrep CLI
//!
//! These tests verify end-to-end functionality of the ricegrep command-line tool.

use std::process::Command;
use tempfile::{NamedTempFile, TempDir};
use std::io::Write;
use std::path::PathBuf;
use std::fs;

#[test]
fn test_cli_basic_search() {
    // Create a temporary file with test content
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fn main() {{").unwrap();
    writeln!(temp_file, "    println!(\"Hello, world!\");").unwrap();
    writeln!(temp_file, "}}").unwrap();

    let file_path = temp_file.path();

    // Run ricegrep search
    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "ricegrep", "--", "search", "println", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("println"));
    assert!(output.status.success());
}

#[test]
fn test_cli_case_insensitive_search() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "FUNCTION test() {{").unwrap();
    writeln!(temp_file, "    return TRUE;").unwrap();
    writeln!(temp_file, "}}").unwrap();

    let file_path = temp_file.path();

    // Test case insensitive search
    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "ricegrep", "--", "search", "--ignore-case", "function", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FUNCTION"));
    assert!(output.status.success());
}

#[test]
fn test_cli_replace_preview() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fn main() {{").unwrap();
    writeln!(temp_file, "    println!(\"hello\");").unwrap();
    writeln!(temp_file, "}}").unwrap();

    let file_path = temp_file.path();

    // Test replace preview
    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "ricegrep", "--", "search", "println", "--replace", "eprintln", "--preview", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Preview of replace operations"));
    assert!(stdout.contains("println"));
    assert!(stdout.contains("eprintln"));
    assert!(output.status.success());
}

#[test]
fn test_cli_index_operations() {
    // Test index status (should work even without index) - use legacy mode
    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "ricegrep", "--", "--index-status"])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No index found") || stdout.contains("Index"));
    assert!(output.status.success());
}

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "ricegrep", "--", "--help"])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AI-enhanced code search tool"));
    assert!(stdout.contains("Usage:"));
    assert!(output.status.success());
}

#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "ricegrep", "--", "--version"])
        .output()
        .expect("Failed to execute ricegrep");

    assert!(output.status.success());
}

#[test]
fn test_cli_json_output() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fn test() {{").unwrap();
    writeln!(temp_file, "    println!(\"test\");").unwrap();
    writeln!(temp_file, "}}").unwrap();

    let file_path = temp_file.path();

    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "ricegrep", "--", "--json", "println", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // JSON output should contain valid JSON
    assert!(stdout.contains("{") || stdout.is_empty());
    assert!(output.status.success());
}

#[test]
fn test_cli_word_regex() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "function test() {{").unwrap();
    writeln!(temp_file, "    println(\"test\");").unwrap();
    writeln!(temp_file, "    print(\"not a match\");").unwrap();
    writeln!(temp_file, "}}").unwrap();

    let file_path = temp_file.path();

    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "ricegrep", "--", "search", "--word-regexp", "println", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("println"));
    assert!(!stdout.contains("print(\"not a match\")"));
    assert!(output.status.success());
}

#[test]
fn test_cli_max_count() {
    let mut temp_file = NamedTempFile::new().unwrap();
    for i in 0..10 {
        writeln!(temp_file, "println!(\"line {}\");", i).unwrap();
    }

    let file_path = temp_file.path();

    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "ricegrep", "--", "search", "--max-count", "3", "println", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line_count = stdout.lines().filter(|line| line.contains("println")).count();
    assert!(line_count <= 3);
    assert!(output.status.success());
}

#[test]
fn test_cli_content_display() {
    // Create a temporary file with test content
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fn main() {{").unwrap();
    writeln!(temp_file, "    println!(\"Hello, world!\");").unwrap();
    writeln!(temp_file, "}}").unwrap();

    let file_path = temp_file.path();

    // Run ricegrep search with --content flag
    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "ricegrep", "--", "search", "--content", "println", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Content mode should show the full file content
    assert!(stdout.contains("fn main()"));
    assert!(stdout.contains("println"));
    assert!(output.status.success());
}

#[test]
fn test_cli_answer_generation() {
    // Create a temporary file with test content
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fn main() {{").unwrap();
    writeln!(temp_file, "    println!(\"Hello, world!\");").unwrap();
    writeln!(temp_file, "}}").unwrap();

    let file_path = temp_file.path();

    // Run ricegrep search with --answer flag
    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "ricegrep", "--", "search", "--answer", "println", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ricegrep");

    // Answer mode should succeed (even if AI is not available, it should fall back gracefully)
    assert!(output.status.success());
}

#[test]
fn test_cli_no_rerank() {
    // Create a temporary file with test content
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fn main() {{").unwrap();
    writeln!(temp_file, "    println!(\"Hello, world!\");").unwrap();
    writeln!(temp_file, "}}").unwrap();

    let file_path = temp_file.path();

    // Run ricegrep search with --no-rerank flag
    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "ricegrep", "--", "search", "--no-rerank", "println", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should find the match without AI reranking
    assert!(stdout.contains("println"));
    assert!(output.status.success());
}

#[test]
fn test_cli_custom_ignore_file_basic() {
    // Create a temporary directory with test files
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    let file1_path = temp_path.join("test1.txt");
    let file2_path = temp_path.join("test2.txt");
    let ignored_file_path = temp_path.join("ignored.txt");

    fs::write(&file1_path, "This is test1 content with pattern").unwrap();
    fs::write(&file2_path, "This is test2 content with pattern").unwrap();
    fs::write(&ignored_file_path, "This is ignored content with pattern").unwrap();

    // Create custom ignore file
    let ignore_file_path = temp_path.join(".ricegrepignore");
    fs::write(&ignore_file_path, "ignored.txt\n").unwrap();

    // Test search with custom ignore file
    let output = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "--ignore-file", ignore_file_path.to_str().unwrap(),
            "pattern", temp_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test1.txt"));
    assert!(stdout.contains("test2.txt"));
    assert!(!stdout.contains("ignored.txt"));
    assert!(output.status.success());
}

#[test]
fn test_cli_custom_ignore_file_gitignore_patterns() {
    // Create a temporary directory with test files
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create directory structure
    let subdir_path = temp_path.join("subdir");
    fs::create_dir(&subdir_path).unwrap();

    // Create test files
    let file1_path = temp_path.join("important.rs");
    let file2_path = subdir_path.join("important.rs");
    let ignored_file_path = temp_path.join("temp.rs");

    fs::write(&file1_path, "fn main() { println!(\"hello\"); }").unwrap();
    fs::write(&file2_path, "fn main() { println!(\"hello\"); }").unwrap();
    fs::write(&ignored_file_path, "fn main() { println!(\"hello\"); }").unwrap();

    // Check files exist
    assert!(file1_path.exists());
    assert!(file2_path.exists());
    assert!(ignored_file_path.exists());

    // Create custom ignore file with .gitignore-style patterns
    let ignore_file_path = temp_path.join(".ricegrepignore");
    fs::write(&ignore_file_path, "*.rs\n!subdir/\n").unwrap();

    // Test search with custom ignore file
    let output = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "--ignore-file", ignore_file_path.to_str().unwrap(),
            "println", temp_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should find files in subdir (due to !subdir/ exception) but not temp.rs
    assert!(stdout.contains("subdir") && stdout.contains("important.rs"));
    assert!(!stdout.contains("temp.rs"));
    assert!(output.status.success());
}

#[test]
fn test_cli_custom_ignore_file_nonexistent() {
    // Create a temporary file for search
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "test content").unwrap();
    let file_path = temp_file.path();

    // Test with non-existent ignore file
    let output = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "--ignore-file", "/nonexistent/ignore/file",
            "test", file_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    // Should fail with error message
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist"));
}

#[test]
fn test_cli_custom_ignore_file_directory() {
    // Create a temporary file for search
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "test content").unwrap();
    let file_path = temp_file.path();

    // Test with directory as ignore file (should fail)
    let temp_dir = TempDir::new().unwrap();
    let output = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "--ignore-file", temp_dir.path().to_str().unwrap(),
            "test", file_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    // Should fail with error message
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not a regular file"));
}

#[test]
fn test_cli_progress_verbosity_levels() {
    // Create a temporary directory with test files
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create multiple test files to trigger progress display
    for i in 0..5 {
        let file_path = temp_path.join(format!("test{}.rs", i));
        fs::write(&file_path, format!("fn test_{}() {{ println!(\"test {}\"); }}", i, i)).unwrap();
    }

    // Test normal progress verbosity (should show progress bar)
    let output = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "println", temp_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Normal verbosity should show search results
    assert!(output.status.success());

    // Test quiet progress verbosity (should suppress progress but still show results)
    let output_quiet = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "--quiet", "println", temp_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout_quiet = String::from_utf8_lossy(&output_quiet.stdout);
    // Quiet mode should still work and show results
    assert!(output_quiet.status.success());
    assert!(stdout_quiet.contains("println"));
}

#[test]
fn test_cli_dry_run_accuracy() {
    // Create a temporary file for replace testing
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fn main() {{").unwrap();
    writeln!(temp_file, "    println!(\"hello world\");").unwrap();
    writeln!(temp_file, "    println!(\"test message\");").unwrap();
    writeln!(temp_file, "}}").unwrap();

    let file_path = temp_file.path();

    // Test dry-run mode (should show preview without making changes)
    let output_dry = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "--dry-run", "println", "--replace", "log::info", file_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout_dry = String::from_utf8_lossy(&output_dry.stdout);
    assert!(output_dry.status.success());
    assert!(stdout_dry.contains("Dry-run of replace operations"));
    assert!(stdout_dry.contains("println"));
    assert!(stdout_dry.contains("log::info"));

    // Verify the file was NOT actually changed
    let content_after = fs::read_to_string(file_path).unwrap();
    assert!(content_after.contains("println!(\"hello world\")"));
    assert!(!content_after.contains("log::info"));

    // Test normal replace mode (should actually make changes)
    let output_replace = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "println", "--replace", "log::info", "--force", file_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    assert!(output_replace.status.success());

    // Verify the file WAS actually changed
    let content_after_replace = fs::read_to_string(file_path).unwrap();
    assert!(!content_after_replace.contains("println!(\"hello world\")"));
    assert!(content_after_replace.contains("log::info"));
}

#[test]
fn test_performance_regression_basic() {
    // Create test files for performance testing
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create multiple files with searchable content
    for i in 0..50 {
        let file_path = temp_path.join(format!("perf_test_{}.rs", i));
        let content = format!("fn function_{}() {{\n    println!(\"Hello from function {}\");\n    let x = {};\n    return x;\n}}\n", i, i, i);
        fs::write(&file_path, content).unwrap();
    }

    // Measure search performance
    use std::time::Instant;
    let start = Instant::now();

    let output = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "println", temp_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    let duration = start.elapsed();

    // Verify search succeeded
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should find matches in all files
    assert!(stdout.contains("perf_test_0.rs"));
    assert!(stdout.contains("perf_test_49.rs"));

    // Performance check: should complete in reasonable time (less than 60 seconds for 50 files in debug mode)
    assert!(duration.as_secs_f64() < 60.0, "Search took too long: {:.2}s", duration.as_secs_f64());

    println!("Performance test completed in {:.2}s", duration.as_secs_f64());
}

#[test]
fn test_cli_quota_limits() {
    // Create test files
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create multiple files with matches
    for i in 0..10 {
        let file_path = temp_path.join(format!("file{}.txt", i));
        fs::write(&file_path, format!("This file {} contains a test match", i)).unwrap();
    }

    // Test max-files quota
    let output_max_files = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "--max-files", "3", "test", temp_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout_max_files = String::from_utf8_lossy(&output_max_files.stdout);
    assert!(output_max_files.status.success());
    // Should only find matches in 3 files
    let file_count = stdout_max_files.lines()
        .filter(|line| line.contains("file") && line.contains(".txt"))
        .count();
    assert_eq!(file_count, 3);

    // Test max-matches quota
    let output_max_matches = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "--max-matches", "2", "test", temp_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout_max_matches = String::from_utf8_lossy(&output_max_matches.stdout);
    assert!(output_max_matches.status.success());
    // Should only find 2 matches total
    let match_count = stdout_max_matches.lines()
        .filter(|line| line.contains("test match"))
        .count();
    assert_eq!(match_count, 2);
}

#[test]
fn test_cli_max_file_size_filtering() {
    // Create test files of different sizes
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a small file (should be included)
    let small_file = temp_path.join("small.txt");
    fs::write(&small_file, "This is a small file with test content").unwrap();

    // Create a larger file (should be excluded by max-file-size)
    let large_file = temp_path.join("large.txt");
    let large_content = "x".repeat(2000); // 2000 bytes
    fs::write(&large_file, large_content).unwrap();

    // Test search with max-file-size limit
    let output = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "--max-file-size", "1000", "test", temp_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    // Should find the small file but not the large file
    assert!(stdout.contains("small.txt"));
    assert!(!stdout.contains("large.txt"));
}

#[test]
fn test_cli_custom_ignore_file_comments_and_empty_lines() {
    // Create a temporary directory with test files
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    let file1_path = temp_path.join("keep.txt");
    let file2_path = temp_path.join("ignore.txt");

    fs::write(&file1_path, "This file should be found").unwrap();
    fs::write(&file2_path, "This file should be ignored").unwrap();

    // Create custom ignore file with comments and empty lines
    let ignore_file_path = temp_path.join(".ricegrepignore");
    fs::write(&ignore_file_path, "# This is a comment\n\nignore.txt\n  \n# Another comment\n").unwrap();

    // Test search with custom ignore file
    let output = Command::new("cargo")
        .args(&[
            "run", "--release", "--bin", "ricegrep", "--",
            "search", "--ignore-file", ignore_file_path.to_str().unwrap(),
            "file", temp_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("keep.txt"));
    assert!(!stdout.contains("ignore.txt"));
    assert!(output.status.success());
}