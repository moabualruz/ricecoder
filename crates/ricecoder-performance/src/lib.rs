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
pub mod enterprise;
pub mod memory;
pub mod monitor;
pub mod optimization;
pub mod profiler;
pub mod regression;
pub mod simulation;
pub mod validation;

pub use baseline::{PerformanceBaseline, BaselineData};
pub use detector::PerformanceRegressionDetector;
pub use enterprise::{EnterpriseMonitor, AlertConfig, AlertDestination, AlertSeverity, SmtpConfig};
pub use memory::MemoryProfiler;
pub use monitor::{PerformanceMonitor, PerformanceMetrics};
pub use optimization::{OptimizationPipeline, OptimizationResult, OptimizationPriority, create_default_pipeline};
pub use profiler::{PerformanceProfiler, ProfileResult};
pub use regression::{RegressionAlert, RegressionConfig};
pub use simulation::{EnterpriseSimulator, SimulationResult};
pub use validation::{ValidationResult, PerformanceValidator};