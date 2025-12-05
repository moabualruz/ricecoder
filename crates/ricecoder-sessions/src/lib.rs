//! RiceCoder Sessions Module
//!
//! This module provides multi-session support with persistence, sharing, and background agent execution.
//! Sessions allow developers to run multiple agents in parallel, persist session state, and share sessions with teammates.

pub mod error;
pub mod models;
pub mod manager;
pub mod store;
pub mod history;
pub mod context;
pub mod router;
pub mod share;
pub mod background_agent;

// Re-export commonly used types
pub use error::{SessionError, SessionResult};
pub use models::{
    Session, SessionStatus, SessionContext, SessionMode,
    Message, MessageRole, MessageMetadata,
    BackgroundAgent, AgentStatus,
};
pub use manager::SessionManager;
pub use store::SessionStore;
pub use history::HistoryManager;
pub use context::ContextManager;
pub use router::SessionRouter;
pub use share::{ShareService, SessionShare, SharePermissions};
pub use background_agent::BackgroundAgentManager;
