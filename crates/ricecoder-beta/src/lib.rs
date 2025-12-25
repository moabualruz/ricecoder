//! # RiceCoder Beta Testing Infrastructure
//!
//! This crate provides comprehensive beta testing capabilities for RiceCoder,
//! including user feedback collection, analytics, enterprise requirements validation,
//! and compliance testing for enterprise deployment scenarios.
//!
//! ## Features
//!
//! - **User Feedback Collection**: Structured feedback forms and analytics
//! - **Enterprise Validation**: SOC 2, GDPR, HIPAA compliance testing
//! - **Performance Monitoring**: Beta user performance metrics and regression detection
//! - **Analytics Dashboard**: Real-time beta testing insights and reporting
//! - **Compliance Reporting**: Automated compliance validation and reporting
//! - **Continuous Improvement**: Orchestrates feedback, analytics, issue detection, and roadmap planning

pub mod analytics;
pub mod compliance;
pub mod feedback;
pub mod validation;
pub mod improvement;

pub use analytics::*;
pub use compliance::*;
pub use feedback::*;
pub use validation::*;
