//! Progress tracking and status reporting for workflows

use crate::models::{WorkflowState, WorkflowStatus};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Tracks workflow progress and provides status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressTracker {
    /// Total number of steps in the workflow
    pub total_steps: usize,
    /// Step durations in milliseconds (for estimation)
    pub step_durations: Vec<u64>,
}

/// Status report for a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusReport {
    /// Current executing step ID
    pub current_step: Option<String>,
    /// Progress percentage (0-100)
    pub progress_percentage: u32,
    /// Estimated completion time
    pub estimated_completion_time: Option<DateTime<Utc>>,
    /// Workflow status
    pub workflow_status: WorkflowStatus,
    /// Number of completed steps
    pub completed_steps_count: usize,
    /// Total steps in workflow
    pub total_steps: usize,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(total_steps: usize) -> Self {
        ProgressTracker {
            total_steps,
            step_durations: Vec::new(),
        }
    }

    /// Record a step duration
    pub fn record_step_duration(&mut self, duration_ms: u64) {
        self.step_durations.push(duration_ms);
    }

    /// Calculate progress percentage (0-100)
    pub fn calculate_progress(&self, completed_steps: usize) -> u32 {
        if self.total_steps == 0 {
            return 0;
        }

        ((completed_steps as u32 * 100) / self.total_steps as u32).min(100)
    }

    /// Estimate completion time based on step durations
    pub fn estimate_completion_time(
        &self,
        state: &WorkflowState,
        now: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        if self.step_durations.is_empty() || self.total_steps == 0 {
            return None;
        }

        // Calculate average step duration
        let total_duration: u64 = self.step_durations.iter().sum();
        let avg_duration_ms = total_duration / self.step_durations.len() as u64;

        // Calculate remaining steps
        let remaining_steps = self.total_steps.saturating_sub(state.completed_steps.len());

        // Estimate remaining time
        let estimated_remaining_ms = remaining_steps as u64 * avg_duration_ms;

        // Add to current time
        now.checked_add_signed(Duration::milliseconds(estimated_remaining_ms as i64))
    }

    /// Generate a status report
    pub fn generate_status_report(
        &self,
        state: &WorkflowState,
        now: DateTime<Utc>,
    ) -> StatusReport {
        let completed_steps_count = state.completed_steps.len();
        let progress_percentage = self.calculate_progress(completed_steps_count);
        let estimated_completion_time = self.estimate_completion_time(state, now);

        StatusReport {
            current_step: state.current_step.clone(),
            progress_percentage,
            estimated_completion_time,
            workflow_status: state.status,
            completed_steps_count,
            total_steps: self.total_steps,
        }
    }

    /// Get average step duration in milliseconds
    pub fn get_average_step_duration(&self) -> Option<u64> {
        if self.step_durations.is_empty() {
            return None;
        }

        let total: u64 = self.step_durations.iter().sum();
        Some(total / self.step_durations.len() as u64)
    }

    /// Get minimum step duration in milliseconds
    pub fn get_min_step_duration(&self) -> Option<u64> {
        self.step_durations.iter().copied().min()
    }

    /// Get maximum step duration in milliseconds
    pub fn get_max_step_duration(&self) -> Option<u64> {
        self.step_durations.iter().copied().max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_workflow_state() -> WorkflowState {
        WorkflowState {
            workflow_id: "test-workflow".to_string(),
            status: WorkflowStatus::Running,
            current_step: Some("step1".to_string()),
            completed_steps: vec!["step0".to_string()],
            step_results: Default::default(),
            started_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_create_progress_tracker() {
        let tracker = ProgressTracker::new(10);
        assert_eq!(tracker.total_steps, 10);
        assert!(tracker.step_durations.is_empty());
    }

    #[test]
    fn test_record_step_duration() {
        let mut tracker = ProgressTracker::new(10);
        tracker.record_step_duration(100);
        tracker.record_step_duration(200);

        assert_eq!(tracker.step_durations.len(), 2);
        assert_eq!(tracker.step_durations[0], 100);
        assert_eq!(tracker.step_durations[1], 200);
    }

    #[test]
    fn test_calculate_progress() {
        let tracker = ProgressTracker::new(10);

        assert_eq!(tracker.calculate_progress(0), 0);
        assert_eq!(tracker.calculate_progress(5), 50);
        assert_eq!(tracker.calculate_progress(10), 100);
        assert_eq!(tracker.calculate_progress(15), 100); // Capped at 100
    }

    #[test]
    fn test_calculate_progress_zero_steps() {
        let tracker = ProgressTracker::new(0);
        assert_eq!(tracker.calculate_progress(0), 0);
    }

    #[test]
    fn test_get_average_step_duration() {
        let mut tracker = ProgressTracker::new(10);
        tracker.record_step_duration(100);
        tracker.record_step_duration(200);
        tracker.record_step_duration(300);

        assert_eq!(tracker.get_average_step_duration(), Some(200));
    }

    #[test]
    fn test_get_average_step_duration_empty() {
        let tracker = ProgressTracker::new(10);
        assert_eq!(tracker.get_average_step_duration(), None);
    }

    #[test]
    fn test_get_min_step_duration() {
        let mut tracker = ProgressTracker::new(10);
        tracker.record_step_duration(100);
        tracker.record_step_duration(200);
        tracker.record_step_duration(50);

        assert_eq!(tracker.get_min_step_duration(), Some(50));
    }

    #[test]
    fn test_get_max_step_duration() {
        let mut tracker = ProgressTracker::new(10);
        tracker.record_step_duration(100);
        tracker.record_step_duration(200);
        tracker.record_step_duration(50);

        assert_eq!(tracker.get_max_step_duration(), Some(200));
    }

    #[test]
    fn test_estimate_completion_time() {
        let mut tracker = ProgressTracker::new(10);
        tracker.record_step_duration(100);
        tracker.record_step_duration(100);

        let state = create_test_workflow_state();
        let now = Utc::now();

        let estimated = tracker.estimate_completion_time(&state, now);
        assert!(estimated.is_some());

        // With 1 completed step out of 10, and avg 100ms per step,
        // remaining 9 steps should take ~900ms
        let estimated_time = estimated.unwrap();
        let diff = estimated_time.signed_duration_since(now);
        assert!(diff.num_milliseconds() > 800 && diff.num_milliseconds() < 1000);
    }

    #[test]
    fn test_estimate_completion_time_no_durations() {
        let tracker = ProgressTracker::new(10);
        let state = create_test_workflow_state();
        let now = Utc::now();

        let estimated = tracker.estimate_completion_time(&state, now);
        assert!(estimated.is_none());
    }

    #[test]
    fn test_generate_status_report() {
        let mut tracker = ProgressTracker::new(10);
        tracker.record_step_duration(100);

        let state = create_test_workflow_state();
        let now = Utc::now();

        let report = tracker.generate_status_report(&state, now);

        assert_eq!(report.current_step, Some("step1".to_string()));
        assert_eq!(report.progress_percentage, 10); // 1 out of 10
        assert_eq!(report.completed_steps_count, 1);
        assert_eq!(report.total_steps, 10);
        assert_eq!(report.workflow_status, WorkflowStatus::Running);
        assert!(report.estimated_completion_time.is_some());
    }
}
