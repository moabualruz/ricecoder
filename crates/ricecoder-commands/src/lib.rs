//! RiceCoder Custom Commands System
//!
//! This crate provides a system for defining and managing custom commands in RiceCoder.
//! Commands can be executed with template substitution and output injection into chat.
//!
//! # Examples
//!
//! ```ignore
//! use ricecoder_commands::{CommandDefinition, CommandRegistry, ArgumentType, CommandArgument};
//!
//! let mut registry = CommandRegistry::new();
//!
//! // Create a custom command
//! let cmd = CommandDefinition::new("greet", "Greet User", "echo Hello {{name}}")
//!     .with_description("Greet a user by name")
//!     .with_argument(
//!         CommandArgument::new("name", ArgumentType::String)
//!             .with_description("User name")
//!             .with_required(true)
//!     )
//!     .with_inject_output(true);
//!
//! // Register the command
//! registry.register(cmd)?;
//!
//! // Get the command
//! let cmd = registry.get("greet")?;
//! ```

pub mod config;
pub mod di;
pub mod error;
pub mod executor;
pub mod manager;
pub mod output_injection;
pub mod registry;
pub mod template;
pub mod types;

pub use config::ConfigManager;
pub use error::{CommandError, Result};
pub use executor::CommandExecutor;
pub use manager::CommandManager;
pub use output_injection::{OutputFormat, OutputInjectionConfig, OutputInjector};
pub use registry::CommandRegistry;
pub use template::TemplateProcessor;
pub use types::{
    ArgumentType, CommandArgument, CommandContext, CommandDefinition, CommandExecutionResult,
};
