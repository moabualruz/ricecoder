//! RiceCoder Enterprise Industry Integrations
//!
//! This crate provides enterprise-grade integrations and OAuth authentication
//! for RiceCoder, enabling seamless connections with industry tools and platforms.
//!
//! ## Features
//!
//! - **OAuth Authentication**: Secure OAuth 2.0 flows for enterprise integrations
//! - **Tool Connections**: Abstractions for connecting to external industry tools
//! - **Security Validation**: Enterprise-grade security validation and compliance
//! - **Audit Logging**: Comprehensive audit trails for enterprise compliance
//! - **Provider Integrations**: Support for major enterprise AI and development platforms
//!
//! ## Architecture
//!
//! The industry crate serves as the enterprise integration layer, providing:
//!
//! - Authentication flows for enterprise platforms (GitHub Enterprise, GitLab, etc.)
//! - Tool connection abstractions with security validation
//! - Compliance features and audit logging
//! - Provider integrations with enterprise security requirements
//!
//! ## Security Considerations
//!
//! All integrations implement enterprise security standards:
//! - Secure token storage and rotation
//! - Audit logging for all operations
//! - Input validation and sanitization
//! - Rate limiting and abuse prevention
//! - Compliance with enterprise security policies

pub mod auth;
pub mod compliance;
pub mod connections;
pub mod error;
pub mod providers;
pub mod tools;

// Re-export commonly used types
pub use auth::{OAuthClient, OAuthConfig, OAuthFlow, OAuthToken};
pub use compliance::{AuditLogger, ComplianceManager, SecurityValidator};
pub use connections::{ConnectionManager, ToolConnection, ToolConnector};
pub use error::{IndustryError, IndustryResult};
pub use providers::{EnterpriseProvider, ProviderManager};
pub use tools::{IndustryTool, ToolRegistry};
