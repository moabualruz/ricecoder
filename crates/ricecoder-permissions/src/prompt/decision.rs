//! Permission prompt decision handling

/// User decision in response to a permission prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserDecision {
    /// User approved the action
    Approved,
    /// User denied the action
    Denied,
    /// User cancelled the action
    Cancelled,
    /// Prompt timed out
    TimedOut,
}

/// Result of a permission prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptResult {
    /// Action was approved
    Approved,
    /// Action was denied
    Denied,
}

impl UserDecision {
    /// Convert user decision to prompt result
    pub fn to_prompt_result(self) -> PromptResult {
        match self {
            UserDecision::Approved => PromptResult::Approved,
            UserDecision::Denied | UserDecision::Cancelled | UserDecision::TimedOut => {
                PromptResult::Denied
            }
        }
    }

    /// Execute on approval - returns true if approved
    pub fn execute_on_approval(self) -> bool {
        self == UserDecision::Approved
    }

    /// Block on denial - returns true if denied
    pub fn block_on_denial(self) -> bool {
        self != UserDecision::Approved
    }

    /// Handle cancellation - treats cancellation as denial
    pub fn handle_cancellation(self) -> PromptResult {
        self.to_prompt_result()
    }
}

impl PromptResult {
    /// Check if the result is approved
    pub fn is_approved(self) -> bool {
        self == PromptResult::Approved
    }

    /// Check if the result is denied
    pub fn is_denied(self) -> bool {
        self == PromptResult::Denied
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_decision_equality() {
        assert_eq!(UserDecision::Approved, UserDecision::Approved);
        assert_ne!(UserDecision::Approved, UserDecision::Denied);
    }

    #[test]
    fn test_prompt_result_equality() {
        assert_eq!(PromptResult::Approved, PromptResult::Approved);
        assert_ne!(PromptResult::Approved, PromptResult::Denied);
    }

    #[test]
    fn test_user_decision_to_prompt_result() {
        assert_eq!(
            UserDecision::Approved.to_prompt_result(),
            PromptResult::Approved
        );
        assert_eq!(
            UserDecision::Denied.to_prompt_result(),
            PromptResult::Denied
        );
        assert_eq!(
            UserDecision::Cancelled.to_prompt_result(),
            PromptResult::Denied
        );
        assert_eq!(
            UserDecision::TimedOut.to_prompt_result(),
            PromptResult::Denied
        );
    }

    #[test]
    fn test_execute_on_approval() {
        assert!(UserDecision::Approved.execute_on_approval());
        assert!(!UserDecision::Denied.execute_on_approval());
        assert!(!UserDecision::Cancelled.execute_on_approval());
        assert!(!UserDecision::TimedOut.execute_on_approval());
    }

    #[test]
    fn test_block_on_denial() {
        assert!(!UserDecision::Approved.block_on_denial());
        assert!(UserDecision::Denied.block_on_denial());
        assert!(UserDecision::Cancelled.block_on_denial());
        assert!(UserDecision::TimedOut.block_on_denial());
    }

    #[test]
    fn test_handle_cancellation() {
        assert_eq!(
            UserDecision::Cancelled.handle_cancellation(),
            PromptResult::Denied
        );
        assert_eq!(
            UserDecision::Approved.handle_cancellation(),
            PromptResult::Approved
        );
    }

    #[test]
    fn test_prompt_result_is_approved() {
        assert!(PromptResult::Approved.is_approved());
        assert!(!PromptResult::Denied.is_approved());
    }

    #[test]
    fn test_prompt_result_is_denied() {
        assert!(!PromptResult::Approved.is_denied());
        assert!(PromptResult::Denied.is_denied());
    }
}
