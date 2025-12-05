//! Format conversion utilities for converting between YAML and Markdown spec formats

use crate::error::SpecError;
use crate::parsers::{MarkdownParser, YamlParser};
use crate::validation::ValidationEngine;

/// Converts specs between YAML and Markdown formats
pub struct FormatConverter;

impl FormatConverter {
    /// Convert a spec from YAML format to Markdown format
    ///
    /// # Arguments
    /// * `yaml_content` - The YAML spec content as a string
    ///
    /// # Returns
    /// The spec converted to Markdown format, or an error if parsing/validation fails
    ///
    /// # Errors
    /// Returns `SpecError` if:
    /// - The YAML content cannot be parsed
    /// - The parsed spec fails validation
    pub fn yaml_to_markdown(yaml_content: &str) -> Result<String, SpecError> {
        // Parse YAML to Spec
        let spec = YamlParser::parse(yaml_content)?;

        // Validate the parsed spec
        ValidationEngine::validate(&spec)?;

        // Serialize to Markdown
        MarkdownParser::serialize(&spec)
    }

    /// Convert a spec from Markdown format to YAML format
    ///
    /// # Arguments
    /// * `markdown_content` - The Markdown spec content as a string
    ///
    /// # Returns
    /// The spec converted to YAML format, or an error if parsing/validation fails
    ///
    /// # Errors
    /// Returns `SpecError` if:
    /// - The Markdown content cannot be parsed
    /// - The parsed spec fails validation
    pub fn markdown_to_yaml(markdown_content: &str) -> Result<String, SpecError> {
        // Parse Markdown to Spec
        let spec = MarkdownParser::parse(markdown_content)?;

        // Validate the parsed spec
        ValidationEngine::validate(&spec)?;

        // Serialize to YAML
        YamlParser::serialize(&spec)
    }

    /// Convert a spec from one format to another
    ///
    /// # Arguments
    /// * `content` - The spec content as a string
    /// * `from_format` - The source format ("yaml" or "markdown")
    /// * `to_format` - The target format ("yaml" or "markdown")
    ///
    /// # Returns
    /// The spec converted to the target format, or an error if conversion fails
    ///
    /// # Errors
    /// Returns `SpecError` if:
    /// - The source format is invalid
    /// - The content cannot be parsed
    /// - The parsed spec fails validation
    pub fn convert(content: &str, from_format: &str, to_format: &str) -> Result<String, SpecError> {
        let from_lower = from_format.to_lowercase();
        let to_lower = to_format.to_lowercase();

        match (from_lower.as_str(), to_lower.as_str()) {
            ("yaml", "markdown") => Self::yaml_to_markdown(content),
            ("markdown", "yaml") => Self::markdown_to_yaml(content),
            ("yaml", "yaml") => {
                // YAML to YAML: parse and re-serialize to normalize
                let spec = YamlParser::parse(content)?;
                ValidationEngine::validate(&spec)?;
                YamlParser::serialize(&spec)
            }
            ("markdown", "markdown") => {
                // Markdown to Markdown: parse and re-serialize to normalize
                let spec = MarkdownParser::parse(content)?;
                ValidationEngine::validate(&spec)?;
                MarkdownParser::serialize(&spec)
            }
            _ => Err(SpecError::InvalidFormat(format!(
                "Unsupported format conversion: {} to {}",
                from_format, to_format
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use chrono::Utc;

    #[test]
    fn test_yaml_to_markdown_conversion() {
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Test Author".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let yaml = YamlParser::serialize(&spec).expect("Failed to serialize to YAML");
        let markdown =
            FormatConverter::yaml_to_markdown(&yaml).expect("Failed to convert YAML to Markdown");

        assert!(markdown.contains("# Test Spec"));
        assert!(markdown.contains("test-spec"));
        assert!(markdown.contains("Test Author"));
    }

    #[test]
    fn test_markdown_to_yaml_conversion() {
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Test Author".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let markdown = MarkdownParser::serialize(&spec).expect("Failed to serialize to Markdown");
        let yaml = FormatConverter::markdown_to_yaml(&markdown)
            .expect("Failed to convert Markdown to YAML");

        assert!(yaml.contains("id: test-spec"));
        assert!(yaml.contains("name: Test Spec"));
        assert!(yaml.contains("version: '1.0'"));
    }

    #[test]
    fn test_convert_yaml_to_markdown() {
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let yaml = YamlParser::serialize(&spec).expect("Failed to serialize to YAML");
        let markdown =
            FormatConverter::convert(&yaml, "yaml", "markdown").expect("Failed to convert");

        assert!(markdown.contains("# Test Spec"));
    }

    #[test]
    fn test_convert_markdown_to_yaml() {
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let markdown = MarkdownParser::serialize(&spec).expect("Failed to serialize to Markdown");
        let yaml =
            FormatConverter::convert(&markdown, "markdown", "yaml").expect("Failed to convert");

        assert!(yaml.contains("id: test-spec"));
    }

    #[test]
    fn test_convert_same_format_yaml() {
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let yaml = YamlParser::serialize(&spec).expect("Failed to serialize to YAML");
        let normalized =
            FormatConverter::convert(&yaml, "yaml", "yaml").expect("Failed to normalize YAML");

        // Should be able to parse both
        let parsed_original = YamlParser::parse(&yaml).expect("Failed to parse original");
        let parsed_normalized = YamlParser::parse(&normalized).expect("Failed to parse normalized");

        assert_eq!(parsed_original.id, parsed_normalized.id);
    }

    #[test]
    fn test_convert_same_format_markdown() {
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let markdown = MarkdownParser::serialize(&spec).expect("Failed to serialize to Markdown");
        let normalized = FormatConverter::convert(&markdown, "markdown", "markdown")
            .expect("Failed to normalize Markdown");

        // Should be able to parse both
        let parsed_original = MarkdownParser::parse(&markdown).expect("Failed to parse original");
        let parsed_normalized =
            MarkdownParser::parse(&normalized).expect("Failed to parse normalized");

        assert_eq!(parsed_original.id, parsed_normalized.id);
    }

    #[test]
    fn test_convert_invalid_format() {
        let result = FormatConverter::convert("content", "invalid", "yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_case_insensitive() {
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let yaml = YamlParser::serialize(&spec).expect("Failed to serialize to YAML");

        // Test with uppercase
        let result1 = FormatConverter::convert(&yaml, "YAML", "MARKDOWN");
        assert!(result1.is_ok());

        // Test with mixed case
        let result2 = FormatConverter::convert(&yaml, "YaMl", "MaRkDoWn");
        assert!(result2.is_ok());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::models::*;
    use chrono::Utc;
    use proptest::prelude::*;

    fn arb_spec() -> impl Strategy<Value = Spec> {
        let valid_id = r"[a-z0-9][a-z0-9\-_]{0,20}";
        let valid_name = r"[a-zA-Z0-9][a-zA-Z0-9 ]{0,29}";
        let valid_version = r"[0-9]\.[0-9](\.[0-9])?";

        (valid_id, valid_name, valid_version).prop_map(|(id, name, version)| {
            let now = Utc::now();
            Spec {
                id,
                name: name.trim().to_string(),
                version,
                requirements: vec![],
                design: None,
                tasks: vec![],
                metadata: SpecMetadata {
                    author: Some("Test".to_string()),
                    created_at: now,
                    updated_at: now,
                    phase: SpecPhase::Requirements,
                    status: SpecStatus::Draft,
                },
                inheritance: None,
            }
        })
    }

    proptest! {
        /// **Feature: ricecoder-specs, Property 1: Spec Parsing Round-Trip**
        /// **Validates: Requirements 1.8**
        ///
        /// For any valid spec, converting YAML → Markdown → YAML SHALL produce semantically equivalent output.
        #[test]
        fn prop_yaml_markdown_yaml_roundtrip(spec in arb_spec()) {
            // Serialize to YAML
            let yaml1 = YamlParser::serialize(&spec)
                .expect("Failed to serialize to YAML");

            // Convert YAML to Markdown
            let markdown = FormatConverter::yaml_to_markdown(&yaml1)
                .expect("Failed to convert YAML to Markdown");

            // Convert Markdown back to YAML
            let yaml2 = FormatConverter::markdown_to_yaml(&markdown)
                .expect("Failed to convert Markdown to YAML");

            // Parse both YAML versions
            let parsed1 = YamlParser::parse(&yaml1)
                .expect("Failed to parse original YAML");
            let parsed2 = YamlParser::parse(&yaml2)
                .expect("Failed to parse roundtrip YAML");

            // Verify semantic equivalence
            prop_assert_eq!(parsed1.id, parsed2.id, "ID should be preserved in YAML→MD→YAML");
            prop_assert_eq!(parsed1.name, parsed2.name, "Name should be preserved in YAML→MD→YAML");
            prop_assert_eq!(parsed1.version, parsed2.version, "Version should be preserved in YAML→MD→YAML");
            prop_assert_eq!(parsed1.metadata.phase, parsed2.metadata.phase, "Phase should be preserved in YAML→MD→YAML");
            prop_assert_eq!(parsed1.metadata.status, parsed2.metadata.status, "Status should be preserved in YAML→MD→YAML");
        }

        /// **Feature: ricecoder-specs, Property 1: Spec Parsing Round-Trip**
        /// **Validates: Requirements 1.8**
        ///
        /// For any valid spec, converting Markdown → YAML → Markdown SHALL produce semantically equivalent output.
        #[test]
        fn prop_markdown_yaml_markdown_roundtrip(spec in arb_spec()) {
            // Serialize to Markdown
            let markdown1 = MarkdownParser::serialize(&spec)
                .expect("Failed to serialize to Markdown");

            // Convert Markdown to YAML
            let yaml = FormatConverter::markdown_to_yaml(&markdown1)
                .expect("Failed to convert Markdown to YAML");

            // Convert YAML back to Markdown
            let markdown2 = FormatConverter::yaml_to_markdown(&yaml)
                .expect("Failed to convert YAML to Markdown");

            // Parse both Markdown versions
            let parsed1 = MarkdownParser::parse(&markdown1)
                .expect("Failed to parse original Markdown");
            let parsed2 = MarkdownParser::parse(&markdown2)
                .expect("Failed to parse roundtrip Markdown");

            // Verify semantic equivalence
            prop_assert_eq!(parsed1.id, parsed2.id, "ID should be preserved in MD→YAML→MD");
            prop_assert_eq!(parsed1.name, parsed2.name, "Name should be preserved in MD→YAML→MD");
            prop_assert_eq!(parsed1.version, parsed2.version, "Version should be preserved in MD→YAML→MD");
            prop_assert_eq!(parsed1.metadata.phase, parsed2.metadata.phase, "Phase should be preserved in MD→YAML→MD");
            prop_assert_eq!(parsed1.metadata.status, parsed2.metadata.status, "Status should be preserved in MD→YAML→MD");
        }

        /// **Feature: ricecoder-specs, Property 1: Spec Parsing Round-Trip**
        /// **Validates: Requirements 1.8**
        ///
        /// For any valid spec, converting between formats SHALL preserve all semantic data.
        #[test]
        fn prop_format_conversion_preserves_semantics(spec in arb_spec()) {
            // Serialize to YAML
            let yaml = YamlParser::serialize(&spec)
                .expect("Failed to serialize to YAML");

            // Convert to Markdown
            let markdown = FormatConverter::yaml_to_markdown(&yaml)
                .expect("Failed to convert to Markdown");

            // Parse both formats
            let from_yaml = YamlParser::parse(&yaml)
                .expect("Failed to parse YAML");
            let from_markdown = MarkdownParser::parse(&markdown)
                .expect("Failed to parse Markdown");

            // Verify semantic equivalence
            prop_assert_eq!(from_yaml.id, from_markdown.id, "ID should match across formats");
            prop_assert_eq!(from_yaml.name, from_markdown.name, "Name should match across formats");
            prop_assert_eq!(from_yaml.version, from_markdown.version, "Version should match across formats");
            prop_assert_eq!(from_yaml.metadata.phase, from_markdown.metadata.phase, "Phase should match across formats");
            prop_assert_eq!(from_yaml.metadata.status, from_markdown.metadata.status, "Status should match across formats");
        }
    }
}
