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
pub mod context;
pub mod di;
pub mod edit;
pub mod error;
pub mod filetype;
pub mod format;
pub mod locale;
pub mod lsp;
pub mod patch;
pub mod provider;
pub mod read;
pub mod registry;
pub mod result;
pub mod search;
pub mod todo;
pub mod tool;
pub mod webfetch;
pub mod write;

// Re-export commonly used types
pub use batch::{BatchInput, BatchOutput, BatchTool, InvocationResult, ToolInvocation};
pub use context::{MetadataUpdate, ToolContext};
pub use edit::{
    BatchFileEditInput, BatchFileEditOutput, FileEditInput, FileEditOutput, FileEditTool,
};
pub use error::ToolError;
pub use locale::Locale;
pub use lsp::{
    ExternalLspClient, LspError, LspMetadata, LspOperation, LspPosition, LspTool, LspToolInput,
    LspToolOutput,
};
pub use provider::{Provider, ProviderRegistry};
pub use read::{
    BatchFileReadInput, BatchFileReadOutput, ContentFilter, FileReadInput, FileReadOutput,
    FileReadResult, FileReadTool,
};
pub use registry::{AgentPermissions, PluginTool, ToolMetadata, ToolRegistry};
pub use result::{FileAttachment, ResultMetadata, ToolErrorInfo, ToolResult};
pub use search::{SearchInput, SearchOutput, SearchResult, SearchTool};
pub use todo::{
    Todo, TodoPriority, TodoStatus, TodoTools, TodoreadInput, TodoreadOutput, TodowriteInput,
    TodowriteOutput,
};
pub use tool::{
    ParameterSchema, Tool, ToolDefinition, ToolExecutionResult, ToolParameters, ToolWrapper,
};
pub use write::{WriteError, WriteInput, WriteMetadata, WriteOutput, WriteTool};
