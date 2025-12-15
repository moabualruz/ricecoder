//! Performance monitoring and regression detection for RiceCoder
//!
//! This crate provides comprehensive performance baseline validation including:
//! - Startup time monitoring (<3s cold start)
//! - Response time tracking (<500ms for typical operations)
//! - Memory usage monitoring (<300MB)
//! - Large project support validation
//! - Concurrent session testing
//! - Performance regression detection with automated alerting

pub mod baseline;
pub mod detector;
pub mod memory;
pub mod monitor;
pub mod regression;
pub mod validation;

pub use baseline::{PerformanceBaseline, BaselineData};
pub use detector::PerformanceRegressionDetector;
pub use memory::MemoryProfiler;
pub use monitor::{PerformanceMonitor, PerformanceMetrics};
pub use regression::{RegressionAlert, RegressionConfig};
pub use validation::{ValidationResult, PerformanceValidator};