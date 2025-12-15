//! Benchmark error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BenchmarkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Test execution error: {0}")]
    TestExecution(String),

    #[error("Exercise not found: {0}")]
    ExerciseNotFound(String),

    #[error("Timeout error")]
    Timeout,

    #[error("Configuration error: {0}")]
    Config(String),
}