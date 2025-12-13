//! Property-based tests for activity logging
//! **Feature: ricecoder-workflows, Property 12: Activity Logging**
//! **Validates: Requirements 2.6**



    // Strategy for generating step IDs
    fn step_id_strategy() -> impl Strategy<Value = Option<String>> {
        prop_oneof![Just(None), "step-[a-z0-9]{5}".prop_map(Some),]
    }

    proptest! {
        /// Property 12: Activity Logging
        /// For any workflow execution, the system SHALL log all step starts,
        /// completions, errors, and state transitions.
        #[test]
        fn prop_activity_logger_records_all_activities(
            activity_type in activity_type_strategy(),
            step_id in step_id_strategy(),
            message in ".*",
        ) {
            let mut logger = ActivityLogger::new(1000);

            // Log an activity
            logger.log(
                activity_type,
                step_id.clone(),
                message.clone(),
                serde_json::json!({}),
            );

            // Verify the activity was recorded
            prop_assert_eq!(logger.len(), 1);

            let entries = logger.get_entries();
            prop_assert_eq!(entries.len(), 1);
            prop_assert_eq!(entries[0].activity_type, activity_type);
            prop_assert_eq!(&entries[0].step_id, &step_id);
            prop_assert_eq!(&entries[0].message, &message);
        }

        /// Property: Activity log entries have timestamps
        /// For any logged activity, the entry should have a valid timestamp
        #[test]
        fn prop_activity_log_entries_have_timestamps(
            activity_type in activity_type_strategy(),
        ) {
            let mut logger = ActivityLogger::new(1000);

            logger.log(
                activity_type,
                None,
                "Test activity".to_string(),
                serde_json::json!({}),
            );

            let entries = logger.get_entries();
            prop_assert_eq!(entries.len(), 1);

            // Timestamp should be recent (within last minute)
            let now = chrono::Utc::now();
            let diff = now.signed_duration_since(entries[0].timestamp);
            prop_assert!(diff.num_seconds() >= 0 && diff.num_seconds() < 60);
        }

        /// Property: Activity log respects max_entries limit
        /// For any activity logger with a max_entries limit,
        /// the number of entries should never exceed that limit
        #[test]
        fn prop_activity_log_respects_max_entries(
            max_entries in 1usize..100,
            num_activities in 1usize..200,
        ) {
            let mut logger = ActivityLogger::new(max_entries);

            for i in 0..num_activities {
                logger.log(
                    ActivityType::StepStarted,
                    Some(format!("step-{}", i)),
                    format!("Activity {}", i),
                    serde_json::json!({}),
                );
            }

            // Should not exceed max_entries
            prop_assert!(logger.len() <= max_entries);
        }

        /// Property: Activity log entries can be filtered by type
        /// For any set of logged activities, filtering by type should return
        /// only entries of that type
        #[test]
        fn prop_activity_log_filtering_by_type_is_correct(
            num_started in 1usize..20,
            num_completed in 1usize..20,
        ) {
            let mut logger = ActivityLogger::new(1000);

            // Log some started activities
            for i in 0..num_started {
                logger.log(
                    ActivityType::StepStarted,
                    Some(format!("step-{}", i)),
                    "Started".to_string(),
                    serde_json::json!({}),
                );
            }

            // Log some completed activities
            for i in 0..num_completed {
                logger.log(
                    ActivityType::StepCompleted,
                    Some(format!("step-{}", i)),
                    "Completed".to_string(),
                    serde_json::json!({}),
                );
            }

            // Filter by type
            let started = logger.get_entries_by_type(ActivityType::StepStarted);
            let completed = logger.get_entries_by_type(ActivityType::StepCompleted);

            prop_assert_eq!(started.len(), num_started);
            prop_assert_eq!(completed.len(), num_completed);

            // All started entries should have StepStarted type
            for entry in started {
                prop_assert_eq!(entry.activity_type, ActivityType::StepStarted);
            }

            // All completed entries should have StepCompleted type
            for entry in completed {
                prop_assert_eq!(entry.activity_type, ActivityType::StepCompleted);
            }
        }

        /// Property: Activity log entries can be filtered by step ID
        /// For any set of logged activities, filtering by step ID should return
        /// only entries for that step
        #[test]
        fn prop_activity_log_filtering_by_step_is_correct(
            num_steps in 1usize..10,
            activities_per_step in 1usize..10,
        ) {
            let mut logger = ActivityLogger::new(1000);

            // Log activities for multiple steps
            for step_num in 0..num_steps {
                let step_id = format!("step-{}", step_num);
                for activity_num in 0..activities_per_step {
                    logger.log(
                        ActivityType::StepStarted,
                        Some(step_id.clone()),
                        format!("Activity {}", activity_num),
                        serde_json::json!({}),
                    );
                }
            }

            // Filter by step ID
            for step_num in 0..num_steps {
                let step_id = format!("step-{}", step_num);
                let entries = logger.get_entries_for_step(&step_id);

                prop_assert_eq!(entries.len(), activities_per_step);

                // All entries should be for this step
                for entry in entries {
                    prop_assert_eq!(entry.step_id, Some(step_id.clone()));
                }
            }
        }

        /// Property: Activity log can be cleared
        /// For any activity logger with entries, clearing should remove all entries
        #[test]
        fn prop_activity_log_can_be_cleared(
            num_activities in 1usize..100,
        ) {
            let mut logger = ActivityLogger::new(1000);

            // Log some activities
            for i in 0..num_activities {
                logger.log(
                    ActivityType::StepStarted,
                    Some(format!("step-{}", i)),
                    "Activity".to_string(),
                    serde_json::json!({}),
                );
            }

            prop_assert_eq!(logger.len(), num_activities);

            // Clear the logger
            logger.clear();

            prop_assert!(logger.is_empty());
            prop_assert_eq!(logger.len(), 0);
        }

        /// Property: Workflow lifecycle activities are logged correctly
        /// For a complete workflow lifecycle, all key activities should be logged
        #[test]
        fn prop_workflow_lifecycle_activities_are_logged(
            workflow_id in "workflow-[a-z0-9]{5}",
        ) {
            let mut logger = ActivityLogger::new(1000);

            // Log workflow lifecycle
            logger.log_workflow_started(&workflow_id);
            logger.log_step_started("step1", "Step 1");
            logger.log_step_completed("step1", "Step 1", 100);
            logger.log_step_started("step2", "Step 2");
            logger.log_step_completed("step2", "Step 2", 150);
            logger.log_workflow_completed(&workflow_id, 250);

            // Verify all activities were logged
            prop_assert_eq!(logger.len(), 6);

            let entries = logger.get_entries();

            // First should be workflow started
            prop_assert_eq!(entries[0].activity_type, ActivityType::WorkflowStarted);

            // Last should be workflow completed
            prop_assert_eq!(entries[5].activity_type, ActivityType::WorkflowCompleted);

            // Should have step activities in between
            let step_started = logger.get_entries_by_type(ActivityType::StepStarted);
            let step_completed = logger.get_entries_by_type(ActivityType::StepCompleted);

            prop_assert_eq!(step_started.len(), 2);
            prop_assert_eq!(step_completed.len(), 2);
        }

        /// Property: Error activities are logged with context
        /// For any logged error, the entry should contain error information
        #[test]
        fn prop_error_activities_contain_context(
            error_message in "error: .*",
        ) {
            let mut logger = ActivityLogger::new(1000);

            logger.log_error(Some("step1"), &error_message);

            let entries = logger.get_entries();
            prop_assert_eq!(entries.len(), 1);
            prop_assert_eq!(entries[0].activity_type, ActivityType::Error);
            prop_assert_eq!(&entries[0].step_id, &Some("step1".to_string()));
            prop_assert!(entries[0].message.contains(&error_message));
        }

        /// Property: Approval activities are logged correctly
        /// For approval workflow, all approval activities should be logged
        #[test]
        fn prop_approval_activities_are_logged(
            step_id in "step-[a-z0-9]{5}",
        ) {
            let mut logger = ActivityLogger::new(1000);

            logger.log_approval_requested(&step_id, "Please review");
            logger.log_approval_granted(&step_id);

            let entries = logger.get_entries();
            prop_assert_eq!(entries.len(), 2);

            prop_assert_eq!(entries[0].activity_type, ActivityType::ApprovalRequested);
            prop_assert_eq!(entries[1].activity_type, ActivityType::ApprovalGranted);

            // Both should reference the same step
            prop_assert_eq!(&entries[0].step_id, &Some(step_id.clone()));
            prop_assert_eq!(&entries[1].step_id, &Some(step_id));
        }

        /// Property: Activity log entries maintain order
        /// For any sequence of logged activities, entries should be in order
        #[test]
        fn prop_activity_log_maintains_order(
            num_activities in 1usize..100,
        ) {
            let mut logger = ActivityLogger::new(1000);

            // Log activities in sequence
            for i in 0..num_activities {
                logger.log(
                    ActivityType::StepStarted,
                    Some(format!("step-{}", i)),
                    format!("Activity {}", i),
                    serde_json::json!({}),
                );
            }

            let entries = logger.get_entries();

            // Verify order is maintained
            for (i, entry) in entries.iter().enumerate() {
                prop_assert_eq!(&entry.step_id, &Some(format!("step-{}", i)));
            }
        }
    }
}
