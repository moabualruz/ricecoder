use ricecoder_workflows::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_activity_logger() {
        let logger = ActivityLogger::new(100);
        assert!(logger.is_empty());
        assert_eq!(logger.len(), 0);
    }

    #[test]
    fn test_log_activity() {
        let mut logger = ActivityLogger::new(100);

        logger.log(
            ActivityType::WorkflowStarted,
            None,
            "Workflow started".to_string(),
            serde_json::json!({}),
        );

        assert_eq!(logger.len(), 1);
        assert!(!logger.is_empty());
    }

    #[test]
    fn test_log_workflow_started() {
        let mut logger = ActivityLogger::new(100);
        logger.log_workflow_started("test-workflow");

        assert_eq!(logger.len(), 1);
        let entries = logger.get_entries();
        assert_eq!(entries[0].activity_type, ActivityType::WorkflowStarted);
    }

    #[test]
    fn test_log_step_started() {
        let mut logger = ActivityLogger::new(100);
        logger.log_step_started("step1", "Step 1");

        assert_eq!(logger.len(), 1);
        let entries = logger.get_entries();
        assert_eq!(entries[0].activity_type, ActivityType::StepStarted);
        assert_eq!(entries[0].step_id, Some("step1".to_string()));
    }

    #[test]
    fn test_get_entries_by_type() {
        let mut logger = ActivityLogger::new(100);
        logger.log_workflow_started("test-workflow");
        logger.log_step_started("step1", "Step 1");
        logger.log_workflow_completed("test-workflow", 100);

        let workflow_entries = logger.get_entries_by_type(ActivityType::WorkflowStarted);
        assert_eq!(workflow_entries.len(), 1);

        let step_entries = logger.get_entries_by_type(ActivityType::StepStarted);
        assert_eq!(step_entries.len(), 1);
    }

    #[test]
    fn test_get_entries_for_step() {
        let mut logger = ActivityLogger::new(100);
        logger.log_step_started("step1", "Step 1");
        logger.log_step_completed("step1", "Step 1", 100);
        logger.log_step_started("step2", "Step 2");

        let step1_entries = logger.get_entries_for_step("step1");
        assert_eq!(step1_entries.len(), 2);

        let step2_entries = logger.get_entries_for_step("step2");
        assert_eq!(step2_entries.len(), 1);
    }

    #[test]
    fn test_max_entries_limit() {
        let mut logger = ActivityLogger::new(3);

        logger.log_workflow_started("workflow1");
        logger.log_workflow_started("workflow2");
        logger.log_workflow_started("workflow3");
        logger.log_workflow_started("workflow4");

        // Should only keep the last 3 entries
        assert_eq!(logger.len(), 3);
    }

    #[test]
    fn test_clear_entries() {
        let mut logger = ActivityLogger::new(100);
        logger.log_workflow_started("test-workflow");
        logger.log_step_started("step1", "Step 1");

        assert_eq!(logger.len(), 2);

        logger.clear();
        assert!(logger.is_empty());
        assert_eq!(logger.len(), 0);
    }

    #[test]
    fn test_log_error() {
        let mut logger = ActivityLogger::new(100);
        logger.log_error(Some("step1"), "Something went wrong");

        assert_eq!(logger.len(), 1);
        let entries = logger.get_entries();
        assert_eq!(entries[0].activity_type, ActivityType::Error);
        assert_eq!(entries[0].step_id, Some("step1".to_string()));
    }

    #[test]
    fn test_log_approval_workflow() {
        let mut logger = ActivityLogger::new(100);
        logger.log_approval_requested("step1", "Please review");
        logger.log_approval_granted("step1");

        assert_eq!(logger.len(), 2);

        let approval_entries = logger.get_entries_by_type(ActivityType::ApprovalRequested);
        assert_eq!(approval_entries.len(), 1);

        let granted_entries = logger.get_entries_by_type(ActivityType::ApprovalGranted);
        assert_eq!(granted_entries.len(), 1);
    }
}