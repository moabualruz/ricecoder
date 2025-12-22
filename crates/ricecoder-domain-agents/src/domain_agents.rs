//! Specialized domain agents

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    error::Result,
    models::{DomainAgentConfig, DomainAgentMetadata},
};

/// Input for domain agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAgentInput {
    /// Domain for this agent
    pub domain: String,
    /// Task description
    pub task: String,
    /// Context or code to analyze
    pub context: String,
    /// Additional parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Output from domain agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAgentOutput {
    /// Domain that processed this
    pub domain: String,
    /// Generated response
    pub response: String,
    /// Suggestions or recommendations
    pub suggestions: Vec<String>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Metadata about the execution
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Trait for domain-specific agents
#[async_trait]
pub trait DomainAgent: Send + Sync {
    /// Get agent metadata
    fn metadata(&self) -> &DomainAgentMetadata;

    /// Get agent configuration
    fn config(&self) -> &DomainAgentConfig;

    /// Execute the agent on input
    async fn execute(&self, input: DomainAgentInput) -> Result<DomainAgentOutput>;

    /// Validate input for this agent
    fn validate_input(&self, input: &DomainAgentInput) -> Result<()>;
}

/// Frontend domain agent
pub struct FrontendAgent {
    metadata: DomainAgentMetadata,
    config: DomainAgentConfig,
}

impl FrontendAgent {
    /// Create a new frontend agent
    pub fn new() -> Self {
        Self {
            metadata: DomainAgentMetadata {
                domain: "frontend".to_string(),
                name: "Frontend Agent".to_string(),
                version: "1.0.0".to_string(),
                capabilities: vec![
                    "component-design".to_string(),
                    "state-management".to_string(),
                    "performance-optimization".to_string(),
                    "accessibility".to_string(),
                ],
                required_tools: vec!["code-analysis".to_string()],
                optional_tools: vec!["performance-profiling".to_string()],
            },
            config: DomainAgentConfig {
                domain: "frontend".to_string(),
                name: "Frontend Agent".to_string(),
                description: "Specialized agent for frontend development".to_string(),
                system_prompt: "You are an expert frontend developer specializing in React, Vue, and Angular. Provide best practices for component design, state management, and performance optimization.".to_string(),
                tools: vec!["code-analysis".to_string(), "performance-profiling".to_string()],
                model: None,
                temperature: Some(0.7),
                max_tokens: Some(2000),
                custom_config: HashMap::new(),
            },
        }
    }
}

impl Default for FrontendAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DomainAgent for FrontendAgent {
    fn metadata(&self) -> &DomainAgentMetadata {
        &self.metadata
    }

    fn config(&self) -> &DomainAgentConfig {
        &self.config
    }

    async fn execute(&self, input: DomainAgentInput) -> Result<DomainAgentOutput> {
        debug!("Frontend agent executing task: {}", input.task);

        self.validate_input(&input)?;

        Ok(DomainAgentOutput {
            domain: "frontend".to_string(),
            response: format!("Frontend analysis for: {}", input.task),
            suggestions: vec![
                "Consider using React hooks for state management".to_string(),
                "Implement proper error boundaries".to_string(),
                "Optimize component re-renders".to_string(),
            ],
            confidence: 0.85,
            metadata: HashMap::new(),
        })
    }

    fn validate_input(&self, input: &DomainAgentInput) -> Result<()> {
        if input.domain != "frontend" {
            return Err(crate::error::DomainAgentError::InvalidConfiguration(
                "Domain mismatch".to_string(),
            ));
        }
        Ok(())
    }
}

/// Backend domain agent
pub struct BackendAgent {
    metadata: DomainAgentMetadata,
    config: DomainAgentConfig,
}

impl BackendAgent {
    /// Create a new backend agent
    pub fn new() -> Self {
        Self {
            metadata: DomainAgentMetadata {
                domain: "backend".to_string(),
                name: "Backend Agent".to_string(),
                version: "1.0.0".to_string(),
                capabilities: vec![
                    "api-design".to_string(),
                    "database-optimization".to_string(),
                    "security".to_string(),
                    "scalability".to_string(),
                ],
                required_tools: vec!["code-analysis".to_string()],
                optional_tools: vec!["performance-profiling".to_string()],
            },
            config: DomainAgentConfig {
                domain: "backend".to_string(),
                name: "Backend Agent".to_string(),
                description: "Specialized agent for backend development".to_string(),
                system_prompt: "You are an expert backend developer specializing in Node.js, Python, and Go. Provide best practices for API design, database optimization, and security.".to_string(),
                tools: vec!["code-analysis".to_string(), "performance-profiling".to_string()],
                model: None,
                temperature: Some(0.7),
                max_tokens: Some(2000),
                custom_config: HashMap::new(),
            },
        }
    }
}

impl Default for BackendAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DomainAgent for BackendAgent {
    fn metadata(&self) -> &DomainAgentMetadata {
        &self.metadata
    }

    fn config(&self) -> &DomainAgentConfig {
        &self.config
    }

    async fn execute(&self, input: DomainAgentInput) -> Result<DomainAgentOutput> {
        debug!("Backend agent executing task: {}", input.task);

        self.validate_input(&input)?;

        Ok(DomainAgentOutput {
            domain: "backend".to_string(),
            response: format!("Backend analysis for: {}", input.task),
            suggestions: vec![
                "Implement proper error handling".to_string(),
                "Add database indexing for performance".to_string(),
                "Use connection pooling".to_string(),
            ],
            confidence: 0.88,
            metadata: HashMap::new(),
        })
    }

    fn validate_input(&self, input: &DomainAgentInput) -> Result<()> {
        if input.domain != "backend" {
            return Err(crate::error::DomainAgentError::InvalidConfiguration(
                "Domain mismatch".to_string(),
            ));
        }
        Ok(())
    }
}

/// DevOps domain agent
pub struct DevOpsAgent {
    metadata: DomainAgentMetadata,
    config: DomainAgentConfig,
}

impl DevOpsAgent {
    /// Create a new DevOps agent
    pub fn new() -> Self {
        Self {
            metadata: DomainAgentMetadata {
                domain: "devops".to_string(),
                name: "DevOps Agent".to_string(),
                version: "1.0.0".to_string(),
                capabilities: vec![
                    "infrastructure-design".to_string(),
                    "deployment".to_string(),
                    "monitoring".to_string(),
                    "security".to_string(),
                ],
                required_tools: vec!["infrastructure-analysis".to_string()],
                optional_tools: vec!["monitoring-setup".to_string()],
            },
            config: DomainAgentConfig {
                domain: "devops".to_string(),
                name: "DevOps Agent".to_string(),
                description: "Specialized agent for DevOps and infrastructure".to_string(),
                system_prompt: "You are an expert DevOps engineer specializing in Kubernetes, Docker, and cloud infrastructure. Provide best practices for deployment, monitoring, and security.".to_string(),
                tools: vec!["infrastructure-analysis".to_string(), "monitoring-setup".to_string()],
                model: None,
                temperature: Some(0.7),
                max_tokens: Some(2000),
                custom_config: HashMap::new(),
            },
        }
    }
}

impl Default for DevOpsAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DomainAgent for DevOpsAgent {
    fn metadata(&self) -> &DomainAgentMetadata {
        &self.metadata
    }

    fn config(&self) -> &DomainAgentConfig {
        &self.config
    }

    async fn execute(&self, input: DomainAgentInput) -> Result<DomainAgentOutput> {
        debug!("DevOps agent executing task: {}", input.task);

        self.validate_input(&input)?;

        Ok(DomainAgentOutput {
            domain: "devops".to_string(),
            response: format!("DevOps analysis for: {}", input.task),
            suggestions: vec![
                "Implement infrastructure as code".to_string(),
                "Set up proper monitoring and alerting".to_string(),
                "Use container orchestration".to_string(),
            ],
            confidence: 0.82,
            metadata: HashMap::new(),
        })
    }

    fn validate_input(&self, input: &DomainAgentInput) -> Result<()> {
        if input.domain != "devops" {
            return Err(crate::error::DomainAgentError::InvalidConfiguration(
                "Domain mismatch".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_frontend_agent_creation() {
        let agent = FrontendAgent::new();
        assert_eq!(agent.metadata().domain, "frontend");
        assert_eq!(agent.config().domain, "frontend");
    }

    #[tokio::test]
    async fn test_backend_agent_creation() {
        let agent = BackendAgent::new();
        assert_eq!(agent.metadata().domain, "backend");
        assert_eq!(agent.config().domain, "backend");
    }

    #[tokio::test]
    async fn test_devops_agent_creation() {
        let agent = DevOpsAgent::new();
        assert_eq!(agent.metadata().domain, "devops");
        assert_eq!(agent.config().domain, "devops");
    }

    #[tokio::test]
    async fn test_frontend_agent_execution() {
        let agent = FrontendAgent::new();
        let input = DomainAgentInput {
            domain: "frontend".to_string(),
            task: "Design a React component".to_string(),
            context: "Component for user profile".to_string(),
            parameters: HashMap::new(),
        };

        let output = agent.execute(input).await.unwrap();
        assert_eq!(output.domain, "frontend");
        assert!(!output.suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_backend_agent_execution() {
        let agent = BackendAgent::new();
        let input = DomainAgentInput {
            domain: "backend".to_string(),
            task: "Design an API endpoint".to_string(),
            context: "User management API".to_string(),
            parameters: HashMap::new(),
        };

        let output = agent.execute(input).await.unwrap();
        assert_eq!(output.domain, "backend");
        assert!(!output.suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_devops_agent_execution() {
        let agent = DevOpsAgent::new();
        let input = DomainAgentInput {
            domain: "devops".to_string(),
            task: "Set up deployment pipeline".to_string(),
            context: "Kubernetes cluster".to_string(),
            parameters: HashMap::new(),
        };

        let output = agent.execute(input).await.unwrap();
        assert_eq!(output.domain, "devops");
        assert!(!output.suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_domain_mismatch_validation() {
        let agent = FrontendAgent::new();
        let input = DomainAgentInput {
            domain: "backend".to_string(),
            task: "Design a React component".to_string(),
            context: "Component for user profile".to_string(),
            parameters: HashMap::new(),
        };

        let result = agent.execute(input).await;
        assert!(result.is_err());
    }
}
