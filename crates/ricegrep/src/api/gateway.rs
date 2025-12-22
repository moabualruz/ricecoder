use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{serve, Router};
use prometheus::Registry;
use tokio::net::TcpListener;
use tracing::info;

use crate::performance::BenchmarkHarness;
use crate::{
    admin::AdminToolset,
    api::{
        auth::{AuthConfig, AuthCredentials, AuthManager, AuthMethod},
        error::GatewayError,
        execution::SearchExecutor,
        graphql::{build_graphql_schema, GraphQLSchema},
        handler::{RequestHandler, RequestHandlerConfig},
        http::{build_router, GatewayState},
        models::{SearchRequest, SearchResponse},
        openapi::{openapi_document, OpenApiDocument},
    },
    benchmarking::BenchmarkCoordinator,
    search::coordinator::SearchCoordinator,
    vector::{
        alerting::AlertManager,
        metrics_storage::MetricsStorage,
        observability::{SystemResourceSampler, VectorMetrics, VectorTelemetry},
    },
};

#[derive(Clone)]
pub struct APIGatewayConfig {
    pub bind_addr: SocketAddr,
    pub auth_config: AuthConfig,
    pub handler_config: RequestHandlerConfig,
    pub search_coordinator: Arc<SearchCoordinator>,
    pub index_directory: PathBuf,
    pub benchmark_root: PathBuf,
}

pub struct APIGateway {
    config: Arc<APIGatewayConfig>,
    executor: Arc<SearchExecutor>,
    graphql_schema: Arc<GraphQLSchema>,
    openapi_doc: Arc<OpenApiDocument>,
    metrics_registry: Arc<Registry>,
    vector_metrics: Arc<VectorMetrics>,
    vector_telemetry: Arc<VectorTelemetry>,
    alert_manager: Arc<AlertManager>,
    metrics_storage: Arc<MetricsStorage>,
    resource_sampler: Arc<Mutex<SystemResourceSampler>>,
    admin_toolset: Arc<AdminToolset>,
    benchmark_coordinator: Arc<BenchmarkCoordinator>,
    benchmark_root: PathBuf,
}

impl APIGateway {
    pub fn new(config: APIGatewayConfig) -> Self {
        let handler = Arc::new(RequestHandler::new(
            config.handler_config.clone(),
            config.search_coordinator.clone(),
        ));
        let auth = Arc::new(AuthManager::new(
            config.auth_config.clone(),
            AuthMethod::ApiKey,
        ));
        let executor = Arc::new(SearchExecutor::new(handler.clone(), auth.clone()));
        let schema = Arc::new(build_graphql_schema(executor.clone()));
        let openapi_doc = Arc::new(openapi_document());
        let metrics_registry = Arc::new(Registry::new());
        let vector_metrics =
            VectorMetrics::register(&metrics_registry).expect("registering vector metrics failed");
        let vector_telemetry = Arc::new(VectorTelemetry::default());
        let alert_manager = Arc::new(AlertManager::new(vector_telemetry.clone()));
        let metrics_storage = Arc::new(MetricsStorage::new(
            Duration::from_secs(90 * 24 * 60 * 60),
            Duration::from_secs(300),
        ));
        let resource_sampler = Arc::new(Mutex::new(SystemResourceSampler::new()));
        let benchmark_root = config.benchmark_root.clone();
        let admin_toolset = Arc::new(AdminToolset::new(
            config.index_directory.clone(),
            Some(vector_telemetry.clone()),
        ));
        let benchmark_coordinator = Arc::new(
            BenchmarkCoordinator::new(
                config.index_directory.clone(),
                config.benchmark_root.clone(),
                BenchmarkHarness::default_queries(),
                alert_manager.clone(),
            )
            .expect("initializing benchmark coordinator"),
        );

        Self {
            config: Arc::new(config),
            executor,
            graphql_schema: schema,
            openapi_doc,
            metrics_registry,
            vector_metrics,
            vector_telemetry,
            alert_manager,
            metrics_storage,
            resource_sampler,
            admin_toolset,
            benchmark_coordinator,
            benchmark_root,
        }
    }

    fn router(&self) -> Router {
        let state = GatewayState {
            executor: self.executor.clone(),
            schema: self.graphql_schema.clone(),
            openapi: self.openapi_doc.clone(),
            metrics_registry: self.metrics_registry.clone(),
            vector_metrics: self.vector_metrics.clone(),
            vector_telemetry: self.vector_telemetry.clone(),
            alert_manager: self.alert_manager.clone(),
            metrics_storage: self.metrics_storage.clone(),
            resource_sampler: self.resource_sampler.clone(),
            admin_toolset: self.admin_toolset.clone(),
            benchmark_coordinator: self.benchmark_coordinator.clone(),
            benchmark_root: self.benchmark_root.clone(),
        };

        build_router(state)
    }

    pub async fn start(&self) -> Result<(), GatewayError> {
        info!(address = %self.config.bind_addr, "starting API gateway");
        let router = self.router();
        let listener = TcpListener::bind(self.config.bind_addr)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))?;

        serve(listener, router)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub async fn handle_search(
        &self,
        request: SearchRequest,
        credentials: AuthCredentials,
        endpoint: &str,
    ) -> Result<SearchResponse, GatewayError> {
        self.executor.execute(request, credentials, endpoint).await
    }
}
