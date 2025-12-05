//! Mode trait definition

use crate::error::Result;
use crate::models::{
    Capability, ModeConfig, ModeConstraints, ModeContext, ModeResponse, Operation,
};
use async_trait::async_trait;

/// Trait that all modes must implement
///
/// This trait defines the interface for all modes in the RiceCoder system.
/// Each mode must provide its own implementation of these methods.
#[async_trait]
pub trait Mode: Send + Sync {
    /// Mode identifier (e.g., "code", "ask", "vibe")
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Mode description
    fn description(&self) -> &str;

    /// Get system prompt for this mode
    fn system_prompt(&self) -> &str;

    /// Process user input in this mode
    async fn process(&self, input: &str, context: &ModeContext) -> Result<ModeResponse>;

    /// Get mode-specific capabilities
    fn capabilities(&self) -> Vec<Capability>;

    /// Get mode configuration
    fn config(&self) -> &ModeConfig;

    /// Validate if operation is allowed in this mode
    fn can_execute(&self, operation: &Operation) -> bool;

    /// Get mode-specific constraints
    fn constraints(&self) -> ModeConstraints;
}
