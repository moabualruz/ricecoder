//! Health check endpoint

use axum::{
    extract::{Query, State},
    Json,
};

use crate::{models::HealthResponse, state::AppState};

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: state.uptime_seconds(),
        database: "connected".to_string(), // TODO: Add actual database health check
    })
}

/// Load test endpoint for performance validation
#[utoipa::path(
    get,
    path = "/load-test",
    params(("requests" = Option<u32>, Query, description = "Number of simulated requests")),
    responses(
        (status = 200, description = "Load test completed", body = serde_json::Value)
    )
)]
pub async fn load_test(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let requests: u32 = params
        .get("requests")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let start = std::time::Instant::now();

    // Simulate load by doing some CPU work
    for i in 0..requests {
        // Simulate some processing work
        let _work = (0..1000).map(|x| x * x).sum::<u64>();
        if i % 10 == 0 {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }

    let duration = start.elapsed();

    Json(serde_json::json!({
        "status": "completed",
        "requests_processed": requests,
        "total_time_ms": duration.as_millis(),
        "avg_time_per_request_ms": duration.as_millis() as f64 / requests as f64,
        "requests_per_second": requests as f64 / duration.as_secs_f64()
    }))
}
