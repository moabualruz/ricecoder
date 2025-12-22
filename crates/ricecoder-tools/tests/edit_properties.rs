//! Property tests for file edit strategy correctness
//!
//! This module contains property-based tests that validate the correctness
//! of file editing strategies, ensuring they work correctly across various
//! inputs and edge cases.

use std::fs;

use proptest::prelude::*;
use ricecoder_tools::edit::{FileEditInput, FileEditTool};
use tempfile::NamedTempFile;

/// Property 4: File Edit Strategy Correctness
/// Validates: Requirements 20.1, 20.2
///
/// Ensures that file edit strategies correctly apply changes and generate valid diffs.
proptest! {
    #[test]
    fn prop_file_edit_strategies_correctness(
        old_content in ".{1,1000}",
        new_content in ".{1,1000}",
        file_name in "[a-zA-Z0-9_]{1,20}.txt"
    ) {
        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Write initial content
        fs::write(&temp_file, &old_content).unwrap();

        // Create edit input
        let input = FileEditInput {
            file_path: file_path.clone(),
            old_string: old_content.clone(),
            new_string: new_content.clone(),
            start_line: None,
            end_line: None,
        };

        // Apply edit
        let result = FileEditTool::edit_file(&input);

        // The operation should not panic
        prop_assert!(result.is_ok());

        let output = result.unwrap();

        // If successful, should have used a strategy
        if output.success {
            prop_assert!(output.strategy_used.is_some());
            prop_assert!(output.diff.is_some());
            prop_assert!(output.error.is_none());

            // Should have attempted at least one strategy
            prop_assert!(!output.strategies_attempted.is_empty());

            // The strategy used should be in the attempted list
            prop_assert!(output.strategies_attempted.contains(&output.strategy_used.as_ref().unwrap()));
        } else {
            // If failed, should have error message
            prop_assert!(output.error.is_some() || output.closest_match.is_some());
        }

        // Should always have attempted strategies
        prop_assert!(!output.strategies_attempted.is_empty());

        // Strategies attempted should be valid strategy names
        let valid_strategies = vec![
            "Simple", "Line-trimmed", "Block-anchor", "Indent-normalized",
            "Whitespace-normalized", "Levenshtein", "Line-by-line", "Regex", "AST-based"
        ];

        for strategy in &output.strategies_attempted {
            prop_assert!(valid_strategies.contains(&strategy.as_str()));
        }
    }

    #[test]
    fn prop_file_edit_preserves_file_integrity(
        initial_content in ".{1,500}",
        edit_content in ".{1,200}",
        replacement in ".{1,200}"
    ) {
        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Write initial content
        fs::write(&temp_file, &initial_content).unwrap();

        // Create edit input that should match
        let input = FileEditInput {
            file_path: file_path.clone(),
            old_string: edit_content.clone(),
            new_string: replacement.clone(),
            start_line: None,
            end_line: None,
        };

        // Apply edit
        let result = FileEditTool::edit_file(&input);

        // Should not panic
        prop_assert!(result.is_ok());

        // File should still exist and be readable
        let final_content = fs::read_to_string(&temp_file);
        prop_assert!(final_content.is_ok());
    }

    #[test]
    fn prop_file_edit_strategy_order_matters(
        content in ".{1,300}",
        target in ".{1,50}",
        replacement in ".{1,50}"
    ) {
        // Only test if the target actually exists in content
        prop_assume!(content.contains(&target));

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Write content
        fs::write(&temp_file, &content).unwrap();

        let input = FileEditInput {
            file_path: file_path.clone(),
            old_string: target.clone(),
            new_string: replacement.clone(),
            start_line: None,
            end_line: None,
        };

        let result = FileEditTool::edit_file(&input);

        // Should not panic
        prop_assert!(result.is_ok());

        let output = result.unwrap();

        // If successful, the first strategy that works should be used
        if output.success {
            let strategy_used = output.strategy_used.as_ref().unwrap();

            // Simple strategy should be tried first and should work for exact matches
            if content.contains(&target) {
                // For exact matches, Simple should work
                prop_assert!(output.strategies_attempted[0] == "Simple");
            }
        }
    }

    #[test]
    fn prop_file_edit_handles_edge_cases(
        empty_old in "",
        empty_new in "",
        whitespace_old in "[\t\n\r ]{1,20}",
        unicode_old in "[\\u{0080}-\\u{FFFF}]{1,10}",
        unicode_new in "[\\u{0080}-\\u{FFFF}]{1,10}"
    ) {
        // Test empty strings
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        fs::write(&temp_file, "test content").unwrap();

        let input = FileEditInput {
            file_path: file_path.clone(),
            old_string: empty_old.clone(),
            new_string: empty_new.clone(),
            start_line: None,
            end_line: None,
        };

        let result = FileEditTool::edit_file(&input);
        // Should not panic even with empty strings
        prop_assert!(result.is_ok());

        // Test whitespace
        let input2 = FileEditInput {
            file_path: file_path.clone(),
            old_string: whitespace_old.clone(),
            new_string: "replacement".to_string(),
            start_line: None,
            end_line: None,
        };

        let result2 = FileEditTool::edit_file(&input2);
        prop_assert!(result2.is_ok());

        // Test unicode
        let input3 = FileEditInput {
            file_path,
            old_string: unicode_old,
            new_string: unicode_new,
            start_line: None,
            end_line: None,
        };

        let result3 = FileEditTool::edit_file(&input3);
        prop_assert!(result3.is_ok());
    }
}
