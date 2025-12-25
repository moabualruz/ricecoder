//! RiceCoder Enhanced Tools
//!
//! Provides webfetch, patch, todo, and web search tools with hybrid MCP provider architecture.
//!
//! # Overview
//!
//! Enhanced Tools implements a hybrid provider pattern where built-in implementations provide
//! default functionality, while optional MCP servers can override or extend behavior for
//! advanced use cases.
//!
//! # Architecture
//!
//! All tools follow a provider priority chain:
//!
//! 1. **MCP Server** (if configured and available) - Custom implementations via external MCP servers
//! 2. **Built-in Implementation** (fallback) - Default ricecoder implementation
//! 3. **Error** (if both unavailable) - Report that tool is not available
//!
//! This enables:
//! - Out-of-the-box functionality with built-in tools
//! - Advanced customization via custom MCP servers
//! - Graceful fallback when MCP is unavailable
//! - Zero configuration required for basic use
//!
//! # Modules
//!
//! - [`error`] - Error types with context and suggestions
//! - [`result`] - Result types with metadata about execution
//! - [`provider`] - Provider trait and registry for tool implementations
//! - [`webfetch`] - Webfetch tool for fetching web content
//! - [`patch`] - Patch tool for applying unified diff patches
//! - [`todo`] - Todo tools for managing task lists
//! - [`search`] - Web search tool for searching the web
//!
//! # Example
//!
//! ```ignore
//! use ricecoder_tools::provider::ProviderRegistry;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let registry = ProviderRegistry::new();
//!
//!     // Register providers
//!     // registry.register_builtin_provider("webfetch", builtin_provider).await;
//!
//!     // Get provider and execute
//!     // let provider = registry.get_provider("webfetch").await?;
//!     // let result = provider.execute("https://example.com").await?;
//!
//!     Ok(())
//! }
//! ```

pub mod batch;
pub mod di;
pub mod edit;
pub mod error;
pub mod patch;
pub mod provider;
pub mod read;
pub mod result;
pub mod search;
pub mod todo;
pub mod webfetch;

// Re-export commonly used types
pub use edit::{
    BatchFileEditInput, BatchFileEditOutput, FileEditInput, FileEditOutput, FileEditTool,
};
pub use error::ToolError;
pub use provider::{Provider, ProviderRegistry};
pub use read::{
    BatchFileReadInput, BatchFileReadOutput, ContentFilter, FileReadInput, FileReadOutput,
    FileReadResult, FileReadTool,
};
pub use result::{ResultMetadata, ToolErrorInfo, ToolResult};
pub use search::{SearchInput, SearchOutput, SearchResult, SearchTool};
pub use batch::{BatchInput, BatchOutput, BatchTool, InvocationResult, ToolInvocation};
pub use todo::{
    Todo, TodoPriority, TodoStatus, TodoTools, TodoreadInput, TodoreadOutput, TodowriteInput,
    TodowriteOutput,
};
