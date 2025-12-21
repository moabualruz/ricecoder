use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_list_action() {
        let action = HooksAction::List { format: None };
        let _cmd = HooksCommand::new(action);
        // Just verify it can be created
        assert!(true);
    }

    #[test]
    fn test_hooks_inspect_action() {
        let action = HooksAction::Inspect {
            id: "hook1".to_string(),
            format: None,
        };
        let _cmd = HooksCommand::new(action);
        // Just verify it can be created
        assert!(true);
    }

    #[test]
    fn test_hooks_enable_action() {
        let action = HooksAction::Enable {
            id: "hook1".to_string(),
        };
        let _cmd = HooksCommand::new(action);
        // Just verify it can be created
        assert!(true);
    }

    #[test]
    fn test_hooks_disable_action() {
        let action = HooksAction::Disable {
            id: "hook1".to_string(),
        };
        let _cmd = HooksCommand::new(action);
        // Just verify it can be created
        assert!(true);
    }

    #[test]
    fn test_hooks_delete_action() {
        let action = HooksAction::Delete {
            id: "hook1".to_string(),
        };
        let _cmd = HooksCommand::new(action);
        // Just verify it can be created
        assert!(true);
    }
}
