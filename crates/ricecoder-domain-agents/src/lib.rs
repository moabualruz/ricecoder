//! Domain-Specific Agents for RiceCoder
//!
//! This crate provides a framework for specialized agents that focus on specific domains
//! (frontend, backend, DevOps, etc.) with domain-specific knowledge bases and capabilities.
//!
//! # Architecture
//!
//! The framework consists of:
//! - **Domain Agents**: Specialized agents for specific domains (Frontend, Backend, DevOps)
//! - **Knowledge Base**: Domain-specific knowledge entries and best practices
//! - **Registry**: Central registry for discovering and managing domain agents
//! - **Models**: Data structures for domains, agents, and knowledge
//!
//! # Example
//!
//! ```ignore
//! use ricecoder_domain_agents::{DomainAgentRegistryManager, DomainAgentInput};
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create registry with default agents
//!     let registry = DomainAgentRegistryManager::with_defaults();
//!
//!     // Execute frontend agent
//!     let input = DomainAgentInput {
//!         domain: "frontend".to_string(),
//!         task: "Design a React component".to_string(),
//!         context: "User profile component".to_string(),
//!         parameters: HashMap::new(),
//!     };
//!
//!     let output = registry.execute_agent("frontend", input).await?;
//!     println!("Response: {}", output.response);
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

pub mod domain_agents;
pub mod error;
pub mod knowledge_base;
pub mod models;
pub mod registry;

pub use domain_agents::{
    BackendAgent, DevOpsAgent, DomainAgent, DomainAgentInput, DomainAgentOutput, FrontendAgent,
};
pub use error::{DomainAgentError, Result};
pub use knowledge_base::KnowledgeBaseManager;
pub use models::{
    Domain, DomainAgentConfig, DomainAgentMetadata, DomainAgentRegistry, KnowledgeBase,
    KnowledgeEntry,
};
pub use registry::DomainAgentRegistryManager;
