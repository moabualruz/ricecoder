//! Minimal model types for backward compatibility with CLI
//!
//! These types were part of the old TEA system model.rs that was deleted during cleanup.
//! They're retained here as stubs to satisfy CLI dependencies during the transition.

/// Provider connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderConnectionState {
    Connected,
    Disconnected,
    Error,
    Disabled,
}

/// Provider information for TUI
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub state: ProviderConnectionState,
    pub models: Vec<String>,
    pub error_message: Option<String>,
    pub last_checked: Option<chrono::DateTime<chrono::Utc>>,
}
