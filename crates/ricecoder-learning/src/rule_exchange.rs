/// Rule export and import functionality
use crate::error::{LearningError, Result};
use crate::models::Rule;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Metadata for exported rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Version of the export format
    pub version: String,
    /// When the export was created
    pub exported_at: String,
    /// Number of rules in the export
    pub rule_count: usize,
    /// Description of the export
    pub description: Option<String>,
}

/// Container for exported rules with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleExport {
    /// Export metadata
    pub metadata: ExportMetadata,
    /// Exported rules
    pub rules: Vec<Rule>,
}

impl RuleExport {
    /// Create a new rule export
    pub fn new(rules: Vec<Rule>, description: Option<String>) -> Self {
        let rule_count = rules.len();
        Self {
            metadata: ExportMetadata {
                version: "1.0".to_string(),
                exported_at: chrono::Utc::now().to_rfc3339(),
                rule_count,
                description,
            },
            rules,
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(LearningError::SerializationError)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(LearningError::SerializationError)
    }

    /// Write to file
    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)
            .map_err(LearningError::IoError)
    }

    /// Read from file
    pub fn read_from_file(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)
            .map_err(LearningError::IoError)?;
        Self::from_json(&json)
    }
}

/// Rule exporter for exporting rules with metrics
pub struct RuleExporter;

impl RuleExporter {
    /// Export rules to JSON format
    pub fn export_rules(rules: Vec<Rule>, description: Option<String>) -> Result<RuleExport> {
        Ok(RuleExport::new(rules, description))
    }

    /// Export rules to JSON string
    pub fn export_to_json(rules: Vec<Rule>, description: Option<String>) -> Result<String> {
        let export = RuleExport::new(rules, description);
        export.to_json()
    }

    /// Export rules to file
    pub fn export_to_file(
        rules: Vec<Rule>,
        path: &Path,
        description: Option<String>,
    ) -> Result<()> {
        let export = RuleExport::new(rules, description);
        export.write_to_file(path)
    }
}

/// Rule importer for importing rules with validation
pub struct RuleImporter;

impl RuleImporter {
    /// Import rules from JSON string
    pub fn import_from_json(json: &str) -> Result<Vec<Rule>> {
        let export = RuleExport::from_json(json)?;
        Ok(export.rules)
    }

    /// Import rules from file
    pub fn import_from_file(path: &Path) -> Result<Vec<Rule>> {
        let export = RuleExport::read_from_file(path)?;
        Ok(export.rules)
    }

    /// Import and validate rules
    pub fn import_and_validate(json: &str) -> Result<(Vec<Rule>, Vec<String>)> {
        let export = RuleExport::from_json(json)?;
        let mut valid_rules = Vec::new();
        let mut validation_errors = Vec::new();

        for rule in export.rules {
            match Self::validate_rule(&rule) {
                Ok(_) => valid_rules.push(rule),
                Err(e) => validation_errors.push(format!("Rule {}: {}", rule.id, e)),
            }
        }

        Ok((valid_rules, validation_errors))
    }

    /// Validate a single rule
    fn validate_rule(rule: &Rule) -> Result<()> {
        // Validate pattern is not empty
        if rule.pattern.is_empty() {
            return Err(LearningError::RuleValidationFailed(
                "Rule pattern cannot be empty".to_string(),
            ));
        }

        // Validate action is not empty
        if rule.action.is_empty() {
            return Err(LearningError::RuleValidationFailed(
                "Rule action cannot be empty".to_string(),
            ));
        }

        // Validate confidence is in valid range
        if !(0.0..=1.0).contains(&rule.confidence) {
            return Err(LearningError::RuleValidationFailed(
                "Rule confidence must be between 0.0 and 1.0".to_string(),
            ));
        }

        // Validate success rate is in valid range
        if !(0.0..=1.0).contains(&rule.success_rate) {
            return Err(LearningError::RuleValidationFailed(
                "Rule success_rate must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RuleScope, RuleSource};

    fn create_test_rule() -> Rule {
        Rule::new(
            RuleScope::Global,
            "test_pattern".to_string(),
            "test_action".to_string(),
            RuleSource::Learned,
        )
    }

    #[test]
    fn test_rule_export_creation() {
        let rules = vec![create_test_rule()];
        let export = RuleExport::new(rules, Some("Test export".to_string()));

        assert_eq!(export.metadata.version, "1.0");
        assert_eq!(export.metadata.rule_count, 1);
        assert_eq!(export.rules.len(), 1);
    }

    #[test]
    fn test_rule_export_to_json() {
        let rules = vec![create_test_rule()];
        let export = RuleExport::new(rules, None);
        let json = export.to_json().unwrap();

        assert!(json.contains("version") && json.contains("1.0"));
        assert!(json.contains("rule_count") && json.contains("1"));
    }

    #[test]
    fn test_rule_export_from_json() {
        let rules = vec![create_test_rule()];
        let export = RuleExport::new(rules.clone(), None);
        let json = export.to_json().unwrap();

        let imported = RuleExport::from_json(&json).unwrap();
        assert_eq!(imported.rules.len(), 1);
        assert_eq!(imported.metadata.rule_count, 1);
    }

    #[test]
    fn test_rule_exporter_export_rules() {
        let rules = vec![create_test_rule()];
        let export = RuleExporter::export_rules(rules, None).unwrap();

        assert_eq!(export.rules.len(), 1);
        assert_eq!(export.metadata.rule_count, 1);
    }

    #[test]
    fn test_rule_exporter_export_to_json() {
        let rules = vec![create_test_rule()];
        let json = RuleExporter::export_to_json(rules, None).unwrap();

        assert!(json.contains("version") && json.contains("1.0"));
    }

    #[test]
    fn test_rule_importer_import_from_json() {
        let rules = vec![create_test_rule()];
        let export = RuleExport::new(rules, None);
        let json = export.to_json().unwrap();

        let imported = RuleImporter::import_from_json(&json).unwrap();
        assert_eq!(imported.len(), 1);
    }

    #[test]
    fn test_rule_importer_validate_rule_valid() {
        let rule = create_test_rule();
        assert!(RuleImporter::validate_rule(&rule).is_ok());
    }

    #[test]
    fn test_rule_importer_validate_rule_empty_pattern() {
        let mut rule = create_test_rule();
        rule.pattern = String::new();

        assert!(RuleImporter::validate_rule(&rule).is_err());
    }

    #[test]
    fn test_rule_importer_validate_rule_empty_action() {
        let mut rule = create_test_rule();
        rule.action = String::new();

        assert!(RuleImporter::validate_rule(&rule).is_err());
    }

    #[test]
    fn test_rule_importer_validate_rule_invalid_confidence() {
        let mut rule = create_test_rule();
        rule.confidence = 1.5;

        assert!(RuleImporter::validate_rule(&rule).is_err());
    }

    #[test]
    fn test_rule_importer_validate_rule_invalid_success_rate() {
        let mut rule = create_test_rule();
        rule.success_rate = -0.1;

        assert!(RuleImporter::validate_rule(&rule).is_err());
    }

    #[test]
    fn test_rule_importer_import_and_validate() {
        let mut rules = vec![create_test_rule()];
        let mut invalid_rule = create_test_rule();
        invalid_rule.pattern = String::new();
        rules.push(invalid_rule);

        let export = RuleExport::new(rules, None);
        let json = export.to_json().unwrap();

        let (valid, errors) = RuleImporter::import_and_validate(&json).unwrap();
        assert_eq!(valid.len(), 1);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_rule_export_write_and_read_file() {
        let rules = vec![create_test_rule()];
        let export = RuleExport::new(rules, None);

        let temp_file = std::env::temp_dir().join("test_rules.json");
        export.write_to_file(&temp_file).unwrap();

        let imported = RuleExport::read_from_file(&temp_file).unwrap();
        assert_eq!(imported.rules.len(), 1);

        // Cleanup
        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn test_rule_exporter_export_to_file() {
        let rules = vec![create_test_rule()];
        let temp_file = std::env::temp_dir().join("test_export.json");

        RuleExporter::export_to_file(rules, &temp_file, None).unwrap();

        assert!(temp_file.exists());

        // Cleanup
        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn test_rule_importer_import_from_file() {
        let rules = vec![create_test_rule()];
        let export = RuleExport::new(rules, None);

        let temp_file = std::env::temp_dir().join("test_import.json");
        export.write_to_file(&temp_file).unwrap();

        let imported = RuleImporter::import_from_file(&temp_file).unwrap();
        assert_eq!(imported.len(), 1);

        // Cleanup
        let _ = std::fs::remove_file(&temp_file);
    }
}
