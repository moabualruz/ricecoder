//! MCP Integration for RiceCoder
//!
//! Provides Model Context Protocol support for extending RiceCoder with custom tools
//! and service integrations. Includes MCP client implementation, tool registry,
//! permission system integration, and error handling.

pub mod agent_integration;
pub mod client;
pub mod config;
pub mod connection_pool;
pub mod error;
pub mod error_recovery;
pub mod error_reporting;
pub mod executor;
pub mod health_check;
pub mod hot_reload;
pub mod lifecycle;
pub mod marshaler;
pub mod metadata;
pub mod permissions;
pub mod permissions_integration;
pub mod registry;
pub mod storage_integration;

pub use agent_integration::{
    AgentToolCapabilities, ToolDiscovery, ToolExecutionContext, ToolExecutionResult,
    ToolInvoker, ToolWorkflowIntegration,
};
pub use client::MCPClient;
pub use config::{MCPConfig, MCPConfigLoader};
pub use connection_pool::{ConnectionPool, PoolConfig, PooledConnection, PoolStats};
pub use error::{Error, ErrorContext, ErrorLogEntry, Result, ToolError};
pub use error_recovery::{
    BackoffConfig, GracefulDegradationHandler, RecoveryStrategy, RetryHandler,
    determine_recovery_strategy,
};
pub use error_reporting::{ErrorMessageFormatter, ErrorReporter, ErrorStatistics};
pub use executor::CustomToolExecutor;
pub use health_check::{HealthChecker, HealthCheckConfig, HealthStatus, ServerAvailability};
pub use hot_reload::ConfigWatcher;
pub use lifecycle::{ServerLifecycle, ServerLifecycleInfo, ServerState};
pub use marshaler::ToolMarshaler;
pub use metadata::{ParameterMetadata, ToolMetadata, ToolSource};
pub use permissions::{MCPPermissionManager, PermissionLevelConfig, PermissionRule};
pub use permissions_integration::{
    PermissionAwareToolExecution, ToolPermissionChecker, ToolPermissionDecision,
    ToolPermissionEnforcer, ToolPermissionLevel, ToolPermissionPrompt, UserPermissionDecision,
};
pub use registry::ToolRegistry;
pub use storage_integration::{
    JsonToolRegistryStorage, ToolRegistryCache, ToolRegistryPersistence, ToolRegistryStorage,
};
