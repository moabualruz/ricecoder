use std::sync::Arc;

use crate::api::{
    auth::{AuthCredentials, AuthManager},
    error::GatewayError,
    handler::RequestHandler,
    models::{SearchRequest, SearchResponse},
};

#[derive(Clone)]
pub struct SearchExecutor {
    handler: Arc<RequestHandler>,
    auth: Arc<AuthManager>,
}

impl SearchExecutor {
    pub fn new(handler: Arc<RequestHandler>, auth: Arc<AuthManager>) -> Self {
        Self { handler, auth }
    }

    pub async fn execute(
        &self,
        request: SearchRequest,
        credentials: AuthCredentials,
        endpoint: &str,
    ) -> Result<SearchResponse, GatewayError> {
        self.auth
            .authenticate(endpoint, credentials)
            .await
            .map_err(GatewayError::Authentication)?;
        self.handler
            .process_request(request)
            .await
            .map_err(GatewayError::Handler)
    }
}
