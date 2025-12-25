//! Context Providers Module
//!
//! This module provides various context providers for the TUI,
//! managing state and providing access to global resources.

pub mod args;
pub mod helper;
pub mod local;
pub mod prompt;
pub mod sdk;

pub use args::{Args, ArgsProvider};
pub use helper::{LazyProvider, SimpleProvider};
pub use local::{AgentInfo, LocalProvider, LocalState, McpStatus, ModelId};
pub use prompt::{PromptOps, PromptRef, PromptRefProvider};
pub use sdk::{SdkClient, SdkEvent, SdkProvider};
