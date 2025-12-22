//! API gateway and SDK entry points for the hybrid search service.
//! This module currently provides lightweight scaffolding tied to the spec.

pub mod auth;
pub mod error;
pub mod execution;
pub mod gateway;
pub mod graphql;
pub mod handler;
pub mod http;
pub mod models;
pub mod openapi;

pub use auth::{AuthConfig, AuthCredentials, AuthManager, AuthMethod, RateLimitConfig};
pub use error::{GatewayError, HandlerError, SDKError};
pub use execution::SearchExecutor;
pub use gateway::{APIGateway, APIGatewayConfig};
pub use handler::{RequestHandler, RequestHandlerConfig};
pub use models::{
    EnrichedQuery, HealthStatus, RankingConfig, SDKConfig, SearchFilters, SearchOptions,
    SearchRequest, SearchResponse, SearchResult,
};
