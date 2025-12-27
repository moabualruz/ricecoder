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
//! // Create orchestrator with default dependencies
//! let orchestrator = AgentOrchestrator::with_defaults(registry);
//!
//! // Execute agents
//! let results = orchestrator.execute(tasks).await?;
//! ```

#![warn(missing_docs)]

pub mod agents;
pub mod chat;
pub mod coordinator;
pub mod domain;
pub mod error;
pub mod executor;
pub mod mcp_integration;
pub mod metrics;
pub mod models;
pub mod orchestrator;
pub mod registry;
pub mod scheduler;
pub mod tool_invokers;
pub mod tool_registry;
pub mod tools;
pub mod use_cases;

#[cfg(test)]
mod scheduler_properties;

#[cfg(test)]
mod coordinator_properties;

#[cfg(test)]
mod orchestrator_properties;

pub use agents::{Agent, CodeReviewAgent, WebAgent};
pub use chat::{
    ApprovalCallback, ChatContext, ChatError, ChatMessage, ChatResponse, ChatService,
    ContentBlock, Role, StopReason, ToolApprovalInfo, ToolCall, TrackedFile, Usage,
};
pub use coordinator::AgentCoordinator;
pub use error::AgentError;
pub use executor::{ExecutionConfig, ExecutionResult, ParallelExecutor};
pub use mcp_integration::{
    ExternalToolBackend, ExternalToolIntegrationService, ToolExecutionResult, ToolExecutor,
};
pub use metrics::{AgentStats, ExecutionMetrics, MetricsCollector};
pub use models::{
    AgentConfig, AgentInput, AgentMetadata, AgentMetrics, AgentOutput, AgentTask, Finding,
    Severity, Suggestion, TaskScope, TaskTarget, TaskType,
};
pub use orchestrator::AgentOrchestrator;
pub use registry::AgentRegistry;
pub use scheduler::{AgentScheduler, ExecutionPhase, ExecutionSchedule, TaskDAG};
pub use tool_invokers::{
    ExtensibleToolInvoker, GlobToolInvoker, GrepToolInvoker, ListToolInvoker, PatchToolInvoker,
    ReadToolInvoker, TodoreadToolInvoker, TodowriteToolInvoker, ToolBackend, WebfetchToolInvoker,
    WebsearchToolInvoker, WriteToolInvoker, EditToolInvoker,
};
pub use tool_registry::{ToolInvoker, ToolMetadata, ToolRegistry};
pub use tools::{
    ModelConfig, SessionManager, SubagentType, TaskExecutionContext, TaskParams, TaskProgress,
    TaskResult, TaskTool,
};
pub use use_cases::{
    ConfigureToolBackendUseCase, ExecuteExternalToolUseCase, ProviderCommunityUseCase,
    ProviderFailoverUseCase, ProviderHealthUseCase, ProviderModelUseCase,
    ProviderPerformanceUseCase, ProviderSwitchingUseCase, SessionLifecycleUseCase,
    SessionSharingUseCase, SessionStateManagementUseCase, ToolManagementUseCase,
};
