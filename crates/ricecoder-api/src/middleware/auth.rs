//! Authentication middleware

use crate::{models::UserInfo, state::AppState};
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Authentication middleware
pub async fn auth_middleware(
    State(_state): State<AppState>,
    mut request: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..]; // Remove "Bearer " prefix

                // TODO: Validate JWT token
                // For now, accept any token that looks like a JWT
                if token.contains('.') && token.split('.').count() == 3 {
                    // Add user info to request extensions for handlers to use
                    request.extensions_mut().insert(UserInfo {
                        id: "user-123".to_string(),
                        username: "authenticated-user".to_string(),
                        email: "user@example.com".to_string(),
                        role: "user".to_string(),
                    });
                } else {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        // Allow health check and auth endpoints without authentication
        let path = request.uri().path();
        if path == "/health" || path.starts_with("/api/v1/auth/") {
            // No auth required
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    let response = next.run(request).await;
    Ok(response)
}
