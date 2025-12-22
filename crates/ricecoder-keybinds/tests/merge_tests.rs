use ricecoder_keybinds::*;

use crate::models::Context;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_no_conflicts() {
        let defaults = vec![
            Keybind::new_default("action1", "Ctrl+A", "global", "Action 1"),
            Keybind::new_default("action2", "Ctrl+B", "global", "Action 2"),
        ];

        let user = vec![Keybind::new("action3", "Ctrl+C", "global", "Action 3")];

        let result = KeybindMerger::merge_with_contexts(&defaults, &user).unwrap();

        assert_eq!(result.merged.len(), 3);
        assert!(result.unresolved_conflicts.is_empty());
    }

    #[test]
    fn test_merge_user_overrides_default() {
        let defaults = vec![Keybind::new_default(
            "action1",
            "Ctrl+A",
            "global",
            "Default Action 1",
        )];

        let user = vec![Keybind::new("action1", "Ctrl+A", "global", "User Action 1")];

        let result = KeybindMerger::merge_with_contexts(&defaults, &user).unwrap();

        assert_eq!(result.merged.len(), 1);
        assert_eq!(result.merged[0].description, "User Action 1");
        assert_eq!(result.resolved_conflicts.len(), 1);
    }

    #[test]
    fn test_merge_context_specific() {
        let defaults = vec![Keybind::new_default_with_contexts(
            "action1",
            "Ctrl+A",
            "global",
            "Global Action",
            vec![Context::Global],
        )];

        let user = vec![Keybind::new_with_contexts(
            "action1",
            "Ctrl+A",
            "input",
            "Input Action",
            vec![Context::Input],
        )];

        let result = KeybindMerger::merge_with_contexts(&defaults, &user).unwrap();

        // Should have both - they apply to different contexts
        assert_eq!(result.merged.len(), 2);
        assert!(result.unresolved_conflicts.is_empty());
    }

    #[test]
    fn test_merge_context_conflict() {
        let defaults = vec![Keybind::new_default_with_contexts(
            "action1",
            "Ctrl+A",
            "global",
            "Default Action",
            vec![Context::Input],
        )];

        let user = vec![Keybind::new_with_contexts(
            "action1",
            "Ctrl+A",
            "input",
            "User Action",
            vec![Context::Input],
        )];

        let result = KeybindMerger::merge_with_contexts(&defaults, &user).unwrap();

        // User should override default in same context
        assert_eq!(result.merged.len(), 1);
        assert_eq!(result.merged[0].description, "User Action");
        assert_eq!(result.resolved_conflicts.len(), 1);
    }
}
