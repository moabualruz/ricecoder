//! API server implementation

use crate::{
    middleware::{auth, logging, rate_limit},
    routes,
    state::AppState,
};
use axum::Router;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

/// API server
pub struct ApiServer {
    app: Router,
}

impl ApiServer {
    /// Create new API server
    pub fn new(state: AppState) -> Self {
        let app = routes::all_routes()
            // Add middleware
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(axum::middleware::from_fn_with_state(
                        state.clone(),
                        logging::logging_middleware,
                    ))
                    .layer(axum::middleware::from_fn_with_state(
                        state.clone(),
                        rate_limit::rate_limit_middleware,
                    ))
                    .layer(axum::middleware::from_fn_with_state(
                        state.clone(),
                        auth::auth_middleware,
                    )),
            )
            .with_state(state);

        Self { app }
    }

    /// Run the server
    pub async fn run(self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.app).await?;
        Ok(())
    }

    /// Get the router for testing
    pub fn router(&self) -> &Router {
        &self.app
    }
}
