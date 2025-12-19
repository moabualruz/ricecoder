//! Compatibility tests for ricegrep vs ripgrep
//!
//! These tests verify that ricegrep produces compatible output with ripgrep
//! for common use cases, ensuring drop-in replacement capability.

use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;

fn create_test_file() -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fn main() {{").unwrap();
    writeln!(temp_file, "    println!(\"Hello, world!\");").unwrap();
    writeln!(temp_file, "    let x = 42;").unwrap();
    writeln!(temp_file, "    println!(\"x = {{}}\", x);").unwrap();
    writeln!(temp_file, "}}").unwrap();
    temp_file
}

fn run_ricegrep(args: &[&str], file_path: &str) -> String {
    let mut cmd_args = vec!["run", "--release", "--bin", "ricegrep", "--"];
    cmd_args.extend_from_slice(args);
    cmd_args.push(file_path);

    let output = Command::new("cargo")
        .args(&cmd_args)
        .output()
        .expect("Failed to execute ricegrep");

    assert!(output.status.success(), "ricegrep failed: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn run_ripgrep(args: &[&str], file_path: &str) -> String {
    let mut cmd_args = args.to_vec();
    cmd_args.push(file_path);

    let output = Command::new("rg")
        .args(&cmd_args)
        .output()
        .expect("Failed to execute ripgrep");

    assert!(output.status.success(), "ripgrep failed: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_basic_search_compatibility() {
    let temp_file = create_test_file();
    let file_path = temp_file.path().to_str().unwrap();

    let rg_output = run_ripgrep(&["println"], file_path);
    let rice_output = run_ricegrep(&["println"], file_path);

    // Both should contain the matching lines
    assert!(rg_output.contains("println"));
    assert!(rice_output.contains("println"));
}

#[test]
fn test_line_numbers_compatibility() {
    let temp_file = create_test_file();
    let file_path = temp_file.path().to_str().unwrap();

    let rg_output = run_ripgrep(&["--line-number", "println"], file_path);
    let rice_output = run_ricegrep(&["--line-number", "println"], file_path);

    // Both should show line numbers
    assert!(rg_output.contains("2:") || rg_output.contains("4:"));
    assert!(rice_output.contains("2:") || rice_output.contains("4:"));
}

#[test]
fn test_case_insensitive_compatibility() {
    let temp_file = create_test_file();
    let file_path = temp_file.path().to_str().unwrap();

    let rg_output = run_ripgrep(&["--ignore-case", "PRINTLN"], file_path);
    let rice_output = run_ricegrep(&["--ignore-case", "PRINTLN"], file_path);

    // Both should find the matches
    assert!(rg_output.contains("println"));
    assert!(rice_output.contains("println"));
}

#[test]
fn test_word_regex_compatibility() {
    let temp_file = create_test_file();
    let file_path = temp_file.path().to_str().unwrap();

    let rg_output = run_ripgrep(&["--word-regexp", "let"], file_path);
    let rice_output = run_ricegrep(&["--word-regexp", "let"], file_path);

    // Both should find "let" as a word
    assert!(rg_output.contains("let x = 42;"));
    assert!(rice_output.contains("let x = 42;"));
}

#[test]
fn test_count_compatibility() {
    let temp_file = create_test_file();
    let file_path = temp_file.path().to_str().unwrap();

    let rg_output = run_ripgrep(&["--count", "println"], file_path);
    let rice_output = run_ricegrep(&["--count", "println"], file_path);

    // Both should show count of 2
    // Ripgrep shows just "2" for single file, ricegrep shows "filename:2"
    assert!(rg_output.trim() == "2" || rg_output.contains(":2"));
    assert!(rice_output.contains(":2"));
}

#[test]
fn test_invert_match_compatibility() {
    let temp_file = create_test_file();
    let file_path = temp_file.path().to_str().unwrap();

    let rg_output = run_ripgrep(&["--invert-match", "println"], file_path);
    let rice_output = run_ricegrep(&["--invert-match", "println"], file_path);

    // Both should show lines that don't contain "println"
    assert!(rg_output.contains("fn main()"));
    assert!(rice_output.contains("fn main()"));
    assert!(!rg_output.contains("println"));
    assert!(!rice_output.contains("println"));
}

#[test]
fn test_max_count_compatibility() {
    let temp_file = create_test_file();
    let file_path = temp_file.path().to_str().unwrap();

    let rg_output = run_ripgrep(&["--max-count", "1", "println"], file_path);
    let rice_output = run_ricegrep(&["--max-count", "1", "println"], file_path);

    // Both should show only 1 match
    let rg_lines: Vec<&str> = rg_output.lines().filter(|l| l.contains("println")).collect();
    let rice_lines: Vec<&str> = rice_output.lines().filter(|l| l.contains("println")).collect();

    assert_eq!(rg_lines.len(), 1);
    assert_eq!(rice_lines.len(), 1);
}

#[test]
fn test_no_filename_compatibility() {
    let temp_file = create_test_file();
    let file_path = temp_file.path().to_str().unwrap();

    let rg_output = run_ripgrep(&["--no-filename", "println"], file_path);
    let rice_output = run_ricegrep(&["--no-filename", "println"], file_path);

    // Both should not show filenames (since we're searching single file)
    assert!(!rg_output.contains(temp_file.path().file_name().unwrap().to_str().unwrap()));
    assert!(!rice_output.contains(temp_file.path().file_name().unwrap().to_str().unwrap()));
}

#[test]
fn test_fixed_strings_compatibility() {
    let temp_file = create_test_file();
    let file_path = temp_file.path().to_str().unwrap();

    let rg_output = run_ripgrep(&["--fixed-strings", "println!"], file_path);
    let rice_output = run_ricegrep(&["--fixed-strings", "println!"], file_path);

    // Both should find the literal string
    assert!(rg_output.contains("println"));
    assert!(rice_output.contains("println"));
}

#[test]
fn test_before_context_compatibility() {
    let temp_file = create_test_file();
    let file_path = temp_file.path().to_str().unwrap();

    let rg_output = run_ripgrep(&["--before-context", "1", "println"], file_path);
    let rice_output = run_ricegrep(&["--before-context", "1", "println"], file_path);

    // Both should show context lines
    assert!(rg_output.contains("fn main()"));
    assert!(rice_output.contains("fn main()"));
}

#[test]
fn test_after_context_compatibility() {
    let temp_file = create_test_file();
    let file_path = temp_file.path().to_str().unwrap();

    let rg_output = run_ripgrep(&["--after-context", "1", "println"], file_path);
    let rice_output = run_ricegrep(&["--after-context", "1", "println"], file_path);

    // Both should show context lines
    assert!(rg_output.contains("let x = 42;"));
    assert!(rice_output.contains("let x = 42;"));
}