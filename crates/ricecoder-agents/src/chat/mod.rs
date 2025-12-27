//! Chat Module - LLM Conversation Service
//!
//! This module provides the core conversation loop for interacting with LLM providers,
//! including tool execution, context management, and approval flows.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                       ChatService                                 │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  - Manages conversation with LLM providers                       │
//! │  - Executes tool calls in a loop                                │
//! │  - Handles user approval for dangerous operations               │
//! │  - Tracks token usage and context window                        │
//! └─────────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                       ChatContext                                 │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  - Token counting and estimation                                 │
//! │  - Message history management                                    │
//! │  - Context window pruning                                        │
//! │  - File tracking                                                │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use ricecoder_agents::chat::{ChatService, ChatContext};
//! use std::sync::Arc;
//!
//! // Create service with provider
//! let service = ChatService::new(provider)
//!     .with_system_prompt("You are a helpful coding assistant.".into())
//!     .with_tool_registry(tool_registry)
//!     .with_auto_approve_tools(false);
//!
//! // Create conversation context
//! let mut context = ChatContext::new(session_id, 128_000); // 128k context
//!
//! // Send message with tool execution
//! let response = service.send_message_with_tools(
//!     &mut context,
//!     "Read the file src/main.rs".into(),
//!     None,
//! ).await?;
//! ```

pub mod context;
pub mod error;
pub mod service;
pub mod types;

// Re-exports for convenience
pub use context::{ChatContext, TrackedFile};
pub use error::{ChatError, Result};
pub use service::{ApprovalCallback, ChatResponse, ChatService, ToolCall};
pub use types::{
    ChatMessage, ContentBlock, Role, StopReason, ToolApprovalInfo, ToolDefinition, Usage,
};
