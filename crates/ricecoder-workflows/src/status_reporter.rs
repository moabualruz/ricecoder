//! Status reporting for workflow execution

use crate::models::WorkflowState;
use crate::progress::{ProgressTracker, StatusReport};
use chrono::Utc;

use std::sync::{Arc, Mutex};

/// Status reporter for providing real-time workflow status updates
#[derive(Debug, Clone)]
pub struct StatusReporter {
    /// Progress tracker
    progress_tracker: Arc<Mutex<ProgressTracker>>,
    /// Last reported status
    last_status: Arc<Mutex<Option<StatusReport>>>,
}

impl StatusReporter {
    /// Create a new status reporter
    pub fn new(total_steps: usize) -> Self {
        StatusReporter {
            progress_tracker: Arc::new(Mutex::new(ProgressTracker::new(total_steps))),
            last_status: Arc::new(Mutex::new(None)),
        }
    }

    /// Record a step duration
    pub fn record_step_duration(&self, duration_ms: u64) {
        if let Ok(mut tracker) = self.progress_tracker.lock() {
            tracker.record_step_duration(duration_ms);
        }
    }

    /// Get current status report
    pub fn get_status(&self, state: &WorkflowState) -> StatusReport {
        let now = Utc::now();
        let tracker = self.progress_tracker.lock().unwrap();
        let report = tracker.generate_status_report(state, now);

        // Update last status
        if let Ok(mut last_status) = self.last_status.lock() {
            *last_status = Some(report.clone());
        }

        report
    }

    /// Get last reported status
    pub fn get_last_status(&self) -> Option<StatusReport> {
        self.last_status.lock().ok().and_then(|status| status.clone())
    }

    /// Get average step duration
    pub fn get_average_step_duration(&self) -> Option<u64> {
        self.progress_tracker
            .lock()
            .ok()
            .and_then(|tracker| tracker.get_average_step_duration())
    }

    /// Get minimum step duration
    pub fn get_min_step_duration(&self) -> Option<u64> {
        self.progress_tracker
            .lock()
            .ok()
            .and_then(|tracker| tracker.get_min_step_duration())
    }

    /// Get maximum step duration
    pub fn get_max_step_duration(&self) -> Option<u64> {
        self.progress_tracker
            .lock()
            .ok()
            .and_then(|tracker| tracker.get_max_step_duration())
    }

    /// Format status report as a human-readable string
    pub fn format_status(&self, state: &WorkflowState) -> String {
        let report = self.get_status(state);

        let mut output = String::new();
        output.push_str(&format!("Workflow Status: {:?}\n", report.workflow_status));
        output.push_str(&format!(
            "Progress: {}/{} steps ({}%)\n",
            report.completed_steps_count, report.total_steps, report.progress_percentage
        ));

        if let Some(current_step) = &report.current_step {
            output.push_str(&format!("Current Step: {}\n", current_step));
        }

        if let Some(eta) = report.estimated_completion_time {
            output.push_str(&format!("Estimated Completion: {}\n", eta.format("%Y-%m-%d %H:%M:%S")));
        }

        output
    }
}

/// Real-time status update callback
pub type StatusUpdateCallback = Box<dyn Fn(&StatusReport) + Send + Sync>;

/// Status update listener for receiving real-time updates
#[derive(Clone)]
pub struct StatusUpdateListener {
    callbacks: Arc<Mutex<Vec<StatusUpdateCallback>>>,
}

impl StatusUpdateListener {
    /// Create a new status update listener
    pub fn new() -> Self {
        StatusUpdateListener {
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a callback for status updates
    pub fn on_status_update<F>(&self, callback: F)
    where
        F: Fn(&StatusReport) + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.callbacks.lock() {
            callbacks.push(Box::new(callback));
        }
    }

    /// Notify all listeners of a status update
    pub fn notify(&self, report: &StatusReport) {
        if let Ok(callbacks) = self.callbacks.lock() {
            for callback in callbacks.iter() {
                callback(report);
            }
        }
    }

    /// Clear all callbacks
    pub fn clear(&self) {
        if let Ok(mut callbacks) = self.callbacks.lock() {
            callbacks.clear();
        }
    }
}

impl Default for StatusUpdateListener {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::WorkflowStatus;

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
    fn test_create_status_reporter() {
        let reporter = StatusReporter::new(10);
        let state = create_test_workflow_state();

        let report = reporter.get_status(&state);
        assert_eq!(report.total_steps, 10);
        assert_eq!(report.completed_steps_count, 1);
    }

    #[test]
    fn test_record_step_duration() {
        let reporter = StatusReporter::new(10);
        reporter.record_step_duration(100);
        reporter.record_step_duration(200);

        assert_eq!(reporter.get_average_step_duration(), Some(150));
    }

    #[test]
    fn test_get_status() {
        let reporter = StatusReporter::new(10);
        let state = create_test_workflow_state();

        let report = reporter.get_status(&state);
        assert_eq!(report.current_step, Some("step1".to_string()));
        assert_eq!(report.progress_percentage, 10);
        assert_eq!(report.completed_steps_count, 1);
    }

    #[test]
    fn test_get_last_status() {
        let reporter = StatusReporter::new(10);
        let state = create_test_workflow_state();

        assert!(reporter.get_last_status().is_none());

        reporter.get_status(&state);
        assert!(reporter.get_last_status().is_some());
    }

    #[test]
    fn test_format_status() {
        let reporter = StatusReporter::new(10);
        let state = create_test_workflow_state();

        let formatted = reporter.format_status(&state);
        assert!(formatted.contains("Workflow Status"));
        assert!(formatted.contains("Progress"));
        assert!(formatted.contains("Current Step"));
    }

    #[test]
    fn test_status_update_listener() {
        let listener = StatusUpdateListener::new();
        let called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let called_clone = called.clone();

        listener.on_status_update(move |_report| {
            *called_clone.lock().unwrap() = true;
        });

        let state = create_test_workflow_state();
        let reporter = StatusReporter::new(10);
        let report = reporter.get_status(&state);

        listener.notify(&report);
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_status_update_listener_clear() {
        let listener = StatusUpdateListener::new();
        listener.on_status_update(|_report| {});

        let state = create_test_workflow_state();
        let reporter = StatusReporter::new(10);
        let report = reporter.get_status(&state);

        listener.clear();
        listener.notify(&report);
        // If we get here without panicking, clear worked
    }
}
