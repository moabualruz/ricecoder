//! RiceCoder Hooks System
//!
//! Event-driven automation through a registry of hooks that trigger on specific events.
//!
//! # Overview
//!
//! The Hooks System enables users to define automated actions that trigger on specific events.
//! Hooks can execute shell commands, call ricecoder tools with parameter binding, send prompts
//! to AI assistants, or trigger other hooks in chains.
//!
//! # Architecture
//!
//! The system consists of four main components:
//!
//! 1. **Hook Registry** (`registry`): Stores and manages hooks
//! 2. **Event Dispatcher** (`dispatcher`): Routes events to matching hooks
//! 3. **Hook Executor** (`executor`): Executes hook actions
//! 4. **Configuration** (`config`): Loads and manages hook configuration
//!
//! # Quick Start
//!
//! ```ignore
//! use ricecoder_hooks::{
//!     InMemoryHookRegistry, Hook, Event, EventContext, Action, CommandAction,
//! };
//!
//! // Create a registry
//! let mut registry = InMemoryHookRegistry::new();
//!
//! // Create a hook
//! let hook = Hook {
//!     id: "format-hook".to_string(),
//!     name: "Format on save".to_string(),
//!     event: "file_modified".to_string(),
//!     action: Action::Command(CommandAction {
//!         command: "prettier".to_string(),
//!         args: vec!["{{file_path}}".to_string()],
//!         timeout_ms: Some(5000),
//!         capture_output: true,
//!     }),
//!     enabled: true,
//!     tags: vec!["formatting".to_string()],
//!     metadata: serde_json::json!({}),
//!     condition: None,
//! };
//!
//! // Register the hook
//! let hook_id = registry.register_hook(hook)?;
//! println!("Registered hook: {}", hook_id);
//!
//! // Create an event
//! let event = Event {
//!     event_type: "file_modified".to_string(),
//!     context: EventContext {
//!         data: serde_json::json!({
//!             "file_path": "/path/to/file.ts",
//!             "old_hash": "abc123",
//!             "new_hash": "def456",
//!         }),
//!         metadata: serde_json::json!({}),
//!     },
//!     timestamp: std::time::SystemTime::now(),
//! };
//!
//! // Dispatch the event (hooks will be triggered)
//! // dispatcher.dispatch_event(event)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Configuration
//!
//! Hooks are configured in YAML files (`.ricecoder/hooks.yaml`):
//!
//! ```yaml
//! hooks:
//!   - name: "Format on save"
//!     event: "file_modified"
//!     action:
//!       type: "command"
//!       command: "prettier"
//!       args:
//!         - "--write"
//!         - "{{file_path}}"
//!     enabled: true
//! ```
//!
//! # Action Types
//!
//! Hooks support four action types:
//!
//! - **Command**: Execute shell commands
//! - **Tool Call**: Call ricecoder tools with parameter binding
//! - **AI Prompt**: Send prompts to AI assistants
//! - **Chain**: Execute multiple hooks in sequence
//!
//! # Events
//!
//! The system emits events for:
//!
//! - File operations (created, modified, deleted, renamed, moved, read)
//! - Directory operations (created, deleted)
//! - System events (test passed/failed, generation complete, build complete, deployment complete)
//!
//! # Variable Substitution
//!
//! Variables are substituted in hook actions using `{{variable_name}}` syntax:
//!
//! ```yaml
//! action:
//!   type: "command"
//!   command: "echo"
//!   args:
//!     - "File: {{file_path}}"
//!     - "Size: {{file_size}}"
//! ```
//!
//! Available variables depend on the event type. See the `events` module for details.
//!
//! # Error Handling
//!
//! All operations return `Result<T>` which is an alias for `std::result::Result<T, HooksError>`.
//! Errors are explicit and provide context for debugging.
//!
//! # Thread Safety
//!
//! All components are thread-safe (`Send + Sync`) and can be used in concurrent contexts.

pub mod cli;
pub mod config;
pub mod dispatcher;
pub mod error;
pub mod events;
pub mod executor;
pub mod registry;
pub mod types;

// Re-export public types
pub use cli::{HookCli, HookCommand};
pub use error::{HooksError, Result};
pub use events::{
    BuildFailedEvent, BuildSuccessEvent, CustomEvent, DeploymentCompleteEvent,
    DirectoryOperationEvent, FileOperationEvent, FileSavedEvent, FileSystemMonitor,
    GenerationCompleteEvent, RefactoringCompleteEvent, ReviewCompleteEvent, SystemEvent,
    TestFailedEvent, TestPassedEvent,
};
pub use registry::{HookRegistry, InMemoryHookRegistry};
pub use types::{
    Action, AiPromptAction, ChainAction, CommandAction, Condition, Event, EventContext, Hook,
    HookResult, HookStatus, ParameterBindings, ParameterValue, ToolCallAction,
};
