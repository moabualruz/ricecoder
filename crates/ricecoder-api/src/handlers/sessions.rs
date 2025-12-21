//! Session management API handlers

use crate::{
    error::{ApiError, ApiResult},
    models::{
        CreateSessionRequest, SessionListResponse, SessionResponse, ShareSessionRequest,
        ShareSessionResponse,
    },
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::collections::HashMap;

/// Query parameters for session listing
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListSessionsQuery {
    /// Filter by status
    pub status: Option<String>,
    /// Limit results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Create a new session
#[utoipa::path(
    post,
    path = "/api/v1/sessions",
    request_body = CreateSessionRequest,
    responses(
        (status = 201, description = "Session created successfully", body = SessionResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_session(
    State(_state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> ApiResult<Json<SessionResponse>> {
    // Validate request
    if request.name.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "Session name cannot be empty".to_string(),
        ));
    }
    if request.name.len() > 100 {
        return Err(ApiError::BadRequest(
            "Session name too long (max 100 characters)".to_string(),
        ));
    }
    if !request
        .name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ApiError::BadRequest(
            "Session name can only contain alphanumeric characters, underscores, and hyphens"
                .to_string(),
        ));
    }
    if request.provider.trim().is_empty() {
        return Err(ApiError::BadRequest("Provider cannot be empty".to_string()));
    }
    if let Some(ref context) = request.context {
        if context.len() > 10000 {
            return Err(ApiError::BadRequest(
                "Context too long (max 10000 characters)".to_string(),
            ));
        }
    }

    // TODO: Implement actual session creation through use cases
    // For now, return a mock response
    let response = SessionResponse {
        id: format!("session-{}", uuid::Uuid::new_v4()),
        name: request.name,
        provider: request.provider,
        status: "active".to_string(),
        created_at: chrono::Utc::now().timestamp(),
        last_activity: chrono::Utc::now().timestamp(),
    };

    Ok(Json(response))
}

/// List sessions
#[utoipa::path(
    get,
    path = "/api/v1/sessions",
    params(ListSessionsQuery),
    responses(
        (status = 200, description = "List of sessions", body = SessionListResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_sessions(
    State(_state): State<AppState>,
    Query(_query): Query<ListSessionsQuery>,
) -> ApiResult<Json<SessionListResponse>> {
    // TODO: Implement actual session listing
    // For now, return mock data
    let sessions = vec![SessionResponse {
        id: "session-1".to_string(),
        name: "Test Session".to_string(),
        provider: "anthropic".to_string(),
        status: "active".to_string(),
        created_at: chrono::Utc::now().timestamp(),
        last_activity: chrono::Utc::now().timestamp(),
    }];

    let response = SessionListResponse { sessions, total: 1 };

    Ok(Json(response))
}

/// Get session by ID
#[utoipa::path(
    get,
    path = "/api/v1/sessions/{id}",
    params(("id" = String, Path, description = "Session ID")),
    responses(
        (status = 200, description = "Session details", body = SessionResponse),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_session(
    State(_state): State<AppState>,
    Path(session_id): Path<String>,
) -> ApiResult<Json<SessionResponse>> {
    // TODO: Implement actual session retrieval
    // For now, return mock data if ID matches
    if session_id == "session-1" {
        let response = SessionResponse {
            id: session_id,
            name: "Test Session".to_string(),
            provider: "anthropic".to_string(),
            status: "active".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            last_activity: chrono::Utc::now().timestamp(),
        };
        Ok(Json(response))
    } else {
        Err(ApiError::SessionNotFound(session_id))
    }
}

/// Delete session
#[utoipa::path(
    delete,
    path = "/api/v1/sessions/{id}",
    params(("id" = String, Path, description = "Session ID")),
    responses(
        (status = 204, description = "Session deleted"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_session(
    State(_state): State<AppState>,
    Path(session_id): Path<String>,
) -> ApiResult<StatusCode> {
    // TODO: Implement actual session deletion
    // For now, just return success
    if session_id == "session-1" {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::SessionNotFound(session_id))
    }
}

/// Share session
#[utoipa::path(
    post,
    path = "/api/v1/sessions/{id}/share",
    params(("id" = String, Path, description = "Session ID")),
    request_body = ShareSessionRequest,
    responses(
        (status = 200, description = "Session shared successfully", body = ShareSessionResponse),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn share_session(
    State(_state): State<AppState>,
    Path(session_id): Path<String>,
    Json(_request): Json<ShareSessionRequest>,
) -> ApiResult<Json<ShareSessionResponse>> {
    // TODO: Implement actual session sharing
    // For now, return mock data
    let response = ShareSessionResponse {
        share_id: format!("share-{}", uuid::Uuid::new_v4()),
        share_url: format!("https://ricecoder.com/share/{}", session_id),
        expires_at: Some((chrono::Utc::now() + chrono::Duration::hours(24)).timestamp()),
    };

    Ok(Json(response))
}

/// Get session shares
#[utoipa::path(
    get,
    path = "/api/v1/sessions/{id}/shares",
    params(("id" = String, Path, description = "Session ID")),
    responses(
        (status = 200, description = "List of session shares", body = Vec<ShareSessionResponse>),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_session_shares(
    State(_state): State<AppState>,
    Path(_session_id): Path<String>,
) -> ApiResult<Json<Vec<ShareSessionResponse>>> {
    // TODO: Implement actual share listing
    // For now, return empty list
    Ok(Json(vec![]))
}
