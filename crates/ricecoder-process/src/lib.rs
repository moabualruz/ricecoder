//! # ricecoder-process
//!
//! **Purpose**: Shared process lifecycle management for RiceCoder
//!
//! Provides cross-platform process spawning, PID tracking, graceful shutdown,
//! timeout handling, and tree-kill functionality.
//!
//! ## Features
//!
//! - **Process Spawning**: Async process creation with full stdio control
//! - **PID Tracking**: Track process IDs for monitoring and cleanup
//! - **Graceful Shutdown**: SIGTERM→SIGKILL escalation with configurable timeouts
//! - **Timeout Support**: Per-process and per-operation timeouts
//! - **Output Capture**: Capture stdout/stderr with buffering
//! - **Signal Handling**: Cross-platform signal delivery
//! - **Process Tree Kill**: Kill process groups on Unix, task trees on Windows
//!
//! ## Usage
//!
//! ```rust,no_run
//! use ricecoder_process::{ProcessManager, ProcessConfig};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create process manager
//! let manager = ProcessManager::new();
//!
//! // Configure process
//! let config = ProcessConfig::new("rust-analyzer")
//!     .args(&["--stdio"])
//!     .timeout_secs(30);
//!
//! // Spawn process
//! let child = manager.spawn(config).await?;
//!
//! // Clean shutdown
//! manager.shutdown(child).await?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod error;
pub mod manager;
pub mod child;

pub use config::ProcessConfig;
pub use error::{ProcessError, Result};
pub use manager::ProcessManager;
pub use child::ManagedChild;
