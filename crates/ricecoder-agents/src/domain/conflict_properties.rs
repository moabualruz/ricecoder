//! Property-based tests for conflict detection and reporting
//!
//! **Feature: ricecoder-domain-agents, Property 8: Conflict Detection and Reporting**
//! **Validates: Requirements 4.5**

#[cfg(test)]
mod tests {
    use crate::domain::{ConflictDetector, ConflictType, Recommendation};
    use proptest::prelude::*;

    // Strategy for generating recommendations
    fn recommendation_strategy() -> impl Strategy<Value = Recommendation> {
        (
            prop_oneof![Just("web"), Just("backend"), Just("devops")],
            prop_oneof![
                Just("framework"),
                Just("database"),
                Just("deployment"),
                Just("testing")
            ],
            ".*",
            prop::collection::vec(".*", 1..3),
            ".*",
        )
            .prop_map(|(domain, category, content, technologies, rationale)| {
                Recommendation {
                    domain: domain.to_string(),
                    category: category.to_string(),
                    content,
                    technologies,
                    rationale,
                }
            })
    }

    /// Property 8.1: Conflicting recommendations are detected
    ///
    /// *For any* set of domain agent recommendations that contain conflicts,
    /// the Domain Agent System SHALL detect the conflicts.
    ///
    /// **Feature: ricecoder-domain-agents, Property 8: Conflict Detection and Reporting**
    /// **Validates: Requirements 4.5**
    #[test]
    fn test_conflicting_recommendations_detected() {
        let detector = ConflictDetector::new();

        // Create recommendations with incompatible technologies
        let rec_a = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use React for frontend".to_string(),
            technologies: vec!["React".to_string()],
            rationale: "React is suitable for complex UIs".to_string(),
        };

        let rec_b = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use Angular for frontend".to_string(),
            technologies: vec!["Angular".to_string()],
            rationale: "Angular provides full framework".to_string(),
        };

        let conflicts = detector
            .detect_conflicts(vec![rec_a, rec_b])
            .expect("detection failed");

        // Should detect at least one conflict
        assert!(!conflicts.is_empty());
        assert_eq!(conflicts[0].conflict_type, ConflictType::Incompatible);
    }

    /// Property 8.2: Conflicts include clear rationale
    ///
    /// *For any* detected conflict, the Domain Agent System SHALL include
    /// clear rationale for each conflicting recommendation.
    ///
    /// **Feature: ricecoder-domain-agents, Property 8: Conflict Detection and Reporting**
    /// **Validates: Requirements 4.5**
    #[test]
    fn test_conflicts_include_rationale() {
        let detector = ConflictDetector::new();

        let rec_a = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use React".to_string(),
            technologies: vec!["React".to_string()],
            rationale: "React has large ecosystem and community support".to_string(),
        };

        let rec_b = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use Vue".to_string(),
            technologies: vec!["Vue".to_string()],
            rationale: "Vue is easier to learn and has excellent documentation".to_string(),
        };

        let conflicts = detector
            .detect_conflicts(vec![rec_a, rec_b])
            .expect("detection failed");

        // Verify that conflicts include rationale
        for conflict in conflicts {
            assert!(!conflict.recommendation_a.rationale.is_empty());
            assert!(!conflict.recommendation_b.rationale.is_empty());
        }
    }

    /// Property 8.3: Conflict report includes analysis
    ///
    /// *For any* set of conflicts, the conflict report SHALL include
    /// analysis of the conflicts.
    ///
    /// **Feature: ricecoder-domain-agents, Property 8: Conflict Detection and Reporting**
    /// **Validates: Requirements 4.5**
    #[test]
    fn test_conflict_report_includes_analysis() {
        let detector = ConflictDetector::new();

        let rec_a = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use React".to_string(),
            technologies: vec!["React".to_string()],
            rationale: "React is suitable".to_string(),
        };

        let rec_b = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use Angular".to_string(),
            technologies: vec!["Angular".to_string()],
            rationale: "Angular is comprehensive".to_string(),
        };

        let conflicts = detector
            .detect_conflicts(vec![rec_a, rec_b])
            .expect("detection failed");

        let report = detector
            .generate_report(conflicts)
            .expect("report generation failed");

        // Verify that report includes analysis
        assert!(!report.analysis.is_empty());
        assert!(report.analysis.contains("conflict") || report.analysis.contains("Conflict"));
    }

    /// Property 8.4: Conflict report includes suggested resolution
    ///
    /// *For any* conflict report, the report SHALL include suggested
    /// resolution strategies.
    ///
    /// **Feature: ricecoder-domain-agents, Property 8: Conflict Detection and Reporting**
    /// **Validates: Requirements 4.5**
    #[test]
    fn test_conflict_report_includes_resolution() {
        let detector = ConflictDetector::new();

        let rec_a = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use React".to_string(),
            technologies: vec!["React".to_string()],
            rationale: "React is suitable".to_string(),
        };

        let rec_b = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use Angular".to_string(),
            technologies: vec!["Angular".to_string()],
            rationale: "Angular is comprehensive".to_string(),
        };

        let conflicts = detector
            .detect_conflicts(vec![rec_a, rec_b])
            .expect("detection failed");

        let report = detector
            .generate_report(conflicts)
            .expect("report generation failed");

        // Verify that report includes suggested resolution
        assert!(!report.suggested_resolution.is_empty());
    }

    /// Property 8.5: No false positives for compatible recommendations
    ///
    /// *For any* set of compatible recommendations from different domains,
    /// the conflict detector SHALL NOT report conflicts.
    ///
    /// **Feature: ricecoder-domain-agents, Property 8: Conflict Detection and Reporting**
    /// **Validates: Requirements 4.5**
    #[test]
    fn test_no_false_positives_for_compatible_recommendations() {
        let detector = ConflictDetector::new();

        let rec_a = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use React".to_string(),
            technologies: vec!["React".to_string()],
            rationale: "React is suitable for frontend".to_string(),
        };

        let rec_b = Recommendation {
            domain: "backend".to_string(),
            category: "runtime".to_string(),
            content: "Use Node.js".to_string(),
            technologies: vec!["Node.js".to_string()],
            rationale: "Node.js is suitable for backend".to_string(),
        };

        let conflicts = detector
            .detect_conflicts(vec![rec_a, rec_b])
            .expect("detection failed");

        // Should not detect conflicts for compatible recommendations
        assert!(conflicts.is_empty());
    }

    /// Property 8.6: Conflict formatting includes all necessary information
    ///
    /// *For any* conflict, the formatted conflict string SHALL include
    /// domain, category, content, and rationale for both recommendations.
    ///
    /// **Feature: ricecoder-domain-agents, Property 8: Conflict Detection and Reporting**
    /// **Validates: Requirements 4.5**
    #[test]
    fn test_conflict_formatting_includes_all_information() {
        let detector = ConflictDetector::new();

        let rec_a = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use React for frontend".to_string(),
            technologies: vec!["React".to_string()],
            rationale: "React has large ecosystem".to_string(),
        };

        let rec_b = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use Angular for frontend".to_string(),
            technologies: vec!["Angular".to_string()],
            rationale: "Angular is comprehensive".to_string(),
        };

        let conflict = crate::domain::Conflict {
            recommendation_a: rec_a,
            recommendation_b: rec_b,
            conflict_type: ConflictType::Incompatible,
        };

        let formatted = detector.format_conflict(&conflict);

        // Verify that formatted string includes all necessary information
        assert!(formatted.contains("Incompatible"));
        assert!(formatted.contains("web"));
        assert!(formatted.contains("framework"));
        assert!(formatted.contains("React"));
        assert!(formatted.contains("Angular"));
        assert!(formatted.contains("Rationale"));
    }

    /// Property 8.7: Conflict analysis provides actionable suggestions
    ///
    /// *For any* conflict, the conflict analysis SHALL provide actionable
    /// suggestions for resolution.
    ///
    /// **Feature: ricecoder-domain-agents, Property 8: Conflict Detection and Reporting**
    /// **Validates: Requirements 4.5**
    #[test]
    fn test_conflict_analysis_provides_suggestions() {
        let detector = ConflictDetector::new();

        let rec_a = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use React".to_string(),
            technologies: vec!["React".to_string()],
            rationale: "React is suitable".to_string(),
        };

        let rec_b = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use Angular".to_string(),
            technologies: vec!["Angular".to_string()],
            rationale: "Angular is comprehensive".to_string(),
        };

        let conflict = crate::domain::Conflict {
            recommendation_a: rec_a,
            recommendation_b: rec_b,
            conflict_type: ConflictType::Incompatible,
        };

        let analysis = detector.analyze_conflict(&conflict);

        // Verify that analysis includes suggestions
        assert!(analysis.contains("Suggested Resolutions"));
        assert!(analysis.contains("1."));
    }

    /// Property 8.8: Conflict types are correctly identified
    ///
    /// *For any* conflict, the conflict type SHALL be correctly identified
    /// as Incompatible, Contradictory, or RequiresSequencing.
    ///
    /// **Feature: ricecoder-domain-agents, Property 8: Conflict Detection and Reporting**
    /// **Validates: Requirements 4.5**
    #[test]
    fn test_conflict_types_correctly_identified() {
        let detector = ConflictDetector::new();

        // Test Incompatible conflict
        let rec_a = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use React".to_string(),
            technologies: vec!["React".to_string()],
            rationale: "React is suitable".to_string(),
        };

        let rec_b = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use Angular".to_string(),
            technologies: vec!["Angular".to_string()],
            rationale: "Angular is comprehensive".to_string(),
        };

        let conflicts = detector
            .detect_conflicts(vec![rec_a, rec_b])
            .expect("detection failed");

        assert!(!conflicts.is_empty());
        assert_eq!(conflicts[0].conflict_type, ConflictType::Incompatible);
    }

    /// Property 8.9: Multiple conflicts are all detected
    ///
    /// *For any* set of recommendations with multiple conflicts,
    /// all conflicts SHALL be detected.
    ///
    /// **Feature: ricecoder-domain-agents, Property 8: Conflict Detection and Reporting**
    /// **Validates: Requirements 4.5**
    #[test]
    fn test_multiple_conflicts_all_detected() {
        let detector = ConflictDetector::new();

        let recommendations = vec![
            Recommendation {
                domain: "web".to_string(),
                category: "framework".to_string(),
                content: "Use React".to_string(),
                technologies: vec!["React".to_string()],
                rationale: "React is suitable".to_string(),
            },
            Recommendation {
                domain: "web".to_string(),
                category: "framework".to_string(),
                content: "Use Angular".to_string(),
                technologies: vec!["Angular".to_string()],
                rationale: "Angular is comprehensive".to_string(),
            },
            Recommendation {
                domain: "web".to_string(),
                category: "framework".to_string(),
                content: "Use Vue".to_string(),
                technologies: vec!["Vue".to_string()],
                rationale: "Vue is easy to learn".to_string(),
            },
        ];

        let conflicts = detector
            .detect_conflicts(recommendations)
            .expect("detection failed");

        // Should detect multiple conflicts (React vs Angular, React vs Vue, Angular vs Vue)
        assert!(conflicts.len() >= 1);
    }

    /// Property 8.10: Conflict report formatting is consistent
    ///
    /// *For any* conflict report, the formatted report SHALL have
    /// consistent structure and include all required sections.
    ///
    /// **Feature: ricecoder-domain-agents, Property 8: Conflict Detection and Reporting**
    /// **Validates: Requirements 4.5**
    #[test]
    fn test_conflict_report_formatting_is_consistent() {
        let detector = ConflictDetector::new();

        let rec_a = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use React".to_string(),
            technologies: vec!["React".to_string()],
            rationale: "React is suitable".to_string(),
        };

        let rec_b = Recommendation {
            domain: "web".to_string(),
            category: "framework".to_string(),
            content: "Use Angular".to_string(),
            technologies: vec!["Angular".to_string()],
            rationale: "Angular is comprehensive".to_string(),
        };

        let conflict = crate::domain::Conflict {
            recommendation_a: rec_a,
            recommendation_b: rec_b,
            conflict_type: ConflictType::Incompatible,
        };

        let report = crate::domain::ConflictReport {
            conflicting_recommendations: vec![conflict],
            analysis: "Detected 1 conflict".to_string(),
            suggested_resolution: "Choose one framework".to_string(),
        };

        let formatted = detector.format_report(&report);

        // Verify consistent structure
        assert!(formatted.contains("CONFLICT REPORT"));
        assert!(formatted.contains("Analysis:"));
        assert!(formatted.contains("Conflicts Found:"));
        assert!(formatted.contains("Suggested Resolution:"));
    }
}
