//! API gateway and SDK entry points for the hybrid search service.
//! This module currently provides lightweight scaffolding tied to the spec.

pub mod auth;
pub mod error;
pub mod models;
pub mod openapi;

// Modules requiring local-embeddings for SearchCoordinator/RequestHandler
#[cfg(feature = "local-embeddings")]
pub mod execution;
#[cfg(feature = "local-embeddings")]
pub mod gateway;
#[cfg(feature = "local-embeddings")]
pub mod graphql;
#[cfg(feature = "local-embeddings")]
pub mod handler;
#[cfg(feature = "local-embeddings")]
pub mod http;

pub use auth::{AuthConfig, AuthCredentials, AuthManager, AuthMethod, RateLimitConfig};
pub use error::{GatewayError, HandlerError, SDKError};
pub use models::{
    EnrichedQuery, HealthStatus, RankingConfig, SDKConfig, SearchFilters, SearchOptions,
    SearchRequest, SearchResponse, SearchResult,
};

#[cfg(feature = "local-embeddings")]
pub use execution::SearchExecutor;
#[cfg(feature = "local-embeddings")]
pub use gateway::{APIGateway, APIGatewayConfig};
#[cfg(feature = "local-embeddings")]
pub use handler::{RequestHandler, RequestHandlerConfig};
