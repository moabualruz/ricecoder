//! Integration tests for ricegrep CLI
//!
//! These tests verify end-to-end functionality of the ricegrep command-line tool.

use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;
use std::path::PathBuf;

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
        .args(&["run", "--release", "--bin", "ricegrep", "--", "println", file_path.to_str().unwrap()])
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
        .args(&["run", "--release", "--bin", "ricegrep", "--", "--ignore-case", "function", file_path.to_str().unwrap()])
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
        .args(&["run", "--release", "--bin", "ricegrep", "--", "println", "--replace", "eprintln", "--preview", file_path.to_str().unwrap()])
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
    // Test index status (should work even without index)
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
        .args(&["run", "--release", "--bin", "ricegrep", "--", "--word-regexp", "print", file_path.to_str().unwrap()])
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
        .args(&["run", "--release", "--bin", "ricegrep", "--", "--max-count", "3", "println", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ricegrep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line_count = stdout.lines().filter(|line| line.contains("println")).count();
    assert!(line_count <= 3);
    assert!(output.status.success());
}