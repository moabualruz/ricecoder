//! Domain-specific agents module
//!
//! This module provides specialized agents for different development domains
//! (web, backend, DevOps) with domain-specific knowledge and capabilities.
//! Each agent is configured with domain expertise through configuration files,
//! enabling runtime customization without code changes.
//!
//! # Architecture
//!
//! The domain agents system consists of:
//! - **DomainAgent**: Specialized agent for a specific domain
//! - **DomainRegistry**: Discovers and registers domain agents
//! - **AgentFactory**: Creates domain agent instances from configuration
//! - **KnowledgeBase**: Stores domain-specific knowledge and best practices
//! - **SharedContextManager**: Manages cross-domain context sharing
//! - **ConflictDetector**: Detects and reports conflicting recommendations
//! - **DomainCoordinator**: Coordinates multi-agent workflows
//!
//! # Example
//!
//! ```ignore
//! use ricecoder_agents::domain::{DomainRegistry, AgentFactory};
//!
//! // Create registry and factory
//! let registry = DomainRegistry::new();
//! let factory = AgentFactory::new();
//!
//! // Create domain agents from configuration
//! let web_agent = factory.create_agent("web", &config)?;
//! let backend_agent = factory.create_agent("backend", &config)?;
//!
//! // Register agents
//! registry.register_agent("web", web_agent);
//! registry.register_agent("backend", backend_agent);
//! ```

pub mod config_loader;
pub mod conflict;
pub mod conflict_properties;
pub mod context;
pub mod context_sharing_properties;
pub mod coordination_properties;
pub mod coordinator;
pub mod error;
pub mod extensibility_properties;
pub mod factory;
pub mod knowledge;
pub mod knowledge_properties;
pub mod models;
pub mod registry;
pub mod routing_properties;
pub mod sequencing_properties;

pub use config_loader::ConfigLoader;
pub use conflict::{Conflict, ConflictDetector, ConflictReport, ConflictType};
pub use context::SharedContextManager;
pub use coordinator::{
    CoordinatedResponse, DomainCoordinator, DomainRequest, FullStackCoordination, Operation,
};
pub use error::{DomainError, DomainResult};
pub use factory::{
    AgentConfig, AgentFactory, AntiPatternConfig, BestPracticeConfig, CapabilityConfig,
    PatternConfig, TechRecommendationConfig,
};
pub use knowledge::KnowledgeBase;
pub use models::{
    AntiPattern, BestPractice, DomainAgent, DomainCapability, DomainKnowledge, Pattern,
    Recommendation, SharedContext, TechRecommendation,
};
pub use registry::DomainRegistry;
