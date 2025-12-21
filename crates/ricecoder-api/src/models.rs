//! API request and response models

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Session creation request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSessionRequest {
    /// Session name
    pub name: String,
    /// Provider to use
    pub provider: String,
    /// Initial context (optional)
    pub context: Option<String>,
}

/// Session response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionResponse {
    /// Session ID
    pub id: String,
    /// Session name
    pub name: String,
    /// Provider
    pub provider: String,
    /// Status
    pub status: String,
    /// Created timestamp
    pub created_at: i64,
    /// Last activity timestamp
    pub last_activity: i64,
}

/// Session list response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionListResponse {
    /// List of sessions
    pub sessions: Vec<SessionResponse>,
    /// Total count
    pub total: usize,
}

/// Share session request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShareSessionRequest {
    /// Expiration time in seconds (optional)
    pub expires_in: Option<u64>,
    /// Include history in share
    pub include_history: Option<bool>,
    /// Include context in share
    pub include_context: Option<bool>,
}

/// Share session response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShareSessionResponse {
    /// Share ID
    pub share_id: String,
    /// Share URL
    pub share_url: String,
    /// Expiration timestamp
    pub expires_at: Option<i64>,
}

/// Execute MCP tool request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExecuteToolRequest {
    /// Tool name
    pub tool_name: String,
    /// Tool parameters
    pub parameters: serde_json::Value,
    /// Server ID (optional)
    pub server_id: Option<String>,
    /// Timeout in seconds
    pub timeout_seconds: Option<u64>,
}

/// Execute MCP tool response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExecuteToolResponse {
    /// Execution result
    pub result: serde_json::Value,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Success flag
    pub success: bool,
}

/// Authentication request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthRequest {
    /// Username or email
    pub username: String,
    /// Password
    pub password: String,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    /// JWT token
    pub token: String,
    /// Token expiration timestamp
    pub expires_at: i64,
    /// User information
    pub user: UserInfo,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserInfo {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// Email
    pub email: String,
    /// Role
    pub role: String,
}

/// API health response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Version
    pub version: String,
    /// Uptime in seconds
    pub uptime: u64,
    /// Database status
    pub database: String,
}
