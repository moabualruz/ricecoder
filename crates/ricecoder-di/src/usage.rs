//! # RiceCoder DI Container Usage Guide
//!
//! The DI container provides a unified way to wire services across all RiceCoder crates.
//! This allows for consistent dependency injection patterns throughout the entire codebase.
//!
//! ## Feature Flags
//!
//! Enable specific features in your `Cargo.toml` to include services from different crates:
//!
//! ```toml
//! [dependencies]
//! ricecoder-di = { version = "0.1", features = ["full"] }
//! ```
//!
//! Available features:
//! - `storage` - File and memory storage services
//! - `research` - Code analysis and semantic indexing
//! - `workflows` - Workflow execution and management
//! - `execution` - Command execution services
//! - `mcp` - Model Context Protocol services
//! - `tools` - Tool registry and execution
//! - `config` - Configuration management
//! - `activity-log` - Activity logging and monitoring
//! - `orchestration` - Workspace orchestration services
//! - `specs` - Specification management
//! - `undo-redo` - Undo/redo functionality
//! - `vcs` - Version control integration
//! - `permissions` - Permission management and checking
//! - `security` - Security services (access control, encryption)
//! - `cache` - Caching services and strategies
//! - `domain` - Domain services and repositories
//! - `learning` - Machine learning and pattern recognition
//! - `industry` - Industry-specific services (auth, compliance)
//! - `safety` - Safety monitoring and risk assessment
//! - `files` - File management and watching
//! - `themes` - Theme management and loading
//! - `images` - Image processing and analysis
//! - `full` - All features enabled
//!
//! ## Basic Usage
//!
//! ```rust
//! use ricecoder_di::{create_application_container, DIContainer};
//!
//! // Create container with core services (sessions, providers, agents)
//! let container = create_application_container().unwrap();
//!
//! // Resolve services
//! let session_manager = container.resolve::<ricecoder_sessions::SessionManager>().unwrap();
//! let provider_manager = container.resolve::<ricecoder_providers::provider::manager::ProviderManager>().unwrap();
//! ```
//!
//! ## Advanced Usage with Builder Pattern
//!
//! ```rust
//! use ricecoder_di::{DIContainerBuilder, DIContainerBuilderExt};
//!
//! // Build a container with core services.
//! let builder = DIContainerBuilder::new()
//!     .register_infrastructure_services().unwrap()
//!     .register_use_cases().unwrap();
//! let container = builder.build();
//! ```
//!
//! ## Service Registration
//!
//! ### Core Services (Always Available)
//! - `SessionManager` - Session lifecycle management
//! - `SessionStore` - Session persistence
//! - `ShareService` - Session sharing functionality
//! - `ProviderManager` - AI provider orchestration
//! - All use cases from `ricecoder-agents`
//!
//! ### Optional Services
//!
//! #### Storage Services (`storage` feature)
//! ```rust,ignore
//! let storage_manager = container.resolve::<ricecoder_storage::StorageManager>().unwrap();
//! let file_storage = container.resolve::<ricecoder_storage::FileStorage>().unwrap();
//! let memory_storage = container.resolve::<ricecoder_storage::MemoryStorage>().unwrap();
//! ```
//!
//! #### Research Services (`research` feature)
//! ```rust,ignore
//! let research_manager = container.resolve::<ricecoder_research::ResearchManager>().unwrap();
//! let codebase_scanner = container.resolve::<ricecoder_research::CodebaseScanner>().unwrap();
//! let semantic_indexer = container.resolve::<ricecoder_research::SemanticIndexer>().unwrap();
//! ```
//!
//! #### Workflow Services (`workflows` feature)
//! ```rust,ignore
//! let workflow_engine = container.resolve::<ricecoder_workflows::WorkflowEngine>().unwrap();
//! let workflow_manager = container.resolve::<ricecoder_workflows::WorkflowManager>().unwrap();
//! ```
//!
//! #### Execution Services (`execution` feature)
//! ```rust,ignore
//! let execution_engine = container.resolve::<ricecoder_execution::ExecutionEngine>().unwrap();
//! let command_executor = container.resolve::<ricecoder_execution::CommandExecutor>().unwrap();
//! ```
//!
//! #### MCP Services (`mcp` feature)
//! ```rust,ignore
//! let mcp_client = container.resolve::<ricecoder_mcp::MCPClient>().unwrap();
//! let mcp_server = container.resolve::<ricecoder_mcp::MCPServer>().unwrap();
//! ```
//!
//! #### Tool Services (`tools` feature)
//! ```rust,ignore
//! let tool_registry = container.resolve::<ricecoder_tools::ToolRegistry>().unwrap();
//! let tool_executor = container.resolve::<ricecoder_tools::ToolExecutor>().unwrap();
//! ```
//!
//! #### Config Services (`config` feature)
//! ```rust,ignore
//! let config_manager = container.resolve::<ricecoder_config::ConfigManager>().unwrap();
//! let config_loader = container.resolve::<ricecoder_config::ConfigLoader>().unwrap();
//! ```
//!
//! ## Custom Service Registration
//!
//! You can also register your own services:
//!
//! ```rust
//! use ricecoder_di::{DIContainer, register_service};
//!
//! struct MyCustomService;
//! impl MyCustomService { fn new() -> Self { Self } }
//! struct AnotherService;
//! impl AnotherService { fn new() -> Self { Self } }
//!
//! let container = DIContainer::new();
//!
//! // Register a custom service
//! container.register(|_| Ok(std::sync::Arc::new(MyCustomService::new()))).unwrap();
//!
//! // Or use the macro
//! register_service!(container, AnotherService, |_| Ok(std::sync::Arc::new(AnotherService::new()))).unwrap();
//!
//! // Resolve it
//! let service = container.resolve::<MyCustomService>().unwrap();
//! ```
//!
//! ## Best Practices
//!
//! 1. **Use feature flags** to control which services are included
//! 2. **Register services at startup** to avoid runtime errors
//! 3. **Use Arc for shared state** to ensure thread safety
//! 4. **Handle service resolution errors** gracefully
//! 5. **Test with different feature combinations** to ensure compatibility
//!
//! ## Example Application Setup
//!
//! ```rust
//! use ricecoder_di::{DIContainerBuilder, DIContainerBuilderExt};
//!
//! fn setup_container() -> Result<ricecoder_di::DIContainer, ricecoder_di::DIError> {
//!     let builder = DIContainerBuilder::new()
//!         .register_infrastructure_services()?
//!         .register_use_cases()?;
//!     Ok(builder.build())
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let container = setup_container()?;
//!
//!     // Use services...
//!     let session_manager = container.resolve::<ricecoder_sessions::SessionManager>()?;
//!
//!     Ok(())
//! }
//! ```

pub mod usage_examples {}
