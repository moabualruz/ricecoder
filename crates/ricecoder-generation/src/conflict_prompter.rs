//! User prompting for conflict resolution
//!
//! Handles interactive prompting for users to choose conflict resolution strategies.
//! Implements requirement:
//! - Requirement 4.5: Prompt user to choose strategy for each conflict

use crate::conflict_detector::FileConflictInfo;
use crate::conflict_resolver::ConflictStrategy;
use crate::error::GenerationError;
use std::io::{self, BufRead, Write};

/// Prompts user for conflict resolution decisions
///
/// Implements requirement:
/// - Requirement 4.5: Prompt user to choose strategy for each conflict
pub struct ConflictPrompter;

/// Result of user prompting for a conflict
#[derive(Debug, Clone)]
pub struct PromptResult {
    /// Strategy chosen by user
    pub strategy: ConflictStrategy,
    /// Whether to apply to all remaining conflicts
    pub apply_to_all: bool,
}

impl ConflictPrompter {
    /// Create a new conflict prompter
    pub fn new() -> Self {
        Self
    }

    /// Prompt user for a single conflict
    ///
    /// Shows the conflict details and asks user to choose a resolution strategy.
    ///
    /// # Arguments
    /// * `conflict` - Conflict to prompt for
    /// * `conflict_number` - Number of this conflict (for display)
    /// * `total_conflicts` - Total number of conflicts
    ///
    /// # Returns
    /// User's choice of strategy
    ///
    /// # Requirements
    /// - Requirement 4.5: Prompt user to choose strategy for each conflict
    pub fn prompt_for_conflict(
        &self,
        conflict: &FileConflictInfo,
        conflict_number: usize,
        total_conflicts: usize,
    ) -> Result<PromptResult, GenerationError> {
        // Display conflict information
        self.display_conflict(conflict, conflict_number, total_conflicts)?;

        // Get user input
        loop {
            self.display_options()?;
            let input = self.read_user_input()?;
            let input = input.trim().to_lowercase();

            match input.as_str() {
                "s" | "skip" => {
                    let apply_to_all = self.prompt_apply_to_all()?;
                    return Ok(PromptResult {
                        strategy: ConflictStrategy::Skip,
                        apply_to_all,
                    });
                }
                "o" | "overwrite" => {
                    let apply_to_all = self.prompt_apply_to_all()?;
                    return Ok(PromptResult {
                        strategy: ConflictStrategy::Overwrite,
                        apply_to_all,
                    });
                }
                "m" | "merge" => {
                    let apply_to_all = self.prompt_apply_to_all()?;
                    return Ok(PromptResult {
                        strategy: ConflictStrategy::Merge,
                        apply_to_all,
                    });
                }
                "d" | "diff" => {
                    self.display_diff(conflict)?;
                    // Loop back to show options again
                }
                "q" | "quit" => {
                    return Err(GenerationError::ValidationError {
                        file: conflict.path.to_string_lossy().to_string(),
                        line: 0,
                        message: "User cancelled conflict resolution".to_string(),
                    });
                }
                _ => {
                    println!("Invalid choice. Please try again.");
                }
            }
        }
    }

    /// Prompt user for multiple conflicts
    ///
    /// Iterates through conflicts and prompts for each one, with option to apply
    /// same strategy to all remaining conflicts.
    ///
    /// # Arguments
    /// * `conflicts` - List of conflicts to prompt for
    ///
    /// # Returns
    /// Map of file paths to chosen strategies
    pub fn prompt_for_conflicts(
        &self,
        conflicts: &[FileConflictInfo],
    ) -> Result<Vec<(String, ConflictStrategy)>, GenerationError> {
        let mut results = Vec::new();
        let mut apply_to_all_strategy: Option<ConflictStrategy> = None;

        for (i, conflict) in conflicts.iter().enumerate() {
            let conflict_num = i + 1;

            // If user chose "apply to all", use that strategy
            if let Some(strategy) = apply_to_all_strategy {
                results.push((conflict.path.to_string_lossy().to_string(), strategy));
                continue;
            }

            // Otherwise, prompt for this conflict
            let prompt_result = self.prompt_for_conflict(conflict, conflict_num, conflicts.len())?;

            results.push((
                conflict.path.to_string_lossy().to_string(),
                prompt_result.strategy,
            ));

            // If user chose "apply to all", remember the strategy
            if prompt_result.apply_to_all {
                apply_to_all_strategy = Some(prompt_result.strategy);
            }
        }

        Ok(results)
    }

    /// Display conflict information
    ///
    /// Shows the file path, diff summary, and conflict details.
    fn display_conflict(
        &self,
        conflict: &FileConflictInfo,
        conflict_number: usize,
        total_conflicts: usize,
    ) -> Result<(), GenerationError> {
        println!("\n{}", "=".repeat(70));
        println!(
            "Conflict {}/{}: {}",
            conflict_number,
            total_conflicts,
            conflict.path.display()
        );
        println!("{}", "=".repeat(70));

        println!("\nConflict Summary:");
        println!(
            "  Added lines: {}",
            conflict.diff.added_lines.len()
        );
        println!(
            "  Removed lines: {}",
            conflict.diff.removed_lines.len()
        );
        println!(
            "  Modified lines: {}",
            conflict.diff.modified_lines.len()
        );

        println!("\nFile sizes:");
        println!("  Original: {} bytes", conflict.old_content.len());
        println!("  Generated: {} bytes", conflict.new_content.len());

        Ok(())
    }

    /// Display resolution options
    fn display_options(&self) -> Result<(), GenerationError> {
        println!("\nResolution options:");
        println!("  (s)kip      - Don't write this file");
        println!("  (o)verwrite - Write new file (backup original)");
        println!("  (m)erge     - Merge changes (mark conflicts)");
        println!("  (d)iff      - Show detailed diff");
        println!("  (q)uit      - Cancel generation");
        print!("\nChoose option: ");
        io::stdout().flush().ok();

        Ok(())
    }

    /// Display detailed diff
    fn display_diff(&self, conflict: &FileConflictInfo) -> Result<(), GenerationError> {
        println!("\n{}", "-".repeat(70));
        println!("Detailed Diff:");
        println!("{}", "-".repeat(70));

        if !conflict.diff.removed_lines.is_empty() {
            println!("\nRemoved lines:");
            for line in &conflict.diff.removed_lines {
                println!("  - [{}] {}", line.line_number, line.content);
            }
        }

        if !conflict.diff.added_lines.is_empty() {
            println!("\nAdded lines:");
            for line in &conflict.diff.added_lines {
                println!("  + [{}] {}", line.line_number, line.content);
            }
        }

        if !conflict.diff.modified_lines.is_empty() {
            println!("\nModified lines:");
            for (old, new) in &conflict.diff.modified_lines {
                println!("  - [{}] {}", old.line_number, old.content);
                println!("  + [{}] {}", new.line_number, new.content);
            }
        }

        println!("{}", "-".repeat(70));

        Ok(())
    }

    /// Prompt user if they want to apply strategy to all remaining conflicts
    fn prompt_apply_to_all(&self) -> Result<bool, GenerationError> {
        print!("\nApply this choice to all remaining conflicts? (y/n): ");
        io::stdout().flush().ok();

        let input = self.read_user_input()?;
        let input = input.trim().to_lowercase();

        Ok(input == "y" || input == "yes")
    }

    /// Read user input from stdin
    fn read_user_input(&self) -> Result<String, GenerationError> {
        let stdin = io::stdin();
        let mut line = String::new();
        stdin
            .lock()
            .read_line(&mut line)
            .map_err(|e| GenerationError::ValidationError {
                file: "stdin".to_string(),
                line: 0,
                message: format!("Failed to read user input: {}", e),
            })?;

        Ok(line)
    }

    /// Display a summary of all conflicts
    ///
    /// # Arguments
    /// * `conflicts` - List of conflicts
    pub fn display_summary(&self, conflicts: &[FileConflictInfo]) -> Result<(), GenerationError> {
        println!("\n{}", "=".repeat(70));
        println!("Conflict Summary");
        println!("{}", "=".repeat(70));
        println!("Total conflicts: {}", conflicts.len());

        for (i, conflict) in conflicts.iter().enumerate() {
            println!(
                "  {}. {} ({} changes)",
                i + 1,
                conflict.path.display(),
                conflict.diff.total_changes
            );
        }

        println!("{}", "=".repeat(70));

        Ok(())
    }
}

impl Default for ConflictPrompter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_conflict() -> FileConflictInfo {
        FileConflictInfo {
            path: PathBuf::from("test.rs"),
            old_content: "old content".to_string(),
            new_content: "new content".to_string(),
            diff: crate::conflict_detector::FileDiff {
                added_lines: vec![crate::conflict_detector::DiffLine {
                    line_number: 1,
                    content: "added".to_string(),
                }],
                removed_lines: vec![crate::conflict_detector::DiffLine {
                    line_number: 2,
                    content: "removed".to_string(),
                }],
                modified_lines: vec![],
                total_changes: 2,
            },
        }
    }

    #[test]
    fn test_create_prompter() {
        let _prompter = ConflictPrompter::new();
    }

    #[test]
    fn test_display_conflict() {
        let prompter = ConflictPrompter::new();
        let conflict = create_test_conflict();

        // This should not panic
        let result = prompter.display_conflict(&conflict, 1, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_options() {
        let prompter = ConflictPrompter::new();

        // This should not panic
        let result = prompter.display_options();
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_diff() {
        let prompter = ConflictPrompter::new();
        let conflict = create_test_conflict();

        // This should not panic
        let result = prompter.display_diff(&conflict);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_summary() {
        let prompter = ConflictPrompter::new();
        let conflicts = vec![create_test_conflict(), create_test_conflict()];

        // This should not panic
        let result = prompter.display_summary(&conflicts);
        assert!(result.is_ok());
    }
}
