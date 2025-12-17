#![warn(missing_docs)]

//! RiceCoder RESTful API
//!
//! Provides RESTful endpoints for session management, MCP tool execution,
//! and enterprise features with comprehensive authentication and monitoring.

pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod server;
pub mod state;

pub use server::ApiServer;
pub use state::AppState;