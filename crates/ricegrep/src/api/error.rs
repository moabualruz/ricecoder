use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use thiserror::Error;

/// Errors emitted by the API gateway.
#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("internal handler error: {0}")]
    Handler(#[from] HandlerError),
    #[error("authentication failure: {0}")]
    Authentication(String),
    #[error("rate limit exceeded")]
    RateLimit,
    #[error("internal gateway failure: {0}")]
    Internal(String),
}

impl GatewayError {
    fn status(&self) -> StatusCode {
        match self {
            GatewayError::Handler(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GatewayError::Authentication(_) => StatusCode::UNAUTHORIZED,
            GatewayError::RateLimit => StatusCode::TOO_MANY_REQUESTS,
            GatewayError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            GatewayError::Handler(_) => "handler",
            GatewayError::Authentication(_) => "authentication",
            GatewayError::RateLimit => "rate_limit",
            GatewayError::Internal(_) => "internal",
        }
    }

    fn message(&self) -> String {
        match self {
            GatewayError::Handler(err) => err.to_string(),
            GatewayError::Authentication(msg) => msg.clone(),
            GatewayError::RateLimit => "rate limit exceeded".to_string(),
            GatewayError::Internal(msg) => msg.clone(),
        }
    }
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let payload = json!({
            "error": self.message(),
            "kind": self.kind(),
        });
        (self.status(), Json(payload)).into_response()
    }
}

/// Errors emitted by the request handler.
#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("query execution error: {0}")]
    Query(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        let (status, kind) = match &self {
            HandlerError::Validation(_) => (StatusCode::BAD_REQUEST, "validation"),
            HandlerError::Query(_) => (StatusCode::BAD_GATEWAY, "query"),
            HandlerError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };
        let payload = json!({"error": self.to_string(), "kind": kind});
        (status, Json(payload)).into_response()
    }
}

/// Placeholder SDK error abstraction.
#[derive(Debug, Error)]
pub enum SDKError {
    #[error("SDK configuration error: {0}")]
    Config(String),
    #[error("SDK execution error: {0}")]
    Execution(String),
}
