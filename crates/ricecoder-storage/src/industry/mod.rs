//! Industry-standard file support
//!
//! This module provides support for reading and converting configuration files
//! from other AI coding tools (Cursor, Claude, Windsurf, Cline, Aider, Copilot, Continue, Kiro)
//! into RiceCoder's internal configuration format.
//!
//! The module follows a precedence order when multiple industry files exist:
//! environment > project > legacy > global > defaults

pub mod adapter;
pub mod agents;
pub mod aider;
pub mod claude;
pub mod cline;
pub mod continue_dev;
pub mod copilot;
pub mod cursor;
pub mod kiro;
pub mod windsurf;

// Re-export commonly used types
pub use adapter::{FileDetectionResult, IndustryFileAdapter, IndustryFileDetector};
pub use agents::AgentsAdapter;
pub use aider::AiderAdapter;
pub use claude::ClaudeAdapter;
pub use cline::ClineAdapter;
pub use continue_dev::ContinueDevAdapter;
pub use copilot::CopilotAdapter;
pub use cursor::CursorAdapter;
pub use kiro::KiroAdapter;
pub use windsurf::WindsurfAdapter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_industry_module_exports() {
        // Verify that the module exports are accessible
        let _: &dyn IndustryFileAdapter;
    }
}
