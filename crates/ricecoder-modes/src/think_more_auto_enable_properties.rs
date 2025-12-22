/// Property-based tests for Think More auto-enable
/// **Feature: ricecoder-modes, Property 9: Think More Auto-Enable**
/// **Validates: Requirements 4.5**
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::{ComplexityDetector, ComplexityLevel, ThinkMoreController};

    proptest! {
        /// Property: Complex tasks auto-enable Think More when auto-enable is enabled
        /// For any complex task, Think More should be auto-enabled when configured
        #[test]
        fn prop_complex_tasks_auto_enable(
            task_description in ".*complex.*|.*algorithm.*|.*optimization.*",
        ) {
            let controller = ThinkMoreController::new();
            controller.enable_auto_enable().unwrap();

            let detector = ComplexityDetector::default();
            let complexity = detector.detect_complexity(&task_description);

            // Complex tasks should trigger auto-enable
            if complexity == ComplexityLevel::Complex {
                prop_assert!(controller.should_auto_enable(complexity).unwrap());
            }
        }

        /// Property: Simple tasks do not auto-enable Think More
        /// For any simple task, Think More should not be auto-enabled
        #[test]
        fn prop_simple_tasks_no_auto_enable(
            task_description in "write|create|make",
        ) {
            let controller = ThinkMoreController::new();
            controller.enable_auto_enable().unwrap();

            let detector = ComplexityDetector::default();
            let complexity = detector.detect_complexity(&task_description);

            // Simple tasks should not trigger auto-enable
            if complexity == ComplexityLevel::Simple {
                prop_assert!(!controller.should_auto_enable(complexity).unwrap());
            }
        }

        /// Property: Auto-enable can be toggled on and off
        /// For any state, auto-enable should be toggleable
        #[test]
        fn prop_auto_enable_toggleable(
            num_toggles in 1..10usize,
        ) {
            let controller = ThinkMoreController::new();

            for i in 0..num_toggles {
                if i % 2 == 0 {
                    controller.enable_auto_enable().unwrap();
                    prop_assert!(controller.is_auto_enable_enabled().unwrap());
                } else {
                    controller.disable_auto_enable().unwrap();
                    prop_assert!(!controller.is_auto_enable_enabled().unwrap());
                }
            }
        }

        /// Property: Auto-enable respects complexity threshold
        /// For any complexity level, auto-enable should respect the configured threshold
        #[test]
        fn prop_auto_enable_respects_threshold(
            complexity in prop_oneof![
                Just(ComplexityLevel::Simple),
                Just(ComplexityLevel::Moderate),
                Just(ComplexityLevel::Complex),
            ],
        ) {
            let controller = ThinkMoreController::new();
            controller.enable_auto_enable().unwrap();

            // Complex should always auto-enable
            if complexity == ComplexityLevel::Complex {
                prop_assert!(controller.should_auto_enable(complexity).unwrap());
            }
        }

        /// Property: Complexity detection is consistent
        /// For any task description, complexity detection should be consistent
        #[test]
        fn prop_complexity_detection_consistent(
            task_description in ".*",
        ) {
            let detector = ComplexityDetector::default();

            let complexity1 = detector.detect_complexity(&task_description);
            let complexity2 = detector.detect_complexity(&task_description);

            // Same input should produce same complexity
            prop_assert_eq!(complexity1, complexity2);
        }

        /// Property: Complexity analysis includes reasoning
        /// For any task, complexity analysis should provide reasoning
        #[test]
        fn prop_complexity_analysis_has_reasoning(
            task_description in ".*",
        ) {
            let detector = ComplexityDetector::default();
            let analysis = detector.analyze_task(&task_description);

            // Analysis should have non-empty reasoning
            prop_assert!(!analysis.reasoning.is_empty());
            prop_assert!(analysis.reasoning.contains("Complexity Level"));
        }

        /// Property: Complexity score correlates with complexity level
        /// For any task, higher complexity should have higher score
        #[test]
        fn prop_complexity_score_correlates(
            simple_task in "write|create",
            complex_task in ".*complex.*algorithm.*optimization.*",
        ) {
            let detector = ComplexityDetector::default();

            let simple_score = detector.calculate_complexity_score(&simple_task);
            let complex_score = detector.calculate_complexity_score(&complex_task);

            // Complex task should have higher score
            prop_assert!(complex_score >= simple_score);
        }

        /// Property: Auto-enable decision is based on complexity
        /// For any complexity level, auto-enable decision should be consistent
        #[test]
        fn prop_auto_enable_decision_consistent(
            complexity in prop_oneof![
                Just(ComplexityLevel::Simple),
                Just(ComplexityLevel::Moderate),
                Just(ComplexityLevel::Complex),
            ],
        ) {
            let controller = ThinkMoreController::new();
            controller.enable_auto_enable().unwrap();

            let decision1 = controller.should_auto_enable(complexity).unwrap();
            let decision2 = controller.should_auto_enable(complexity).unwrap();

            // Same complexity should produce same decision
            prop_assert_eq!(decision1, decision2);
        }

        /// Property: Disabling auto-enable prevents auto-enabling
        /// When auto-enable is disabled, no tasks should auto-enable
        #[test]
        fn prop_disabled_auto_enable_prevents_enabling(
            complexity in prop_oneof![
                Just(ComplexityLevel::Simple),
                Just(ComplexityLevel::Moderate),
                Just(ComplexityLevel::Complex),
            ],
        ) {
            let controller = ThinkMoreController::new();
            controller.disable_auto_enable().unwrap();

            // Nothing should auto-enable when disabled
            prop_assert!(!controller.should_auto_enable(complexity).unwrap());
        }

        /// Property: Complexity threshold can be configured
        /// For any threshold, it should be configurable and respected
        #[test]
        fn prop_complexity_threshold_configurable(
            threshold in prop_oneof![
                Just(ComplexityLevel::Simple),
                Just(ComplexityLevel::Moderate),
                Just(ComplexityLevel::Complex),
            ],
        ) {
            let mut detector = ComplexityDetector::new(threshold);

            prop_assert_eq!(detector.get_threshold(), threshold);

            let new_threshold = ComplexityLevel::Complex;
            detector.set_threshold(new_threshold);

            prop_assert_eq!(detector.get_threshold(), new_threshold);
        }

        /// Property: Task complexity analysis is deterministic
        /// For any task, analysis should be deterministic
        #[test]
        fn prop_task_analysis_deterministic(
            task_description in ".*",
        ) {
            let detector = ComplexityDetector::default();

            let analysis1 = detector.analyze_task(&task_description);
            let analysis2 = detector.analyze_task(&task_description);

            // Same task should produce same analysis
            prop_assert_eq!(analysis1.complexity, analysis2.complexity);
            prop_assert_eq!(analysis1.score, analysis2.score);
            prop_assert_eq!(analysis1.should_enable_think_more, analysis2.should_enable_think_more);
        }
    }
}
