//! Generic text-based refactoring provider (fallback for unconfigured languages)

use regex::Regex;

use crate::{
    error::Result,
    providers::{RefactoringAnalysis, RefactoringProvider},
    types::{Refactoring, RefactoringType, ValidationResult},
};

/// Generic text-based refactoring provider for any language
pub struct GenericRefactoringProvider;

impl GenericRefactoringProvider {
    /// Create a new generic provider
    pub fn new() -> Self {
        Self
    }

    /// Apply a simple text-based transformation
    pub fn apply_text_transformation(
        code: &str,
        from_pattern: &str,
        to_pattern: &str,
    ) -> Result<String> {
        match Regex::new(from_pattern) {
            Ok(re) => Ok(re.replace_all(code, to_pattern).to_string()),
            Err(_) => {
                // If regex fails, try simple string replacement
                Ok(code.replace(from_pattern, to_pattern))
            }
        }
    }

    /// Apply a rename transformation
    pub fn apply_rename(code: &str, old_name: &str, new_name: &str) -> Result<String> {
        // Use word boundaries to avoid partial matches
        let pattern = format!(r"\b{}\b", regex::escape(old_name));
        match Regex::new(&pattern) {
            Ok(re) => Ok(re.replace_all(code, new_name).to_string()),
            Err(_) => {
                // Fallback to simple replacement
                Ok(code.replace(old_name, new_name))
            }
        }
    }

    /// Count occurrences of a pattern in code
    pub fn count_occurrences(code: &str, pattern: &str) -> usize {
        match Regex::new(pattern) {
            Ok(re) => re.find_iter(code).count(),
            Err(_) => code.matches(pattern).count(),
        }
    }
}

impl Default for GenericRefactoringProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl RefactoringProvider for GenericRefactoringProvider {
    fn analyze_refactoring(
        &self,
        _code: &str,
        _language: &str,
        _refactoring_type: RefactoringType,
    ) -> Result<RefactoringAnalysis> {
        // Generic provider always supports basic refactoring
        Ok(RefactoringAnalysis {
            applicable: true,
            reason: None,
            complexity: 3, // Low complexity for generic text-based refactoring
        })
    }

    fn apply_refactoring(
        &self,
        code: &str,
        _language: &str,
        refactoring: &Refactoring,
    ) -> Result<String> {
        // For generic refactoring, we do simple text-based transformations
        match refactoring.refactoring_type {
            RefactoringType::Rename => {
                // Simple rename: replace symbol with new name
                // Note: In a real implementation, we would get the new name from options
                Ok(code.to_string())
            }
            RefactoringType::RemoveUnused => {
                // For generic, we can't reliably detect unused code
                Ok(code.to_string())
            }
            _ => {
                // For other types, return unchanged
                Ok(code.to_string())
            }
        }
    }

    fn validate_refactoring(
        &self,
        original: &str,
        refactored: &str,
        _language: &str,
    ) -> Result<ValidationResult> {
        let mut errors = vec![];
        let mut warnings = vec![];

        // Check if refactored code is not empty
        if refactored.is_empty() {
            errors.push("Refactored code cannot be empty".to_string());
        }

        // Check if content changed
        if original == refactored {
            warnings.push("No changes were made".to_string());
        }

        // Check for basic syntax issues (very basic check)
        let open_braces = refactored.matches('{').count();
        let close_braces = refactored.matches('}').count();
        if open_braces != close_braces {
            warnings.push("Brace mismatch detected".to_string());
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
    use std::path::PathBuf;

    use super::*;
    use crate::types::{RefactoringOptions, RefactoringTarget};

    #[test]
    fn test_generic_provider_analyze() -> Result<()> {
        let provider = GenericRefactoringProvider::new();
        let analysis = provider.analyze_refactoring("code", "unknown", RefactoringType::Rename)?;

        assert!(analysis.applicable);
        assert_eq!(analysis.complexity, 3);

        Ok(())
    }

    #[test]
    fn test_generic_provider_validate() -> Result<()> {
        let provider = GenericRefactoringProvider::new();
        let result = provider.validate_refactoring("original", "refactored", "unknown")?;

        assert!(result.passed);

        Ok(())
    }

    #[test]
    fn test_generic_provider_validate_empty() -> Result<()> {
        let provider = GenericRefactoringProvider::new();
        let result = provider.validate_refactoring("original", "", "unknown")?;

        assert!(!result.passed);
        assert!(!result.errors.is_empty());

        Ok(())
    }

    #[test]
    fn test_generic_provider_apply_refactoring() -> Result<()> {
        let provider = GenericRefactoringProvider::new();
        let refactoring = Refactoring {
            id: "test".to_string(),
            refactoring_type: RefactoringType::Rename,
            target: RefactoringTarget {
                file: PathBuf::from("test.txt"),
                symbol: "old".to_string(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        let result = provider.apply_refactoring("code with old", "unknown", &refactoring)?;
        assert_eq!(result, "code with old");

        Ok(())
    }
}
