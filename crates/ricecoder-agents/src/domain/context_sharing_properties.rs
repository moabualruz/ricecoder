//! Property-based tests for cross-domain context sharing
//!
//! **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
//! **Validates: Requirements 4.1-4.2**

#[cfg(test)]
mod tests {
    use crate::domain::{Recommendation, SharedContextManager};
    use proptest::prelude::*;

    // Helper function to create test recommendations
    fn create_recommendation(domain: &str, category: &str, technology: &str) -> Recommendation {
        Recommendation {
            domain: domain.to_string(),
            category: category.to_string(),
            content: format!("Recommendation for {}", technology),
            technologies: vec![technology.to_string()],
            rationale: format!("This is a good choice for {}", domain),
        }
    }

    /// Property 5.1: Context is shared between agents
    ///
    /// *For any* cross-domain request, the Domain Agent System SHALL share relevant context
    /// between domain agents such that all agents have access to shared project context.
    ///
    /// **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
    /// **Validates: Requirements 4.1-4.2**
    #[test]
    fn test_context_shared_between_agents() {
        let manager = SharedContextManager::new();

        // Set up shared context
        manager.set_project_type("web-application").unwrap();
        manager.add_technology("React").unwrap();
        manager.add_technology("Node.js").unwrap();
        manager.add_constraint("Must support IE11").unwrap();

        // Store recommendations from different agents
        let web_recommendations = vec![create_recommendation("web", "framework", "React")];
        let backend_recommendations = vec![create_recommendation("backend", "runtime", "Node.js")];

        manager
            .store_agent_recommendations("web-agent", web_recommendations)
            .unwrap();
        manager
            .store_agent_recommendations("backend-agent", backend_recommendations)
            .unwrap();

        // Verify that all agents have access to shared context
        let project_type = manager.get_project_type().unwrap();
        assert_eq!(project_type, "web-application");

        let tech_stack = manager.get_tech_stack().unwrap();
        assert_eq!(tech_stack.len(), 2);
        assert!(tech_stack.contains(&"React".to_string()));
        assert!(tech_stack.contains(&"Node.js".to_string()));

        let constraints = manager.get_constraints().unwrap();
        assert_eq!(constraints.len(), 1);
        assert_eq!(constraints[0], "Must support IE11");

        // Verify that each agent can access all recommendations
        let all_recommendations = manager.get_all_recommendations().unwrap();
        assert_eq!(all_recommendations.len(), 2);
    }

    /// Property 5.2: All agents have access to shared project context
    ///
    /// *For any* cross-domain request, all agents SHALL have access to shared project context
    /// (project type, tech stack, constraints).
    ///
    /// **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
    /// **Validates: Requirements 4.1-4.2**
    #[test]
    fn test_all_agents_access_shared_context() {
        let manager = SharedContextManager::new();

        // Set up shared context
        manager.set_project_type("backend-service").unwrap();
        manager.add_technology("PostgreSQL").unwrap();
        manager.add_technology("Redis").unwrap();
        manager.add_constraint("Must be scalable").unwrap();
        manager.add_constraint("Must support high availability").unwrap();

        // Store recommendations from multiple agents
        let web_recs = vec![create_recommendation("web", "framework", "React")];
        let backend_recs = vec![create_recommendation("backend", "database", "PostgreSQL")];
        let devops_recs = vec![create_recommendation("devops", "orchestration", "Kubernetes")];

        manager
            .store_agent_recommendations("web-agent", web_recs)
            .unwrap();
        manager
            .store_agent_recommendations("backend-agent", backend_recs)
            .unwrap();
        manager
            .store_agent_recommendations("devops-agent", devops_recs)
            .unwrap();

        // Verify that each agent can access the shared context
        for _agent_id in &["web-agent", "backend-agent", "devops-agent"] {
            let project_type = manager.get_project_type().unwrap();
            assert_eq!(project_type, "backend-service");

            let tech_stack = manager.get_tech_stack().unwrap();
            assert_eq!(tech_stack.len(), 2);

            let constraints = manager.get_constraints().unwrap();
            assert_eq!(constraints.len(), 2);

            // Each agent can access all recommendations
            let all_recs = manager.get_all_recommendations().unwrap();
            assert_eq!(all_recs.len(), 3);
        }
    }

    /// Property 5.3: Context updates are visible to all agents
    ///
    /// *For any* context update, all agents SHALL immediately see the updated context.
    ///
    /// **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
    /// **Validates: Requirements 4.1-4.2**
    #[test]
    fn test_context_updates_visible_to_all_agents() {
        let manager = SharedContextManager::new();

        // Initial context
        manager.set_project_type("web-app").unwrap();
        manager.add_technology("React").unwrap();

        // Verify initial state
        assert_eq!(manager.get_project_type().unwrap(), "web-app");
        assert_eq!(manager.get_tech_stack().unwrap().len(), 1);

        // Update context
        manager.set_project_type("full-stack-app").unwrap();
        manager.add_technology("Node.js").unwrap();

        // Verify updates are visible
        assert_eq!(manager.get_project_type().unwrap(), "full-stack-app");
        assert_eq!(manager.get_tech_stack().unwrap().len(), 2);
    }

    /// Property 5.4: Recommendations from different agents are accessible
    ///
    /// *For any* set of agent recommendations, all recommendations SHALL be accessible
    /// through the shared context manager.
    ///
    /// **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
    /// **Validates: Requirements 4.1-4.2**
    #[test]
    fn test_recommendations_from_different_agents_accessible() {
        let manager = SharedContextManager::new();

        // Store recommendations from different agents
        let web_recs = vec![
            create_recommendation("web", "framework", "React"),
            create_recommendation("web", "styling", "Tailwind"),
        ];
        let backend_recs = vec![
            create_recommendation("backend", "api", "REST"),
            create_recommendation("backend", "database", "PostgreSQL"),
        ];
        let devops_recs = vec![create_recommendation("devops", "ci-cd", "GitHub Actions")];

        manager
            .store_agent_recommendations("web-agent", web_recs)
            .unwrap();
        manager
            .store_agent_recommendations("backend-agent", backend_recs)
            .unwrap();
        manager
            .store_agent_recommendations("devops-agent", devops_recs)
            .unwrap();

        // Verify all recommendations are accessible
        let all_recs = manager.get_all_recommendations().unwrap();
        assert_eq!(all_recs.len(), 5);

        // Verify recommendations from each agent are accessible
        let web_recs_retrieved = manager.get_agent_recommendations("web-agent").unwrap();
        assert_eq!(web_recs_retrieved.len(), 2);

        let backend_recs_retrieved = manager.get_agent_recommendations("backend-agent").unwrap();
        assert_eq!(backend_recs_retrieved.len(), 2);

        let devops_recs_retrieved = manager.get_agent_recommendations("devops-agent").unwrap();
        assert_eq!(devops_recs_retrieved.len(), 1);
    }

    /// Property 5.5: Cross-domain state is maintained
    ///
    /// *For any* cross-domain state update, the state SHALL be maintained and accessible
    /// to all agents.
    ///
    /// **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
    /// **Validates: Requirements 4.1-4.2**
    #[test]
    fn test_cross_domain_state_maintained() {
        let manager = SharedContextManager::new();

        // Update cross-domain state
        manager
            .update_context("deployment_target", serde_json::json!("AWS"))
            .unwrap();
        manager
            .update_context("environment", serde_json::json!("production"))
            .unwrap();
        manager
            .update_context("budget_constraint", serde_json::json!(5000))
            .unwrap();

        // Verify state is maintained
        let deployment_target = manager.get_context("deployment_target").unwrap();
        assert_eq!(deployment_target, serde_json::json!("AWS"));

        let environment = manager.get_context("environment").unwrap();
        assert_eq!(environment, serde_json::json!("production"));

        let budget = manager.get_context("budget_constraint").unwrap();
        assert_eq!(budget, serde_json::json!(5000));

        // Verify all state is in shared context
        let shared_context = manager.get_shared_context().unwrap();
        assert_eq!(shared_context.cross_domain_state.len(), 3);
    }

    /// Property 5.6: Context isolation between different managers
    ///
    /// *For any* two different SharedContextManager instances, they SHALL maintain
    /// separate contexts.
    ///
    /// **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
    /// **Validates: Requirements 4.1-4.2**
    #[test]
    fn test_context_isolation_between_managers() {
        let manager1 = SharedContextManager::new();
        let manager2 = SharedContextManager::new();

        // Set different contexts
        manager1.set_project_type("web-app").unwrap();
        manager2.set_project_type("mobile-app").unwrap();

        manager1.add_technology("React").unwrap();
        manager2.add_technology("Flutter").unwrap();

        // Verify contexts are isolated
        assert_eq!(manager1.get_project_type().unwrap(), "web-app");
        assert_eq!(manager2.get_project_type().unwrap(), "mobile-app");

        let tech_stack1 = manager1.get_tech_stack().unwrap();
        let tech_stack2 = manager2.get_tech_stack().unwrap();

        assert_eq!(tech_stack1.len(), 1);
        assert_eq!(tech_stack2.len(), 1);
        assert_eq!(tech_stack1[0], "React");
        assert_eq!(tech_stack2[0], "Flutter");
    }

    /// Property 5.7: Duplicate technologies are not added
    ///
    /// *For any* technology added multiple times, it SHALL only appear once in the tech stack.
    ///
    /// **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
    /// **Validates: Requirements 4.1-4.2**
    #[test]
    fn test_duplicate_technologies_not_added() {
        let manager = SharedContextManager::new();

        manager.add_technology("React").unwrap();
        manager.add_technology("React").unwrap();
        manager.add_technology("React").unwrap();

        let tech_stack = manager.get_tech_stack().unwrap();
        assert_eq!(tech_stack.len(), 1);
        assert_eq!(tech_stack[0], "React");
    }

    /// Property 5.8: Duplicate constraints are not added
    ///
    /// *For any* constraint added multiple times, it SHALL only appear once in the constraints.
    ///
    /// **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
    /// **Validates: Requirements 4.1-4.2**
    #[test]
    fn test_duplicate_constraints_not_added() {
        let manager = SharedContextManager::new();

        manager.add_constraint("Must be scalable").unwrap();
        manager.add_constraint("Must be scalable").unwrap();
        manager.add_constraint("Must be scalable").unwrap();

        let constraints = manager.get_constraints().unwrap();
        assert_eq!(constraints.len(), 1);
        assert_eq!(constraints[0], "Must be scalable");
    }

    /// Property 5.9: Clear operation resets all context
    ///
    /// *For any* context state, calling clear() SHALL reset all context to default state.
    ///
    /// **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
    /// **Validates: Requirements 4.1-4.2**
    #[test]
    fn test_clear_resets_all_context() {
        let manager = SharedContextManager::new();

        // Set up context
        manager.set_project_type("web-app").unwrap();
        manager.add_technology("React").unwrap();
        manager.add_constraint("Must support IE11").unwrap();
        manager
            .store_agent_recommendations(
                "web-agent",
                vec![create_recommendation("web", "framework", "React")],
            )
            .unwrap();

        // Verify context is set
        assert_eq!(manager.get_project_type().unwrap(), "web-app");
        assert_eq!(manager.get_tech_stack().unwrap().len(), 1);
        assert_eq!(manager.get_constraints().unwrap().len(), 1);
        assert_eq!(manager.get_all_recommendations().unwrap().len(), 1);

        // Clear context
        manager.clear().unwrap();

        // Verify context is reset
        assert!(manager.get_project_type().unwrap().is_empty());
        assert!(manager.get_tech_stack().unwrap().is_empty());
        assert!(manager.get_constraints().unwrap().is_empty());
        assert!(manager.get_all_recommendations().unwrap().is_empty());
    }

    /// Property 5.10: Shared context contains all expected fields
    ///
    /// *For any* shared context, it SHALL contain project_type, tech_stack, constraints,
    /// and cross_domain_state fields.
    ///
    /// **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
    /// **Validates: Requirements 4.1-4.2**
    #[test]
    fn test_shared_context_contains_all_fields() {
        let manager = SharedContextManager::new();

        manager.set_project_type("web-app").unwrap();
        manager.add_technology("React").unwrap();
        manager.add_constraint("Must be fast").unwrap();
        manager
            .update_context("custom_field", serde_json::json!("custom_value"))
            .unwrap();

        let context = manager.get_shared_context().unwrap();

        // Verify all fields are present
        assert_eq!(context.project_type, "web-app");
        assert_eq!(context.tech_stack.len(), 1);
        assert_eq!(context.constraints.len(), 1);
        assert_eq!(context.cross_domain_state.len(), 1);
    }

    proptest! {
        /// Property 5.11: Multiple agents can store and retrieve recommendations
        ///
        /// *For any* number of agents, each agent SHALL be able to store and retrieve
        /// its own recommendations independently.
        ///
        /// **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
        /// **Validates: Requirements 4.1-4.2**
        #[test]
        fn prop_multiple_agents_store_retrieve_recommendations(
            num_agents in 1..10usize,
            recs_per_agent in 1..5usize,
        ) {
            let manager = SharedContextManager::new();

            // Store recommendations from multiple agents
            for agent_idx in 0..num_agents {
                let agent_id = format!("agent-{}", agent_idx);
                let mut recommendations = Vec::new();

                for rec_idx in 0..recs_per_agent {
                    let domain = match agent_idx % 3 {
                        0 => "web",
                        1 => "backend",
                        _ => "devops",
                    };
                    let rec = create_recommendation(
                        domain,
                        &format!("category-{}", rec_idx),
                        &format!("tech-{}", rec_idx),
                    );
                    recommendations.push(rec);
                }

                manager
                    .store_agent_recommendations(&agent_id, recommendations)
                    .unwrap();
            }

            // Verify all recommendations are stored
            let all_recs = manager.get_all_recommendations().unwrap();
            prop_assert_eq!(all_recs.len(), num_agents * recs_per_agent);

            // Verify each agent can retrieve its recommendations
            for agent_idx in 0..num_agents {
                let agent_id = format!("agent-{}", agent_idx);
                let agent_recs = manager.get_agent_recommendations(&agent_id).unwrap();
                prop_assert_eq!(agent_recs.len(), recs_per_agent);
            }
        }
    }

    proptest! {
        /// Property 5.12: Context values can be updated multiple times
        ///
        /// *For any* context key, it SHALL be possible to update its value multiple times,
        /// with the latest value being returned.
        ///
        /// **Feature: ricecoder-domain-agents, Property 5: Cross-Domain Context Sharing**
        /// **Validates: Requirements 4.1-4.2**
        #[test]
        fn prop_context_values_updated_multiple_times(
            values in prop::collection::vec("[a-z]+", 1..10),
        ) {
            let manager = SharedContextManager::new();

            // Update context multiple times
            for (_idx, value) in values.iter().enumerate() {
                manager
                    .update_context("key", serde_json::json!(value))
                    .unwrap();

                // Verify the latest value is returned
                let retrieved = manager.get_context("key").unwrap();
                prop_assert_eq!(retrieved, serde_json::json!(value));
            }

            // Verify final value is correct
            let final_value = manager.get_context("key").unwrap();
            prop_assert_eq!(final_value, serde_json::json!(values.last().unwrap().as_str()));
        }
    }
}
