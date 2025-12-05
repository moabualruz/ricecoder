//! Pattern export and import functionality

use crate::error::{RefactoringError, Result};
use super::RefactoringPattern;
use serde_json;
use serde_yaml;

/// Exports and imports refactoring patterns
pub struct PatternExporter;

impl PatternExporter {
    /// Export a pattern to JSON format
    pub fn export_json(pattern: &RefactoringPattern) -> Result<String> {
        serde_json::to_string_pretty(pattern)
            .map_err(|e| RefactoringError::Other(format!("Failed to export pattern to JSON: {}", e)))
    }

    /// Export a pattern to YAML format
    pub fn export_yaml(pattern: &RefactoringPattern) -> Result<String> {
        serde_yaml::to_string(pattern)
            .map_err(|e| RefactoringError::Other(format!("Failed to export pattern to YAML: {}", e)))
    }

    /// Import a pattern from JSON format
    pub fn import_json(data: &str) -> Result<RefactoringPattern> {
        serde_json::from_str(data)
            .map_err(|e| RefactoringError::Other(format!("Failed to import pattern from JSON: {}", e)))
    }

    /// Import a pattern from YAML format
    pub fn import_yaml(data: &str) -> Result<RefactoringPattern> {
        serde_yaml::from_str(data)
            .map_err(|e| RefactoringError::Other(format!("Failed to import pattern from YAML: {}", e)))
    }

    /// Export multiple patterns to JSON format
    pub fn export_patterns_json(patterns: &[RefactoringPattern]) -> Result<String> {
        serde_json::to_string_pretty(patterns)
            .map_err(|e| RefactoringError::Other(format!("Failed to export patterns to JSON: {}", e)))
    }

    /// Export multiple patterns to YAML format
    pub fn export_patterns_yaml(patterns: &[RefactoringPattern]) -> Result<String> {
        serde_yaml::to_string(patterns)
            .map_err(|e| RefactoringError::Other(format!("Failed to export patterns to YAML: {}", e)))
    }

    /// Import multiple patterns from JSON format
    pub fn import_patterns_json(data: &str) -> Result<Vec<RefactoringPattern>> {
        serde_json::from_str(data)
            .map_err(|e| RefactoringError::Other(format!("Failed to import patterns from JSON: {}", e)))
    }

    /// Import multiple patterns from YAML format
    pub fn import_patterns_yaml(data: &str) -> Result<Vec<RefactoringPattern>> {
        serde_yaml::from_str(data)
            .map_err(|e| RefactoringError::Other(format!("Failed to import patterns from YAML: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::{PatternParameter, PatternScope};

    fn create_test_pattern() -> RefactoringPattern {
        RefactoringPattern {
            name: "test_pattern".to_string(),
            description: "A test pattern".to_string(),
            template: "fn {{old_name}}() -> fn {{new_name}}()".to_string(),
            parameters: vec![
                PatternParameter {
                    name: "old_name".to_string(),
                    placeholder: "{{old_name}}".to_string(),
                    description: "Old function name".to_string(),
                },
                PatternParameter {
                    name: "new_name".to_string(),
                    placeholder: "{{new_name}}".to_string(),
                    description: "New function name".to_string(),
                },
            ],
            scope: PatternScope::Global,
        }
    }

    #[test]
    fn test_export_import_json() -> Result<()> {
        let pattern = create_test_pattern();
        let exported = PatternExporter::export_json(&pattern)?;
        let imported = PatternExporter::import_json(&exported)?;

        assert_eq!(imported.name, pattern.name);
        assert_eq!(imported.description, pattern.description);
        assert_eq!(imported.template, pattern.template);
        assert_eq!(imported.parameters.len(), pattern.parameters.len());

        Ok(())
    }

    #[test]
    fn test_export_import_yaml() -> Result<()> {
        let pattern = create_test_pattern();
        let exported = PatternExporter::export_yaml(&pattern)?;
        let imported = PatternExporter::import_yaml(&exported)?;

        assert_eq!(imported.name, pattern.name);
        assert_eq!(imported.description, pattern.description);
        assert_eq!(imported.template, pattern.template);
        assert_eq!(imported.parameters.len(), pattern.parameters.len());

        Ok(())
    }

    #[test]
    fn test_export_import_multiple_json() -> Result<()> {
        let patterns = vec![create_test_pattern(), create_test_pattern()];
        let exported = PatternExporter::export_patterns_json(&patterns)?;
        let imported = PatternExporter::import_patterns_json(&exported)?;

        assert_eq!(imported.len(), patterns.len());
        assert_eq!(imported[0].name, patterns[0].name);

        Ok(())
    }

    #[test]
    fn test_export_import_multiple_yaml() -> Result<()> {
        let patterns = vec![create_test_pattern(), create_test_pattern()];
        let exported = PatternExporter::export_patterns_yaml(&patterns)?;
        let imported = PatternExporter::import_patterns_yaml(&exported)?;

        assert_eq!(imported.len(), patterns.len());
        assert_eq!(imported[0].name, patterns[0].name);

        Ok(())
    }

    #[test]
    fn test_import_invalid_json() {
        let result = PatternExporter::import_json("invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_import_invalid_yaml() {
        let result = PatternExporter::import_yaml("invalid: [yaml");
        assert!(result.is_err());
    }
}
