//! TypeScript-specific refactoring provider

use crate::error::Result;
use crate::providers::{RefactoringAnalysis, RefactoringProvider};
use crate::types::{Refactoring, RefactoringType, ValidationResult};
use regex::Regex;

/// TypeScript-specific refactoring provider
pub struct TypeScriptRefactoringProvider;

impl TypeScriptRefactoringProvider {
    /// Create a new TypeScript provider
    pub fn new() -> Self {
        Self
    }

    /// Check if code is valid TypeScript
    fn is_valid_typescript(code: &str) -> bool {
        // Basic checks for TypeScript syntax
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

    /// Apply a TypeScript-specific rename with word boundaries
    pub fn apply_typescript_rename(code: &str, old_name: &str, new_name: &str) -> Result<String> {
        let pattern = format!(r"\b{}\b", regex::escape(old_name));
        match Regex::new(&pattern) {
            Ok(re) => Ok(re.replace_all(code, new_name).to_string()),
            Err(_) => Ok(code.replace(old_name, new_name)),
        }
    }

    /// Check for TypeScript type issues
    pub fn check_typescript_types(code: &str) -> Vec<String> {
        let mut issues = vec![];

        if code.contains(": any") {
            issues.push("Code uses 'any' type which disables type checking".to_string());
        }

        if code.contains("as any") {
            issues.push("Code uses 'as any' type assertion".to_string());
        }

        if code.contains("!.") {
            issues.push("Code uses non-null assertion operator".to_string());
        }

        issues
    }
}

impl Default for TypeScriptRefactoringProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl RefactoringProvider for TypeScriptRefactoringProvider {
    fn analyze_refactoring(
        &self,
        _code: &str,
        _language: &str,
        refactoring_type: RefactoringType,
    ) -> Result<RefactoringAnalysis> {
        let complexity = match refactoring_type {
            RefactoringType::Rename => 4,
            RefactoringType::Extract => 6,
            RefactoringType::Inline => 5,
            RefactoringType::Move => 7,
            RefactoringType::ChangeSignature => 8,
            RefactoringType::RemoveUnused => 4,
            RefactoringType::Simplify => 5,
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
        // Apply TypeScript-specific refactoring
        match refactoring.refactoring_type {
            RefactoringType::Rename => {
                // TypeScript rename: use word boundaries
                Self::apply_typescript_rename(code, &refactoring.target.symbol, &refactoring.target.symbol)
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

        // Check TypeScript syntax validity
        if !Self::is_valid_typescript(refactored) {
            errors.push("Refactored code has syntax errors (brace/paren mismatch)".to_string());
        }

        // Check for common TypeScript issues
        if refactored.contains("any") && !original.contains("any") {
            warnings.push("Refactoring introduced 'any' type".to_string());
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
    fn test_typescript_provider_analyze() -> Result<()> {
        let provider = TypeScriptRefactoringProvider::new();
        let analysis = provider.analyze_refactoring("function main() {}", "typescript", RefactoringType::Rename)?;

        assert!(analysis.applicable);
        assert_eq!(analysis.complexity, 4);

        Ok(())
    }

    #[test]
    fn test_typescript_provider_validate_valid() -> Result<()> {
        let provider = TypeScriptRefactoringProvider::new();
        let result = provider.validate_refactoring("function main() {}", "function main() { console.log(); }", "typescript")?;

        assert!(result.passed);

        Ok(())
    }

    #[test]
    fn test_typescript_provider_validate_invalid_braces() -> Result<()> {
        let provider = TypeScriptRefactoringProvider::new();
        let result = provider.validate_refactoring("function main() {}", "function main() { ", "typescript")?;

        assert!(!result.passed);

        Ok(())
    }

    #[test]
    fn test_is_valid_typescript() {
        assert!(TypeScriptRefactoringProvider::is_valid_typescript("function main() {}"));
        assert!(TypeScriptRefactoringProvider::is_valid_typescript("const x = [1, 2, 3];"));
        assert!(!TypeScriptRefactoringProvider::is_valid_typescript("function main() {"));
        assert!(!TypeScriptRefactoringProvider::is_valid_typescript("const x = [1, 2, 3;"));
    }
}
