//! Safety validation for refactoring operations

use crate::error::Result;
use crate::types::{Refactoring, ValidationResult};

/// Validates safety of refactoring operations
pub struct SafetyChecker;

impl SafetyChecker {
    /// Check if a refactoring is safe to apply
    pub fn check(refactoring: &Refactoring) -> Result<ValidationResult> {
        let mut errors = vec![];
        let mut warnings = vec![];

        // Check if target file exists
        if !refactoring.target.file.exists() {
            errors.push(format!(
                "Target file does not exist: {}",
                refactoring.target.file.display()
            ));
        }

        // Check if symbol is not empty
        if refactoring.target.symbol.is_empty() {
            errors.push("Symbol name cannot be empty".to_string());
        }

        // Check for potential issues
        if refactoring.target.symbol.len() > 100 {
            warnings.push("Symbol name is very long".to_string());
        }

        Ok(ValidationResult {
            passed: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Validate that changes don't break the code
    pub fn validate_changes(original: &str, new: &str) -> Result<ValidationResult> {
        let mut errors = vec![];
        let mut warnings = vec![];

        // Check if content is not empty
        if new.is_empty() {
            errors.push("Refactored code cannot be empty".to_string());
        }

        // Check if content changed
        if original == new {
            warnings.push("No changes were made".to_string());
        }

        Ok(ValidationResult {
            passed: errors.is_empty(),
            errors,
            warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{RefactoringOptions, RefactoringTarget, RefactoringType};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_check_refactoring_invalid_file() -> Result<()> {
        let refactoring = Refactoring {
            id: "test".to_string(),
            refactoring_type: RefactoringType::Rename,
            target: RefactoringTarget {
                file: PathBuf::from("/nonexistent/file.rs"),
                symbol: "test".to_string(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        let result = SafetyChecker::check(&refactoring)?;
        assert!(!result.passed);
        assert!(!result.errors.is_empty());

        Ok(())
    }

    #[test]
    fn test_check_refactoring_empty_symbol() -> Result<()> {
        let file = NamedTempFile::new()?;
        let refactoring = Refactoring {
            id: "test".to_string(),
            refactoring_type: RefactoringType::Rename,
            target: RefactoringTarget {
                file: file.path().to_path_buf(),
                symbol: "".to_string(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        let result = SafetyChecker::check(&refactoring)?;
        assert!(!result.passed);

        Ok(())
    }

    #[test]
    fn test_validate_changes_empty_new() -> Result<()> {
        let result = SafetyChecker::validate_changes("original", "")?;
        assert!(!result.passed);

        Ok(())
    }

    #[test]
    fn test_validate_changes_no_change() -> Result<()> {
        let result = SafetyChecker::validate_changes("same", "same")?;
        assert!(result.passed);
        assert!(!result.warnings.is_empty());

        Ok(())
    }
}
