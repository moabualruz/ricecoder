//! Python-specific refactoring provider

use crate::error::Result;
use crate::providers::{RefactoringAnalysis, RefactoringProvider};
use crate::types::{Refactoring, RefactoringType, ValidationResult};
use regex::Regex;

/// Python-specific refactoring provider
pub struct PythonRefactoringProvider;

impl PythonRefactoringProvider {
    /// Create a new Python provider
    pub fn new() -> Self {
        Self
    }

    /// Check if code is valid Python
    fn is_valid_python(code: &str) -> bool {
        // Basic checks for Python syntax
        let open_parens = code.matches('(').count();
        let close_parens = code.matches(')').count();
        let open_brackets = code.matches('[').count();
        let close_brackets = code.matches(']').count();
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();

        open_parens == close_parens
            && open_brackets == close_brackets
            && open_braces == close_braces
    }

    /// Apply a Python-specific rename with word boundaries
    pub fn apply_python_rename(code: &str, old_name: &str, new_name: &str) -> Result<String> {
        let pattern = format!(r"\b{}\b", regex::escape(old_name));
        match Regex::new(&pattern) {
            Ok(re) => Ok(re.replace_all(code, new_name).to_string()),
            Err(_) => Ok(code.replace(old_name, new_name)),
        }
    }

    /// Check for Python code quality issues
    pub fn check_python_quality(code: &str) -> Vec<String> {
        let mut issues = vec![];

        if code.contains("eval(") {
            issues.push("Code uses eval() which is a security risk".to_string());
        }

        if code.contains("exec(") {
            issues.push("Code uses exec() which is a security risk".to_string());
        }

        if code.contains("__import__") {
            issues.push("Code uses __import__() for dynamic imports".to_string());
        }

        issues
    }
}

impl Default for PythonRefactoringProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl RefactoringProvider for PythonRefactoringProvider {
    fn analyze_refactoring(
        &self,
        _code: &str,
        _language: &str,
        refactoring_type: RefactoringType,
    ) -> Result<RefactoringAnalysis> {
        let complexity = match refactoring_type {
            RefactoringType::Rename => 3,
            RefactoringType::Extract => 5,
            RefactoringType::Inline => 4,
            RefactoringType::Move => 6,
            RefactoringType::ChangeSignature => 7,
            RefactoringType::RemoveUnused => 3,
            RefactoringType::Simplify => 4,
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
        // Apply Python-specific refactoring
        match refactoring.refactoring_type {
            RefactoringType::Rename => {
                // Python rename: use word boundaries
                Self::apply_python_rename(
                    code,
                    &refactoring.target.symbol,
                    &refactoring.target.symbol,
                )
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

        // Check Python syntax validity
        if !Self::is_valid_python(refactored) {
            errors.push("Refactored code has syntax errors (paren/bracket mismatch)".to_string());
        }

        // Check for common Python issues
        if refactored.contains("exec(") && !original.contains("exec(") {
            warnings.push("Refactoring introduced exec() call".to_string());
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
    fn test_python_provider_analyze() -> Result<()> {
        let provider = PythonRefactoringProvider::new();
        let analysis =
            provider.analyze_refactoring("def main():", "python", RefactoringType::Rename)?;

        assert!(analysis.applicable);
        assert_eq!(analysis.complexity, 3);

        Ok(())
    }

    #[test]
    fn test_python_provider_validate_valid() -> Result<()> {
        let provider = PythonRefactoringProvider::new();
        let result =
            provider.validate_refactoring("def main():", "def main():\n    print()", "python")?;

        assert!(result.passed);

        Ok(())
    }

    #[test]
    fn test_python_provider_validate_invalid_parens() -> Result<()> {
        let provider = PythonRefactoringProvider::new();
        let result = provider.validate_refactoring("def main():", "def main(:", "python")?;

        assert!(!result.passed);

        Ok(())
    }

    #[test]
    fn test_is_valid_python() {
        assert!(PythonRefactoringProvider::is_valid_python("def main():"));
        assert!(PythonRefactoringProvider::is_valid_python("x = [1, 2, 3]"));
        assert!(!PythonRefactoringProvider::is_valid_python("def main("));
        assert!(!PythonRefactoringProvider::is_valid_python("x = [1, 2, 3"));
    }
}
