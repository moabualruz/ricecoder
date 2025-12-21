//! Configuration loader and validator for domain agents
//!
//! This module provides functionality to load domain agent configurations from YAML/JSON files
//! and validate them against the JSON Schema.

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::factory::AgentConfig;
use jsonschema::JSONSchema;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Configuration loader for domain agents
///
/// This struct provides methods to load and validate domain agent configurations
/// from YAML and JSON files.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::ConfigLoader;
/// use std::path::Path;
///
/// let loader = ConfigLoader::new();
/// let config = loader.load_from_file(Path::new("config/domains/web.yaml"))?;
/// ```
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    schema: Option<Arc<JSONSchema>>,
}

impl ConfigLoader {
    /// Create a new configuration loader
    ///
    /// # Returns
    ///
    /// Returns a new ConfigLoader instance
    pub fn new() -> Self {
        Self { schema: None }
    }

    /// Create a new configuration loader with schema validation
    ///
    /// # Arguments
    ///
    /// * `schema_json` - JSON schema as a string
    ///
    /// # Returns
    ///
    /// Returns a new ConfigLoader instance with schema validation enabled
    pub fn with_schema(schema_json: &str) -> DomainResult<Self> {
        let schema_value: Value = serde_json::from_str(schema_json)
            .map_err(|e| DomainError::config_error(format!("Failed to parse schema: {}", e)))?;

        let schema = JSONSchema::compile(&schema_value)
            .map_err(|e| DomainError::config_error(format!("Failed to compile schema: {}", e)))?;

        Ok(Self {
            schema: Some(Arc::new(schema)),
        })
    }

    /// Load configuration from a file
    ///
    /// Automatically detects file format (YAML or JSON) based on file extension.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Returns the loaded configuration
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let config = loader.load_from_file(Path::new("config/domains/web.yaml"))?;
    /// ```
    pub fn load_from_file(&self, path: &Path) -> DomainResult<AgentConfig> {
        // Read file
        let content = fs::read_to_string(path)
            .map_err(|e| DomainError::config_error(format!("Failed to read file: {}", e)))?;

        // Determine format based on extension
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        match extension {
            "yaml" | "yml" => self.load_from_yaml(&content),
            "json" => self.load_from_json(&content),
            _ => Err(DomainError::config_error(
                "Unsupported file format. Use .yaml, .yml, or .json",
            )),
        }
    }

    /// Load configuration from YAML string
    ///
    /// # Arguments
    ///
    /// * `yaml` - YAML string containing configuration
    ///
    /// # Returns
    ///
    /// Returns the loaded configuration
    pub fn load_from_yaml(&self, yaml: &str) -> DomainResult<AgentConfig> {
        // Parse YAML to JSON value
        let value: Value = serde_yaml::from_str(yaml)
            .map_err(|e| DomainError::config_error(format!("Failed to parse YAML: {}", e)))?;

        // Validate against schema if available
        if let Some(schema) = &self.schema {
            schema.validate(&value).map_err(|e| {
                let errors: Vec<String> = e.map(|err| err.to_string()).collect();
                DomainError::config_error(format!(
                    "Configuration validation failed:\n{}",
                    errors.join("\n")
                ))
            })?;
        }

        // Deserialize to AgentConfig
        serde_json::from_value(value).map_err(|e| {
            DomainError::config_error(format!("Failed to deserialize configuration: {}", e))
        })
    }

    /// Load configuration from JSON string
    ///
    /// # Arguments
    ///
    /// * `json` - JSON string containing configuration
    ///
    /// # Returns
    ///
    /// Returns the loaded configuration
    pub fn load_from_json(&self, json: &str) -> DomainResult<AgentConfig> {
        // Parse JSON
        let value: Value = serde_json::from_str(json)
            .map_err(|e| DomainError::config_error(format!("Failed to parse JSON: {}", e)))?;

        // Validate against schema if available
        if let Some(schema) = &self.schema {
            schema.validate(&value).map_err(|e| {
                let errors: Vec<String> = e.map(|err| err.to_string()).collect();
                DomainError::config_error(format!(
                    "Configuration validation failed:\n{}",
                    errors.join("\n")
                ))
            })?;
        }

        // Deserialize to AgentConfig
        serde_json::from_value(value).map_err(|e| {
            DomainError::config_error(format!("Failed to deserialize configuration: {}", e))
        })
    }

    /// Validate configuration against schema
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration to validate
    ///
    /// # Returns
    ///
    /// Returns Ok if configuration is valid, otherwise returns an error
    pub fn validate(&self, config: &AgentConfig) -> DomainResult<()> {
        if let Some(schema) = &self.schema {
            let value = serde_json::to_value(config).map_err(|e| {
                DomainError::config_error(format!("Failed to serialize configuration: {}", e))
            })?;

            schema.validate(&value).map_err(|e| {
                let errors: Vec<String> = e.map(|err| err.to_string()).collect();
                DomainError::config_error(format!(
                    "Configuration validation failed:\n{}",
                    errors.join("\n")
                ))
            })?;
        }

        Ok(())
    }

    /// Load the built-in domain schema
    ///
    /// # Returns
    ///
    /// Returns a ConfigLoader with the built-in schema
    pub fn with_builtin_schema() -> DomainResult<Self> {
        // Try to load schema from file, fall back to embedded schema
        let schema_json = Self::load_builtin_schema_json()?;
        Self::with_schema(&schema_json)
    }

    /// Load the built-in schema JSON
    fn load_builtin_schema_json() -> DomainResult<String> {
        // Try multiple possible paths
        let possible_paths = vec![
            "config/schemas/domain.schema.json",
            "../../../config/schemas/domain.schema.json",
            "../../../../config/schemas/domain.schema.json",
        ];

        for path in possible_paths {
            if let Ok(content) = fs::read_to_string(path) {
                return Ok(content);
            }
        }

        // If file not found, return embedded schema
        Ok(Self::embedded_schema().to_string())
    }

    /// Embedded domain schema (fallback)
    fn embedded_schema() -> &'static str {
        r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Domain Agent Configuration Schema",
  "description": "Schema for domain-specific agent configuration files",
  "type": "object",
  "required": [
    "domain",
    "name",
    "description",
    "capabilities"
  ],
  "properties": {
    "domain": {
      "type": "string",
      "description": "Domain identifier (e.g., 'web', 'backend', 'devops')",
      "minLength": 1,
      "maxLength": 100,
      "pattern": "^[a-z0-9-]+$"
    },
    "name": {
      "type": "string",
      "description": "Human-readable agent name",
      "minLength": 1,
      "maxLength": 200
    },
    "description": {
      "type": "string",
      "description": "Detailed description of the agent and its purpose",
      "minLength": 1,
      "maxLength": 1000
    },
    "capabilities": {
      "type": "array",
      "description": "List of capabilities this agent provides",
      "minItems": 1,
      "items": {
        "type": "object",
        "required": [
          "name",
          "description",
          "technologies"
        ],
        "properties": {
          "name": {
            "type": "string",
            "description": "Capability name",
            "minLength": 1,
            "maxLength": 200
          },
          "description": {
            "type": "string",
            "description": "Capability description",
            "minLength": 1,
            "maxLength": 500
          },
          "technologies": {
            "type": "array",
            "description": "Technologies supported by this capability",
            "minItems": 1,
            "items": {
              "type": "string",
              "minLength": 1,
              "maxLength": 100
            }
          }
        },
        "additionalProperties": false
      }
    },
    "best_practices": {
      "type": "array",
      "description": "List of best practices for this domain",
      "default": [],
      "items": {
        "type": "object",
        "required": [
          "title",
          "description",
          "technologies",
          "implementation"
        ],
        "properties": {
          "title": {
            "type": "string",
            "description": "Best practice title",
            "minLength": 1,
            "maxLength": 200
          },
          "description": {
            "type": "string",
            "description": "Best practice description",
            "minLength": 1,
            "maxLength": 500
          },
          "technologies": {
            "type": "array",
            "description": "Technologies this practice applies to",
            "minItems": 1,
            "items": {
              "type": "string",
              "minLength": 1,
              "maxLength": 100
            }
          },
          "implementation": {
            "type": "string",
            "description": "Implementation guidance",
            "minLength": 1,
            "maxLength": 1000
          }
        },
        "additionalProperties": false
      }
    },
    "technology_recommendations": {
      "type": "array",
      "description": "List of technology recommendations",
      "default": [],
      "items": {
        "type": "object",
        "required": [
          "technology",
          "use_cases",
          "pros",
          "cons",
          "alternatives"
        ],
        "properties": {
          "technology": {
            "type": "string",
            "description": "Technology name",
            "minLength": 1,
            "maxLength": 100
          },
          "use_cases": {
            "type": "array",
            "description": "Use cases for this technology",
            "minItems": 1,
            "items": {
              "type": "string",
              "minLength": 1,
              "maxLength": 200
            }
          },
          "pros": {
            "type": "array",
            "description": "Advantages of this technology",
            "minItems": 1,
            "items": {
              "type": "string",
              "minLength": 1,
              "maxLength": 300
            }
          },
          "cons": {
            "type": "array",
            "description": "Disadvantages of this technology",
            "minItems": 1,
            "items": {
              "type": "string",
              "minLength": 1,
              "maxLength": 300
            }
          },
          "alternatives": {
            "type": "array",
            "description": "Alternative technologies",
            "minItems": 1,
            "items": {
              "type": "string",
              "minLength": 1,
              "maxLength": 100
            }
          }
        },
        "additionalProperties": false
      }
    },
    "patterns": {
      "type": "array",
      "description": "List of design patterns for this domain",
      "default": [],
      "items": {
        "type": "object",
        "required": [
          "name",
          "description",
          "technologies",
          "use_cases"
        ],
        "properties": {
          "name": {
            "type": "string",
            "description": "Pattern name",
            "minLength": 1,
            "maxLength": 200
          },
          "description": {
            "type": "string",
            "description": "Pattern description",
            "minLength": 1,
            "maxLength": 500
          },
          "technologies": {
            "type": "array",
            "description": "Technologies this pattern applies to",
            "minItems": 1,
            "items": {
              "type": "string",
              "minLength": 1,
              "maxLength": 100
            }
          },
          "use_cases": {
            "type": "array",
            "description": "Use cases for this pattern",
            "minItems": 1,
            "items": {
              "type": "string",
              "minLength": 1,
              "maxLength": 200
            }
          }
        },
        "additionalProperties": false
      }
    },
    "anti_patterns": {
      "type": "array",
      "description": "List of anti-patterns to avoid",
      "default": [],
      "items": {
        "type": "object",
        "required": [
          "name",
          "description",
          "why_avoid",
          "better_alternative"
        ],
        "properties": {
          "name": {
            "type": "string",
            "description": "Anti-pattern name",
            "minLength": 1,
            "maxLength": 200
          },
          "description": {
            "type": "string",
            "description": "Anti-pattern description",
            "minLength": 1,
            "maxLength": 500
          },
          "why_avoid": {
            "type": "string",
            "description": "Why this anti-pattern should be avoided",
            "minLength": 1,
            "maxLength": 500
          },
          "better_alternative": {
            "type": "string",
            "description": "Better alternative to use instead",
            "minLength": 1,
            "maxLength": 500
          }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"#
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_yaml() -> String {
        r#"
domain: web
name: "Web Development Agent"
description: "Specialized agent for web development"
capabilities:
  - name: "Frontend Framework Selection"
    description: "Recommend frontend frameworks based on project needs"
    technologies: ["React", "Vue", "Angular"]
best_practices: []
technology_recommendations: []
patterns: []
anti_patterns: []
"#
        .to_string()
    }

    fn create_test_json() -> String {
        r#"{
  "domain": "web",
  "name": "Web Development Agent",
  "description": "Specialized agent for web development",
  "capabilities": [
    {
      "name": "Frontend Framework Selection",
      "description": "Recommend frontend frameworks based on project needs",
      "technologies": ["React", "Vue", "Angular"]
    }
  ],
  "best_practices": [],
  "technology_recommendations": [],
  "patterns": [],
  "anti_patterns": []
}"#
        .to_string()
    }

    #[test]
    fn test_config_loader_creation() {
        let loader = ConfigLoader::new();
        assert!(loader.schema.is_none());
    }

    #[test]
    fn test_config_loader_default() {
        let loader = ConfigLoader::default();
        assert!(loader.schema.is_none());
    }

    #[test]
    fn test_load_from_yaml() {
        let loader = ConfigLoader::new();
        let yaml = create_test_yaml();

        let config = loader.load_from_yaml(&yaml).unwrap();
        assert_eq!(config.domain, "web");
        assert_eq!(config.name, "Web Development Agent");
        assert_eq!(config.capabilities.len(), 1);
    }

    #[test]
    fn test_load_from_json() {
        let loader = ConfigLoader::new();
        let json = create_test_json();

        let config = loader.load_from_json(&json).unwrap();
        assert_eq!(config.domain, "web");
        assert_eq!(config.name, "Web Development Agent");
        assert_eq!(config.capabilities.len(), 1);
    }

    #[test]
    fn test_load_from_yaml_invalid() {
        let loader = ConfigLoader::new();
        let yaml = "invalid: yaml: content:";

        assert!(loader.load_from_yaml(yaml).is_err());
    }

    #[test]
    fn test_load_from_json_invalid() {
        let loader = ConfigLoader::new();
        let json = "invalid json";

        assert!(loader.load_from_json(json).is_err());
    }

    #[test]
    fn test_with_builtin_schema() {
        let result = ConfigLoader::with_builtin_schema();
        assert!(result.is_ok());

        let loader = result.unwrap();
        assert!(loader.schema.is_some());
    }

    #[test]
    fn test_validate_with_schema() {
        let loader = ConfigLoader::with_builtin_schema().unwrap();
        let yaml = create_test_yaml();

        let config = loader.load_from_yaml(&yaml).unwrap();
        assert!(loader.validate(&config).is_ok());
    }

    #[test]
    fn test_load_from_yaml_with_schema_validation() {
        let loader = ConfigLoader::with_builtin_schema().unwrap();
        let yaml = create_test_yaml();

        let result = loader.load_from_yaml(&yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_from_json_with_schema_validation() {
        let loader = ConfigLoader::with_builtin_schema().unwrap();
        let json = create_test_json();

        let result = loader.load_from_json(&json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_from_yaml_missing_required_field() {
        let loader = ConfigLoader::with_builtin_schema().unwrap();
        let yaml = r#"
domain: web
name: "Web Agent"
# Missing description and capabilities
"#;

        let result = loader.load_from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_json_missing_required_field() {
        let loader = ConfigLoader::with_builtin_schema().unwrap();
        let json = r#"{
  "domain": "web",
  "name": "Web Agent"
}"#;

        let result = loader.load_from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_yaml_empty_capabilities() {
        let loader = ConfigLoader::with_builtin_schema().unwrap();
        let yaml = r#"
domain: web
name: "Web Agent"
description: "Web development agent"
capabilities: []
"#;

        let result = loader.load_from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_yaml_invalid_domain_format() {
        let loader = ConfigLoader::with_builtin_schema().unwrap();
        let yaml = r#"
domain: "Web Domain!"
name: "Web Agent"
description: "Web development agent"
capabilities:
  - name: "Framework"
    description: "Select frameworks"
    technologies: ["React"]
"#;

        let result = loader.load_from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_yaml_with_best_practices() {
        let loader = ConfigLoader::new();
        let yaml = r#"
domain: web
name: "Web Agent"
description: "Web development agent"
capabilities:
  - name: "Framework"
    description: "Select frameworks"
    technologies: ["React"]
best_practices:
  - title: "Component-Based Architecture"
    description: "Use components"
    technologies: ["React"]
    implementation: "Break UI into components"
"#;

        let config = loader.load_from_yaml(yaml).unwrap();
        assert_eq!(config.best_practices.len(), 1);
        assert_eq!(
            config.best_practices[0].title,
            "Component-Based Architecture"
        );
    }

    #[test]
    fn test_load_from_yaml_with_tech_recommendations() {
        let loader = ConfigLoader::new();
        let yaml = r#"
domain: web
name: "Web Agent"
description: "Web development agent"
capabilities:
  - name: "Framework"
    description: "Select frameworks"
    technologies: ["React"]
technology_recommendations:
  - technology: "React"
    use_cases: ["SPAs"]
    pros: ["Ecosystem"]
    cons: ["Learning curve"]
    alternatives: ["Vue"]
"#;

        let config = loader.load_from_yaml(yaml).unwrap();
        assert_eq!(config.technology_recommendations.len(), 1);
        assert_eq!(config.technology_recommendations[0].technology, "React");
    }

    #[test]
    fn test_load_from_yaml_with_patterns() {
        let loader = ConfigLoader::new();
        let yaml = r#"
domain: web
name: "Web Agent"
description: "Web development agent"
capabilities:
  - name: "Framework"
    description: "Select frameworks"
    technologies: ["React"]
patterns:
  - name: "Component Pattern"
    description: "Component-based architecture"
    technologies: ["React"]
    use_cases: ["UI development"]
"#;

        let config = loader.load_from_yaml(yaml).unwrap();
        assert_eq!(config.patterns.len(), 1);
        assert_eq!(config.patterns[0].name, "Component Pattern");
    }

    #[test]
    fn test_load_from_yaml_with_anti_patterns() {
        let loader = ConfigLoader::new();
        let yaml = r#"
domain: web
name: "Web Agent"
description: "Web development agent"
capabilities:
  - name: "Framework"
    description: "Select frameworks"
    technologies: ["React"]
anti_patterns:
  - name: "God Component"
    description: "Component that does too much"
    why_avoid: "Violates SRP"
    better_alternative: "Break into smaller components"
"#;

        let config = loader.load_from_yaml(yaml).unwrap();
        assert_eq!(config.anti_patterns.len(), 1);
        assert_eq!(config.anti_patterns[0].name, "God Component");
    }

    #[test]
    fn test_validate_config_without_schema() {
        let loader = ConfigLoader::new();
        let config = AgentConfig {
            domain: "web".to_string(),
            name: "Web Agent".to_string(),
            description: "Web development agent".to_string(),
            capabilities: vec![],
            best_practices: vec![],
            technology_recommendations: vec![],
            patterns: vec![],
            anti_patterns: vec![],
        };

        // Should pass validation even without schema
        assert!(loader.validate(&config).is_ok());
    }

    #[test]
    fn test_load_from_yaml_with_all_fields() {
        let loader = ConfigLoader::new();
        let yaml = r#"
domain: backend
name: "Backend Agent"
description: "Backend development agent"
capabilities:
  - name: "API Design"
    description: "API design patterns"
    technologies: ["REST", "GraphQL"]
best_practices:
  - title: "API Versioning"
    description: "Maintain backward compatibility"
    technologies: ["REST"]
    implementation: "Use URL versioning"
technology_recommendations:
  - technology: "PostgreSQL"
    use_cases: ["Relational data"]
    pros: ["Reliable"]
    cons: ["Vertical scaling"]
    alternatives: ["MySQL"]
patterns:
  - name: "MVC Pattern"
    description: "Model-View-Controller"
    technologies: ["Django"]
    use_cases: ["Web applications"]
anti_patterns:
  - name: "God Object"
    description: "Class that does too much"
    why_avoid: "Violates SRP"
    better_alternative: "Break into smaller classes"
"#;

        let config = loader.load_from_yaml(yaml).unwrap();
        assert_eq!(config.domain, "backend");
        assert_eq!(config.capabilities.len(), 1);
        assert_eq!(config.best_practices.len(), 1);
        assert_eq!(config.technology_recommendations.len(), 1);
        assert_eq!(config.patterns.len(), 1);
        assert_eq!(config.anti_patterns.len(), 1);
    }
}
