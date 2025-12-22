/// Property-based tests for Think More activation
/// **Feature: ricecoder-modes, Property 7: Think More Activation**
/// **Validates: Requirements 4.1, 4.2**
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::{ThinkMoreController, ThinkingDepth};

    proptest! {
        /// Property: When Think More is enabled, thinking can be started and content is visible
        /// For any thinking depth, enabling Think More and starting thinking should result in
        /// active thinking state with visible content
        #[test]
        fn prop_think_more_activation_enables_thinking(
            depth in prop_oneof![
                Just(ThinkingDepth::Light),
                Just(ThinkingDepth::Medium),
                Just(ThinkingDepth::Deep),
            ],
            content in ".*",
        ) {
            let controller = ThinkMoreController::new();

            // Enable Think More
            prop_assert!(controller.enable().is_ok());
            prop_assert!(controller.is_enabled().unwrap());

            // Start thinking
            prop_assert!(controller.start_thinking(depth).is_ok());
            prop_assert!(controller.is_thinking().unwrap());

            // Add content
            prop_assert!(controller.add_thinking_content(&content).is_ok());

            // Verify content is visible
            let thinking_content = controller.get_thinking_content().unwrap();
            prop_assert!(!thinking_content.is_empty() || content.is_empty());

            // Stop thinking
            let stopped_content = controller.stop_thinking().unwrap();
            prop_assert!(!controller.is_thinking().unwrap());
            prop_assert_eq!(stopped_content, thinking_content);
        }

        /// Property: Thinking depth is preserved during thinking session
        /// For any thinking depth, the depth should remain constant throughout the session
        #[test]
        fn prop_thinking_depth_preserved(
            depth in prop_oneof![
                Just(ThinkingDepth::Light),
                Just(ThinkingDepth::Medium),
                Just(ThinkingDepth::Deep),
            ],
        ) {
            let controller = ThinkMoreController::new();
            controller.set_depth(depth).unwrap();

            controller.start_thinking(depth).unwrap();

            // Depth should be preserved
            let metadata = controller.get_thinking_metadata().unwrap();
            prop_assert_eq!(metadata.depth, depth);

            controller.stop_thinking().unwrap();
        }

        /// Property: Multiple content additions accumulate
        /// For any sequence of content strings, all content should be accumulated
        #[test]
        fn prop_thinking_content_accumulates(
            contents in prop::collection::vec(".*", 1..10),
        ) {
            let controller = ThinkMoreController::new();
            controller.start_thinking(ThinkingDepth::Medium).unwrap();

            for content in &contents {
                prop_assert!(controller.add_thinking_content(content).is_ok());
            }

            let accumulated = controller.get_thinking_content().unwrap();

            // All non-empty contents should be in the accumulated string
            for content in &contents {
                if !content.is_empty() {
                    prop_assert!(accumulated.contains(content));
                }
            }

            controller.stop_thinking().unwrap();
        }

        /// Property: Thinking state is properly tracked
        /// For any sequence of start/stop operations, the state should be consistent
        #[test]
        fn prop_thinking_state_consistency(
            num_cycles in 1..5usize,
        ) {
            let controller = ThinkMoreController::new();

            for _ in 0..num_cycles {
                // Initially not thinking
                prop_assert!(!controller.is_thinking().unwrap());

                // Start thinking
                prop_assert!(controller.start_thinking(ThinkingDepth::Medium).is_ok());
                prop_assert!(controller.is_thinking().unwrap());

                // Stop thinking
                prop_assert!(controller.stop_thinking().is_ok());
                prop_assert!(!controller.is_thinking().unwrap());
            }
        }

        /// Property: Metadata reflects current thinking state
        /// For any thinking session, metadata should accurately reflect the state
        #[test]
        fn prop_metadata_reflects_state(
            depth in prop_oneof![
                Just(ThinkingDepth::Light),
                Just(ThinkingDepth::Medium),
                Just(ThinkingDepth::Deep),
            ],
        ) {
            let controller = ThinkMoreController::new();
            controller.enable().unwrap();
            controller.start_thinking(depth).unwrap();

            let metadata = controller.get_thinking_metadata().unwrap();

            prop_assert!(metadata.enabled);
            prop_assert!(metadata.active);
            prop_assert_eq!(metadata.depth, depth);
            prop_assert!(metadata.elapsed_time.is_some());

            controller.stop_thinking().unwrap();
        }

        /// Property: Thinking content is empty after stop
        /// After stopping thinking, the content should be retrievable but new additions should not work
        #[test]
        fn prop_thinking_content_after_stop(
            content in ".*",
        ) {
            let controller = ThinkMoreController::new();
            controller.start_thinking(ThinkingDepth::Medium).unwrap();
            controller.add_thinking_content(&content).unwrap();

            let stopped_content = controller.stop_thinking().unwrap();

            // Content should be preserved
            if !content.is_empty() {
                prop_assert!(stopped_content.contains(&content));
            }

            // New additions should not be added (thinking is stopped)
            controller.add_thinking_content("new content").unwrap();
            let final_content = controller.get_thinking_content().unwrap();

            // Final content should not include the new content since thinking was stopped
            prop_assert_eq!(final_content, stopped_content);
        }

        /// Property: Canceling thinking stops the session
        /// For any active thinking session, canceling should stop it
        #[test]
        fn prop_cancel_thinking_stops_session(
            content in ".*",
        ) {
            let controller = ThinkMoreController::new();
            controller.start_thinking(ThinkingDepth::Medium).unwrap();
            controller.add_thinking_content(&content).unwrap();

            prop_assert!(controller.is_thinking().unwrap());

            controller.cancel_thinking().unwrap();

            prop_assert!(!controller.is_thinking().unwrap());
        }
    }
}
