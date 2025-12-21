//! Factory for creating domain agents from configuration

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::models::{
    AntiPattern, BestPractice, DomainAgent, DomainCapability, DomainKnowledge, Pattern,
    TechRecommendation,
};
use serde::{Deserialize, Serialize};

/// Configuration for a domain agent
///
/// This struct defines the configuration for creating a domain agent,
/// including capabilities, best practices, and technology recommendations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Domain identifier
    pub domain: String,
    /// Agent name
    pub name: String,
    /// Agent description
    pub description: String,
    /// Capabilities configuration
    pub capabilities: Vec<CapabilityConfig>,
    /// Best practices configuration
    #[serde(default)]
    pub best_practices: Vec<BestPracticeConfig>,
    /// Technology recommendations configuration
    #[serde(default)]
    pub technology_recommendations: Vec<TechRecommendationConfig>,
    /// Patterns configuration
    #[serde(default)]
    pub patterns: Vec<PatternConfig>,
    /// Anti-patterns configuration
    #[serde(default)]
    pub anti_patterns: Vec<AntiPatternConfig>,
}

/// Configuration for a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityConfig {
    /// Capability name
    pub name: String,
    /// Capability description
    pub description: String,
    /// Technologies
    pub technologies: Vec<String>,
}

/// Configuration for a best practice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestPracticeConfig {
    /// Practice title
    pub title: String,
    /// Practice description
    pub description: String,
    /// Technologies
    pub technologies: Vec<String>,
    /// Implementation guidance
    pub implementation: String,
}

/// Configuration for a technology recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechRecommendationConfig {
    /// Technology name
    pub technology: String,
    /// Use cases
    pub use_cases: Vec<String>,
    /// Pros
    pub pros: Vec<String>,
    /// Cons
    pub cons: Vec<String>,
    /// Alternatives
    pub alternatives: Vec<String>,
}

/// Configuration for a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    /// Pattern name
    pub name: String,
    /// Pattern description
    pub description: String,
    /// Technologies
    pub technologies: Vec<String>,
    /// Use cases
    pub use_cases: Vec<String>,
}

/// Configuration for an anti-pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternConfig {
    /// Anti-pattern name
    pub name: String,
    /// Anti-pattern description
    pub description: String,
    /// Why avoid
    pub why_avoid: String,
    /// Better alternative
    pub better_alternative: String,
}

/// Factory for creating domain agents
///
/// This struct creates domain agent instances from configuration,
/// loading domain knowledge and capabilities from configuration files.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::AgentFactory;
///
/// let factory = AgentFactory::new();
/// let config = AgentConfig { /* ... */ };
/// let agent = factory.create_agent("web", &config)?;
/// ```
#[derive(Debug, Clone)]
pub struct AgentFactory;

impl AgentFactory {
    /// Create a new agent factory
    pub fn new() -> Self {
        Self
    }

    /// Create a domain agent from configuration
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    /// * `config` - Agent configuration
    ///
    /// # Returns
    ///
    /// Returns a new domain agent instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let agent = factory.create_agent("web", &config)?;
    /// ```
    pub fn create_agent(&self, domain: &str, config: &AgentConfig) -> DomainResult<DomainAgent> {
        // Validate configuration
        self.validate_config(config)?;

        // Create capabilities
        let capabilities = config
            .capabilities
            .iter()
            .map(|cap| DomainCapability {
                name: cap.name.clone(),
                description: cap.description.clone(),
                technologies: cap.technologies.clone(),
                patterns: vec![],
            })
            .collect();

        // Create best practices
        let best_practices = config
            .best_practices
            .iter()
            .map(|bp| BestPractice {
                title: bp.title.clone(),
                description: bp.description.clone(),
                domain: domain.to_string(),
                technologies: bp.technologies.clone(),
                implementation: bp.implementation.clone(),
            })
            .collect();

        // Create technology recommendations
        let technology_recommendations = config
            .technology_recommendations
            .iter()
            .map(|tr| TechRecommendation {
                technology: tr.technology.clone(),
                domain: domain.to_string(),
                use_cases: tr.use_cases.clone(),
                pros: tr.pros.clone(),
                cons: tr.cons.clone(),
                alternatives: tr.alternatives.clone(),
            })
            .collect();

        // Create patterns
        let patterns = config
            .patterns
            .iter()
            .map(|p| Pattern {
                name: p.name.clone(),
                description: p.description.clone(),
                domain: domain.to_string(),
                technologies: p.technologies.clone(),
                use_cases: p.use_cases.clone(),
            })
            .collect();

        // Create anti-patterns
        let anti_patterns = config
            .anti_patterns
            .iter()
            .map(|ap| AntiPattern {
                name: ap.name.clone(),
                description: ap.description.clone(),
                domain: domain.to_string(),
                why_avoid: ap.why_avoid.clone(),
                better_alternative: ap.better_alternative.clone(),
            })
            .collect();

        // Create knowledge
        let knowledge = DomainKnowledge {
            best_practices,
            technology_recommendations,
            patterns,
            anti_patterns,
        };

        // Create agent
        let agent = DomainAgent {
            id: format!("{}-agent", domain),
            domain: domain.to_string(),
            capabilities,
            knowledge,
        };

        Ok(agent)
    }

    /// Validate agent configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Agent configuration to validate
    ///
    /// # Returns
    ///
    /// Returns Ok if configuration is valid, otherwise returns an error
    pub fn validate_config(&self, config: &AgentConfig) -> DomainResult<()> {
        // Validate domain
        if config.domain.is_empty() {
            return Err(DomainError::config_error("Domain cannot be empty"));
        }

        // Validate name
        if config.name.is_empty() {
            return Err(DomainError::config_error("Agent name cannot be empty"));
        }

        // Validate capabilities
        if config.capabilities.is_empty() {
            return Err(DomainError::config_error(
                "At least one capability is required",
            ));
        }

        // Validate each capability
        for cap in &config.capabilities {
            if cap.name.is_empty() {
                return Err(DomainError::config_error("Capability name cannot be empty"));
            }
            if cap.technologies.is_empty() {
                return Err(DomainError::config_error(
                    "Capability must have at least one technology",
                ));
            }
        }

        Ok(())
    }

    /// Load configuration from JSON
    ///
    /// # Arguments
    ///
    /// * `json` - JSON string containing configuration
    ///
    /// # Returns
    ///
    /// Returns parsed configuration
    pub fn load_from_json(&self, json: &str) -> DomainResult<AgentConfig> {
        serde_json::from_str(json)
            .map_err(|e| DomainError::serialization_error(format!("Failed to parse JSON: {}", e)))
    }

    /// Load configuration from YAML
    ///
    /// # Arguments
    ///
    /// * `yaml` - YAML string containing configuration
    ///
    /// # Returns
    ///
    /// Returns parsed configuration
    ///
    /// Note: YAML support requires the `serde_yaml` crate to be added as a dependency
    pub fn load_from_yaml(&self, _yaml: &str) -> DomainResult<AgentConfig> {
        Err(DomainError::config_error(
            "YAML support requires serde_yaml dependency",
        ))
    }
}

impl Default for AgentFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config(domain: &str) -> AgentConfig {
        AgentConfig {
            domain: domain.to_string(),
            name: format!("{} Agent", domain),
            description: format!("Agent for {} development", domain),
            capabilities: vec![CapabilityConfig {
                name: "Test Capability".to_string(),
                description: "A test capability".to_string(),
                technologies: vec!["Tech1".to_string()],
            }],
            best_practices: vec![],
            technology_recommendations: vec![],
            patterns: vec![],
            anti_patterns: vec![],
        }
    }

    #[test]
    fn test_factory_creation() {
        let factory = AgentFactory::new();
        assert_eq!(std::mem::size_of_val(&factory), 0); // Zero-sized type
    }

    #[test]
    fn test_create_agent() {
        let factory = AgentFactory::new();
        let config = create_test_config("web");

        let agent = factory.create_agent("web", &config).unwrap();

        assert_eq!(agent.domain, "web");
        assert_eq!(agent.id, "web-agent");
        assert_eq!(agent.capabilities.len(), 1);
    }

    #[test]
    fn test_validate_config_valid() {
        let factory = AgentFactory::new();
        let config = create_test_config("web");

        assert!(factory.validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_empty_domain() {
        let factory = AgentFactory::new();
        let mut config = create_test_config("web");
        config.domain = String::new();

        assert!(factory.validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_empty_name() {
        let factory = AgentFactory::new();
        let mut config = create_test_config("web");
        config.name = String::new();

        assert!(factory.validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_no_capabilities() {
        let factory = AgentFactory::new();
        let mut config = create_test_config("web");
        config.capabilities = vec![];

        assert!(factory.validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_empty_capability_name() {
        let factory = AgentFactory::new();
        let mut config = create_test_config("web");
        config.capabilities[0].name = String::new();

        assert!(factory.validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_no_technologies() {
        let factory = AgentFactory::new();
        let mut config = create_test_config("web");
        config.capabilities[0].technologies = vec![];

        assert!(factory.validate_config(&config).is_err());
    }

    #[test]
    fn test_create_agent_with_best_practices() {
        let factory = AgentFactory::new();
        let mut config = create_test_config("web");
        config.best_practices = vec![BestPracticeConfig {
            title: "Practice 1".to_string(),
            description: "Description".to_string(),
            technologies: vec!["React".to_string()],
            implementation: "Implementation".to_string(),
        }];

        let agent = factory.create_agent("web", &config).unwrap();

        assert_eq!(agent.knowledge.best_practices.len(), 1);
        assert_eq!(agent.knowledge.best_practices[0].title, "Practice 1");
    }

    #[test]
    fn test_create_agent_with_tech_recommendations() {
        let factory = AgentFactory::new();
        let mut config = create_test_config("web");
        config.technology_recommendations = vec![TechRecommendationConfig {
            technology: "React".to_string(),
            use_cases: vec!["SPAs".to_string()],
            pros: vec!["Ecosystem".to_string()],
            cons: vec!["Learning curve".to_string()],
            alternatives: vec!["Vue".to_string()],
        }];

        let agent = factory.create_agent("web", &config).unwrap();

        assert_eq!(agent.knowledge.technology_recommendations.len(), 1);
        assert_eq!(
            agent.knowledge.technology_recommendations[0].technology,
            "React"
        );
    }

    #[test]
    fn test_create_agent_with_patterns() {
        let factory = AgentFactory::new();
        let mut config = create_test_config("web");
        config.patterns = vec![PatternConfig {
            name: "Pattern 1".to_string(),
            description: "Description".to_string(),
            technologies: vec!["React".to_string()],
            use_cases: vec!["UI".to_string()],
        }];

        let agent = factory.create_agent("web", &config).unwrap();

        assert_eq!(agent.knowledge.patterns.len(), 1);
        assert_eq!(agent.knowledge.patterns[0].name, "Pattern 1");
    }

    #[test]
    fn test_create_agent_with_anti_patterns() {
        let factory = AgentFactory::new();
        let mut config = create_test_config("web");
        config.anti_patterns = vec![AntiPatternConfig {
            name: "Anti-pattern 1".to_string(),
            description: "Description".to_string(),
            why_avoid: "Reason".to_string(),
            better_alternative: "Alternative".to_string(),
        }];

        let agent = factory.create_agent("web", &config).unwrap();

        assert_eq!(agent.knowledge.anti_patterns.len(), 1);
        assert_eq!(agent.knowledge.anti_patterns[0].name, "Anti-pattern 1");
    }

    #[test]
    fn test_load_from_json() {
        let factory = AgentFactory::new();
        let json = r#"{
            "domain": "web",
            "name": "Web Agent",
            "description": "Web development agent",
            "capabilities": [
                {
                    "name": "Framework Selection",
                    "description": "Select frameworks",
                    "technologies": ["React"]
                }
            ],
            "best_practices": [],
            "technology_recommendations": [],
            "patterns": [],
            "anti_patterns": []
        }"#;

        let config = factory.load_from_json(json).unwrap();
        assert_eq!(config.domain, "web");
        assert_eq!(config.name, "Web Agent");
    }

    #[test]
    fn test_load_from_json_invalid() {
        let factory = AgentFactory::new();
        let json = "invalid json";

        assert!(factory.load_from_json(json).is_err());
    }

    #[test]
    fn test_default_factory() {
        let factory = AgentFactory::default();
        let config = create_test_config("web");

        assert!(factory.create_agent("web", &config).is_ok());
    }

    #[test]
    fn test_create_agent_multiple_capabilities() {
        let factory = AgentFactory::new();
        let mut config = create_test_config("web");
        config.capabilities = vec![
            CapabilityConfig {
                name: "Capability 1".to_string(),
                description: "Description 1".to_string(),
                technologies: vec!["Tech1".to_string()],
            },
            CapabilityConfig {
                name: "Capability 2".to_string(),
                description: "Description 2".to_string(),
                technologies: vec!["Tech2".to_string()],
            },
        ];

        let agent = factory.create_agent("web", &config).unwrap();

        assert_eq!(agent.capabilities.len(), 2);
    }

    #[test]
    fn test_create_agent_preserves_domain() {
        let factory = AgentFactory::new();
        let mut config = create_test_config("backend");
        config.best_practices = vec![BestPracticeConfig {
            title: "Practice 1".to_string(),
            description: "Description".to_string(),
            technologies: vec!["Tech1".to_string()],
            implementation: "Implementation".to_string(),
        }];

        let agent = factory.create_agent("backend", &config).unwrap();

        assert_eq!(agent.domain, "backend");
        assert_eq!(agent.knowledge.best_practices[0].domain, "backend");
    }
}
