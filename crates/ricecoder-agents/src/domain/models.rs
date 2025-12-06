//! Data models for domain-specific agents
//!
//! This module contains all the data structures used for domain agent communication,
//! configuration, and result reporting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A domain-specific agent
///
/// This struct represents a specialized agent for a specific development domain
/// (web, backend, DevOps, etc.) with domain-specific capabilities and knowledge.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::DomainAgent;
///
/// let agent = DomainAgent {
///     id: "web-agent".to_string(),
///     domain: "web".to_string(),
///     capabilities: vec![],
///     knowledge: Default::default(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAgent {
    /// Unique agent identifier
    pub id: String,
    /// Domain name (e.g., "web", "backend", "devops")
    pub domain: String,
    /// Agent capabilities
    pub capabilities: Vec<DomainCapability>,
    /// Domain knowledge
    pub knowledge: DomainKnowledge,
}

/// A capability of a domain agent
///
/// This struct represents a specific capability that a domain agent has,
/// including the technologies it supports and patterns it knows about.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::DomainCapability;
///
/// let capability = DomainCapability {
///     name: "Frontend Framework Selection".to_string(),
///     description: "Recommend frontend frameworks based on project needs".to_string(),
///     technologies: vec!["React".to_string(), "Vue".to_string(), "Angular".to_string()],
///     patterns: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCapability {
    /// Capability name
    pub name: String,
    /// Capability description
    pub description: String,
    /// Technologies supported by this capability
    pub technologies: Vec<String>,
    /// Patterns associated with this capability
    pub patterns: Vec<Pattern>,
}

/// Domain-specific knowledge
///
/// This struct contains all the expertise for a specific domain,
/// including best practices, technology recommendations, patterns, and anti-patterns.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::DomainKnowledge;
///
/// let knowledge = DomainKnowledge {
///     best_practices: vec![],
///     technology_recommendations: vec![],
///     patterns: vec![],
///     anti_patterns: vec![],
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainKnowledge {
    /// Best practices for the domain
    pub best_practices: Vec<BestPractice>,
    /// Technology recommendations
    pub technology_recommendations: Vec<TechRecommendation>,
    /// Patterns for the domain
    pub patterns: Vec<Pattern>,
    /// Anti-patterns to avoid
    pub anti_patterns: Vec<AntiPattern>,
}

/// A technology recommendation
///
/// This struct represents a recommendation for a specific technology,
/// including use cases, pros, cons, and alternatives.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::TechRecommendation;
///
/// let recommendation = TechRecommendation {
///     technology: "React".to_string(),
///     domain: "web".to_string(),
///     use_cases: vec!["Single Page Applications".to_string()],
///     pros: vec!["Large ecosystem".to_string()],
///     cons: vec!["Steep learning curve".to_string()],
///     alternatives: vec!["Vue".to_string(), "Angular".to_string()],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechRecommendation {
    /// Technology name
    pub technology: String,
    /// Domain this recommendation applies to
    pub domain: String,
    /// Use cases for this technology
    pub use_cases: Vec<String>,
    /// Pros of using this technology
    pub pros: Vec<String>,
    /// Cons of using this technology
    pub cons: Vec<String>,
    /// Alternative technologies
    pub alternatives: Vec<String>,
}

/// A best practice
///
/// This struct represents a best practice for a specific domain,
/// including implementation guidance and applicable technologies.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::BestPractice;
///
/// let practice = BestPractice {
///     title: "Component-Based Architecture".to_string(),
///     description: "Use component-based architecture for maintainability".to_string(),
///     domain: "web".to_string(),
///     technologies: vec!["React".to_string(), "Vue".to_string()],
///     implementation: "Break UI into small, reusable components".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestPractice {
    /// Practice title
    pub title: String,
    /// Practice description
    pub description: String,
    /// Domain this practice applies to
    pub domain: String,
    /// Technologies this practice applies to
    pub technologies: Vec<String>,
    /// Implementation guidance
    pub implementation: String,
}

/// A pattern
///
/// This struct represents a design or architectural pattern,
/// including its use cases and applicable technologies.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::Pattern;
///
/// let pattern = Pattern {
///     name: "MVC Pattern".to_string(),
///     description: "Model-View-Controller architectural pattern".to_string(),
///     domain: "backend".to_string(),
///     technologies: vec!["Django".to_string(), "Rails".to_string()],
///     use_cases: vec!["Web applications".to_string()],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Pattern name
    pub name: String,
    /// Pattern description
    pub description: String,
    /// Domain this pattern applies to
    pub domain: String,
    /// Technologies this pattern applies to
    pub technologies: Vec<String>,
    /// Use cases for this pattern
    pub use_cases: Vec<String>,
}

/// An anti-pattern
///
/// This struct represents an anti-pattern (something to avoid),
/// including why it should be avoided and better alternatives.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::AntiPattern;
///
/// let anti_pattern = AntiPattern {
///     name: "God Object".to_string(),
///     description: "A class that does too much".to_string(),
///     domain: "backend".to_string(),
///     why_avoid: "Violates single responsibility principle".to_string(),
///     better_alternative: "Break into smaller, focused classes".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPattern {
    /// Anti-pattern name
    pub name: String,
    /// Anti-pattern description
    pub description: String,
    /// Domain this anti-pattern applies to
    pub domain: String,
    /// Why this anti-pattern should be avoided
    pub why_avoid: String,
    /// Better alternative to use instead
    pub better_alternative: String,
}

/// A recommendation from a domain agent
///
/// This struct represents a recommendation provided by a domain agent,
/// including the domain, category, content, and rationale.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::Recommendation;
///
/// let recommendation = Recommendation {
///     domain: "web".to_string(),
///     category: "framework".to_string(),
///     content: "Use React for this project".to_string(),
///     technologies: vec!["React".to_string()],
///     rationale: "React is well-suited for complex UIs".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Domain providing the recommendation
    pub domain: String,
    /// Category of the recommendation
    pub category: String,
    /// Recommendation content
    pub content: String,
    /// Technologies mentioned in the recommendation
    pub technologies: Vec<String>,
    /// Rationale for the recommendation
    pub rationale: String,
}

/// Shared context across domain agents
///
/// This struct maintains cross-domain context that is shared between agents,
/// including project type, tech stack, constraints, and cross-domain state.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::SharedContext;
/// use std::collections::HashMap;
///
/// let context = SharedContext {
///     project_type: "web-application".to_string(),
///     tech_stack: vec!["React".to_string(), "Node.js".to_string()],
///     constraints: vec!["Must support IE11".to_string()],
///     cross_domain_state: HashMap::new(),
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SharedContext {
    /// Project type
    pub project_type: String,
    /// Technology stack
    pub tech_stack: Vec<String>,
    /// Project constraints
    pub constraints: Vec<String>,
    /// Cross-domain state
    pub cross_domain_state: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_agent_creation() {
        let agent = DomainAgent {
            id: "web-agent".to_string(),
            domain: "web".to_string(),
            capabilities: vec![],
            knowledge: DomainKnowledge::default(),
        };

        assert_eq!(agent.id, "web-agent");
        assert_eq!(agent.domain, "web");
        assert!(agent.capabilities.is_empty());
    }

    #[test]
    fn test_domain_capability_creation() {
        let capability = DomainCapability {
            name: "Frontend Framework Selection".to_string(),
            description: "Recommend frontend frameworks".to_string(),
            technologies: vec!["React".to_string(), "Vue".to_string()],
            patterns: vec![],
        };

        assert_eq!(capability.name, "Frontend Framework Selection");
        assert_eq!(capability.technologies.len(), 2);
    }

    #[test]
    fn test_tech_recommendation_creation() {
        let recommendation = TechRecommendation {
            technology: "React".to_string(),
            domain: "web".to_string(),
            use_cases: vec!["SPAs".to_string()],
            pros: vec!["Large ecosystem".to_string()],
            cons: vec!["Steep learning curve".to_string()],
            alternatives: vec!["Vue".to_string()],
        };

        assert_eq!(recommendation.technology, "React");
        assert_eq!(recommendation.domain, "web");
        assert_eq!(recommendation.use_cases.len(), 1);
        assert_eq!(recommendation.pros.len(), 1);
        assert_eq!(recommendation.cons.len(), 1);
        assert_eq!(recommendation.alternatives.len(), 1);
    }

    #[test]
    fn test_best_practice_creation() {
        let practice = BestPractice {
            title: "Component-Based Architecture".to_string(),
            description: "Use component-based architecture".to_string(),
            domain: "web".to_string(),
            technologies: vec!["React".to_string()],
            implementation: "Break UI into components".to_string(),
        };

        assert_eq!(practice.title, "Component-Based Architecture");
        assert_eq!(practice.domain, "web");
    }

    #[test]
    fn test_pattern_creation() {
        let pattern = Pattern {
            name: "MVC Pattern".to_string(),
            description: "Model-View-Controller pattern".to_string(),
            domain: "backend".to_string(),
            technologies: vec!["Django".to_string()],
            use_cases: vec!["Web applications".to_string()],
        };

        assert_eq!(pattern.name, "MVC Pattern");
        assert_eq!(pattern.domain, "backend");
    }

    #[test]
    fn test_anti_pattern_creation() {
        let anti_pattern = AntiPattern {
            name: "God Object".to_string(),
            description: "A class that does too much".to_string(),
            domain: "backend".to_string(),
            why_avoid: "Violates SRP".to_string(),
            better_alternative: "Break into smaller classes".to_string(),
        };

        assert_eq!(anti_pattern.name, "God Object");
        assert_eq!(anti_pattern.domain, "backend");
    }

    #[test]
    fn test_recommendation_creation() {
        let recommendation = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use React".to_string(),
            technologies: vec!["React".to_string()],
            rationale: "Well-suited for complex UIs".to_string(),
        };

        assert_eq!(recommendation.domain, "web");
        assert_eq!(recommendation.category, "framework");
    }

    #[test]
    fn test_shared_context_creation() {
        let context = SharedContext {
            project_type: "web-application".to_string(),
            tech_stack: vec!["React".to_string(), "Node.js".to_string()],
            constraints: vec!["Must support IE11".to_string()],
            cross_domain_state: HashMap::new(),
        };

        assert_eq!(context.project_type, "web-application");
        assert_eq!(context.tech_stack.len(), 2);
        assert_eq!(context.constraints.len(), 1);
    }

    #[test]
    fn test_domain_knowledge_default() {
        let knowledge = DomainKnowledge::default();
        assert!(knowledge.best_practices.is_empty());
        assert!(knowledge.technology_recommendations.is_empty());
        assert!(knowledge.patterns.is_empty());
        assert!(knowledge.anti_patterns.is_empty());
    }

    #[test]
    fn test_shared_context_default() {
        let context = SharedContext::default();
        assert!(context.project_type.is_empty());
        assert!(context.tech_stack.is_empty());
        assert!(context.constraints.is_empty());
        assert!(context.cross_domain_state.is_empty());
    }

    #[test]
    fn test_domain_agent_serialization() {
        let agent = DomainAgent {
            id: "web-agent".to_string(),
            domain: "web".to_string(),
            capabilities: vec![],
            knowledge: DomainKnowledge::default(),
        };

        let json = serde_json::to_string(&agent).expect("serialization failed");
        let deserialized: DomainAgent =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.id, agent.id);
        assert_eq!(deserialized.domain, agent.domain);
    }

    #[test]
    fn test_recommendation_serialization() {
        let recommendation = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use React".to_string(),
            technologies: vec!["React".to_string()],
            rationale: "Well-suited".to_string(),
        };

        let json = serde_json::to_string(&recommendation).expect("serialization failed");
        let deserialized: Recommendation =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.domain, recommendation.domain);
        assert_eq!(deserialized.category, recommendation.category);
    }

    #[test]
    fn test_shared_context_with_state() {
        let mut context = SharedContext::default();
        context
            .cross_domain_state
            .insert("key".to_string(), serde_json::json!("value"));

        assert_eq!(context.cross_domain_state.len(), 1);
        assert_eq!(
            context.cross_domain_state.get("key").unwrap(),
            &serde_json::json!("value")
        );
    }

    #[test]
    fn test_tech_recommendation_multiple_alternatives() {
        let recommendation = TechRecommendation {
            technology: "React".to_string(),
            domain: "web".to_string(),
            use_cases: vec!["SPAs".to_string(), "Complex UIs".to_string()],
            pros: vec!["Ecosystem".to_string(), "Community".to_string()],
            cons: vec!["Learning curve".to_string()],
            alternatives: vec!["Vue".to_string(), "Angular".to_string(), "Svelte".to_string()],
        };

        assert_eq!(recommendation.use_cases.len(), 2);
        assert_eq!(recommendation.pros.len(), 2);
        assert_eq!(recommendation.alternatives.len(), 3);
    }

    #[test]
    fn test_domain_capability_with_patterns() {
        let patterns = vec![Pattern {
            name: "Component Pattern".to_string(),
            description: "Component-based architecture".to_string(),
            domain: "web".to_string(),
            technologies: vec!["React".to_string()],
            use_cases: vec!["UI development".to_string()],
        }];

        let capability = DomainCapability {
            name: "Frontend Framework".to_string(),
            description: "Frontend framework selection".to_string(),
            technologies: vec!["React".to_string()],
            patterns,
        };

        assert_eq!(capability.patterns.len(), 1);
        assert_eq!(capability.patterns[0].name, "Component Pattern");
    }

    #[test]
    fn test_domain_knowledge_with_all_fields() {
        let knowledge = DomainKnowledge {
            best_practices: vec![BestPractice {
                title: "Practice".to_string(),
                description: "Description".to_string(),
                domain: "web".to_string(),
                technologies: vec!["React".to_string()],
                implementation: "Implementation".to_string(),
            }],
            technology_recommendations: vec![TechRecommendation {
                technology: "React".to_string(),
                domain: "web".to_string(),
                use_cases: vec!["SPAs".to_string()],
                pros: vec!["Ecosystem".to_string()],
                cons: vec!["Learning curve".to_string()],
                alternatives: vec!["Vue".to_string()],
            }],
            patterns: vec![Pattern {
                name: "Pattern".to_string(),
                description: "Description".to_string(),
                domain: "web".to_string(),
                technologies: vec!["React".to_string()],
                use_cases: vec!["UI".to_string()],
            }],
            anti_patterns: vec![AntiPattern {
                name: "Anti-pattern".to_string(),
                description: "Description".to_string(),
                domain: "web".to_string(),
                why_avoid: "Reason".to_string(),
                better_alternative: "Alternative".to_string(),
            }],
        };

        assert_eq!(knowledge.best_practices.len(), 1);
        assert_eq!(knowledge.technology_recommendations.len(), 1);
        assert_eq!(knowledge.patterns.len(), 1);
        assert_eq!(knowledge.anti_patterns.len(), 1);
    }
}
