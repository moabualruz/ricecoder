//! RiceCoder Sessions Module
//!
//! This module provides multi-session support with persistence, sharing, and background agent execution.
//! Sessions allow developers to run multiple agents in parallel, persist session state, and share sessions with teammates.

pub mod background_agent;
pub mod compliance;
pub mod context;
pub mod error;
pub mod history;
pub mod manager;
pub mod models;
pub mod performance_monitor;
pub mod router;
pub mod session_integration;
pub mod session_manager;
pub mod sessions;
pub mod share;
pub mod store;
pub mod token_estimator;
pub mod tui_session_manager;

// Re-export commonly used types
pub use background_agent::BackgroundAgentManager;
pub use compliance::ComplianceManager;
pub use context::ContextManager;
pub use error::{SessionError, SessionResult};
pub use history::HistoryManager;
pub use manager::{SessionManager, SessionSummary};
pub use models::{
    AgentStatus, BackgroundAgent, CodePart, ComplianceAlertLevel, ComplianceEvent,
    ComplianceEventType, DataErasureRequest, DataExportFormat, DataExportRequest,
    DataMinimizationSettings, DataRetentionPolicy, DataType, EnterpriseSessionAnalytics,
    ErasureReason, FileReferencePart, ImagePart, Message, MessageMetadata, MessagePart,
    MessageRole, PrivacySettings, Session, SessionContext, SessionMode, SessionStatus,
    SharingTrendPoint, ToolInvocationPart, ToolResultPart, ToolStatus,
};
pub use performance_monitor::{
    SessionMetrics, SessionPerformanceMonitor, SessionPerformanceSummary,
};
pub use router::SessionRouter;
pub use session_integration::SessionIntegration;
pub use share::{
    DataClassification, EnterpriseShareMetrics, EnterpriseSharingPolicy, SessionShare,
    ShareAnalyticsData, SharePermissions, ShareService,
};
pub use store::{
    EnterpriseBackupInfo, GarbageCollectionConfig, GarbageCollectionResult, SessionStore,
};
pub use token_estimator::{
    ModelPricing, TokenEstimate, TokenEstimator, TokenLimitStatus, TokenUsage, TokenUsageTracker,
};
pub use tui_session_manager::{TuiSessionData, TuiSessionManager};
