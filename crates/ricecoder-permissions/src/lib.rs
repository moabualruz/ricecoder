//! Permissions System for RiceCoder
//!
//! Provides fine-grained access control with allow/ask/deny levels,
//! per-agent overrides, glob patterns, and audit logging.

pub mod agent;
pub mod audit;
pub mod error;
pub mod glob_matcher;
pub mod permission;
pub mod prompt;
pub mod storage;

pub use agent::{AgentExecutionResult, AgentExecutor};
pub use audit::{AuditAction, AuditLogEntry, AuditLogger, AuditResult};
pub use error::{Error, Result};
pub use glob_matcher::GlobMatcher;
pub use permission::{PermissionConfig, PermissionLevel, PermissionManager, ToolPermission};
pub use prompt::{PermissionPrompt, PromptResult, UserDecision};
pub use storage::{FilePermissionRepository, InMemoryPermissionRepository, PermissionRepository};
