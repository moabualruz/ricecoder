//! Permissions System for RiceCoder
//!
//! Provides fine-grained access control with allow/ask/deny levels,
//! per-agent overrides, glob patterns, and audit logging.

pub mod error;
pub mod permission;
pub mod glob_matcher;
pub mod audit;
pub mod prompt;
pub mod agent;
pub mod storage;

pub use error::{Error, Result};
pub use permission::{PermissionLevel, ToolPermission, PermissionConfig, PermissionManager};
pub use glob_matcher::GlobMatcher;
pub use audit::{AuditLogger, AuditLogEntry};
pub use prompt::{PermissionPrompt, UserDecision, PromptResult};
pub use agent::{AgentExecutor, AgentExecutionResult};
pub use storage::{PermissionRepository, FilePermissionRepository, InMemoryPermissionRepository};
