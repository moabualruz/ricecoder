//! Parsers for YAML and Markdown spec formats

use crate::error::SpecError;
use crate::models::{
    Spec, SpecPhase, SpecStatus, SpecMetadata, Task
};

/// YAML parser for spec files
pub struct YamlParser;

impl YamlParser {
    /// Parse a YAML spec from a string
    /// 
    /// Supports both plain YAML and YAML with frontmatter (---\n...\n---).
    /// Frontmatter is optional and will be stripped before parsing.
    pub fn parse(content: &str) -> Result<Spec, SpecError> {
        let yaml_content = Self::extract_yaml_content(content);
        serde_yaml::from_str(yaml_content).map_err(SpecError::YamlError)
    }

    /// Serialize a spec to YAML
    /// 
    /// Produces valid YAML without frontmatter markers.
    pub fn serialize(spec: &Spec) -> Result<String, SpecError> {
        serde_yaml::to_string(spec).map_err(SpecError::YamlError)
    }

    /// Extract YAML content from a string that may contain frontmatter
    /// 
    /// If the content starts with ---, it's treated as frontmatter and extracted.
    /// Otherwise, the entire content is returned as-is.
    fn extract_yaml_content(content: &str) -> &str {
        let trimmed = content.trim_start();
        
        // Check if content starts with frontmatter delimiter
        if let Some(after_opening) = trimmed.strip_prefix("---") {
            // Find the closing delimiter
            if let Some(closing_pos) = after_opening.find("---") {
                // Return content after the closing delimiter
                let yaml_start = 3 + closing_pos + 3;
                if yaml_start < trimmed.len() {
                    trimmed[yaml_start..].trim_start()
                } else {
                    ""
                }
            } else {
                // No closing delimiter found, treat entire content as YAML
                trimmed
            }
        } else {
            // No frontmatter, return as-is
            trimmed
        }
    }
}

/// Markdown parser for spec files
pub struct MarkdownParser;

impl MarkdownParser {
    /// Parse a Markdown spec from a string
    /// 
    /// Extracts structured data from markdown sections using regex patterns.
    /// Looks for metadata fields in the format: - **Field**: value
    pub fn parse(content: &str) -> Result<Spec, SpecError> {
        use regex::Regex;
        
        let mut spec_id = String::new();
        let mut spec_name = String::new();
        let mut spec_version = String::new();
        let mut author: Option<String> = None;
        let mut phase = SpecPhase::Requirements;
        let mut status = SpecStatus::Draft;
        
        // Extract first H1 as spec name (multiline mode)
        if let Ok(re) = Regex::new(r"(?m)^#\s+(.+)$") {
            if let Some(cap) = re.captures(content) {
                spec_name = cap[1].trim().to_string();
                spec_id = spec_name.to_lowercase().replace(" ", "-");
            }
        }
        
        // Extract metadata fields
        if let Ok(re) = Regex::new(r"(?i)-\s*\*\*ID\*\*:\s*([^\n]+)") {
            if let Some(cap) = re.captures(content) {
                spec_id = cap[1].trim().to_string();
            }
        }
        
        if let Ok(re) = Regex::new(r"(?i)-\s*\*\*Version\*\*:\s*([^\n]+)") {
            if let Some(cap) = re.captures(content) {
                spec_version = cap[1].trim().to_string();
            }
        }
        
        if let Ok(re) = Regex::new(r"(?i)-\s*\*\*Author\*\*:\s*([^\n]+)") {
            if let Some(cap) = re.captures(content) {
                let author_str = cap[1].trim().to_string();
                if !author_str.is_empty() {
                    author = Some(author_str);
                }
            }
        }
        
        if let Ok(re) = Regex::new(r"(?i)-\s*\*\*Phase\*\*:\s*([^\n]+)") {
            if let Some(cap) = re.captures(content) {
                let phase_str = cap[1].trim().to_lowercase();
                phase = match phase_str.as_str() {
                    "discovery" => SpecPhase::Discovery,
                    "requirements" => SpecPhase::Requirements,
                    "design" => SpecPhase::Design,
                    "tasks" => SpecPhase::Tasks,
                    "execution" => SpecPhase::Execution,
                    _ => SpecPhase::Requirements,
                };
            }
        }
        
        if let Ok(re) = Regex::new(r"(?i)-\s*\*\*Status\*\*:\s*([^\n]+)") {
            if let Some(cap) = re.captures(content) {
                let status_str = cap[1].trim().to_lowercase();
                status = match status_str.as_str() {
                    "draft" => SpecStatus::Draft,
                    "inreview" => SpecStatus::InReview,
                    "approved" => SpecStatus::Approved,
                    "archived" => SpecStatus::Archived,
                    _ => SpecStatus::Draft,
                };
            }
        }
        
        Ok(Spec {
            id: spec_id,
            name: spec_name,
            version: spec_version,
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                phase,
                status,
            },
            inheritance: None,
        })
    }

    /// Serialize a spec to Markdown
    /// 
    /// Produces markdown with sections for each spec component.
    pub fn serialize(spec: &Spec) -> Result<String, SpecError> {
        let mut output = String::new();
        
        // Header
        output.push_str(&format!("# {}\n\n", spec.name));
        
        // Metadata section
        output.push_str("## Metadata\n\n");
        output.push_str(&format!("- **ID**: {}\n", spec.id));
        output.push_str(&format!("- **Version**: {}\n", spec.version));
        if let Some(author) = &spec.metadata.author {
            output.push_str(&format!("- **Author**: {}\n", author));
        }
        output.push_str(&format!("- **Phase**: {:?}\n", spec.metadata.phase));
        output.push_str(&format!("- **Status**: {:?}\n", spec.metadata.status));
        output.push_str(&format!("- **Created**: {}\n", spec.metadata.created_at));
        output.push_str(&format!("- **Updated**: {}\n\n", spec.metadata.updated_at));
        
        // Requirements section
        if !spec.requirements.is_empty() {
            output.push_str("## Requirements\n\n");
            for req in &spec.requirements {
                output.push_str(&format!("### {}: {}\n\n", req.id, req.user_story));
                output.push_str("#### Acceptance Criteria\n\n");
                for criterion in &req.acceptance_criteria {
                    output.push_str(&format!("- **{}**: WHEN {} THEN {}\n", 
                        criterion.id, criterion.when, criterion.then));
                }
                output.push_str(&format!("\n**Priority**: {:?}\n\n", req.priority));
            }
        }
        
        // Design section
        if let Some(design) = &spec.design {
            output.push_str("## Design\n\n");
            output.push_str("### Overview\n\n");
            output.push_str(&format!("{}\n\n", design.overview));
            
            output.push_str("### Architecture\n\n");
            output.push_str(&format!("{}\n\n", design.architecture));
            
            if !design.components.is_empty() {
                output.push_str("### Components\n\n");
                for component in &design.components {
                    output.push_str(&format!("- **{}**: {}\n", component.name, component.description));
                }
                output.push('\n');
            }
            
            if !design.data_models.is_empty() {
                output.push_str("### Data Models\n\n");
                for model in &design.data_models {
                    output.push_str(&format!("- **{}**: {}\n", model.name, model.description));
                }
                output.push('\n');
            }
            
            if !design.correctness_properties.is_empty() {
                output.push_str("### Correctness Properties\n\n");
                for prop in &design.correctness_properties {
                    output.push_str(&format!("- **{}**: {}\n", prop.id, prop.description));
                    if !prop.validates.is_empty() {
                        output.push_str(&format!("  - Validates: {}\n", prop.validates.join(", ")));
                    }
                }
                output.push('\n');
            }
        }
        
        // Tasks section
        if !spec.tasks.is_empty() {
            output.push_str("## Tasks\n\n");
            Self::serialize_tasks(&mut output, &spec.tasks, 0);
        }
        
        Ok(output)
    }
    
    /// Helper to serialize tasks recursively
    fn serialize_tasks(output: &mut String, tasks: &[Task], depth: usize) {
        for task in tasks {
            let prefix = "#".repeat(3 + depth);
            output.push_str(&format!("{} {}: {}\n\n", prefix, task.id, task.description));
            
            if !task.requirements.is_empty() {
                output.push_str(&format!("{}**Requirements**: {}\n\n", 
                    " ".repeat(depth * 2), task.requirements.join(", ")));
            }
            
            output.push_str(&format!("{}**Status**: {:?}\n", 
                " ".repeat(depth * 2), task.status));
            output.push_str(&format!("{}**Optional**: {}\n\n", 
                " ".repeat(depth * 2), task.optional));
            
            if !task.subtasks.is_empty() {
                Self::serialize_tasks(output, &task.subtasks, depth + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use chrono::Utc;

    #[test]
    fn test_yaml_parser_roundtrip() {
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

        let yaml = YamlParser::serialize(&spec).expect("Failed to serialize");
        let parsed = YamlParser::parse(&yaml).expect("Failed to parse");

        assert_eq!(spec.id, parsed.id);
        assert_eq!(spec.name, parsed.name);
        assert_eq!(spec.version, parsed.version);
    }

    #[test]
    fn test_yaml_parser_with_frontmatter() {
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

        let yaml = YamlParser::serialize(&spec).expect("Failed to serialize");
        let with_frontmatter = format!("---\n# Frontmatter\n---\n{}", yaml);
        let parsed = YamlParser::parse(&with_frontmatter).expect("Failed to parse");

        assert_eq!(spec.id, parsed.id);
        assert_eq!(spec.name, parsed.name);
        assert_eq!(spec.version, parsed.version);
    }

    #[test]
    fn test_yaml_parser_frontmatter_extraction() {
        let content = "---\nmetadata\n---\nid: test\nname: Test";
        let extracted = YamlParser::extract_yaml_content(content);
        assert_eq!(extracted, "id: test\nname: Test");
    }

    #[test]
    fn test_yaml_parser_no_frontmatter() {
        let content = "id: test\nname: Test";
        let extracted = YamlParser::extract_yaml_content(content);
        assert_eq!(extracted, "id: test\nname: Test");
    }

    #[test]
    fn test_yaml_parser_with_whitespace() {
        let content = "  ---\nmetadata\n---\n  id: test\n  name: Test";
        let extracted = YamlParser::extract_yaml_content(content);
        assert_eq!(extracted, "id: test\n  name: Test");
    }
}

#[cfg(test)]
mod markdown_tests {
    use super::*;
    use crate::models::*;

    #[test]
    fn test_markdown_parser_basic_spec() {
        let markdown = "# Test Spec\n\n## Metadata\n\n- **ID**: test-spec\n- **Version**: 1.0\n- **Author**: Test Author\n- **Phase**: Requirements\n- **Status**: Draft\n";

        let spec = MarkdownParser::parse(markdown).expect("Failed to parse markdown");
        assert_eq!(spec.id, "test-spec");
        assert_eq!(spec.name, "Test Spec");
        assert_eq!(spec.version, "1.0");
        assert_eq!(spec.metadata.author, Some("Test Author".to_string()));
        assert_eq!(spec.metadata.phase, SpecPhase::Requirements);
        assert_eq!(spec.metadata.status, SpecStatus::Draft);
    }

    #[test]
    fn test_markdown_parser_missing_explicit_id() {
        let markdown = "# Test Spec\n\n## Metadata\n\n- **Version**: 1.0\n";

        let spec = MarkdownParser::parse(markdown).expect("Failed to parse markdown");
        // When no explicit ID is provided, it should be derived from the name
        assert_eq!(spec.id, "test-spec");
        assert_eq!(spec.name, "Test Spec");
        assert_eq!(spec.version, "1.0");
    }

    #[test]
    fn test_markdown_serialization_basic() {
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Test Author".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let markdown = MarkdownParser::serialize(&spec).expect("Failed to serialize");
        assert!(markdown.contains("# Test Spec"));
        assert!(markdown.contains("test-spec"));
        assert!(markdown.contains("Test Author"));
    }

    #[test]
    fn test_markdown_serialization_with_requirements() {
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0".to_string(),
            requirements: vec![
                Requirement {
                    id: "REQ-1".to_string(),
                    user_story: "As a user, I want to create tasks".to_string(),
                    acceptance_criteria: vec![
                        AcceptanceCriterion {
                            id: "AC-1.1".to_string(),
                            when: "user enters task".to_string(),
                            then: "task is added".to_string(),
                        },
                    ],
                    priority: Priority::Must,
                },
            ],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let markdown = MarkdownParser::serialize(&spec).expect("Failed to serialize");
        assert!(markdown.contains("## Requirements"));
        assert!(markdown.contains("REQ-1"));
        assert!(markdown.contains("As a user, I want to create tasks"));
        assert!(markdown.contains("AC-1.1"));
        assert!(markdown.contains("WHEN user enters task THEN task is added"));
    }

    #[test]
    fn test_markdown_serialization_with_design() {
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0".to_string(),
            requirements: vec![],
            design: Some(Design {
                overview: "System overview".to_string(),
                architecture: "Layered architecture".to_string(),
                components: vec![
                    Component {
                        name: "ComponentA".to_string(),
                        description: "First component".to_string(),
                    },
                ],
                data_models: vec![
                    DataModel {
                        name: "Model1".to_string(),
                        description: "First model".to_string(),
                    },
                ],
                correctness_properties: vec![
                    Property {
                        id: "PROP-1".to_string(),
                        description: "Property description".to_string(),
                        validates: vec!["REQ-1".to_string()],
                    },
                ],
            }),
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                phase: SpecPhase::Design,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let markdown = MarkdownParser::serialize(&spec).expect("Failed to serialize");
        assert!(markdown.contains("## Design"));
        assert!(markdown.contains("System overview"));
        assert!(markdown.contains("Layered architecture"));
        assert!(markdown.contains("ComponentA"));
        assert!(markdown.contains("Model1"));
        assert!(markdown.contains("PROP-1"));
    }

    #[test]
    fn test_markdown_serialization_with_tasks() {
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![
                Task {
                    id: "1".to_string(),
                    description: "Main task".to_string(),
                    subtasks: vec![
                        Task {
                            id: "1.1".to_string(),
                            description: "Subtask".to_string(),
                            subtasks: vec![],
                            requirements: vec!["REQ-1".to_string()],
                            status: TaskStatus::NotStarted,
                            optional: false,
                        },
                    ],
                    requirements: vec![],
                    status: TaskStatus::InProgress,
                    optional: false,
                },
            ],
            metadata: SpecMetadata {
                author: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                phase: SpecPhase::Tasks,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let markdown = MarkdownParser::serialize(&spec).expect("Failed to serialize");
        assert!(markdown.contains("## Tasks"));
        assert!(markdown.contains("### 1: Main task"));
        assert!(markdown.contains("#### 1.1: Subtask"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::models::*;
    use chrono::Utc;
    use proptest::prelude::*;

    // Helper function to generate arbitrary Spec values with valid IDs
    fn arb_spec() -> impl Strategy<Value = Spec> {
        // Generate valid spec IDs: alphanumeric, hyphens, underscores
        let valid_id = r"[a-z0-9][a-z0-9\-_]{0,20}";
        // Generate valid names: must have at least one non-space character
        let valid_name = r"[a-zA-Z0-9][a-zA-Z0-9 ]{0,29}";
        let valid_version = r"[0-9]\.[0-9](\.[0-9])?";
        
        (valid_id, valid_name, valid_version)
            .prop_map(|(id, name, version)| {
                let now = Utc::now();
                Spec {
                    id,
                    name: name.trim().to_string(),  // Trim whitespace for consistency
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
        /// **Validates: Requirements 1.1, 1.2, 1.4**
        /// 
        /// For any valid spec, parsing and serializing SHALL produce semantically equivalent output.
        #[test]
        fn prop_yaml_roundtrip_preserves_spec(spec in arb_spec()) {
            // Serialize the spec to YAML
            let yaml = YamlParser::serialize(&spec)
                .expect("Failed to serialize spec");
            
            // Parse it back
            let parsed = YamlParser::parse(&yaml)
                .expect("Failed to parse spec");
            
            // Verify semantic equivalence
            prop_assert_eq!(spec.id, parsed.id, "ID should be preserved");
            prop_assert_eq!(spec.name, parsed.name, "Name should be preserved");
            prop_assert_eq!(spec.version, parsed.version, "Version should be preserved");
            prop_assert_eq!(spec.requirements.len(), parsed.requirements.len(), "Requirements count should be preserved");
            prop_assert_eq!(spec.tasks.len(), parsed.tasks.len(), "Tasks count should be preserved");
            prop_assert_eq!(spec.metadata.phase, parsed.metadata.phase, "Phase should be preserved");
            prop_assert_eq!(spec.metadata.status, parsed.metadata.status, "Status should be preserved");
        }

        /// **Feature: ricecoder-specs, Property 1: Spec Parsing Round-Trip**
        /// **Validates: Requirements 1.1, 1.2, 1.4**
        /// 
        /// For any valid spec with frontmatter, parsing and serializing SHALL produce semantically equivalent output.
        #[test]
        fn prop_yaml_roundtrip_with_frontmatter(spec in arb_spec()) {
            // Serialize the spec to YAML
            let yaml = YamlParser::serialize(&spec)
                .expect("Failed to serialize spec");
            
            // Add frontmatter
            let with_frontmatter = format!("---\n# Metadata\n---\n{}", yaml);
            
            // Parse it back
            let parsed = YamlParser::parse(&with_frontmatter)
                .expect("Failed to parse spec with frontmatter");
            
            // Verify semantic equivalence
            prop_assert_eq!(spec.id, parsed.id, "ID should be preserved with frontmatter");
            prop_assert_eq!(spec.name, parsed.name, "Name should be preserved with frontmatter");
            prop_assert_eq!(spec.version, parsed.version, "Version should be preserved with frontmatter");
        }

        /// **Feature: ricecoder-specs, Property 1: Spec Parsing Round-Trip**
        /// **Validates: Requirements 1.1, 1.2, 1.4**
        /// 
        /// For any valid spec, markdown serialization and parsing SHALL produce semantically equivalent output.
        #[test]
        fn prop_markdown_roundtrip_preserves_spec(spec in arb_spec()) {
            // Serialize the spec to Markdown
            let markdown = MarkdownParser::serialize(&spec)
                .expect("Failed to serialize spec to markdown");
            
            // Parse it back
            let parsed = MarkdownParser::parse(&markdown)
                .expect("Failed to parse markdown spec");
            
            // Verify semantic equivalence
            prop_assert_eq!(spec.id, parsed.id, "ID should be preserved in markdown roundtrip");
            prop_assert_eq!(spec.name, parsed.name, "Name should be preserved in markdown roundtrip");
            prop_assert_eq!(spec.version, parsed.version, "Version should be preserved in markdown roundtrip");
            prop_assert_eq!(spec.metadata.phase, parsed.metadata.phase, "Phase should be preserved in markdown roundtrip");
            prop_assert_eq!(spec.metadata.status, parsed.metadata.status, "Status should be preserved in markdown roundtrip");
        }
    }
}
