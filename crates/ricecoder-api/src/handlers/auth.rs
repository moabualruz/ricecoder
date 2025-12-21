//! Authentication API handlers

use crate::{
    error::{ApiError, ApiResult},
    models::{AuthRequest, AuthResponse, UserInfo},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, Json};

/// Authenticate user
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = AuthRequest,
    responses(
        (status = 200, description = "Authentication successful", body = AuthResponse),
        (status = 401, description = "Authentication failed"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn login(
    State(_state): State<AppState>,
    Json(request): Json<AuthRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // TODO: Implement actual user authentication against database
    // For now, accept demo/demo credentials

    if request.username == "demo" && request.password == "demo" {
        // Create JWT token (simplified - in production use proper JWT library)
        let header = base64::encode(r#"{"alg":"HS256","typ":"JWT"}"#);
        let payload = base64::encode(format!(
            r#"{{"sub":"user-123","username":"{}","role":"user","exp":{}}}"#,
            request.username,
            (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp()
        ));
        let signature = base64::encode("mock-signature"); // In production, use proper HMAC
        let token = format!("{}.{}.{}", header, payload, signature);

        let user = UserInfo {
            id: "user-123".to_string(),
            username: request.username,
            email: "demo@example.com".to_string(),
            role: "user".to_string(),
        };

        let response = AuthResponse {
            token,
            expires_at: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp(),
            user,
        };

        Ok(Json(response))
    } else {
        Err(crate::error::ApiError::Authentication(
            "Invalid credentials".to_string(),
        ))
    }
}

/// Logout user
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 204, description = "Logout successful"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn logout() -> StatusCode {
    // TODO: Implement logout logic (invalidate token, etc.)
    StatusCode::NO_CONTENT
}
