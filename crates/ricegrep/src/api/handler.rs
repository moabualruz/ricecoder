use std::sync::Arc;

use crate::{
    api::{
        error::HandlerError,
        models::{HealthStatus, SearchRequest, SearchResponse},
    },
    search::coordinator::SearchCoordinator,
};

#[derive(Debug, Clone)]
pub struct RequestHandlerConfig {
    pub max_query_length: usize,
}

#[derive(Clone)]
pub struct RequestHandler {
    config: Arc<RequestHandlerConfig>,
    coordinator: Arc<SearchCoordinator>,
}

impl RequestHandler {
    pub fn new(config: RequestHandlerConfig, coordinator: Arc<SearchCoordinator>) -> Self {
        Self {
            config: Arc::new(config),
            coordinator,
        }
    }

    pub async fn process_request(
        &self,
        request: SearchRequest,
    ) -> Result<SearchResponse, HandlerError> {
        if request.query.len() > self.config.max_query_length {
            return Err(HandlerError::Validation(format!(
                "query exceeds max length {}",
                self.config.max_query_length
            )));
        }

        match self.coordinator.execute(request).await {
            Ok(response) => Ok(response),
            Err(err) => Err(HandlerError::Query(err.to_string())),
        }
    }

    pub async fn health_check(&self) -> Result<HealthStatus, HandlerError> {
        Ok(HealthStatus {
            healthy: true,
            message: "ok".to_string(),
            alerts: Vec::new(),
        })
    }
}
