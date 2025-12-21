//! Property-based tests for domain agent coordination
//!
//! **Feature: ricecoder-domain-agents, Property 6: Full-Stack Coordination**
//! **Validates: Requirements 4.3**

#[cfg(test)]
mod tests {
    use crate::domain::coordinator::DomainCoordinator;
    use crate::domain::models::Recommendation;

    fn create_recommendation(
        domain: &str,
        category: &str,
        technologies: Vec<&str>,
    ) -> Recommendation {
        Recommendation {
            domain: domain.to_string(),
            category: category.to_string(),
            content: format!("Recommendation for {}", domain),
            technologies: technologies.iter().map(|t| t.to_string()).collect(),
            rationale: format!("Rationale for {}", domain),
        }
    }

    /// Property 6: Full-Stack Coordination
    /// For any full-stack planning request, the Domain Agent System SHALL coordinate
    /// recommendations across frontend (web), backend, and infrastructure (DevOps) domains
    /// such that recommendations are consistent and complementary.
    ///
    /// This property tests that:
    /// 1. Recommendations from all three domains are properly grouped
    /// 2. Full-stack flag is set correctly when all domains are present
    /// 3. Recommendations are consistent across domains
    /// 4. Technology stacks are complementary
    #[test]
    fn property_full_stack_coordination_all_domains_present() {
        let coordinator = DomainCoordinator::new();

        // Create recommendations from all three domains
        let recommendations = vec![
            create_recommendation("web", "framework", vec!["React", "TypeScript"]),
            create_recommendation("web", "styling", vec!["Tailwind CSS"]),
            create_recommendation("backend", "api", vec!["REST", "Node.js"]),
            create_recommendation("backend", "database", vec!["PostgreSQL"]),
            create_recommendation("devops", "containerization", vec!["Docker"]),
            create_recommendation("devops", "orchestration", vec!["Kubernetes"]),
        ];

        // Coordinate multiple times to ensure consistency
        for _ in 0..5 {
            let coordination = coordinator
                .coordinate_full_stack(recommendations.clone())
                .unwrap();

            // Property: All three domains should be present
            assert!(
                !coordination.web_recommendations.is_empty(),
                "Web recommendations should be present"
            );
            assert!(
                !coordination.backend_recommendations.is_empty(),
                "Backend recommendations should be present"
            );
            assert!(
                !coordination.devops_recommendations.is_empty(),
                "DevOps recommendations should be present"
            );

            // Property: Full-stack flag should be true
            assert!(coordination.is_full_stack, "Should be marked as full-stack");

            // Property: Total recommendations should match input
            assert_eq!(
                coordination.total_recommendations, 6,
                "Should have all 6 recommendations"
            );

            // Property: Each domain should have correct number of recommendations
            assert_eq!(
                coordination.web_recommendations.len(),
                2,
                "Web should have 2 recommendations"
            );
            assert_eq!(
                coordination.backend_recommendations.len(),
                2,
                "Backend should have 2 recommendations"
            );
            assert_eq!(
                coordination.devops_recommendations.len(),
                2,
                "DevOps should have 2 recommendations"
            );
        }
    }

    /// Property 6: Full-Stack Coordination (Partial Stack)
    /// For any partial-stack planning request, the system SHALL correctly identify
    /// that not all domains are represented.
    #[test]
    fn property_full_stack_coordination_partial_stack() {
        let coordinator = DomainCoordinator::new();

        // Create recommendations from only two domains
        let recommendations = vec![
            create_recommendation("web", "framework", vec!["React"]),
            create_recommendation("backend", "api", vec!["REST"]),
        ];

        for _ in 0..5 {
            let coordination = coordinator
                .coordinate_full_stack(recommendations.clone())
                .unwrap();

            // Property: Full-stack flag should be false
            assert!(
                !coordination.is_full_stack,
                "Should not be marked as full-stack"
            );

            // Property: Present domains should have recommendations
            assert!(
                !coordination.web_recommendations.is_empty(),
                "Web recommendations should be present"
            );
            assert!(
                !coordination.backend_recommendations.is_empty(),
                "Backend recommendations should be present"
            );

            // Property: Missing domain should have no recommendations
            assert!(
                coordination.devops_recommendations.is_empty(),
                "DevOps recommendations should be empty"
            );

            // Property: Total should be 2
            assert_eq!(
                coordination.total_recommendations, 2,
                "Should have 2 recommendations"
            );
        }
    }

    /// Property 6: Full-Stack Coordination (Consistency)
    /// For any set of recommendations, coordinating them multiple times SHALL produce
    /// identical results.
    #[test]
    fn property_full_stack_coordination_deterministic() {
        let coordinator = DomainCoordinator::new();

        let recommendations = vec![
            create_recommendation("web", "framework", vec!["React", "Vue"]),
            create_recommendation("backend", "api", vec!["REST", "GraphQL"]),
            create_recommendation("devops", "containerization", vec!["Docker"]),
        ];

        // Coordinate multiple times
        let mut results = Vec::new();
        for _ in 0..5 {
            let coordination = coordinator
                .coordinate_full_stack(recommendations.clone())
                .unwrap();
            results.push(coordination);
        }

        // Property: All coordinations should be identical
        let first = &results[0];
        for result in &results[1..] {
            assert_eq!(
                result.is_full_stack, first.is_full_stack,
                "Full-stack flag should be consistent"
            );
            assert_eq!(
                result.total_recommendations, first.total_recommendations,
                "Total recommendations should be consistent"
            );
            assert_eq!(
                result.web_recommendations.len(),
                first.web_recommendations.len(),
                "Web recommendations count should be consistent"
            );
            assert_eq!(
                result.backend_recommendations.len(),
                first.backend_recommendations.len(),
                "Backend recommendations count should be consistent"
            );
            assert_eq!(
                result.devops_recommendations.len(),
                first.devops_recommendations.len(),
                "DevOps recommendations count should be consistent"
            );
        }
    }

    /// Property 6: Full-Stack Coordination (Technology Stack Consistency)
    /// For any full-stack coordination, the technology stack across domains
    /// SHALL be consistent and complementary.
    #[test]
    fn property_full_stack_coordination_tech_stack_consistency() {
        let coordinator = DomainCoordinator::new();

        let recommendations = vec![
            create_recommendation("web", "framework", vec!["React", "TypeScript"]),
            create_recommendation("backend", "runtime", vec!["Node.js", "TypeScript"]),
            create_recommendation("devops", "containerization", vec!["Docker"]),
        ];

        for _ in 0..5 {
            let coordination = coordinator
                .coordinate_full_stack(recommendations.clone())
                .unwrap();

            // Property: All domains should have technologies
            assert!(
                !coordination.web_recommendations.is_empty(),
                "Web should have recommendations"
            );
            assert!(
                !coordination.backend_recommendations.is_empty(),
                "Backend should have recommendations"
            );
            assert!(
                !coordination.devops_recommendations.is_empty(),
                "DevOps should have recommendations"
            );

            // Property: Each recommendation should have technologies
            for rec in &coordination.web_recommendations {
                assert!(
                    !rec.technologies.is_empty(),
                    "Web recommendations should have technologies"
                );
            }
            for rec in &coordination.backend_recommendations {
                assert!(
                    !rec.technologies.is_empty(),
                    "Backend recommendations should have technologies"
                );
            }
            for rec in &coordination.devops_recommendations {
                assert!(
                    !rec.technologies.is_empty(),
                    "DevOps recommendations should have technologies"
                );
            }
        }
    }

    /// Property 6: Full-Stack Coordination (Empty Input)
    /// For any empty input, coordination SHALL produce empty output.
    #[test]
    fn property_full_stack_coordination_empty_input() {
        let coordinator = DomainCoordinator::new();

        for _ in 0..5 {
            let coordination = coordinator.coordinate_full_stack(vec![]).unwrap();

            // Property: Empty input should produce empty coordination
            assert!(
                !coordination.is_full_stack,
                "Empty input should not be full-stack"
            );
            assert_eq!(
                coordination.total_recommendations, 0,
                "Should have no recommendations"
            );
            assert!(
                coordination.web_recommendations.is_empty(),
                "Web should be empty"
            );
            assert!(
                coordination.backend_recommendations.is_empty(),
                "Backend should be empty"
            );
            assert!(
                coordination.devops_recommendations.is_empty(),
                "DevOps should be empty"
            );
        }
    }

    /// Property 6: Full-Stack Coordination (Single Domain)
    /// For any single-domain input, coordination SHALL correctly identify it as partial.
    #[test]
    fn property_full_stack_coordination_single_domain() {
        let coordinator = DomainCoordinator::new();

        let domains = vec!["web", "backend", "devops"];

        for domain in domains {
            let recommendations = vec![create_recommendation(domain, "test", vec!["Tech1"])];

            for _ in 0..3 {
                let coordination = coordinator
                    .coordinate_full_stack(recommendations.clone())
                    .unwrap();

                // Property: Single domain should not be full-stack
                assert!(
                    !coordination.is_full_stack,
                    "Single domain should not be full-stack"
                );

                // Property: Only the specified domain should have recommendations
                match domain {
                    "web" => {
                        assert_eq!(coordination.web_recommendations.len(), 1);
                        assert_eq!(coordination.backend_recommendations.len(), 0);
                        assert_eq!(coordination.devops_recommendations.len(), 0);
                    }
                    "backend" => {
                        assert_eq!(coordination.web_recommendations.len(), 0);
                        assert_eq!(coordination.backend_recommendations.len(), 1);
                        assert_eq!(coordination.devops_recommendations.len(), 0);
                    }
                    "devops" => {
                        assert_eq!(coordination.web_recommendations.len(), 0);
                        assert_eq!(coordination.backend_recommendations.len(), 0);
                        assert_eq!(coordination.devops_recommendations.len(), 1);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Property 6: Full-Stack Coordination (Multiple Recommendations Per Domain)
    /// For any full-stack coordination with multiple recommendations per domain,
    /// all recommendations SHALL be preserved.
    #[test]
    fn property_full_stack_coordination_multiple_per_domain() {
        let coordinator = DomainCoordinator::new();

        let recommendations = vec![
            create_recommendation("web", "framework", vec!["React"]),
            create_recommendation("web", "styling", vec!["Tailwind"]),
            create_recommendation("web", "testing", vec!["Jest"]),
            create_recommendation("backend", "api", vec!["REST"]),
            create_recommendation("backend", "database", vec!["PostgreSQL"]),
            create_recommendation("backend", "caching", vec!["Redis"]),
            create_recommendation("devops", "containerization", vec!["Docker"]),
            create_recommendation("devops", "orchestration", vec!["Kubernetes"]),
            create_recommendation("devops", "monitoring", vec!["Prometheus"]),
        ];

        for _ in 0..3 {
            let coordination = coordinator
                .coordinate_full_stack(recommendations.clone())
                .unwrap();

            // Property: All recommendations should be preserved
            assert_eq!(
                coordination.total_recommendations, 9,
                "All 9 recommendations should be preserved"
            );
            assert_eq!(
                coordination.web_recommendations.len(),
                3,
                "Web should have 3 recommendations"
            );
            assert_eq!(
                coordination.backend_recommendations.len(),
                3,
                "Backend should have 3 recommendations"
            );
            assert_eq!(
                coordination.devops_recommendations.len(),
                3,
                "DevOps should have 3 recommendations"
            );

            // Property: Should be marked as full-stack
            assert!(coordination.is_full_stack, "Should be marked as full-stack");
        }
    }

    /// Property 6: Full-Stack Coordination (Recommendation Integrity)
    /// For any coordination, individual recommendation data SHALL be preserved.
    #[test]
    fn property_full_stack_coordination_recommendation_integrity() {
        let coordinator = DomainCoordinator::new();

        let recommendations = vec![
            create_recommendation("web", "framework", vec!["React", "TypeScript"]),
            create_recommendation("backend", "api", vec!["REST"]),
            create_recommendation("devops", "containerization", vec!["Docker"]),
        ];

        for _ in 0..3 {
            let coordination = coordinator
                .coordinate_full_stack(recommendations.clone())
                .unwrap();

            // Property: Web recommendation should be intact
            assert_eq!(coordination.web_recommendations[0].domain, "web");
            assert_eq!(coordination.web_recommendations[0].category, "framework");
            assert_eq!(coordination.web_recommendations[0].technologies.len(), 2);
            assert!(coordination.web_recommendations[0]
                .technologies
                .contains(&"React".to_string()));
            assert!(coordination.web_recommendations[0]
                .technologies
                .contains(&"TypeScript".to_string()));

            // Property: Backend recommendation should be intact
            assert_eq!(coordination.backend_recommendations[0].domain, "backend");
            assert_eq!(coordination.backend_recommendations[0].category, "api");
            assert_eq!(
                coordination.backend_recommendations[0].technologies.len(),
                1
            );
            assert!(coordination.backend_recommendations[0]
                .technologies
                .contains(&"REST".to_string()));

            // Property: DevOps recommendation should be intact
            assert_eq!(coordination.devops_recommendations[0].domain, "devops");
            assert_eq!(
                coordination.devops_recommendations[0].category,
                "containerization"
            );
            assert_eq!(coordination.devops_recommendations[0].technologies.len(), 1);
            assert!(coordination.devops_recommendations[0]
                .technologies
                .contains(&"Docker".to_string()));
        }
    }
}
