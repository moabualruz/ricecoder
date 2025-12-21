//! TUI integration for VCS functionality
//!
//! This module provides TUI-specific integration code for VCS features,
//! including status display and repository monitoring.

pub mod vcs_integration;

// Re-export public API
pub use vcs_integration::{VcsIntegration, VcsStatus};
