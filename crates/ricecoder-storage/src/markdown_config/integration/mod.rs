//! Integration modules for markdown configuration with ricecoder subsystems
//!
//! This module provides integration with ricecoder-agents, ricecoder-modes, and ricecoder-commands,
//! enabling markdown-based configuration to be registered with these subsystems.
//!
//! The integration uses trait-based design to avoid circular dependencies:
//! - `AgentRegistrar` trait for agent registration
//! - `ModeRegistrar` trait for mode registration
//! - `CommandRegistrar` trait for command registration
//!
//! This allows ricecoder-storage to remain independent while providing integration points
//! for other crates to implement.

pub mod agents;
pub mod commands;
pub mod modes;

pub use agents::{AgentConfigIntegration, AgentRegistrar};
pub use commands::{CommandConfigIntegration, CommandRegistrar};
pub use modes::{ModeConfigIntegration, ModeRegistrar};
