use ricecoder_workflows::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_approval_request() {
        let request = ApprovalRequest::new(
            "step1".to_string(),
            "Please approve this step".to_string(),
            5000,
        );

        assert_eq!(request.step_id, "step1");
        assert_eq!(request.message, "Please approve this step");
        assert_eq!(request.timeout_ms, 5000);
        assert!(!request.approved);
        assert!(request.is_pending());
    }

    #[test]
    fn test_approval_request_timeout() {
        let request = ApprovalRequest::new(
            "step1".to_string(),
            "Please approve this step".to_string(),
            1, // 1ms timeout
        );

        // Wait a bit to ensure timeout
        std::thread::sleep(std::time::Duration::from_millis(10));

        assert!(request.is_timed_out());
        assert!(!request.is_pending());
    }

    #[test]
    fn test_approve_request() {
        let mut request = ApprovalRequest::new(
            "step1".to_string(),
            "Please approve this step".to_string(),
            5000,
        );

        request.approve(Some("Looks good".to_string()));

        assert!(request.approved);
        assert_eq!(request.decision, Some(ApprovalDecision::Approved));
        assert_eq!(request.comments, Some("Looks good".to_string()));
        assert!(!request.is_pending());
    }

    #[test]
    fn test_reject_request() {
        let mut request = ApprovalRequest::new(
            "step1".to_string(),
            "Please approve this step".to_string(),
            5000,
        );

        request.reject(Some("Needs changes".to_string()));

        assert!(request.approved);
        assert_eq!(request.decision, Some(ApprovalDecision::Rejected));
        assert_eq!(request.comments, Some("Needs changes".to_string()));
        assert!(!request.is_pending());
    }

    #[test]
    fn test_approval_gate_request_approval() {
        let mut gate = ApprovalGate::new();

        let request_id = gate
            .request_approval("step1".to_string(), "Please approve".to_string(), 5000)
            .unwrap();

        assert!(!request_id.is_empty());
        assert_eq!(gate.get_pending_requests().len(), 1);
    }

    #[test]
    fn test_approval_gate_approve() {
        let mut gate = ApprovalGate::new();

        let request_id = gate
            .request_approval("step1".to_string(), "Please approve".to_string(), 5000)
            .unwrap();

        gate.approve(&request_id, Some("Approved".to_string()))
            .unwrap();

        assert!(gate.is_approved(&request_id).unwrap());
        assert!(!gate.is_rejected(&request_id).unwrap());
        assert!(!gate.is_pending(&request_id).unwrap());
    }

    #[test]
    fn test_approval_gate_reject() {
        let mut gate = ApprovalGate::new();

        let request_id = gate
            .request_approval("step1".to_string(), "Please approve".to_string(), 5000)
            .unwrap();

        gate.reject(&request_id, Some("Rejected".to_string()))
            .unwrap();

        assert!(!gate.is_approved(&request_id).unwrap());
        assert!(gate.is_rejected(&request_id).unwrap());
        assert!(!gate.is_pending(&request_id).unwrap());
    }

    #[test]
    fn test_approval_gate_get_step_requests() {
        let mut gate = ApprovalGate::new();

        let _req1 = gate
            .request_approval("step1".to_string(), "Please approve".to_string(), 5000)
            .unwrap();

        let _req2 = gate
            .request_approval(
                "step1".to_string(),
                "Please approve again".to_string(),
                5000,
            )
            .unwrap();

        let _req3 = gate
            .request_approval("step2".to_string(), "Please approve".to_string(), 5000)
            .unwrap();

        let step1_requests = gate.get_step_requests("step1");
        assert_eq!(step1_requests.len(), 2);

        let step2_requests = gate.get_step_requests("step2");
        assert_eq!(step2_requests.len(), 1);
    }

    #[test]
    fn test_approval_gate_cannot_approve_twice() {
        let mut gate = ApprovalGate::new();

        let request_id = gate
            .request_approval("step1".to_string(), "Please approve".to_string(), 5000)
            .unwrap();

        gate.approve(&request_id, None).unwrap();

        let result = gate.approve(&request_id, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_approval_gate_cannot_approve_after_timeout() {
        let mut gate = ApprovalGate::new();

        let request_id = gate
            .request_approval(
                "step1".to_string(),
                "Please approve".to_string(),
                1, // 1ms timeout
            )
            .unwrap();

        // Wait for timeout
        std::thread::sleep(std::time::Duration::from_millis(10));

        let result = gate.approve(&request_id, None);
        assert!(result.is_err());
    }
}