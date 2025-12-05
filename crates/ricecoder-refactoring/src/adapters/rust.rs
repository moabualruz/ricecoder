//! Rust-specific refactoring provider

use crate::error::Result;
use crate::providers::{RefactoringAnalysis, RefactoringProvider};
use crate::types::{Refactoring, RefactoringType, ValidationResult};
use regex::Regex;

/// Rust-specific refactoring provider
pub struct RustRefactoringProvider;

impl RustRefactoringProvider {
    /// Create a new Rust provider
    pub fn new() -> Self {
        Self
    }

    /// Check if code is valid Rust
    fn is_valid_rust(code: &str) -> bool {
        // Basic checks for Rust syntax
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        let open_parens = code.matches('(').count();
        let close_parens = code.matches(')').count();
        let open_brackets = code.matches('[').count();
        let close_brackets = code.matches(']').count();

        open_braces == close_braces
            && open_parens == close_parens
            && open_brackets == close_brackets
    }

    /// Apply a Rust-specific rename with word boundaries
    pub fn apply_rust_rename(code: &str, old_name: &str, new_name: &str) -> Result<String> {
        let pattern = format!(r"\b{}\b", regex::escape(old_name));
        match Regex::new(&pattern) {
            Ok(re) => Ok(re.replace_all(code, new_name).to_string()),
            Err(_) => Ok(code.replace(old_name, new_name)),
        }
    }

    /// Check for unsafe code
    pub fn has_unsafe_code(code: &str) -> bool {
        code.contains("unsafe")
    }

    /// Check for common Rust patterns
    pub fn check_rust_patterns(code: &str) -> Vec<String> {
        let mut issues = vec![];

        if code.contains("unwrap()") {
            issues.push("Code uses unwrap() which can panic".to_string());
        }

        if code.contains("panic!") {
            issues.push("Code uses panic! macro".to_string());
        }

        if code.contains("todo!") {
            issues.push("Code contains unimplemented todo!".to_string());
        }

        issues
    }
}

impl Default for RustRefactoringProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl RefactoringProvider for RustRefactoringProvider {
    fn analyze_refactoring(
        &self,
        _code: &str,
        _language: &str,
        refactoring_type: RefactoringType,
    ) -> Result<RefactoringAnalysis> {
        let complexity = match refactoring_type {
            RefactoringType::Rename => 4,
            RefactoringType::Extract => 7,
            RefactoringType::Inline => 6,
            RefactoringType::Move => 8,
            RefactoringType::ChangeSignature => 9,
            RefactoringType::RemoveUnused => 5,
            RefactoringType::Simplify => 6,
        };

        Ok(RefactoringAnalysis {
            applicable: true,
            reason: None,
            complexity,
        })
    }

    fn apply_refactoring(
        &self,
        code: &str,
        _language: &str,
        refactoring: &Refactoring,
    ) -> Result<String> {
        // Apply Rust-specific refactoring
        match refactoring.refactoring_type {
            RefactoringType::Rename => {
                // Rust rename: use word boundaries to avoid partial matches
                Self::apply_rust_rename(code, &refactoring.target.symbol, &refactoring.target.symbol)
            }
            _ => Ok(code.to_string()),
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

        // Check Rust syntax validity
        if !Self::is_valid_rust(refactored) {
            errors.push("Refactored code has syntax errors (brace/paren mismatch)".to_string());
        }

        // Check for common Rust issues
        if refactored.contains("unsafe") && !original.contains("unsafe") {
            warnings.push("Refactoring introduced unsafe code".to_string());
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

    #[test]
    fn test_rust_provider_analyze() -> Result<()> {
        let provider = RustRefactoringProvider::new();
        let analysis = provider.analyze_refactoring("fn main() {}", "rust", RefactoringType::Rename)?;

        assert!(analysis.applicable);
        assert_eq!(analysis.complexity, 4);

        Ok(())
    }

    #[test]
    fn test_rust_provider_validate_valid() -> Result<()> {
        let provider = RustRefactoringProvider::new();
        let result = provider.validate_refactoring("fn main() {}", "fn main() { println!(); }", "rust")?;

        assert!(result.passed);

        Ok(())
    }

    #[test]
    fn test_rust_provider_validate_invalid_braces() -> Result<()> {
        let provider = RustRefactoringProvider::new();
        let result = provider.validate_refactoring("fn main() {}", "fn main() { ", "rust")?;

        assert!(!result.passed);

        Ok(())
    }

    #[test]
    fn test_is_valid_rust() {
        assert!(RustRefactoringProvider::is_valid_rust("fn main() {}"));
        assert!(RustRefactoringProvider::is_valid_rust("let x = [1, 2, 3];"));
        assert!(!RustRefactoringProvider::is_valid_rust("fn main() {"));
        assert!(!RustRefactoringProvider::is_valid_rust("let x = [1, 2, 3;"));
    }
}
