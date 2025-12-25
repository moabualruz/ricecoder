//! MCP Integration for RiceCoder
#![forbid(unsafe_code)]

//!
//! Provides Model Context Protocol support for extending RiceCoder with custom tools
//! and service integrations. Includes MCP client implementation, tool registry,
//! permission system integration, and error handling.

pub mod agent_integration;
pub mod analytics;
pub mod audit;
pub mod client;
pub mod compliance;
pub mod config;
pub mod connection_pool;
pub mod di;
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
pub mod protocol_validation;
pub mod rbac;
pub mod registry;
pub mod server_management;
pub mod storage_integration;
pub mod tool_execution;
pub mod tool_orchestration;
pub mod transport;

pub use agent_integration::{
    AgentToolCapabilities, ToolDiscovery, ToolInvoker, ToolWorkflowIntegration,
};
pub use analytics::{
    EnterpriseDashboardReport, MCPAnalyticsAggregator, MCPEnterpriseDashboard, MCPUsageReport,
    RealtimeDashboardSnapshot,
};
pub use audit::MCPAuditLogger;
// Re-export batch types from ricecoder-tools (moved for architectural correctness)
// Note: ricecoder-mcp cannot depend on ricecoder-tools due to circular deps
// Consumers should import directly from ricecoder_tools::batch
pub use client::MCPClient;
pub use compliance::{
    ComplianceReport, MCPComplianceMonitor, MCPEnterpriseMonitor, MonitoringReport,
};
pub use config::{MCPConfig, MCPConfigLoader};
pub use connection_pool::{ConnectionPool, PoolConfig, PoolStats, PooledConnection};
pub use error::{Error, ErrorContext, ErrorLogEntry, Result, ToolError};
pub use error_recovery::{
    determine_recovery_strategy, BackoffConfig, GracefulDegradationHandler, RecoveryStrategy,
    RetryHandler,
};
pub use error_reporting::{ErrorMessageFormatter, ErrorReporter, ErrorStatistics};
pub use executor::CustomToolExecutor;
pub use health_check::{HealthCheckConfig, HealthChecker, HealthStatus, ServerAvailability};
pub use hot_reload::ConfigWatcher;
pub use lifecycle::{ServerLifecycle, ServerLifecycleInfo};
pub use marshaler::ToolMarshaler;
pub use metadata::{ParameterMetadata, ToolMetadata, ToolSource};
pub use permissions::{MCPPermissionManager, PermissionLevelConfig, PermissionRule};
pub use permissions_integration::{
    PermissionAwareToolExecution, ToolPermissionChecker, ToolPermissionDecision,
    ToolPermissionEnforcer, ToolPermissionLevel, ToolPermissionPrompt, UserPermissionDecision,
};
pub use protocol_validation::{MCPComplianceChecker, MCPErrorHandler, MCPProtocolValidator};
pub use rbac::{MCPAuthorizationMiddleware, MCRBACManager};
pub use registry::ToolRegistry;
pub use server_management::{
    AuthConfig, AuthType, DiscoveryResult, FileSystemDiscoveryProvider, ServerConfig, ServerHealth,
    ServerManager, ServerRegistration, ServerState,
};
pub use storage_integration::{
    JsonToolRegistryStorage, ToolRegistryCache, ToolRegistryPersistence, ToolRegistryStorage,
};
pub use tool_execution::{
    MCPToolExecutor, ToolExecutionContext, ToolExecutionResult, ToolExecutionStats, ToolExecutor,
    ToolResultProcessor,
};
pub use tool_orchestration::{
    PipelineExecutionContext, PipelineExecutionResult, PipelineStats, PipelineStep,
    ToolOrchestrator, ToolPipeline,
};
pub use transport::{
    HTTPTransport, MCPError, MCPErrorData, MCPMessage, MCPNotification, MCPRequest, MCPResponse,
    MCPTransport, SSETransport, StdioTransport, TransportConfig, TransportFactory, TransportType,
};
