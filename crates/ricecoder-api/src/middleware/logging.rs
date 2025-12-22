//! Request logging middleware

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::state::AppState;

/// Request logging middleware
pub async fn logging_middleware(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();

    tracing::info!("Request: {} {}", method, uri);

    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    // Log performance warnings for slow requests (>100ms)
    if duration > std::time::Duration::from_millis(100) {
        tracing::warn!("Slow request: {} {} took {:?}", method, uri, duration);
    }

    tracing::info!(
        "Response: {} {} - {} in {:.2}ms",
        method,
        uri,
        response.status(),
        duration.as_millis()
    );

    Ok(response)
}
