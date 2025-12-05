//! Property-based tests for status reporting
//! **Feature: ricecoder-workflows, Property 11: Status Reporting**
//! **Validates: Requirements 2.5**

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use crate::{
        StatusReporter, WorkflowState, WorkflowStatus,
    };
    use chrono::Utc;
    use std::collections::HashMap;

    // Strategy for generating valid workflow states with total steps
    fn workflow_state_with_total_steps_strategy() -> impl Strategy<Value = (WorkflowState, usize)> {
        (1usize..100)
            .prop_flat_map(|total_steps| {
                (
                    "workflow-[a-z0-9]{5}",
                    0usize..=total_steps,
                )
                    .prop_map(move |(workflow_id, completed_count)| {
                        let completed_steps: Vec<String> = (0..completed_count)
                            .map(|i| format!("step-{}", i))
                            .collect();

                        let current_step = if completed_count < total_steps {
                            Some(format!("step-{}", completed_count))
                        } else {
                            None
                        };

                        let state = WorkflowState {
                            workflow_id,
                            status: WorkflowStatus::Running,
                            current_step,
                            completed_steps,
                            step_results: HashMap::new(),
                            started_at: Utc::now(),
                            updated_at: Utc::now(),
                        };

                        (state, total_steps)
                    })
            })
    }

    // Strategy for generating valid total step counts
    fn total_steps_strategy() -> impl Strategy<Value = usize> {
        1usize..100
    }

    proptest! {
        /// Property 11: Status Reporting
        /// For any executing workflow, status updates SHALL include the current step,
        /// progress percentage (0-100), and estimated completion time.
        #[test]
        fn prop_status_report_includes_required_fields(
            (state, total_steps) in workflow_state_with_total_steps_strategy(),
        ) {
            let reporter = StatusReporter::new(total_steps);
            let report = reporter.get_status(&state);

            // Progress percentage must be between 0 and 100
            prop_assert!(report.progress_percentage <= 100);

            // Completed steps count must not exceed total steps
            prop_assert!(report.completed_steps_count <= report.total_steps);

            // Total steps must match what we created
            prop_assert_eq!(report.total_steps, total_steps);

            // Current step should match state
            prop_assert_eq!(report.current_step, state.current_step);

            // Workflow status should match state
            prop_assert_eq!(report.workflow_status, state.status);
        }

        /// Property: Progress percentage is accurate
        /// For any workflow state, progress percentage should equal
        /// (completed_steps / total_steps) * 100, capped at 100
        #[test]
        fn prop_progress_percentage_is_accurate(
            (state, total_steps) in workflow_state_with_total_steps_strategy(),
        ) {
            let reporter = StatusReporter::new(total_steps);
            let report = reporter.get_status(&state);

            let expected_percentage = if total_steps == 0 {
                0
            } else {
                ((state.completed_steps.len() as u32 * 100) / total_steps as u32).min(100)
            };

            prop_assert_eq!(report.progress_percentage, expected_percentage);
        }

        /// Property: Completed steps count matches state
        /// For any workflow state, the reported completed steps count
        /// should match the actual completed steps in the state
        #[test]
        fn prop_completed_steps_count_matches_state(
            (state, total_steps) in workflow_state_with_total_steps_strategy(),
        ) {
            let reporter = StatusReporter::new(total_steps);
            let report = reporter.get_status(&state);

            prop_assert_eq!(report.completed_steps_count, state.completed_steps.len());
        }

        /// Property: Status report is consistent across calls
        /// For any workflow state, calling get_status multiple times
        /// should produce consistent results
        #[test]
        fn prop_status_report_is_consistent(
            (state, total_steps) in workflow_state_with_total_steps_strategy(),
        ) {
            let reporter = StatusReporter::new(total_steps);

            let report1 = reporter.get_status(&state);
            let report2 = reporter.get_status(&state);

            // Progress percentage should be the same
            prop_assert_eq!(report1.progress_percentage, report2.progress_percentage);

            // Completed steps count should be the same
            prop_assert_eq!(report1.completed_steps_count, report2.completed_steps_count);

            // Current step should be the same
            prop_assert_eq!(report1.current_step, report2.current_step);

            // Workflow status should be the same
            prop_assert_eq!(report1.workflow_status, report2.workflow_status);
        }

        /// Property: Recording step durations enables estimation
        /// For any workflow with recorded step durations,
        /// estimated completion time should be available
        #[test]
        fn prop_step_durations_enable_estimation(
            (state, total_steps) in workflow_state_with_total_steps_strategy(),
            durations in prop::collection::vec(1u64..1000, 1..10),
        ) {
            let reporter = StatusReporter::new(total_steps);

            // Record step durations
            for duration in durations {
                reporter.record_step_duration(duration);
            }

            let report = reporter.get_status(&state);

            // If we have recorded durations and remaining steps,
            // we should have an estimated completion time
            if !state.completed_steps.is_empty() || state.completed_steps.len() < total_steps {
                // Estimation should be available
                prop_assert!(report.estimated_completion_time.is_some());
            }
        }

        /// Property: Average step duration is calculated correctly
        /// For any set of recorded step durations,
        /// the average should be the sum divided by count
        #[test]
        fn prop_average_step_duration_is_correct(
            durations in prop::collection::vec(1u64..1000, 1..100),
        ) {
            let reporter = StatusReporter::new(10);

            for duration in &durations {
                reporter.record_step_duration(*duration);
            }

            let avg = reporter.get_average_step_duration();
            let expected_avg = durations.iter().sum::<u64>() / durations.len() as u64;

            prop_assert_eq!(avg, Some(expected_avg));
        }

        /// Property: Min and max step durations are correct
        /// For any set of recorded step durations,
        /// min should be the smallest and max should be the largest
        #[test]
        fn prop_min_max_step_durations_are_correct(
            durations in prop::collection::vec(1u64..1000, 1..100),
        ) {
            let reporter = StatusReporter::new(10);

            for duration in &durations {
                reporter.record_step_duration(*duration);
            }

            let min = reporter.get_min_step_duration();
            let max = reporter.get_max_step_duration();

            let expected_min = durations.iter().copied().min();
            let expected_max = durations.iter().copied().max();

            prop_assert_eq!(min, expected_min);
            prop_assert_eq!(max, expected_max);
        }

        /// Property: Last status is updated after get_status
        /// For any workflow state, calling get_status should update last_status
        #[test]
        fn prop_last_status_is_updated(
            (state, total_steps) in workflow_state_with_total_steps_strategy(),
        ) {
            let reporter = StatusReporter::new(total_steps);

            // Initially, last_status should be None
            prop_assert!(reporter.get_last_status().is_none());

            // After calling get_status, last_status should be Some
            let _ = reporter.get_status(&state);
            prop_assert!(reporter.get_last_status().is_some());

            // The last_status should match the current status
            let last = reporter.get_last_status().unwrap();
            let current = reporter.get_status(&state);

            prop_assert_eq!(last.progress_percentage, current.progress_percentage);
            prop_assert_eq!(last.completed_steps_count, current.completed_steps_count);
        }

        /// Property: Formatted status contains all required information
        /// For any workflow state, the formatted status string should contain
        /// workflow status, progress, and current step information
        #[test]
        fn prop_formatted_status_contains_required_info(
            (state, total_steps) in workflow_state_with_total_steps_strategy(),
        ) {
            let reporter = StatusReporter::new(total_steps);
            let formatted = reporter.format_status(&state);

            // Should contain workflow status
            prop_assert!(formatted.contains("Workflow Status"));

            // Should contain progress information
            prop_assert!(formatted.contains("Progress"));

            // Should contain step count information
            prop_assert!(formatted.contains("steps"));
        }
    }
}
