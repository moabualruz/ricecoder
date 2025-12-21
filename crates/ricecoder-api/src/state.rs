//! Application state for the API server

use ricecoder_activity_log::ActivityLogger;
use ricecoder_agents::use_cases::{SessionLifecycleUseCase, SessionSharingUseCase};
use ricecoder_di::DIContainer;
use ricecoder_mcp::agent_integration::ToolInvoker;
use std::sync::Arc;

/// Application state shared across all API handlers
#[derive(Clone)]
pub struct AppState {
    /// Session lifecycle use case
    pub session_lifecycle: Arc<SessionLifecycleUseCase>,
    /// Session sharing use case
    pub session_sharing: Arc<SessionSharingUseCase>,
    /// MCP tool invoker
    pub mcp_tool_invoker: Option<Arc<dyn ToolInvoker>>,
    /// Activity logger
    pub activity_logger: Arc<ActivityLogger>,
    /// Server start time for uptime calculation
    pub start_time: std::time::Instant,
}

impl AppState {
    /// Create new application state
    pub async fn new(container: DIContainer) -> Result<Self, Box<dyn std::error::Error>> {
        let session_lifecycle = container
            .resolve::<SessionLifecycleUseCase>()
            .map_err(|_| "SessionLifecycleUseCase not found in container")?;

        let session_sharing = container
            .resolve::<SessionSharingUseCase>()
            .map_err(|_| "SessionSharingUseCase not found in container")?;

        let activity_logger = container
            .resolve::<ActivityLogger>()
            .map_err(|_| "ActivityLogger not found in container")?;

        // MCP tool invoker is optional for now
        let mcp_tool_invoker = None; // TODO: Properly resolve from container

        Ok(Self {
            session_lifecycle,
            session_sharing,
            mcp_tool_invoker,
            activity_logger,
            start_time: std::time::Instant::now(),
        })
    }

    /// Get server uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
