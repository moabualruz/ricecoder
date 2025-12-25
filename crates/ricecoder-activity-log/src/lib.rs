//! RiceCoder Activity Logging and Audit Trails
//!
//! This crate provides comprehensive activity logging, audit trails, and monitoring
//! capabilities for RiceCoder operations. It enables structured logging of user actions,
//! system events, and compliance-related activities.
//!
//! ## Features
//!
//! - **Structured Event Logging**: Hierarchical logging with different severity levels
//! - **Audit Trails**: Immutable records of important actions for compliance
//! - **Session Activity Tracking**: Monitor user sessions and activity patterns
//! - **Performance Monitoring**: Track system performance and bottlenecks
//! - **Compliance Logging**: Regulatory-compliant logging for enterprise use
//!
//! ## Architecture
//!
//! The activity logging system is designed for:
//!
//! - **High Performance**: Async logging with minimal overhead
//! - **Structured Data**: JSON-based logging for easy querying and analysis
//! - **Multiple Outputs**: Console, file, database, and external service logging
//! - **Filtering & Search**: Powerful filtering and search capabilities
//! - **Retention Policies**: Configurable log retention and archiving
//!
//! ## Usage
//!
//! ```rust,no_run
//! use ricecoder_activity_log::{ActivityLogger, LogLevel, ActivityEvent, EventCategory};
//!
//! # async fn example() {
//! // Create a logger
//! let logger = ActivityLogger::new();
//!
//! // Log an activity event
//! logger.log_activity(ActivityEvent {
//!     level: LogLevel::Info,
//!     category: EventCategory::Custom("user_action".to_string()),
//!     action: "file_opened".to_string(),
//!     actor: "user123".to_string(),
//!     resource: "/path/to/file.rs".to_string(),
//!     details: serde_json::json!({"size": 1024}),
//!     session_id: Some("session-abc".to_string()),
//!     ..Default::default()
//! }).await;
//! # }
//! ```

pub mod audit;
pub mod di;
pub mod error;
pub mod events;
pub mod logger;
pub mod monitoring;
pub mod session_tracking;
pub mod storage;

// Re-export commonly used types
pub use audit::{AuditLogger, AuditTrail, ComplianceEvent};
pub use error::{ActivityLogError, ActivityLogResult};
pub use events::{ActivityEvent, EventCategory, LogLevel};
pub use logger::{ActivityLogger, LogStorage, LoggerConfig};
pub use monitoring::{MetricsCollector, PerformanceMonitor};
pub use session_tracking::{SessionActivity, SessionTracker};
pub use storage::RetentionPolicy;
