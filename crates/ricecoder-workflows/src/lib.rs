#![warn(missing_docs)]

//! RiceCoder Core Workflows
//!
//! Provides clean workflow orchestration interfaces with session integration.
//! Core workflow functionality for declarative workflow definitions and execution.

pub mod engine;
pub mod error;
pub mod models;
pub mod parameters;
pub mod parser;
pub mod resolver;

// Re-export core types
pub use engine::WorkflowEngine;
pub use error::*;
pub use models::*;
pub use parameters::*;
pub use parser::WorkflowParser;
pub use resolver::DependencyResolver;
