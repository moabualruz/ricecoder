use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::{Path, Query};
use axum::{
    extract::State,
    http::HeaderMap,
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use prometheus::{Encoder, Registry, TextEncoder};
use serde::Deserialize;

use crate::admin::{
    AdminAction, AdminCommandRequest, AdminCommandResponse, AdminError, AdminToolset,
};
use crate::api::error::HandlerError;
use crate::api::{
    auth::AuthCredentials,
    error::GatewayError,
    execution::SearchExecutor,
    graphql::GraphQLSchema,
    handler::RequestHandler,
    models::{
        AlertActionRequest, BenchmarkRequest, BenchmarkResponse, BenchmarkSuiteResponse,
        HealthStatus, SearchRequest, SearchResponse,
    },
    openapi::OpenApiDocument,
};
use crate::benchmarking::BenchmarkCoordinator;
use crate::performance::{BenchmarkHarness, BenchmarkMode};
use crate::vector::{
    alerting::{AlertManager, AlertSeverity, AlertState, AlertSummary},
    metrics_storage::{MetricsHistoryEntry, MetricsStorage},
    observability::{SystemResourceSampler, VectorMetrics, VectorTelemetry},
};

const MAX_HISTORY_MINUTES: u64 = 90 * 24 * 60;
const MIN_BUCKET_MINUTES: u64 = 1;
const MAX_BUCKET_MINUTES: u64 = 60;

#[derive(Clone)]
pub struct GatewayState {
    pub executor: Arc<SearchExecutor>,
    pub schema: Arc<GraphQLSchema>,
    pub openapi: Arc<OpenApiDocument>,
    pub metrics_registry: Arc<Registry>,
    pub vector_metrics: Arc<VectorMetrics>,
    pub vector_telemetry: Arc<VectorTelemetry>,
    pub alert_manager: Arc<AlertManager>,
    pub metrics_storage: Arc<MetricsStorage>,
    pub resource_sampler: Arc<Mutex<SystemResourceSampler>>,
    pub admin_toolset: Arc<AdminToolset>,
    pub benchmark_coordinator: Arc<BenchmarkCoordinator>,
    pub benchmark_root: PathBuf,
}

pub fn build_router(state: GatewayState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/search", post(search_handler))
        .route("/openapi.json", get(openapi_handler))
        .route("/graphql", post(graphql_handler))
        .route("/graphql", get(graphql_playground))
        .route("/alerts", get(alerts_handler))
        .route("/alerts/:name/ack", post(ack_alert_handler))
        .route("/alerts/:name/resolve", post(resolve_alert_handler))
        .route("/metrics", get(metrics_handler))
        .route("/metrics/history", get(metrics_history_handler))
        .route("/admin/command", post(admin_command_handler))
        .route("/benchmarks/run", post(benchmark_handler))
        .with_state(state)
}

#[axum::debug_handler]
async fn health_handler(State(state): State<GatewayState>) -> Json<HealthStatus> {
    let alerts = state.alert_manager.current_summary();
    let healthy = !alerts.iter().any(|alert| {
        alert.state == AlertState::Firing && alert.severity == AlertSeverity::Critical
    });
    let message = if healthy {
        "ok".to_string()
    } else {
        "critical alert firing".to_string()
    };
    Json(HealthStatus {
        healthy,
        message,
        alerts,
    })
}

async fn alerts_handler(
    State(state): State<GatewayState>,
) -> Result<Json<Vec<AlertSummary>>, GatewayError> {
    let alerts = state.alert_manager.check_alerts();
    Ok(Json(alerts))
}

#[axum::debug_handler]
async fn ack_alert_handler(
    Path(rule_name): Path<String>,
    State(state): State<GatewayState>,
    Json(payload): Json<AlertActionRequest>,
) -> Result<Json<AlertSummary>, GatewayError> {
    state
        .alert_manager
        .acknowledge_alert(&rule_name, payload.actor)
        .map(Json)
        .ok_or_else(|| GatewayError::Internal(format!("alert {} not found", rule_name)))
}

#[axum::debug_handler]
async fn resolve_alert_handler(
    Path(rule_name): Path<String>,
    State(state): State<GatewayState>,
    Json(payload): Json<AlertActionRequest>,
) -> Result<Json<AlertSummary>, GatewayError> {
    state
        .alert_manager
        .resolve_alert(&rule_name, payload.note, payload.actor)
        .map(Json)
        .ok_or_else(|| GatewayError::Internal(format!("alert {} not found", rule_name)))
}

async fn metrics_handler(State(state): State<GatewayState>) -> Result<String, GatewayError> {
    let snapshot = {
        let mut sampler = state.resource_sampler.lock().unwrap();
        sampler.refresh()
    };
    state.vector_metrics.record_resource_usage(&snapshot);
    state
        .vector_telemetry
        .record_resource_usage(snapshot.clone());
    let telemetry_snapshot = state.vector_telemetry.snapshot();
    state.metrics_storage.record(telemetry_snapshot);
    let encoder = TextEncoder::new();
    let metric_families = state.metrics_registry.gather();
    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .map_err(|err| GatewayError::Internal(err.to_string()))?;
    Ok(String::from_utf8(buffer).map_err(|err| GatewayError::Internal(err.to_string()))?)
}

async fn metrics_history_handler(
    State(state): State<GatewayState>,
    Query(query): Query<MetricsHistoryQuery>,
) -> Result<Json<Vec<MetricsHistoryEntry>>, GatewayError> {
    let lookback_minutes = query.minutes.unwrap_or(60).clamp(1, MAX_HISTORY_MINUTES);
    let lookback_duration = Duration::from_secs(lookback_minutes.saturating_mul(60));

    let history = if let Some(bucket_minutes) = query.bucket_minutes {
        let clamped = bucket_minutes.clamp(MIN_BUCKET_MINUTES, MAX_BUCKET_MINUTES);
        let bucket_duration = Duration::from_secs(clamped.saturating_mul(60));
        state
            .metrics_storage
            .aggregated_history(lookback_duration, bucket_duration)
    } else {
        state
            .metrics_storage
            .aggregated_history_default(lookback_duration)
    };

    Ok(Json(history))
}

#[derive(Debug, Deserialize)]
struct MetricsHistoryQuery {
    minutes: Option<u64>,
    bucket_minutes: Option<u64>,
}

async fn admin_command_handler(
    State(state): State<GatewayState>,
    Json(payload): Json<AdminCommandRequest>,
) -> Result<Json<AdminCommandResponse>, GatewayError> {
    let action = payload.action;
    match action {
        AdminAction::Reindex => {
            let repo_path = payload.repository_path.as_ref().ok_or_else(|| {
                GatewayError::Handler(HandlerError::Validation(
                    "repository_path is required for reindex".into(),
                ))
            })?;
            let stats = state
                .admin_toolset
                .reindex_repository(repo_path)
                .await
                .map_err(map_admin_error)?;
            Ok(Json(AdminCommandResponse {
                action,
                summary: format!(
                    "reindexed {:?} ({} chunks)",
                    repo_path, stats.chunks_indexed
                ),
                stats: Some(stats),
            }))
        }
        AdminAction::Optimize => {
            state
                .admin_toolset
                .optimize_index()
                .map_err(map_admin_error)?;
            Ok(Json(AdminCommandResponse {
                action,
                summary: "index optimized".into(),
                stats: None,
            }))
        }
        AdminAction::ClearCache => {
            state.admin_toolset.clear_cache().map_err(map_admin_error)?;
            Ok(Json(AdminCommandResponse {
                action,
                summary: "cache cleared".into(),
                stats: None,
            }))
        }
        AdminAction::UpdateConfig => {
            let key = payload.config_key.clone().ok_or_else(|| {
                GatewayError::Handler(HandlerError::Validation(
                    "config_key is required for update_config".into(),
                ))
            })?;
            let value = payload.config_value.clone().ok_or_else(|| {
                GatewayError::Handler(HandlerError::Validation(
                    "config_value is required for update_config".into(),
                ))
            })?;
            state
                .admin_toolset
                .update_config(key.clone(), value.clone());
            Ok(Json(AdminCommandResponse {
                action,
                summary: format!("updated config {key}", key = key),
                stats: None,
            }))
        }
    }
}

async fn benchmark_handler(
    State(state): State<GatewayState>,
    Json(payload): Json<BenchmarkRequest>,
) -> Result<Response, GatewayError> {
    if payload.run_suite.unwrap_or(false) {
        let results = state
            .benchmark_coordinator
            .run_suite()
            .map_err(|err| GatewayError::Handler(HandlerError::Internal(err.to_string())))?;
        let suite = BenchmarkSuiteResponse {
            summary: format!("completed {} benchmark runs", results.len()),
            results,
        };
        return Ok(Json(suite).into_response());
    }

    let mode = payload.mode.unwrap_or(BenchmarkMode::Bm25);
    let queries = payload
        .queries
        .clone()
        .unwrap_or_else(BenchmarkHarness::default_queries);
    let harness = BenchmarkHarness::new(state.benchmark_root.clone(), queries);
    let result = harness
        .run(mode)
        .map_err(|err| GatewayError::Handler(HandlerError::Internal(err.to_string())))?;
    Ok(Json(BenchmarkResponse {
        mode,
        summary: format!("run {} benchmark queries", result.total_queries),
        result,
    })
    .into_response())
}

fn map_admin_error(err: AdminError) -> GatewayError {
    GatewayError::Handler(HandlerError::Internal(err.to_string()))
}

async fn search_handler(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, GatewayError> {
    let credentials = AuthCredentials::from_headers(&headers);
    let response = state
        .executor
        .execute(request, credentials, "rest.search")
        .await?;
    Ok(Json(response))
}

async fn openapi_handler(State(state): State<GatewayState>) -> Json<OpenApiDocument> {
    Json((*state.openapi).clone())
}

async fn graphql_handler(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    request: GraphQLRequest,
) -> GraphQLResponse {
    let credentials = AuthCredentials::from_headers(&headers);
    let request = request.into_inner().data(credentials);
    state.schema.execute(request).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
