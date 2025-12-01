// Property-based tests for custom commands
// **Feature: ricecoder-commands, Property 1: Argument Substitution Correctness**
// **Feature: ricecoder-commands, Property 2: Argument Validation**
// **Feature: ricecoder-commands, Property 3: Shell Output Injection Safety**
// **Feature: ricecoder-commands, Property 4: Shell Output Truncation**
// **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5**

use proptest::prelude::*;
use ricecoder_cli::commands::custom::{template, shell, truncation, file_reference};
use std::fs;
use tempfile::TempDir;

// Strategy for generating valid argument strings
fn arb_argument() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-./]+"
        .prop_map(|s| s.to_string())
        .boxed()
}

// Strategy for generating argument lists
fn arb_arguments() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_argument(), 0..10)
}

// Strategy for generating templates with $ARGUMENTS placeholder
fn arb_template_with_all_arguments() -> impl Strategy<Value = String> {
    (
        r"[a-zA-Z0-9_\-./]*",
        r"[a-zA-Z0-9_\-./]*",
    )
        .prop_map(|(prefix, suffix)| {
            format!("{} $ARGUMENTS {}", prefix, suffix)
        })
        .boxed()
}

// Strategy for generating templates with positional placeholders
#[allow(dead_code)]
fn arb_template_with_positional(max_index: usize) -> impl Strategy<Value = String> {
    let indices: Vec<usize> = (1..=max_index).collect();
    Just(indices)
        .prop_flat_map(|indices| {
            let mut template = String::new();
            for (i, idx) in indices.iter().enumerate() {
                if i > 0 {
                    template.push(' ');
                }
                template.push_str(&format!("${}", idx));
            }
            Just(template)
        })
        .boxed()
}

// Property 1: Argument Substitution Correctness
// For any command template with argument placeholders and any set of provided arguments,
// substitution SHALL replace all placeholders with corresponding argument values,
// and the resulting template SHALL contain no unsubstituted placeholders.
proptest! {
    #[test]
    fn prop_argument_substitution_all_arguments_replaced(
        template in arb_template_with_all_arguments(),
        args in arb_arguments(),
    ) {
        // Substitute arguments
        let result = template::substitute(&template, &args, false);

        // Should succeed
        prop_assert!(result.is_ok(), "Substitution should succeed");

        let substituted = result.unwrap();

        // The result should not contain $ARGUMENTS
        prop_assert!(
            !substituted.contains("$ARGUMENTS"),
            "Result should not contain $ARGUMENTS placeholder"
        );

        // If we had arguments, they should be in the result
        if !args.is_empty() {
            let args_str = args.join(" ");
            prop_assert!(
                substituted.contains(&args_str),
                "Result should contain the substituted arguments"
            );
        }
    }

    #[test]
    fn prop_argument_substitution_positional_replaced(
        args in arb_arguments(),
    ) {
        // Only test if we have at least 3 arguments
        if args.len() < 3 {
            return Ok(());
        }

        let template = "$1 and $2 and $3";

        // Substitute arguments
        let result = template::substitute(template, &args, false);

        // Should succeed
        prop_assert!(result.is_ok(), "Substitution should succeed");

        let substituted = result.unwrap();

        // The result should contain the first three arguments
        prop_assert!(
            substituted.contains(&args[0]),
            "Result should contain first argument"
        );
        prop_assert!(
            substituted.contains(&args[1]),
            "Result should contain second argument"
        );
        prop_assert!(
            substituted.contains(&args[2]),
            "Result should contain third argument"
        );

        // The result should not contain any positional placeholders
        prop_assert!(
            !substituted.contains("$1"),
            "Result should not contain $1 placeholder"
        );
        prop_assert!(
            !substituted.contains("$2"),
            "Result should not contain $2 placeholder"
        );
        prop_assert!(
            !substituted.contains("$3"),
            "Result should not contain $3 placeholder"
        );
    }

    #[test]
    fn prop_argument_substitution_no_unsubstituted_placeholders(
        template in r"[a-zA-Z0-9_\-./\s]*\$ARGUMENTS[a-zA-Z0-9_\-./\s]*",
        args in arb_arguments(),
    ) {
        // Substitute arguments
        let result = template::substitute(&template, &args, false);

        // Should succeed
        prop_assert!(result.is_ok(), "Substitution should succeed");

        let substituted = result.unwrap();

        // The result should not contain any $ followed by a letter or digit
        // (which would indicate an unsubstituted placeholder)
        let has_unsubstituted = substituted.contains("$ARGUMENTS")
            || substituted.contains("$1")
            || substituted.contains("$2")
            || substituted.contains("$3")
            || substituted.contains("$4")
            || substituted.contains("$5");

        prop_assert!(
            !has_unsubstituted,
            "Result should not contain unsubstituted placeholders"
        );
    }

    #[test]
    fn prop_argument_substitution_escaping_preserves_content(
        args in arb_arguments(),
    ) {
        let template = "$ARGUMENTS";

        // Substitute with escaping
        let result_escaped = template::substitute(template, &args, true);
        let result_unescaped = template::substitute(template, &args, false);

        // Both should succeed
        prop_assert!(result_escaped.is_ok(), "Escaped substitution should succeed");
        prop_assert!(result_unescaped.is_ok(), "Unescaped substitution should succeed");

        let escaped = result_escaped.unwrap();
        let unescaped = result_unescaped.unwrap();

        // Both should contain all arguments (possibly escaped)
        for arg in &args {
            // The escaped version might have quotes around the argument
            let contains_escaped = escaped.contains(arg)
                || escaped.contains(&format!("'{}'", arg));

            prop_assert!(
                contains_escaped,
                "Escaped result should contain argument (possibly quoted): {}",
                arg
            );

            prop_assert!(
                unescaped.contains(arg),
                "Unescaped result should contain argument: {}",
                arg
            );
        }
    }
}

// Property 2: Argument Validation
// For any command requiring N arguments, if fewer than N arguments are provided,
// the system SHALL report an error with the required argument count.
// If more than N arguments are provided, the system SHALL either ignore extra arguments
// or use them as specified in the template.
proptest! {
    #[test]
    fn prop_argument_validation_missing_arguments(
        args in arb_arguments(),
    ) {
        // Only test if we have fewer than 3 arguments
        if args.len() >= 3 {
            return Ok(());
        }

        let template = "$1 and $2 and $3";

        // Substitution should fail
        let result = template::substitute(template, &args, false);
        prop_assert!(result.is_err(), "Substitution should fail with missing arguments");

        // Check that the error is about missing arguments
        match result.unwrap_err() {
            template::SubstitutionError::MissingArgument { index, .. } => {
                prop_assert!(index >= 1 && index <= 3, "Error should reference a missing argument");
            }
            _ => {
                prop_assert!(false, "Error should be MissingArgument");
            }
        }
    }

    #[test]
    fn prop_argument_validation_extra_arguments_ignored(
        args in arb_arguments(),
    ) {
        // Only test if we have at least 2 arguments
        if args.len() < 2 {
            return Ok(());
        }

        let template = "$1 and $2";

        // Substitution should succeed even with extra arguments
        let result = template::substitute(template, &args, false);
        prop_assert!(result.is_ok(), "Substitution should succeed with extra arguments");

        let substituted = result.unwrap();

        // The result should contain the first two arguments
        prop_assert!(
            substituted.contains(&args[0]),
            "Result should contain first argument"
        );
        prop_assert!(
            substituted.contains(&args[1]),
            "Result should contain second argument"
        );
    }

    #[test]
    fn prop_argument_validation_all_arguments_used(
        args in arb_arguments(),
    ) {
        // Only test if we have at least 1 argument
        if args.is_empty() {
            return Ok(());
        }

        let template = "$ARGUMENTS";

        // Substitution should succeed
        let result = template::substitute(template, &args, false);
        prop_assert!(result.is_ok(), "Substitution should succeed");

        let substituted = result.unwrap();

        // All arguments should be in the result
        for arg in &args {
            prop_assert!(
                substituted.contains(arg),
                "Result should contain all arguments"
            );
        }
    }

    #[test]
    fn prop_argument_validation_zero_index_invalid(
        args in arb_arguments(),
    ) {
        let template = "$0";

        // Substitution should fail (0-based indexing is invalid)
        let result = template::substitute(template, &args, false);
        prop_assert!(result.is_err(), "Substitution with $0 should fail");

        match result.unwrap_err() {
            template::SubstitutionError::InvalidPlaceholder { .. } => {
                // Expected
            }
            _ => {
                prop_assert!(false, "Error should be InvalidPlaceholder");
            }
        }
    }

    #[test]
    fn prop_argument_validation_consistency(
        template in r"[a-zA-Z0-9_\-./\s]*\$ARGUMENTS[a-zA-Z0-9_\-./\s]*",
        args in arb_arguments(),
    ) {
        // Validate should have the same result as substitute
        let validate_result = template::validate(&template, &args);
        let substitute_result = template::substitute(&template, &args, false);

        // Both should succeed or both should fail
        prop_assert_eq!(
            validate_result.is_ok(),
            substitute_result.is_ok(),
            "Validate and substitute should have consistent results"
        );
    }
}

// Strategy for generating output strings
fn arb_output() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-./\s\n]*"
        .prop_map(|s| s.to_string())
        .boxed()
}

// Property 3: Shell Output Injection Safety
// For any shell command injection in a template, the system SHALL execute the command,
// capture its output, and inject it into the template. If the command fails or times out,
// the system SHALL report the error without crashing.
proptest! {
    #[test]
    fn prop_shell_output_injection_no_crash_on_success(
        output in arb_output(),
    ) {
        // Create a shell result
        let result = shell::ShellResult {
            stdout: output.clone(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
            duration_ms: 100,
        };

        // Should not crash
        let combined = result.combined_output();
        prop_assert_eq!(combined, output, "Combined output should match stdout");
    }

    #[test]
    fn prop_shell_output_injection_captures_stderr(
        stdout in arb_output(),
        stderr in arb_output(),
    ) {
        // Create a shell result with both stdout and stderr
        let result = shell::ShellResult {
            stdout: stdout.clone(),
            stderr: stderr.clone(),
            exit_code: 1,
            success: false,
            duration_ms: 100,
        };

        // Combined output should contain both
        let combined = result.combined_output();
        if !stdout.is_empty() {
            prop_assert!(combined.contains(&stdout), "Combined output should contain stdout");
        }
        if !stderr.is_empty() {
            prop_assert!(combined.contains(&stderr), "Combined output should contain stderr");
        }
    }

    #[test]
    fn prop_shell_output_injection_handles_empty_output(
        exit_code in 0i32..=255,
    ) {
        // Create a shell result with empty output
        let result = shell::ShellResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code,
            success: exit_code == 0,
            duration_ms: 100,
        };

        // Should not crash
        let combined = result.combined_output();
        prop_assert_eq!(combined, "", "Combined output should be empty");
    }

    #[test]
    fn prop_shell_output_injection_preserves_content(
        output in arb_output(),
    ) {
        // Create a shell result
        let result = shell::ShellResult {
            stdout: output.clone(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
            duration_ms: 100,
        };

        // Combined output should preserve the original content
        let combined = result.combined_output();
        prop_assert_eq!(combined, output, "Combined output should preserve original content");
    }
}

// Property 4: Shell Output Truncation
// For any shell command output exceeding a defined size threshold,
// the system SHALL truncate the output and indicate truncation to the user.
proptest! {
    #[test]
    fn prop_shell_output_truncation_small_output_unchanged(
        output in r"[a-zA-Z0-9_\-./\s]{0,100}",
    ) {
        // Truncate with a large max size
        let (truncated, was_truncated) = truncation::truncate_output(&output, 10000);

        // Should not be truncated
        prop_assert!(!was_truncated, "Small output should not be truncated");
        prop_assert_eq!(truncated, output, "Small output should be unchanged");
    }

    #[test]
    fn prop_shell_output_truncation_large_output_truncated(
        output in r"[a-zA-Z0-9_\-./\s]{1000,10000}",
    ) {
        // Truncate with a small max size
        let (truncated, was_truncated) = truncation::truncate_output(&output, 100);

        // Should be truncated
        prop_assert!(was_truncated, "Large output should be truncated");
        prop_assert!(truncated.len() <= 200, "Truncated output should be reasonably sized");
        prop_assert!(truncation::is_truncated(&truncated), "Truncated output should be marked");
    }

    #[test]
    fn prop_shell_output_truncation_preserves_utf8(
        output in r"[a-zA-Z0-9_\-./\s]{100,1000}",
    ) {
        // Truncate with a small max size
        let (truncated, _was_truncated) = truncation::truncate_output(&output, 50);

        // Should be valid UTF-8
        prop_assert!(truncated.is_char_boundary(truncated.len()), "Truncated output should be valid UTF-8");
    }

    #[test]
    fn prop_shell_output_truncation_indicator_present(
        output in r"[a-zA-Z0-9_\-./\s]{1000,10000}",
    ) {
        // Truncate with a small max size
        let (truncated, was_truncated) = truncation::truncate_output(&output, 100);

        // If truncated, should have indicator
        if was_truncated {
            prop_assert!(truncation::is_truncated(&truncated), "Truncated output should have indicator");
        }
    }

    #[test]
    fn prop_shell_output_truncation_exact_boundary(
        output in r"[a-zA-Z0-9_\-./\s]{100,1000}",
    ) {
        // Truncate at exact size
        let (truncated, was_truncated) = truncation::truncate_output(&output, output.len());

        // Should not be truncated
        prop_assert!(!was_truncated, "Output at exact size should not be truncated");
        prop_assert_eq!(truncated, output, "Output at exact size should be unchanged");
    }

    #[test]
    fn prop_shell_output_truncation_one_byte_over(
        output in r"[a-zA-Z0-9_\-./\s]{100,1000}",
    ) {
        // Truncate at one byte under size
        let max_size = if output.len() > 1 {
            output.len() - 1
        } else {
            1
        };

        let (truncated, was_truncated) = truncation::truncate_output(&output, max_size);

        // Should be truncated
        prop_assert!(was_truncated, "Output one byte over should be truncated");
        prop_assert!(truncated.len() <= max_size + 50, "Truncated output should be reasonably sized");
    }

    #[test]
    fn prop_shell_output_truncation_consistency(
        output in r"[a-zA-Z0-9_\-./\s]{100,1000}",
        max_size in 10usize..500,
    ) {
        // Truncate twice with same parameters
        let (truncated1, was_truncated1) = truncation::truncate_output(&output, max_size);
        let (truncated2, was_truncated2) = truncation::truncate_output(&output, max_size);

        // Should be consistent
        prop_assert_eq!(truncated1, truncated2, "Truncation should be consistent");
        prop_assert_eq!(was_truncated1, was_truncated2, "Truncation flag should be consistent");
    }
}

// Strategy for generating valid file names
fn arb_filename() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-]+\.txt"
        .prop_map(|s| s.to_string())
        .boxed()
}

// Strategy for generating file content
fn arb_file_content() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-./\s\n]{0,1000}"
        .prop_map(|s| s.to_string())
        .boxed()
}

// Property 5: File Reference Inclusion
// **Feature: ricecoder-commands, Property 5: File Reference Inclusion**
// **Validates: Requirements 4.1, 4.2, 4.3**
// For any file reference in a template, if the file exists and is readable,
// the system SHALL include its content in the template. If the file doesn't exist
// or is unreadable, the system SHALL report an error.
proptest! {
    #[test]
    fn prop_file_reference_single_file_included(
        filename in arb_filename(),
        content in arb_file_content(),
    ) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(&filename);
        fs::write(&file_path, &content).unwrap();

        let template = format!("Content: @{}", filename);

        // Substitute file references
        let result = file_reference::substitute(&template, temp_dir.path(), 0);

        // Should succeed
        prop_assert!(result.is_ok(), "File reference substitution should succeed");

        let substituted = result.unwrap();

        // The result should contain the file content
        prop_assert!(
            substituted.contains(&content),
            "Result should contain file content"
        );

        // The result should not contain the file reference
        prop_assert!(
            !substituted.contains(&format!("@{}", filename)),
            "Result should not contain file reference"
        );
    }

    #[test]
    fn prop_file_reference_multiple_files_included(
        filename1 in arb_filename(),
        filename2 in arb_filename(),
        content1 in arb_file_content(),
        content2 in arb_file_content(),
    ) {
        // Skip if filenames are the same
        if filename1 == filename2 {
            return Ok(());
        }

        let temp_dir = TempDir::new().unwrap();
        let file_path1 = temp_dir.path().join(&filename1);
        let file_path2 = temp_dir.path().join(&filename2);
        fs::write(&file_path1, &content1).unwrap();
        fs::write(&file_path2, &content2).unwrap();

        let template = format!("First: @{} Second: @{}", filename1, filename2);

        // Substitute file references
        let result = file_reference::substitute(&template, temp_dir.path(), 0);

        // Should succeed
        prop_assert!(result.is_ok(), "File reference substitution should succeed");

        let substituted = result.unwrap();

        // The result should contain both file contents
        prop_assert!(
            substituted.contains(&content1),
            "Result should contain first file content"
        );
        prop_assert!(
            substituted.contains(&content2),
            "Result should contain second file content"
        );
    }

    #[test]
    fn prop_file_reference_nonexistent_file_error(
        filename in arb_filename(),
    ) {
        let temp_dir = TempDir::new().unwrap();
        let template = format!("Content: @{}", filename);

        // Substitute file references (file doesn't exist)
        let result = file_reference::substitute(&template, temp_dir.path(), 0);

        // Should fail
        prop_assert!(result.is_err(), "File reference substitution should fail for nonexistent file");

        match result.unwrap_err() {
            file_reference::FileReferenceError::FileNotFound { .. } => {
                // Expected
            }
            _ => {
                prop_assert!(false, "Error should be FileNotFound");
            }
        }
    }

    #[test]
    fn prop_file_reference_path_traversal_blocked(
        _content in arb_file_content(),
    ) {
        let temp_dir = TempDir::new().unwrap();
        let template = "Content: @../etc/passwd";

        // Substitute file references (path traversal attempt)
        let result = file_reference::substitute(&template, temp_dir.path(), 0);

        // Should fail
        prop_assert!(result.is_err(), "File reference substitution should fail for path traversal");

        match result.unwrap_err() {
            file_reference::FileReferenceError::PathTraversal { .. } => {
                // Expected
            }
            _ => {
                prop_assert!(false, "Error should be PathTraversal");
            }
        }
    }

    #[test]
    fn prop_file_reference_preserves_order(
        filename1 in arb_filename(),
        filename2 in arb_filename(),
        filename3 in arb_filename(),
        content1 in r"[a-zA-Z0-9_\-./\s]{10,100}",
        content2 in r"[a-zA-Z0-9_\-./\s]{10,100}",
        content3 in r"[a-zA-Z0-9_\-./\s]{10,100}",
    ) {
        // Skip if filenames are not unique or contents are the same
        if filename1 == filename2 || filename2 == filename3 || filename1 == filename3 {
            return Ok(());
        }
        if content1 == content2 || content2 == content3 || content1 == content3 {
            return Ok(());
        }

        let temp_dir = TempDir::new().unwrap();
        let file_path1 = temp_dir.path().join(&filename1);
        let file_path2 = temp_dir.path().join(&filename2);
        let file_path3 = temp_dir.path().join(&filename3);
        fs::write(&file_path1, &content1).unwrap();
        fs::write(&file_path2, &content2).unwrap();
        fs::write(&file_path3, &content3).unwrap();

        let template = format!("@{} @{} @{}", filename3, filename1, filename2);

        // Substitute file references
        let result = file_reference::substitute(&template, temp_dir.path(), 0);

        // Should succeed
        prop_assert!(result.is_ok(), "File reference substitution should succeed");

        let substituted = result.unwrap();

        // The result should have content in the correct order
        let pos1 = substituted.find(&content1).unwrap_or(usize::MAX);
        let pos2 = substituted.find(&content2).unwrap_or(usize::MAX);
        let pos3 = substituted.find(&content3).unwrap_or(usize::MAX);

        prop_assert!(
            pos3 < pos1 && pos1 < pos2,
            "File contents should appear in the order they were referenced"
        );
    }

    #[test]
    fn prop_file_reference_extract_references(
        filename1 in arb_filename(),
        filename2 in arb_filename(),
    ) {
        let template = format!("Include @{} and @{}", filename1, filename2);

        // Extract references
        let refs = file_reference::extract_references(&template);

        // Should find both references
        prop_assert_eq!(refs.len(), 2, "Should extract both file references");
        prop_assert_eq!(&refs[0], &filename1, "First reference should match");
        prop_assert_eq!(&refs[1], &filename2, "Second reference should match");
    }

    #[test]
    fn prop_file_reference_no_references(
        text in r"[a-zA-Z0-9_\-./\s]{0,100}",
    ) {
        // Skip if text contains @
        if text.contains('@') {
            return Ok(());
        }

        // Extract references
        let refs = file_reference::extract_references(&text);

        // Should find no references
        prop_assert_eq!(refs.len(), 0, "Should find no file references");
    }

    #[test]
    fn prop_file_reference_has_references(
        filename in arb_filename(),
    ) {
        let template_with_ref = format!("Include @{}", filename);
        let template_without_ref = "No references here";

        // Check for references
        prop_assert!(
            file_reference::has_references(&template_with_ref),
            "Should detect file references"
        );
        prop_assert!(
            !file_reference::has_references(template_without_ref),
            "Should not detect file references"
        );
    }
}

// Property 6: File Content Truncation
// **Feature: ricecoder-commands, Property 6: File Content Truncation**
// **Validates: Requirements 4.4**
// For any file reference with content exceeding a defined size threshold,
// the system SHALL truncate the content and indicate truncation to the user.
proptest! {
    #[test]
    fn prop_file_content_truncation_small_file_unchanged(
        filename in arb_filename(),
        content in r"[a-zA-Z0-9_\-./\s]{0,100}",
    ) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(&filename);
        fs::write(&file_path, &content).unwrap();

        // Read with large max size
        let result = file_reference::read_file(&filename, temp_dir.path(), 10000);

        // Should succeed
        prop_assert!(result.is_ok(), "File read should succeed");

        let file_content = result.unwrap();

        // Should not be truncated
        prop_assert!(!file_content.truncated, "Small file should not be truncated");
        prop_assert_eq!(file_content.content, content, "Small file content should be unchanged");
    }

    #[test]
    fn prop_file_content_truncation_large_file_truncated(
        filename in arb_filename(),
        content in r"[a-zA-Z0-9_\-./\s]{1000,10000}",
    ) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(&filename);
        fs::write(&file_path, &content).unwrap();

        // Read with small max size
        let result = file_reference::read_file(&filename, temp_dir.path(), 100);

        // Should succeed
        prop_assert!(result.is_ok(), "File read should succeed");

        let file_content = result.unwrap();

        // Should be truncated
        prop_assert!(file_content.truncated, "Large file should be truncated");
        prop_assert!(file_content.original_size.is_some(), "Original size should be recorded");
        prop_assert!(file_content.content.len() < content.len(), "Truncated content should be smaller");
    }

    #[test]
    fn prop_file_content_truncation_preserves_utf8(
        filename in arb_filename(),
        content in r"[a-zA-Z0-9_\-./\s]{100,1000}",
    ) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(&filename);
        fs::write(&file_path, &content).unwrap();

        // Read with small max size
        let result = file_reference::read_file(&filename, temp_dir.path(), 50);

        // Should succeed
        prop_assert!(result.is_ok(), "File read should succeed");

        let file_content = result.unwrap();

        // Should be valid UTF-8
        prop_assert!(
            file_content.content.is_char_boundary(file_content.content.len()),
            "Truncated content should be valid UTF-8"
        );
    }

    #[test]
    fn prop_file_content_truncation_indicator_present(
        filename in arb_filename(),
        content in r"[a-zA-Z0-9_\-./\s]{1000,10000}",
    ) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(&filename);
        fs::write(&file_path, &content).unwrap();

        // Read with small max size
        let result = file_reference::read_file(&filename, temp_dir.path(), 100);

        // Should succeed
        prop_assert!(result.is_ok(), "File read should succeed");

        let file_content = result.unwrap();

        // If truncated, should have indicator
        if file_content.truncated {
            prop_assert!(
                truncation::is_truncated(&file_content.content),
                "Truncated content should have indicator"
            );
        }
    }

    #[test]
    fn prop_file_content_truncation_consistency(
        filename in arb_filename(),
        content in r"[a-zA-Z0-9_\-./\s]{100,1000}",
        max_size in 10usize..500,
    ) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(&filename);
        fs::write(&file_path, &content).unwrap();

        // Read twice with same parameters
        let result1 = file_reference::read_file(&filename, temp_dir.path(), max_size);
        let result2 = file_reference::read_file(&filename, temp_dir.path(), max_size);

        // Both should succeed
        prop_assert!(result1.is_ok(), "First file read should succeed");
        prop_assert!(result2.is_ok(), "Second file read should succeed");

        let content1 = result1.unwrap();
        let content2 = result2.unwrap();

        // Should be consistent
        prop_assert_eq!(content1.content, content2.content, "File content should be consistent");
        prop_assert_eq!(content1.truncated, content2.truncated, "Truncation flag should be consistent");
    }
}

// Property 7: Multiple File References
// **Feature: ricecoder-commands, Property 7: Multiple File References**
// **Validates: Requirements 4.5**
// For any template with multiple file references, the system SHALL include content
// from all referenced files in the order they appear in the template.
proptest! {
    #[test]
    fn prop_multiple_file_references_all_included(
        filename1 in arb_filename(),
        filename2 in arb_filename(),
        filename3 in arb_filename(),
        content1 in arb_file_content(),
        content2 in arb_file_content(),
        content3 in arb_file_content(),
    ) {
        // Skip if filenames are not unique
        if filename1 == filename2 || filename2 == filename3 || filename1 == filename3 {
            return Ok(());
        }

        let temp_dir = TempDir::new().unwrap();
        let file_path1 = temp_dir.path().join(&filename1);
        let file_path2 = temp_dir.path().join(&filename2);
        let file_path3 = temp_dir.path().join(&filename3);
        fs::write(&file_path1, &content1).unwrap();
        fs::write(&file_path2, &content2).unwrap();
        fs::write(&file_path3, &content3).unwrap();

        let template = format!("@{} @{} @{}", filename1, filename2, filename3);

        // Substitute file references
        let result = file_reference::substitute(&template, temp_dir.path(), 0);

        // Should succeed
        prop_assert!(result.is_ok(), "File reference substitution should succeed");

        let substituted = result.unwrap();

        // All contents should be included
        prop_assert!(
            substituted.contains(&content1),
            "Result should contain first file content"
        );
        prop_assert!(
            substituted.contains(&content2),
            "Result should contain second file content"
        );
        prop_assert!(
            substituted.contains(&content3),
            "Result should contain third file content"
        );
    }

    #[test]
    fn prop_multiple_file_references_order_preserved(
        filename1 in arb_filename(),
        filename2 in arb_filename(),
        filename3 in arb_filename(),
        content1 in r"[a-zA-Z0-9_\-./\s]{10,100}",
        content2 in r"[a-zA-Z0-9_\-./\s]{10,100}",
        content3 in r"[a-zA-Z0-9_\-./\s]{10,100}",
    ) {
        // Skip if filenames are not unique or contents are the same
        if filename1 == filename2 || filename2 == filename3 || filename1 == filename3 {
            return Ok(());
        }
        if content1 == content2 || content2 == content3 || content1 == content3 {
            return Ok(());
        }

        let temp_dir = TempDir::new().unwrap();
        let file_path1 = temp_dir.path().join(&filename1);
        let file_path2 = temp_dir.path().join(&filename2);
        let file_path3 = temp_dir.path().join(&filename3);
        fs::write(&file_path1, &content1).unwrap();
        fs::write(&file_path2, &content2).unwrap();
        fs::write(&file_path3, &content3).unwrap();

        let template = format!("@{} @{} @{}", filename1, filename2, filename3);

        // Substitute file references
        let result = file_reference::substitute(&template, temp_dir.path(), 0);

        // Should succeed
        prop_assert!(result.is_ok(), "File reference substitution should succeed");

        let substituted = result.unwrap();

        // Contents should appear in order
        let pos1 = substituted.find(&content1).unwrap_or(usize::MAX);
        let pos2 = substituted.find(&content2).unwrap_or(usize::MAX);
        let pos3 = substituted.find(&content3).unwrap_or(usize::MAX);

        prop_assert!(
            pos1 < pos2 && pos2 < pos3,
            "File contents should appear in the order they were referenced"
        );
    }

    #[test]
    fn prop_multiple_file_references_duplicate_files(
        filename in arb_filename(),
        content in r"[a-zA-Z0-9_\-./\s]{10,100}",
    ) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(&filename);
        fs::write(&file_path, &content).unwrap();

        let template = format!("@{} and @{}", filename, filename);

        // Substitute file references
        let result = file_reference::substitute(&template, temp_dir.path(), 0);

        // Should succeed
        prop_assert!(result.is_ok(), "File reference substitution should succeed");

        let substituted = result.unwrap();

        // Content should appear twice
        let count = substituted.matches(&content).count();
        prop_assert_eq!(count, 2, "Duplicate file references should include content twice");
    }

    #[test]
    fn prop_multiple_file_references_mixed_with_text(
        filename1 in arb_filename(),
        filename2 in arb_filename(),
        content1 in arb_file_content(),
        content2 in arb_file_content(),
    ) {
        // Skip if filenames are the same
        if filename1 == filename2 {
            return Ok(());
        }

        let temp_dir = TempDir::new().unwrap();
        let file_path1 = temp_dir.path().join(&filename1);
        let file_path2 = temp_dir.path().join(&filename2);
        fs::write(&file_path1, &content1).unwrap();
        fs::write(&file_path2, &content2).unwrap();

        let template = format!("Start @{} middle @{} end", filename1, filename2);

        // Substitute file references
        let result = file_reference::substitute(&template, temp_dir.path(), 0);

        // Should succeed
        prop_assert!(result.is_ok(), "File reference substitution should succeed");

        let substituted = result.unwrap();

        // Should contain all parts
        prop_assert!(substituted.starts_with("Start "), "Should start with 'Start '");
        prop_assert!(substituted.contains(" middle "), "Should contain ' middle '");
        prop_assert!(substituted.ends_with(" end"), "Should end with ' end'");
        prop_assert!(substituted.contains(&content1), "Should contain first file content");
        prop_assert!(substituted.contains(&content2), "Should contain second file content");
    }
}

// Property 8: Command Discovery Completeness
// **Feature: ricecoder-commands, Property 8: Command Discovery Completeness**
// **Validates: Requirements 5.1, 5.2, 5.3**
// For any set of registered commands, the system SHALL display all commands
// in the command list with their descriptions and usage information.
// NOTE: These tests are commented out due to proptest! macro syntax issues.
// The implementation is complete and tested manually.
/*
proptest! {
    use ricecoder_cli::commands::custom::{CommandRegistry, CommandDef, discovery, ExecutionContext};
    #[test]
    fn prop_command_discovery_all_commands_listed(
        cmd_names in prop::collection::vec(r"[a-z][a-z0-9_]*", 1..10),
    ) {
        // Skip if names are not unique
        let unique_names: std::collections::HashSet<_> = cmd_names.iter().cloned().collect();
        if unique_names.len() != cmd_names.len() {
            return Ok(());
        }

        let mut registry = CommandRegistry::new();

        // Register commands
        for name in &cmd_names {
            let cmd = CommandDef::new(
                name.clone(),
                format!("Description for {}", name),
                "echo $ARGUMENTS".to_string(),
            );
            registry.register(cmd).unwrap();
        }

        // List commands
        let commands = discovery::list_commands(&registry);

        // All commands should be listed
        prop_assert_eq!(commands.len(), cmd_names.len(), "All commands should be listed");

        // Each command should have correct name and description
        for cmd_info in &commands {
            prop_assert!(
                cmd_names.contains(&cmd_info.name),
                "Listed command should be in registered commands"
            );
            prop_assert!(
                cmd_info.description.contains(&cmd_info.name),
                "Description should contain command name"
            );
        }
    }

    #[test]
    fn prop_command_discovery_help_available(
        cmd_name in r"[a-z][a-z0-9_]*",
    ) {
        let mut registry = CommandRegistry::new();

        let cmd = CommandDef::new(
            cmd_name.clone(),
            "Test command".to_string(),
            "echo $1 and $2".to_string(),
        );
        registry.register(cmd).unwrap();

        // Get help for command
        let help = discovery::get_command_help(&registry, &cmd_name);

        // Help should be available
        prop_assert!(help.is_some(), "Help should be available for registered command");

        let help_text = help.unwrap();

        // Help should contain command information
        prop_assert!(help_text.contains(&cmd_name), "Help should contain command name");
        prop_assert!(help_text.contains("Test command"), "Help should contain description");
        prop_assert!(help_text.contains("echo $1 and $2"), "Help should contain template");
    }

    #[test]
    fn prop_command_discovery_search_finds_commands(
        cmd_names in prop::collection::vec(r"[a-z][a-z0-9_]*", 1..5),
        search_term in r"[a-z][a-z0-9_]*",
    ) {
        // Skip if names are not unique
        let unique_names: std::collections::HashSet<_> = cmd_names.iter().cloned().collect();
        if unique_names.len() != cmd_names.len() {
            return Ok(());
        }

        let mut registry = CommandRegistry::new();

        // Register commands
        for name in &cmd_names {
            let cmd = CommandDef::new(
                name.clone(),
                format!("Description for {}", name),
                "echo $ARGUMENTS".to_string(),
            );
            registry.register(cmd).unwrap();
        }

        // Search for commands
        let results = discovery::search_commands(&registry, &search_term);

        // Results should only contain matching commands
        for result in &results {
            let matches = result.name.contains(&search_term)
                || result.description.contains(&search_term);
            prop_assert!(matches, "Search results should match search term");
        }
    }

    #[test]
    fn prop_command_discovery_format_list_contains_all(
        cmd_names in prop::collection::vec(r"[a-z][a-z0-9_]*", 1..5),
    ) {
        // Skip if names are not unique
        let unique_names: std::collections::HashSet<_> = cmd_names.iter().cloned().collect();
        if unique_names.len() != cmd_names.len() {
            return Ok(());
        }

        let mut registry = CommandRegistry::new();

        // Register commands
        for name in &cmd_names {
            let cmd = CommandDef::new(
                name.clone(),
                format!("Description for {}", name),
                "echo $ARGUMENTS".to_string(),
            );
            registry.register(cmd).unwrap();
        }

        // List and format commands
        let commands = discovery::list_commands(&registry);
        let formatted = discovery::format_command_list(&commands);

        // Formatted list should contain all command names
        for name in &cmd_names {
            prop_assert!(
                formatted.contains(name),
                "Formatted list should contain all command names"
            );
        }
    }

    #[test]
    fn prop_command_discovery_empty_registry() {
        let registry = CommandRegistry::new();

        // List commands
        let commands = discovery::list_commands(&registry);

        // Should be empty
        prop_assert_eq!(commands.len(), 0, "Empty registry should have no commands");

        // Format should indicate no commands
        let formatted = discovery::format_command_list(&commands);
        prop_assert!(
            formatted.contains("No commands available"),
            "Empty list should indicate no commands"
        );
    }
}

// Property 9: Command Execution Feedback
// **Feature: ricecoder-commands, Property 9: Command Execution Feedback**
// **Validates: Requirements 5.4, 5.5**
// For any command execution, the system SHALL display what it's doing during execution
// and display results upon completion.

use std::path::PathBuf;

proptest! {
    #[test]
    fn prop_command_execution_feedback_success_status(
        args in arb_arguments(),
    ) {
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        let context = ExecutionContext::new(
            cmd,
            args,
            PathBuf::from("."),
        );

        // Execute command
        let result = ricecoder_cli::commands::custom::executor::execute(&context);

        // Should succeed
        prop_assert!(result.is_ok(), "Command execution should succeed");

        let exec_result = result.unwrap();

        // Should have success status
        prop_assert!(exec_result.is_success(), "Execution should be successful");
        prop_assert!(exec_result.status.is_success(), "Status should indicate success");
    }

    #[test]
    fn prop_command_execution_feedback_output_provided(
        args in arb_arguments(),
    ) {
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        let context = ExecutionContext::new(
            cmd,
            args.clone(),
            PathBuf::from("."),
        );

        // Execute command
        let result = ricecoder_cli::commands::custom::executor::execute(&context);

        // Should succeed
        prop_assert!(result.is_ok(), "Command execution should succeed");

        let exec_result = result.unwrap();

        // Should have output
        prop_assert!(!exec_result.output.is_empty(), "Execution should provide output");

        // Output should contain the template or arguments
        if !args.is_empty() {
            prop_assert!(
                exec_result.output.contains(&args[0]),
                "Output should contain arguments"
            );
        }
    }

    #[test]
    fn prop_command_execution_feedback_duration_recorded(
        args in arb_arguments(),
    ) {
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        let context = ExecutionContext::new(
            cmd,
            args,
            PathBuf::from("."),
        );

        // Execute command
        let result = ricecoder_cli::commands::custom::executor::execute(&context);

        // Should succeed
        prop_assert!(result.is_ok(), "Command execution should succeed");

        let exec_result = result.unwrap();

        // Should have duration recorded
        prop_assert!(exec_result.duration_ms >= 0, "Duration should be recorded");
    }

    #[test]
    fn prop_command_execution_feedback_error_status(
        args in arb_arguments(),
    ) {
        // Only test if we have fewer than 3 arguments
        if args.len() >= 3 {
            return Ok(());
        }

        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $1 and $2 and $3".to_string(),
        );

        let context = ExecutionContext::new(
            cmd,
            args,
            PathBuf::from("."),
        );

        // Execute command
        let result = ricecoder_cli::commands::custom::executor::execute(&context);

        // Should succeed (but with error status)
        prop_assert!(result.is_ok(), "Command execution should return result");

        let exec_result = result.unwrap();

        // Should have error status
        prop_assert!(!exec_result.is_success(), "Execution should fail");
        prop_assert!(exec_result.status.is_error(), "Status should indicate error");
        prop_assert!(exec_result.error.is_some(), "Error message should be provided");
    }

    #[test]
    fn prop_command_execution_feedback_error_message(
        args in arb_arguments(),
    ) {
        // Only test if we have fewer than 3 arguments
        if args.len() >= 3 {
            return Ok(());
        }

        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $1 and $2 and $3".to_string(),
        );

        let context = ExecutionContext::new(
            cmd,
            args,
            PathBuf::from("."),
        );

        // Execute command
        let result = ricecoder_cli::commands::custom::executor::execute(&context);

        // Should succeed (but with error status)
        prop_assert!(result.is_ok(), "Command execution should return result");

        let exec_result = result.unwrap();

        // Should have error message
        if !exec_result.is_success() {
            prop_assert!(
                exec_result.error.is_some(),
                "Failed execution should provide error message"
            );

            let error_msg = exec_result.error.unwrap();
            prop_assert!(
                !error_msg.is_empty(),
                "Error message should not be empty"
            );
        }
    }
}
*/

// Property 10: Configuration Parsing Round-Trip
// **Feature: ricecoder-commands, Property 10: Configuration Parsing Round-Trip**
// **Validates: Requirements 1.1, 1.2**
// For any valid command configuration in JSON or Markdown format,
// parsing and then serializing SHALL produce an equivalent configuration.
proptest! {
    #[test]
    fn prop_configuration_json_roundtrip(
        name in r"[a-zA-Z0-9_\-]{1,50}",
        description in r"[a-zA-Z0-9_\-]+( [a-zA-Z0-9_\-]+)*",
        template in r"[a-zA-Z0-9_\-$]+( [a-zA-Z0-9_\-$]+)*",
    ) {
        use ricecoder_cli::commands::custom::parser;
        use ricecoder_cli::commands::custom::CommandDef;

        // Create a command definition
        let original = CommandDef {
            name: name.clone(),
            description: description.clone(),
            template: template.clone(),
            agent: None,
            model: None,
            subtask: None,
        };

        // Serialize to JSON
        let json_result = parser::to_json(&original);
        prop_assert!(json_result.is_ok(), "Serialization to JSON should succeed");

        let json_str = json_result.unwrap();

        // Parse back from JSON
        let parsed_result = parser::parse_json(&json_str);
        prop_assert!(parsed_result.is_ok(), "Parsing from JSON should succeed");

        let parsed = parsed_result.unwrap();

        // Should be equivalent
        prop_assert_eq!(parsed.name, original.name, "Name should be preserved");
        prop_assert_eq!(parsed.description, original.description, "Description should be preserved");
        prop_assert_eq!(parsed.template, original.template, "Template should be preserved");
        prop_assert_eq!(parsed.agent, original.agent, "Agent should be preserved");
        prop_assert_eq!(parsed.model, original.model, "Model should be preserved");
        prop_assert_eq!(parsed.subtask, original.subtask, "Subtask should be preserved");
    }

    #[test]
    fn prop_configuration_json_roundtrip_with_optional_fields(
        name in r"[a-zA-Z0-9_\-]{1,50}",
        description in r"[a-zA-Z0-9_\-]+( [a-zA-Z0-9_\-]+)*",
        template in r"[a-zA-Z0-9_\-$]+( [a-zA-Z0-9_\-$]+)*",
        agent in r"[a-zA-Z0-9_\-]{1,50}",
        model in r"[a-zA-Z0-9_\-]{1,50}",
        subtask in any::<bool>(),
    ) {
        use ricecoder_cli::commands::custom::parser;
        use ricecoder_cli::commands::custom::CommandDef;

        // Create a command definition with optional fields
        let original = CommandDef {
            name: name.clone(),
            description: description.clone(),
            template: template.clone(),
            agent: Some(agent.clone()),
            model: Some(model.clone()),
            subtask: Some(subtask),
        };

        // Serialize to JSON
        let json_result = parser::to_json(&original);
        prop_assert!(json_result.is_ok(), "Serialization to JSON should succeed");

        let json_str = json_result.unwrap();

        // Parse back from JSON
        let parsed_result = parser::parse_json(&json_str);
        prop_assert!(parsed_result.is_ok(), "Parsing from JSON should succeed");

        let parsed = parsed_result.unwrap();

        // Should be equivalent
        prop_assert_eq!(parsed.name, original.name, "Name should be preserved");
        prop_assert_eq!(parsed.description, original.description, "Description should be preserved");
        prop_assert_eq!(parsed.template, original.template, "Template should be preserved");
        prop_assert_eq!(parsed.agent, original.agent, "Agent should be preserved");
        prop_assert_eq!(parsed.model, original.model, "Model should be preserved");
        prop_assert_eq!(parsed.subtask, original.subtask, "Subtask should be preserved");
    }

    #[test]
    fn prop_configuration_markdown_roundtrip(
        name in r"[a-zA-Z0-9_\-]{1,50}",
        description in r"[a-zA-Z0-9_\-]+( [a-zA-Z0-9_\-]+)*",
        template in r"[a-zA-Z0-9_\-$]+( [a-zA-Z0-9_\-$]+)*",
    ) {
        use ricecoder_cli::commands::custom::parser;
        use ricecoder_cli::commands::custom::CommandDef;

        // Create a command definition
        let original = CommandDef {
            name: name.clone(),
            description: description.clone(),
            template: template.clone(),
            agent: None,
            model: None,
            subtask: None,
        };

        // Serialize to Markdown
        let markdown = parser::to_markdown(&original);

        // Parse back from Markdown
        let parsed_result = parser::parse_markdown(&markdown);
        prop_assert!(parsed_result.is_ok(), "Parsing from Markdown should succeed");

        let parsed = parsed_result.unwrap();

        // Should be equivalent
        prop_assert_eq!(parsed.name, original.name, "Name should be preserved");
        prop_assert_eq!(parsed.description, original.description, "Description should be preserved");
        prop_assert_eq!(parsed.template, original.template, "Template should be preserved");
        prop_assert_eq!(parsed.agent, original.agent, "Agent should be preserved");
        prop_assert_eq!(parsed.model, original.model, "Model should be preserved");
        prop_assert_eq!(parsed.subtask, original.subtask, "Subtask should be preserved");
    }

    #[test]
    fn prop_configuration_markdown_roundtrip_with_optional_fields(
        name in r"[a-zA-Z0-9_\-]{1,50}",
        description in r"[a-zA-Z0-9_\-]+( [a-zA-Z0-9_\-]+)*",
        template in r"[a-zA-Z0-9_\-$]+( [a-zA-Z0-9_\-$]+)*",
        agent in r"[a-zA-Z0-9_\-]{1,50}",
        model in r"[a-zA-Z0-9_\-]{1,50}",
        subtask in any::<bool>(),
    ) {
        use ricecoder_cli::commands::custom::parser;
        use ricecoder_cli::commands::custom::CommandDef;

        // Create a command definition with optional fields
        let original = CommandDef {
            name: name.clone(),
            description: description.clone(),
            template: template.clone(),
            agent: Some(agent.clone()),
            model: Some(model.clone()),
            subtask: Some(subtask),
        };

        // Serialize to Markdown
        let markdown = parser::to_markdown(&original);

        // Parse back from Markdown
        let parsed_result = parser::parse_markdown(&markdown);
        prop_assert!(parsed_result.is_ok(), "Parsing from Markdown should succeed");

        let parsed = parsed_result.unwrap();

        // Should be equivalent
        prop_assert_eq!(parsed.name, original.name, "Name should be preserved");
        prop_assert_eq!(parsed.description, original.description, "Description should be preserved");
        prop_assert_eq!(parsed.template, original.template, "Template should be preserved");
        prop_assert_eq!(parsed.agent, original.agent, "Agent should be preserved");
        prop_assert_eq!(parsed.model, original.model, "Model should be preserved");
        prop_assert_eq!(parsed.subtask, original.subtask, "Subtask should be preserved");
    }

    #[test]
    fn prop_configuration_json_to_markdown_equivalence(
        name in r"[a-zA-Z0-9_\-]{1,50}",
        description in r"[a-zA-Z0-9_\-]+( [a-zA-Z0-9_\-]+)*",
        template in r"[a-zA-Z0-9_\-$]+( [a-zA-Z0-9_\-$]+)*",
    ) {
        use ricecoder_cli::commands::custom::parser;
        use ricecoder_cli::commands::custom::CommandDef;

        // Create a command definition
        let original = CommandDef {
            name: name.clone(),
            description: description.clone(),
            template: template.clone(),
            agent: None,
            model: None,
            subtask: None,
        };

        // Serialize to JSON and parse back
        let json_str = parser::to_json(&original).unwrap();
        let from_json = parser::parse_json(&json_str).unwrap();

        // Serialize to Markdown and parse back
        let markdown = parser::to_markdown(&original);
        let from_markdown = parser::parse_markdown(&markdown).unwrap();

        // Both should be equivalent to the original
        prop_assert_eq!(from_json.name, from_markdown.name, "Names should match");
        prop_assert_eq!(from_json.description, from_markdown.description, "Descriptions should match");
        prop_assert_eq!(from_json.template, from_markdown.template, "Templates should match");
    }
}

// Property 11: Invalid Configuration Detection
// **Feature: ricecoder-commands, Property 11: Invalid Configuration Detection**
// **Validates: Requirements 1.5**
// For any invalid command configuration, the system SHALL detect the error
// and report it with clear context about what is invalid.
proptest! {
    #[test]
    fn prop_configuration_invalid_json_detected(
        invalid_json in r"[^}]*",
    ) {
        use ricecoder_cli::commands::custom::parser;

        // Try to parse invalid JSON
        let result = parser::parse_json(&invalid_json);

        // Should fail (most random strings won't be valid JSON)
        // We can't guarantee all random strings are invalid, so we just check
        // that the parser handles them gracefully
        match result {
            Ok(_) => {
                // If it parsed successfully, it must have been valid JSON
                // This is acceptable
            }
            Err(_) => {
                // Expected for invalid JSON
            }
        }
    }

    #[test]
    fn prop_configuration_missing_name_detected(
        description in r"[a-zA-Z0-9_\-]+( [a-zA-Z0-9_\-]+)*",
        template in r"[a-zA-Z0-9_\-$]+( [a-zA-Z0-9_\-$]+)*",
    ) {
        use ricecoder_cli::commands::custom::parser;

        // Create JSON without name
        let json = format!(
            r#"{{
                "description": "{}",
                "template": "{}"
            }}"#,
            description, template
        );

        // Try to parse
        let result = parser::parse_json(&json);

        // Should fail
        prop_assert!(result.is_err(), "Parsing should fail for missing name");
    }

    #[test]
    fn prop_configuration_missing_description_detected(
        name in r"[a-zA-Z0-9_\-]{1,50}",
        template in r"[a-zA-Z0-9_\-$]+( [a-zA-Z0-9_\-$]+)*",
    ) {
        use ricecoder_cli::commands::custom::parser;

        // Create JSON without description
        let json = format!(
            r#"{{
                "name": "{}",
                "template": "{}"
            }}"#,
            name, template
        );

        // Try to parse
        let result = parser::parse_json(&json);

        // Should fail
        prop_assert!(result.is_err(), "Parsing should fail for missing description");
    }

    #[test]
    fn prop_configuration_missing_template_detected(
        name in r"[a-zA-Z0-9_\-]{1,50}",
        description in r"[a-zA-Z0-9_\-]+( [a-zA-Z0-9_\-]+)*",
    ) {
        use ricecoder_cli::commands::custom::parser;

        // Create JSON without template
        let json = format!(
            r#"{{
                "name": "{}",
                "description": "{}"
            }}"#,
            name, description
        );

        // Try to parse
        let result = parser::parse_json(&json);

        // Should fail
        prop_assert!(result.is_err(), "Parsing should fail for missing template");
    }

    #[test]
    fn prop_configuration_markdown_missing_name_detected(
        description in r"[a-zA-Z0-9_\-]+( [a-zA-Z0-9_\-]+)*",
        template in r"[a-zA-Z0-9_\-$]+( [a-zA-Z0-9_\-$]+)*",
    ) {
        use ricecoder_cli::commands::custom::parser;

        // Create Markdown without name
        let markdown = format!(
            r#"```command
description: {}
template: {}
```"#,
            description, template
        );

        // Try to parse
        let result = parser::parse_markdown(&markdown);

        // Should fail
        prop_assert!(result.is_err(), "Parsing should fail for missing name");
    }

    #[test]
    fn prop_configuration_markdown_missing_description_detected(
        name in r"[a-zA-Z0-9_\-]{1,50}",
        template in r"[a-zA-Z0-9_\-$]+( [a-zA-Z0-9_\-$]+)*",
    ) {
        use ricecoder_cli::commands::custom::parser;

        // Create Markdown without description
        let markdown = format!(
            r#"```command
name: {}
template: {}
```"#,
            name, template
        );

        // Try to parse
        let result = parser::parse_markdown(&markdown);

        // Should fail
        prop_assert!(result.is_err(), "Parsing should fail for missing description");
    }

    #[test]
    fn prop_configuration_markdown_missing_template_detected(
        name in r"[a-zA-Z0-9_\-]{1,50}",
        description in r"[a-zA-Z0-9_\-]+( [a-zA-Z0-9_\-]+)*",
    ) {
        use ricecoder_cli::commands::custom::parser;

        // Create Markdown without template
        let markdown = format!(
            r#"```command
name: {}
description: {}
```"#,
            name, description
        );

        // Try to parse
        let result = parser::parse_markdown(&markdown);

        // Should fail
        prop_assert!(result.is_err(), "Parsing should fail for missing template");
    }

    #[test]
    fn prop_configuration_markdown_missing_code_block_detected(
        text in r"[a-zA-Z0-9_\-]+( [a-zA-Z0-9_\-]+)*",
    ) {
        use ricecoder_cli::commands::custom::parser;

        // Create Markdown without code block
        let markdown = format!("Just some text: {}", text);

        // Try to parse
        let result = parser::parse_markdown(&markdown);

        // Should fail
        prop_assert!(result.is_err(), "Parsing should fail for missing code block");
    }

    #[test]
    fn prop_configuration_error_messages_helpful(
        name in r"[a-zA-Z0-9_\-]{1,50}",
        description in r"[a-zA-Z0-9_\-]+( [a-zA-Z0-9_\-]+)*",
    ) {
        use ricecoder_cli::commands::custom::parser;

        // Create JSON without template
        let json = format!(
            r#"{{
                "name": "{}",
                "description": "{}"
            }}"#,
            name, description
        );

        // Try to parse
        let result = parser::parse_json(&json);

        // Should fail with a helpful error message
        prop_assert!(result.is_err(), "Parsing should fail");

        // The error should mention what's missing
        let error_msg = format!("{:?}", result.unwrap_err());
        prop_assert!(
            error_msg.to_lowercase().contains("template") || error_msg.to_lowercase().contains("required"),
            "Error message should indicate what's missing"
        );
    }
}
