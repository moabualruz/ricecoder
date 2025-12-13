//! Unit tests for activity logging
//! **Feature: ricecoder-workflows, Activity Logging**
//! **Validates: Requirements 2.6**

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity_log::{ActivityLogger, ActivityType};

    #[test]
    fn test_activity_logger_records_activities() {
        let mut logger = ActivityLogger::new(1000);

        // Log an activity
        logger.log(
            ActivityType::StepStarted,
            Some("step1".to_string()),
            "Step started".to_string(),
            serde_json::json!({}),
        );

        // Verify the activity was recorded
        assert_eq!(logger.len(), 1);

        let entries = logger.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].activity_type, ActivityType::StepStarted);
        assert_eq!(&entries[0].step_id, &Some("step1".to_string()));
        assert_eq!(&entries[0].message, "Step started");
    }

    #[test]
    fn test_activity_log_entries_have_timestamps() {
        let mut logger = ActivityLogger::new(1000);

        logger.log(
            ActivityType::StepStarted,
            None,
            "Test activity".to_string(),
            serde_json::json!({}),
        );

        let entries = logger.get_entries();
        assert_eq!(entries.len(), 1);

        // Timestamp should be recent (within last minute)
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(entries[0].timestamp);
        assert!(diff.num_seconds() >= 0 && diff.num_seconds() < 60);
    }

    #[test]
    fn test_activity_log_respects_max_entries() {
        let mut logger = ActivityLogger::new(5);

        // Log more activities than the limit
        for i in 0..10 {
            logger.log(
                ActivityType::StepStarted,
                Some(format!("step-{}", i)),
                format!("Activity {}", i),
                serde_json::json!({}),
            );
        }

        // Should not exceed max_entries
        assert!(logger.len() <= 5);
    }
}