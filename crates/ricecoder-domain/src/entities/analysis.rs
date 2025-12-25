//! Analysis entities for code analysis results and metrics

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_objects::*;

/// Analysis result entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub id: String,
    pub project_id: ProjectId,
    pub file_id: Option<FileId>,
    pub analysis_type: AnalysisType,
    pub status: AnalysisStatus,
    pub results: serde_json::Value,
    pub metrics: AnalysisMetrics,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl AnalysisResult {
    /// Create a new analysis result
    pub fn new(
        project_id: ProjectId,
        file_id: Option<FileId>,
        analysis_type: AnalysisType,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            file_id,
            analysis_type,
            status: AnalysisStatus::Pending,
            results: serde_json::Value::Null,
            metrics: AnalysisMetrics::default(),
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Start analysis execution (Pending → Running)
    pub fn start(&mut self) {
        debug_assert!(
            self.status == AnalysisStatus::Pending,
            "Can only start from Pending state"
        );
        self.status = AnalysisStatus::Running;
    }

    /// Mark analysis as completed
    pub fn complete(&mut self, results: serde_json::Value, metrics: AnalysisMetrics) {
        self.status = AnalysisStatus::Completed;
        self.results = results;
        self.metrics = metrics;
        self.completed_at = Some(Utc::now());
    }

    /// Mark analysis as failed
    pub fn fail(&mut self, error: String) {
        self.status = AnalysisStatus::Failed;
        self.results = serde_json::Value::String(error);
        self.completed_at = Some(Utc::now());
    }

    /// Cancel analysis
    pub fn cancel(&mut self) {
        self.status = AnalysisStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }

    /// Check if analysis is complete (including cancelled)
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            AnalysisStatus::Completed | AnalysisStatus::Failed | AnalysisStatus::Cancelled
        )
    }

    /// Check if analysis is currently running
    pub fn is_running(&self) -> bool {
        self.status == AnalysisStatus::Running
    }

    /// Check if analysis was cancelled
    pub fn is_cancelled(&self) -> bool {
        self.status == AnalysisStatus::Cancelled
    }
}

/// Analysis type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisType {
    Syntax,
    Semantic,
    Complexity,
    Dependencies,
    Patterns,
    Security,
    Performance,
}

/// Analysis status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Analysis metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    pub lines_of_code: usize,
    pub cyclomatic_complexity: f64,
    pub maintainability_index: f64,
    pub technical_debt_ratio: f64,
    pub execution_time_ms: u64,
}

impl Default for AnalysisMetrics {
    fn default() -> Self {
        Self {
            lines_of_code: 0,
            cyclomatic_complexity: 0.0,
            maintainability_index: 100.0,
            technical_debt_ratio: 0.0,
            execution_time_ms: 0,
        }
    }
}
