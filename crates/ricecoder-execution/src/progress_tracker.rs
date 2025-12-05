//! Progress tracking and reporting for execution plans
//!
//! Tracks execution progress and provides callbacks for UI updates.
//! Supports reporting:
//! - Current step and total steps
//! - Overall progress percentage
//! - Estimated time remaining
//! - Progress callbacks for real-time UI updates

use crate::models::ExecutionPlan;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Progress update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    /// Current step index (0-based)
    pub current_step: usize,
    /// Total number of steps
    pub total_steps: usize,
    /// Overall progress percentage (0-100)
    pub progress_percentage: f32,
    /// Estimated time remaining
    pub estimated_time_remaining: Duration,
    /// Timestamp of this update
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Callback function for progress updates
pub type ProgressCallback = Box<dyn Fn(ProgressUpdate) + Send + Sync>;

/// Tracks execution progress and provides real-time updates
///
/// Maintains:
/// - Current step index
/// - Completed steps count
/// - Execution start time
/// - Step durations for time estimation
/// - Progress callbacks for UI updates
pub struct ProgressTracker {
    /// Total number of steps in the plan
    total_steps: usize,
    /// Current step index (0-based)
    current_step: usize,
    /// Number of completed steps
    completed_steps: usize,
    /// Execution start time
    start_time: Instant,
    /// Step durations for time estimation
    step_durations: Vec<Duration>,
    /// Progress callbacks
    callbacks: Arc<Mutex<Vec<ProgressCallback>>>,
}

impl ProgressTracker {
    /// Create a new progress tracker for a plan
    ///
    /// # Arguments
    /// * `plan` - The execution plan to track
    ///
    /// # Returns
    /// A new ProgressTracker initialized for the plan
    pub fn new(plan: &ExecutionPlan) -> Self {
        let total_steps = plan.steps.len();

        info!(total_steps = total_steps, "Creating progress tracker");

        Self {
            total_steps,
            current_step: 0,
            completed_steps: 0,
            start_time: Instant::now(),
            step_durations: Vec::new(),
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a progress callback
    ///
    /// Callbacks are called whenever progress is updated.
    ///
    /// # Arguments
    /// * `callback` - Function to call on progress updates
    pub fn on_progress<F>(&self, callback: F)
    where
        F: Fn(ProgressUpdate) + Send + Sync + 'static,
    {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.push(Box::new(callback));

        debug!(
            callback_count = callbacks.len(),
            "Progress callback registered"
        );
    }

    /// Update progress to the next step
    ///
    /// Increments the current step and records the duration of the previous step.
    ///
    /// # Arguments
    /// * `step_duration` - Duration of the completed step
    pub fn step_completed(&mut self, step_duration: Duration) {
        self.step_durations.push(step_duration);
        self.completed_steps += 1;
        self.current_step += 1;

        debug!(
            current_step = self.current_step,
            completed_steps = self.completed_steps,
            step_duration_ms = step_duration.as_millis(),
            "Step completed"
        );

        self.notify_progress();
    }

    /// Skip a step
    ///
    /// Marks a step as skipped without recording a duration.
    pub fn step_skipped(&mut self) {
        self.step_durations.push(Duration::from_secs(0));
        self.current_step += 1;

        debug!(current_step = self.current_step, "Step skipped");

        self.notify_progress();
    }

    /// Get the current progress update
    ///
    /// # Returns
    /// A ProgressUpdate containing current progress information
    pub fn get_progress(&self) -> ProgressUpdate {
        let progress_percentage = if self.total_steps > 0 {
            (self.completed_steps as f32 / self.total_steps as f32) * 100.0
        } else {
            0.0
        };

        let estimated_time_remaining = self.estimated_time_remaining();

        ProgressUpdate {
            current_step: self.current_step,
            total_steps: self.total_steps,
            progress_percentage,
            estimated_time_remaining,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get the current step index (0-based)
    pub fn current_step(&self) -> usize {
        self.current_step
    }

    /// Get the total number of steps
    pub fn total_steps(&self) -> usize {
        self.total_steps
    }

    /// Get the number of completed steps
    pub fn completed_steps(&self) -> usize {
        self.completed_steps
    }

    /// Get the overall progress percentage (0-100)
    pub fn progress_percentage(&self) -> f32 {
        if self.total_steps > 0 {
            (self.completed_steps as f32 / self.total_steps as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Get the estimated time remaining
    pub fn estimated_time_remaining(&self) -> Duration {
        if self.step_durations.is_empty() || self.completed_steps == 0 {
            // No data yet, estimate based on total steps
            return Duration::from_secs(0);
        }

        // Calculate average step duration
        let total_duration: Duration = self.step_durations.iter().sum();
        let average_duration = total_duration / self.step_durations.len() as u32;

        // Estimate remaining time
        let remaining_steps = self.total_steps.saturating_sub(self.completed_steps);
        average_duration * remaining_steps as u32
    }

    /// Get the total elapsed time
    pub fn elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get the average step duration
    pub fn average_step_duration(&self) -> Duration {
        if self.step_durations.is_empty() {
            return Duration::from_secs(0);
        }

        let total_duration: Duration = self.step_durations.iter().sum();
        total_duration / self.step_durations.len() as u32
    }

    /// Notify all registered callbacks of progress update
    fn notify_progress(&self) {
        let progress = self.get_progress();

        let callbacks = self.callbacks.lock().unwrap();
        for callback in callbacks.iter() {
            callback(progress.clone());
        }
    }

    /// Reset the progress tracker
    ///
    /// Clears all progress data and resets to initial state.
    pub fn reset(&mut self) {
        self.current_step = 0;
        self.completed_steps = 0;
        self.start_time = Instant::now();
        self.step_durations.clear();

        debug!("Progress tracker reset");
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self {
            total_steps: 0,
            current_step: 0,
            completed_steps: 0,
            start_time: Instant::now(),
            step_durations: Vec::new(),
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ExecutionPlan, ExecutionStep, RiskScore, StepAction, StepStatus};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc as StdArc;

    fn create_test_plan(step_count: usize) -> ExecutionPlan {
        let steps = (0..step_count)
            .map(|i| ExecutionStep {
                id: format!("step-{}", i),
                description: format!("Step {}", i),
                action: StepAction::RunCommand {
                    command: "echo".to_string(),
                    args: vec![format!("step {}", i)],
                },
                risk_score: RiskScore::default(),
                dependencies: Vec::new(),
                rollback_action: None,
                status: StepStatus::Pending,
            })
            .collect();

        ExecutionPlan::new("Test Plan".to_string(), steps)
    }

    #[test]
    fn test_create_tracker() {
        let plan = create_test_plan(5);
        let tracker = ProgressTracker::new(&plan);

        assert_eq!(tracker.total_steps(), 5);
        assert_eq!(tracker.current_step(), 0);
        assert_eq!(tracker.completed_steps(), 0);
        assert_eq!(tracker.progress_percentage(), 0.0);
    }

    #[test]
    fn test_step_completed() {
        let plan = create_test_plan(5);
        let mut tracker = ProgressTracker::new(&plan);

        tracker.step_completed(Duration::from_secs(1));

        assert_eq!(tracker.completed_steps(), 1);
        assert_eq!(tracker.current_step(), 1);
        assert_eq!(tracker.progress_percentage(), 20.0);
    }

    #[test]
    fn test_multiple_steps_completed() {
        let plan = create_test_plan(5);
        let mut tracker = ProgressTracker::new(&plan);

        tracker.step_completed(Duration::from_secs(1));
        tracker.step_completed(Duration::from_secs(2));
        tracker.step_completed(Duration::from_secs(1));

        assert_eq!(tracker.completed_steps(), 3);
        assert!((tracker.progress_percentage() - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_step_skipped() {
        let plan = create_test_plan(5);
        let mut tracker = ProgressTracker::new(&plan);

        tracker.step_completed(Duration::from_secs(1));
        tracker.step_skipped();

        assert_eq!(tracker.completed_steps(), 1);
        assert_eq!(tracker.current_step(), 2);
    }

    #[test]
    fn test_progress_percentage() {
        let plan = create_test_plan(10);
        let mut tracker = ProgressTracker::new(&plan);

        for _ in 0..5 {
            tracker.step_completed(Duration::from_secs(1));
        }

        assert_eq!(tracker.progress_percentage(), 50.0);
    }

    #[test]
    fn test_estimated_time_remaining() {
        let plan = create_test_plan(10);
        let mut tracker = ProgressTracker::new(&plan);

        // Complete 2 steps with 1 second each
        tracker.step_completed(Duration::from_secs(1));
        tracker.step_completed(Duration::from_secs(1));

        // Average is 1 second per step
        // 8 remaining steps should estimate to ~8 seconds
        let estimated = tracker.estimated_time_remaining();
        assert!(estimated.as_secs() >= 7 && estimated.as_secs() <= 9);
    }

    #[test]
    fn test_average_step_duration() {
        let plan = create_test_plan(5);
        let mut tracker = ProgressTracker::new(&plan);

        tracker.step_completed(Duration::from_secs(2));
        tracker.step_completed(Duration::from_secs(4));

        let average = tracker.average_step_duration();
        assert_eq!(average, Duration::from_secs(3));
    }

    #[test]
    fn test_elapsed_time() {
        let plan = create_test_plan(5);
        let tracker = ProgressTracker::new(&plan);

        let elapsed = tracker.elapsed_time();
        // Elapsed time should be recorded (even if very small)
        let _ = elapsed;
    }

    #[test]
    fn test_progress_callback() {
        let plan = create_test_plan(5);
        let mut tracker = ProgressTracker::new(&plan);

        let callback_count = StdArc::new(AtomicUsize::new(0));
        let callback_count_clone = callback_count.clone();

        tracker.on_progress(move |_progress| {
            callback_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        tracker.step_completed(Duration::from_secs(1));
        tracker.step_completed(Duration::from_secs(1));

        assert_eq!(callback_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_get_progress() {
        let plan = create_test_plan(5);
        let mut tracker = ProgressTracker::new(&plan);

        tracker.step_completed(Duration::from_secs(1));

        let progress = tracker.get_progress();
        assert_eq!(progress.current_step, 1);
        assert_eq!(progress.total_steps, 5);
        assert_eq!(progress.progress_percentage, 20.0);
    }

    #[test]
    fn test_reset() {
        let plan = create_test_plan(5);
        let mut tracker = ProgressTracker::new(&plan);

        tracker.step_completed(Duration::from_secs(1));
        tracker.step_completed(Duration::from_secs(1));

        tracker.reset();

        assert_eq!(tracker.completed_steps(), 0);
        assert_eq!(tracker.current_step(), 0);
        assert_eq!(tracker.progress_percentage(), 0.0);
    }

    #[test]
    fn test_empty_plan() {
        let plan = create_test_plan(0);
        let tracker = ProgressTracker::new(&plan);

        assert_eq!(tracker.total_steps(), 0);
        assert_eq!(tracker.progress_percentage(), 0.0);
    }

    #[test]
    fn test_progress_update_serialization() {
        let update = ProgressUpdate {
            current_step: 1,
            total_steps: 5,
            progress_percentage: 20.0,
            estimated_time_remaining: Duration::from_secs(4),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&update).unwrap();
        let deserialized: ProgressUpdate = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.current_step, 1);
        assert_eq!(deserialized.total_steps, 5);
        assert_eq!(deserialized.progress_percentage, 20.0);
    }
}
