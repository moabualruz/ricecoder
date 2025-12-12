//! RiceCoder Sessions Module
//!
//! This module provides multi-session support with persistence, sharing, and background agent execution.
//! Sessions allow developers to run multiple agents in parallel, persist session state, and share sessions with teammates.

pub mod background_agent;
pub mod context;
pub mod error;
pub mod history;
pub mod manager;
pub mod models;
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
pub use context::ContextManager;
pub use error::{SessionError, SessionResult};
pub use history::HistoryManager;
pub use manager::SessionManager;
pub use manager::SessionSummary;
pub use models::{
    AgentStatus, BackgroundAgent, CodePart, FileReferencePart, ImagePart, Message, MessageMetadata,
    MessagePart, MessageRole, Session, SessionContext, SessionMode, SessionStatus, ToolInvocationPart,
    ToolResultPart, ToolStatus,
};
pub use router::SessionRouter;
pub use session_integration::SessionIntegration;
pub use share::{SessionShare, SharePermissions, ShareService};
pub use store::SessionStore;
pub use token_estimator::{
    TokenEstimate, TokenEstimator, TokenLimitStatus, TokenUsage, TokenUsageTracker,
};
pub use tui_session_manager::{TuiSessionManager, TuiSessionData};
