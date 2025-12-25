//! Centralized HTTP client for RiceCoder
//!
//! Provides a mockable, configurable HTTP client wrapper for all RiceCoder crates.
//!
//! ## Features
//!
//! - **Trait-based design**: Mockable via `HttpClientTrait`
//! - **Configurable**: Timeouts, retries, proxy, user-agent
//! - **Middleware support**: Retry logic with exponential backoff
//! - **Connection pooling**: Managed by underlying reqwest client
//! - **Testing support**: Easy mocking with wiremock

pub mod client;
pub mod config;
pub mod error;
pub mod middleware;

pub use client::{HttpClient, HttpClientTrait};
pub use config::HttpConfig;
pub use error::{HttpError, Result};
pub use middleware::{RetryConfig, RetryMiddleware};

/// Re-export commonly used types
pub use reqwest::{header, Method, Response, StatusCode};
