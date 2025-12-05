#![warn(missing_docs)]

//! RiceCoder Modes - Mode system for different interaction patterns
//!
//! This crate provides a flexible mode system for RiceCoder with support for:
//! - Code Mode: Focused code generation and modification
//! - Ask Mode: Question answering without file modifications
//! - Vibe Mode: Free-form exploration and rapid prototyping
//! - Think More: Extended reasoning for complex tasks

/// Ask Mode implementation
pub mod ask_mode;
/// Property-based tests for Ask Mode
#[cfg(test)]
mod ask_mode_properties;
/// Auto-enable logic for Think More based on task complexity
pub mod auto_enable;
/// Code Mode implementation
pub mod code_mode;
/// Property-based tests for Code Mode
#[cfg(test)]
mod code_mode_properties;
/// Error types for the modes system
pub mod error;
/// Mode manager for lifecycle and transitions
pub mod manager;
/// Mode trait definition
pub mod mode;
/// Mode switcher for handling transitions with context preservation
pub mod mode_switcher;
/// Property-based tests for Mode Switching
#[cfg(test)]
mod mode_switching_properties;
/// Data models for modes
pub mod models;
/// Per-task configuration management
pub mod task_config;
/// Property-based tests for Think More Activation
#[cfg(test)]
mod think_more_activation_properties;
/// Property-based tests for Think More Auto-Enable
#[cfg(test)]
mod think_more_auto_enable_properties;
/// Property-based tests for Think More Configuration
#[cfg(test)]
mod think_more_configuration_properties;
/// Think More controller for extended thinking
pub mod think_more_controller;
/// Property-based tests for Think More Performance Trade-off
#[cfg(test)]
mod think_more_performance_properties;
/// Thinking display and formatting
pub mod thinking_display;
/// Vibe Mode implementation
pub mod vibe_mode;
/// Property-based tests for Vibe Mode
#[cfg(test)]
mod vibe_mode_properties;

pub use ask_mode::AskMode;
pub use auto_enable::{ComplexityAnalysis, ComplexityDetector};
pub use code_mode::CodeMode;
pub use error::{ModeError, Result};
pub use manager::ModeManager;
pub use mode::Mode;
pub use mode_switcher::ModeSwitcher;
pub use models::{
    Capability, ChangeSummary, ComplexityLevel, Message, MessageRole, ModeAction, ModeConfig,
    ModeConstraints, ModeContext, ModeResponse, Operation, ResponseMetadata, ThinkMoreConfig,
    ThinkingDepth,
};
pub use task_config::{TaskConfig, TaskConfigManager};
pub use think_more_controller::{ThinkMoreController, ThinkingMetadata};
pub use thinking_display::{ThinkingDisplay, ThinkingStatistics};
pub use vibe_mode::VibeMode;
