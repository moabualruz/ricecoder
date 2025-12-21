//! External Language Server Protocol (LSP) integration for RiceCoder
//!
//! This crate provides integration with external LSP servers to provide real semantic
//! intelligence for code completion, diagnostics, hover, and navigation across multiple
//! programming languages.
//!
//! # Features
//!
//! - **Configuration-Driven**: Support unlimited LSP servers through YAML configuration
//! - **Process Management**: Automatic spawning, monitoring, and restart of LSP servers
//! - **Output Mapping**: Transform LSP server responses to ricecoder models via configuration
//! - **Graceful Degradation**: Fall back to internal providers when external LSP unavailable
//! - **Multi-Language Support**: Pre-configured for Rust, TypeScript, Python, Go, Java, and more
//!
//! # Architecture
//!
//! The external LSP integration follows a layered architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                         Ricecoder LSP Proxy                              │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐ │
//! │  │  LSP Server     │  │  Request        │  │  Response               │ │
//! │  │  Registry       │  │  Router         │  │  Merger                 │ │
//! │  │  (Config)       │  │  (Language)     │  │  (External + Internal)  │ │
//! │  └────────┬────────┘  └────────┬────────┘  └────────────┬────────────┘ │
//! │           │                    │                        │              │
//! │  ┌────────▼────────────────────▼────────────────────────▼────────────┐ │
//! │  │                    External LSP Client Pool                        │ │
//! │  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │ │
//! │  │  │ rust-analyzer│  │  tsserver    │  │   pylsp      │  ...        │ │
//! │  │  │   Client     │  │   Client     │  │   Client     │             │ │
//! │  │  └──────────────┘  └──────────────┘  └──────────────┘             │ │
//! │  └───────────────────────────────────────────────────────────────────┘ │
//! │                                                                         │
//! │  ┌─────────────────────────────────────────────────────────────────┐   │
//! │  │                    Process Manager                               │   │
//! │  │  - Spawn/terminate LSP server processes                         │   │
//! │  │  - Health monitoring and auto-restart                           │   │
//! │  │  - Resource management (memory, CPU limits)                     │   │
//! │  └─────────────────────────────────────────────────────────────────┘   │
//! │                                                                         │
//! │  ┌─────────────────────────────────────────────────────────────────┐   │
//! │  │                    Fallback Provider                             │   │
//! │  │  - Internal completion (existing ricecoder-completion)          │   │
//! │  │  - Internal diagnostics (existing ricecoder-lsp)                │   │
//! │  │  - Used when external LSP unavailable                           │   │
//! │  └─────────────────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Module Organization
//!
//! - `registry`: LSP server registry and configuration management
//! - `client`: LSP client communication and protocol handling
//! - `process`: LSP server process management
//! - `mapping`: Output mapping and transformation
//! - `merger`: Response merging from multiple sources
//! - `error`: Error types and result types
//! - `types`: Core data structures

pub mod client;
pub mod error;
pub mod mapping;
pub mod merger;
pub mod process;
pub mod registry;
pub mod semantic;
pub mod storage_integration;
pub mod types;

// Re-export public API
pub use client::{
    CapabilityNegotiator, ClientCapabilities, JsonRpcError, JsonRpcHandler, JsonRpcMessage,
    JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, LspConnection, PendingRequest, RequestId,
    ServerCapabilities,
};
pub use error::{ExternalLspError, Result};
pub use mapping::{
    CompletionMapper, DiagnosticsMapper, HoverMapper, JsonPathParser, OutputTransformer,
};
pub use merger::{CompletionMerger, DiagnosticsMerger, HoverMerger};
pub use process::{ClientPool, HealthChecker, ProcessManager};
pub use registry::{ConfigLoader, DefaultServerConfigs, ServerDiscovery};
pub use semantic::SemanticFeatures;
pub use storage_integration::StorageConfigLoader;
pub use types::{
    ClientState, CompletionMappingRules, DiagnosticsMappingRules, ExternalLspResult,
    GlobalLspSettings, HealthStatus, HoverMappingRules, LspServerConfig, LspServerRegistry,
    MergeConfig, OutputMappingConfig, ResultSource,
};
