//! Minimal stub for old error_handling types
//! Used by real_time_updates.rs but not critical for TUI functionality

/// Rice error stub (cloneable version for StreamData)
#[derive(Debug, Clone)]
pub struct RiceError {
    pub message: String,
    pub category: ErrorCategory,
    pub severity: ErrorSeverity,
}

impl RiceError {
    pub fn new(message: impl Into<String>, category: ErrorCategory) -> Self {
        Self {
            message: message.into(),
            category,
            severity: ErrorSeverity::Medium,
        }
    }
}

impl std::fmt::Display for RiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RiceError {}

/// Error category stub  
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Provider,
    Session,
    Storage,
    Network,
    Internal,
}

/// Error severity stub
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Error manager stub
#[derive(Debug, Clone)]
pub struct ErrorManager {
    // Placeholder - not actually used
}

impl ErrorManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_error(&self, _error: RiceError) {
        // Stub - just log it
        tracing::warn!("Error handled: {}", _error);
    }
}

impl Default for ErrorManager {
    fn default() -> Self {
        Self::new()
    }
}
