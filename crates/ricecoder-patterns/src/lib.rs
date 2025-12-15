#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! # RiceCoder Patterns
//!
//! Code pattern detection and architectural analysis for identifying design patterns,
//! architectural styles, and coding conventions in software projects.
//!
//! ## Features
//!
//! - **Architectural Pattern Detection**: Identify layered, microservices, event-driven, and monolithic architectures
//! - **Design Pattern Detection**: Detect common design patterns like factory, observer, repository
//! - **Coding Convention Analysis**: Analyze naming conventions, documentation styles, import organization
//! - **Pattern Stability**: Ensure consistent pattern detection across multiple runs
//! - **Extensible Framework**: Easy to add new pattern detectors

pub mod architectural;
pub mod coding;
pub mod detector;
pub mod error;
pub mod models;

pub use architectural::ArchitecturalPatternDetector;
pub use coding::CodingPatternDetector;
pub use detector::PatternDetector;
pub use error::{PatternError, PatternResult};
pub use models::*;

/// Re-export commonly used types
pub type Result<T> = std::result::Result<T, PatternError>;