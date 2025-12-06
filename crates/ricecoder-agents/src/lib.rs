//! Multi-Agent Framework for RiceCoder
//!
//! This crate provides a framework for specialized agents that perform different tasks
//! (code review, testing, documentation, refactoring) with orchestration capabilities
//! for sequential, parallel, and conditional workflows.
//!
//! # Architecture
//!
//! The framework consists of:
//! - **Agent Trait**: Unified interface for all agents
//! - **AgentRegistry**: Discovers and registers agents at runtime
//! - **AgentScheduler**: Schedules agent execution with dependency management
//! - **AgentCoordinator**: Aggregates and prioritizes results from multiple agents
//! - **AgentOrchestrator**: Central coordinator for agent lifecycle and workflows
//!
//! # Example
//!
//! ```ignore
//! use ricecoder_agents::{Agent, AgentRegistry, AgentOrchestrator};
//!
//! // Create registry and register agents
//! let registry = AgentRegistry::new();
//! // ... register agents ...
//!
//! // Create orchestrator
//! let orchestrator = AgentOrchestrator::new(registry);
//!
//! // Execute agents
//! let results = orchestrator.execute(tasks).await?;
//! ```

#![warn(missing_docs)]

pub mod agents;
pub mod coordinator;
pub mod domain;
pub mod error;
pub mod executor;
pub mod metrics;
pub mod models;
pub mod orchestrator;
pub mod registry;
pub mod scheduler;
pub mod tool_registry;
pub mod tool_invokers;

#[cfg(test)]
mod scheduler_properties;

#[cfg(test)]
mod coordinator_properties;

#[cfg(test)]
mod orchestrator_properties;

pub use agents::{Agent, CodeReviewAgent, WebAgent};
pub use coordinator::AgentCoordinator;
pub use error::AgentError;
pub use executor::{ExecutionConfig, ExecutionResult, ParallelExecutor};
pub use metrics::{AgentStats, ExecutionMetrics, MetricsCollector};
pub use models::{
    AgentConfig, AgentInput, AgentMetadata, AgentMetrics, AgentOutput, AgentTask, Finding,
    Severity, Suggestion, TaskScope, TaskTarget, TaskType,
};
pub use orchestrator::AgentOrchestrator;
pub use registry::AgentRegistry;
pub use scheduler::{AgentScheduler, ExecutionPhase, ExecutionSchedule, TaskDAG};
pub use tool_registry::{ToolInvoker, ToolMetadata, ToolRegistry};
pub use tool_invokers::{
    PatchToolInvoker, TodoreadToolInvoker, TodowriteToolInvoker, WebfetchToolInvoker,
    WebsearchToolInvoker,
};
