//! API error types and handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// API error types
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Authorization failed: {0}")]
    Authorization(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Session error: {0}")]
    Session(#[from] ricecoder_sessions::SessionError),

    #[error("MCP error: {0}")]
    Mcp(#[from] ricecoder_mcp::error::Error),

    #[error("Security error: {0}")]
    Security(#[from] ricecoder_security::SecurityError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            ApiError::Authentication(_) => (StatusCode::UNAUTHORIZED, "authentication_error"),
            ApiError::Authorization(_) => (StatusCode::FORBIDDEN, "authorization_error"),
            ApiError::SessionNotFound(_) => (StatusCode::NOT_FOUND, "session_not_found"),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            ApiError::Session(_) => (StatusCode::BAD_REQUEST, "session_error"),
            ApiError::Mcp(_) => (StatusCode::BAD_REQUEST, "mcp_error"),
            ApiError::Security(_) => (StatusCode::FORBIDDEN, "security_error"),
            ApiError::Json(_) => (StatusCode::BAD_REQUEST, "json_error"),
        };

        let body = Json(json!({
            "error": {
                "type": error_type,
                "message": self.to_string(),
            }
        }));

        (status, body).into_response()
    }
}

/// Result type alias for API operations
pub type ApiResult<T> = Result<T, ApiError>;