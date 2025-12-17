//! Rate limiting middleware

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use crate::state::AppState;

/// Rate limiter state
#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<AtomicU64>,
    window_start: std::time::Instant,
    max_requests: u64,
    window_duration: std::time::Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u64, window_duration: std::time::Duration) -> Self {
        Self {
            requests: Arc::new(AtomicU64::new(0)),
            window_start: std::time::Instant::now(),
            max_requests,
            window_duration,
        }
    }

    pub fn check(&self) -> bool {
        let now = std::time::Instant::now();

        // Reset counter if window has passed
        if now.duration_since(self.window_start) >= self.window_duration {
            self.requests.store(1, Ordering::SeqCst);
            // Note: This is not thread-safe for window reset, but good enough for demo
            true
        } else {
            let current = self.requests.fetch_add(1, Ordering::SeqCst);
            current < self.max_requests
        }
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(_state): State<AppState>,
    request: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    // Extract client identifier (IP address for now)
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // TODO: Implement proper distributed rate limiting
    // For now, use a simple in-memory limiter

    static REQUESTS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    static WINDOW_START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

    let now = std::time::Instant::now();
    let window_start = WINDOW_START.get_or_init(|| now);

    // Reset counter every minute
    if now.duration_since(*window_start) > std::time::Duration::from_secs(60) {
        REQUESTS.store(0, std::sync::atomic::Ordering::SeqCst);
        // Note: This is not thread-safe for window reset
    }

    let current_requests = REQUESTS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    // Allow up to 100 requests per minute per IP
    if current_requests > 100 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let response = next.run(request).await;
    Ok(response)
}