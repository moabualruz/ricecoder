//! API route definitions

use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    handlers::{auth, health, providers, sessions, tools},
    state::AppState,
};

/// API routes
pub fn api_routes() -> Router<AppState> {
    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        .route("/load-test", get(health::load_test))
        // Authentication
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/logout", post(auth::logout))
        // Session management
        .route("/api/v1/sessions", get(sessions::list_sessions))
        .route("/api/v1/sessions", post(sessions::create_session))
        .route("/api/v1/sessions/:id", get(sessions::get_session))
        .route("/api/v1/sessions/:id", delete(sessions::delete_session))
        .route("/api/v1/sessions/:id/share", post(sessions::share_session))
        .route(
            "/api/v1/sessions/:id/shares",
            get(sessions::get_session_shares),
        )
        // MCP tools
        .route("/api/v1/tools", get(tools::list_tools))
        .route("/api/v1/tools/execute", post(tools::execute_tool))
        // Provider management
        .route("/api/v1/providers", get(providers::list_providers))
        .route("/api/v1/providers", post(providers::switch_provider))
        .route(
            "/api/v1/providers/current",
            get(providers::get_current_provider),
        )
        .route("/api/v1/providers/:id", get(providers::get_provider))
        .route(
            "/api/v1/providers/:id/metrics",
            get(providers::get_provider_metrics),
        )
        // CORS
        .layer(CorsLayer::permissive())
}

/// Swagger UI routes
pub fn swagger_routes() -> Router<AppState> {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

/// Combined routes
pub fn all_routes() -> Router<AppState> {
    api_routes().merge(swagger_routes())
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        health::load_test,
        auth::login,
        auth::logout,
        sessions::create_session,
        sessions::list_sessions,
        sessions::get_session,
        sessions::delete_session,
        sessions::share_session,
        sessions::get_session_shares,
        tools::list_tools,
        tools::execute_tool,
        providers::list_providers,
        providers::get_provider,
        providers::get_provider_metrics,
        providers::switch_provider,
        providers::get_current_provider,
    ),
    components(schemas(
        crate::models::CreateSessionRequest,
        crate::models::SessionResponse,
        crate::models::SessionListResponse,
        crate::models::ShareSessionRequest,
        crate::models::ShareSessionResponse,
        crate::models::ExecuteToolRequest,
        crate::models::ExecuteToolResponse,
        crate::models::AuthRequest,
        crate::models::AuthResponse,
        crate::models::UserInfo,
        crate::models::HealthResponse,
        crate::handlers::providers::ProviderResponse,
        crate::handlers::providers::ProviderListResponse,
        crate::handlers::providers::ProviderMetricsResponse,
        crate::handlers::providers::SwitchProviderRequest,
        crate::handlers::providers::SwitchProviderResponse,
        crate::handlers::providers::ListProvidersQuery,
    )),
    info(
        title = "RiceCoder API",
        version = "1.0.0",
        description = "RESTful API for RiceCoder session management and MCP tool execution"
    )
)]
struct ApiDoc;
