//! Integration tests for domain agents multi-agent coordination

use ricecoder_agents::domain::{
    DomainRegistry, KnowledgeBase, SharedContextManager, DomainCoordinator, DomainRequest,
    DomainAgent, DomainCapability, DomainKnowledge, Recommendation, BestPractice,
    TechRecommendation, AgentFactory, AgentConfig, CapabilityConfig,
};
use std::collections::HashMap;

/// Helper function to create a test agent
fn create_test_agent(domain: &str) -> DomainAgent {
    DomainAgent {
        id: format!("{}-agent", domain),
        domain: domain.to_string(),
        capabilities: vec![
            DomainCapability {
                name: "Test Capability".to_string(),
                description: "A test capability".to_string(),
                technologies: vec!["Tech1".to_string()],
                patterns: vec![],
            },
        ],
        knowledge: DomainKnowledge::default(),
    }
}

/// Helper function to create a test recommendation
fn create_test_recommendation(domain: &str) -> Recommendation {
    Recommendation {
        domain: domain.to_string(),
        category: "test".to_string(),
        content: "Test recommendation".to_string(),
        technologies: vec!["Tech1".to_string()],
        rationale: "Test rationale".to_string(),
    }
}

/// Test 13.3: Multi-agent coordination - cross-domain workflows
#[test]
fn test_multi_agent_coordination_cross_domain_workflows() {
    let registry = DomainRegistry::new();
    let coordinator = DomainCoordinator::new();

    // Register agents for multiple domains
    registry.register_agent("web", create_test_agent("web")).unwrap();
    registry.register_agent("backend", create_test_agent("backend")).unwrap();
    registry.register_agent("devops", create_test_agent("devops")).unwrap();

    // Create a cross-domain request
    let request = DomainRequest {
        id: "req-1".to_string(),
        domains: vec!["web".to_string(), "backend".to_string(), "devops".to_string()],
        content: "Help me set up a full-stack application".to_string(),
        context: HashMap::new(),
    };

    // Route the request
    let routed_domains = coordinator.route_request(&request).unwrap();
    assert_eq!(routed_domains.len(), 3);

    // Verify all agents are available
    for domain in &routed_domains {
        let agent = registry.get_agent(domain).unwrap();
        assert_eq!(agent.domain, *domain);
    }
}

/// Test 13.3: Multi-agent coordination - context sharing between agents
#[test]
fn test_multi_agent_coordination_context_sharing() {
    let context_manager = SharedContextManager::new();

    // Set up shared context
    context_manager.set_project_type("full-stack-web-app").unwrap();
    context_manager.add_technology("React").unwrap();
    context_manager.add_technology("Node.js").unwrap();
    context_manager.add_technology("PostgreSQL").unwrap();
    context_manager.add_constraint("Must support modern browsers").unwrap();

    // Store recommendations from different agents
    let web_recs = vec![create_test_recommendation("web")];
    let backend_recs = vec![create_test_recommendation("backend")];
    let devops_recs = vec![create_test_recommendation("devops")];

    context_manager.store_agent_recommendations("web-agent", web_recs).unwrap();
    context_manager.store_agent_recommendations("backend-agent", backend_recs).unwrap();
    context_manager.store_agent_recommendations("devops-agent", devops_recs).unwrap();

    // Verify all agents have access to shared context
    let project_type = context_manager.get_project_type().unwrap();
    assert_eq!(project_type, "full-stack-web-app");

    let tech_stack = context_manager.get_tech_stack().unwrap();
    assert_eq!(tech_stack.len(), 3);

    let constraints = context_manager.get_constraints().unwrap();
    assert_eq!(constraints.len(), 1);

    // Verify all recommendations are accessible
    let all_recs = context_manager.get_all_recommendations().unwrap();
    assert_eq!(all_recs.len(), 3);
}

/// Test 13.3: Multi-agent coordination - conflict detection and reporting
#[test]
fn test_multi_agent_coordination_conflict_detection() {
    let coordinator = DomainCoordinator::new();

    // Create recommendations with potential conflicts
    let mut web_rec = create_test_recommendation("web");
    web_rec.technologies = vec!["Webpack".to_string()];

    let mut backend_rec = create_test_recommendation("backend");
    backend_rec.technologies = vec!["Node.js".to_string()];

    let mut devops_rec = create_test_recommendation("devops");
    devops_rec.technologies = vec!["Vite".to_string()];

    let recommendations = vec![web_rec, backend_rec, devops_rec];

    // Coordinate responses
    let coordinated = coordinator.coordinate_responses(recommendations).unwrap();
    assert_eq!(coordinated.domain_count, 3);
    assert_eq!(coordinated.total_recommendations, 3);

    // Detect conflicts
    let coordination = coordinator.coordinate_full_stack(coordinated.recommendations).unwrap();
    let conflicts = coordinator.detect_full_stack_conflicts(&coordination).unwrap();

    // Webpack and Vite are incompatible
    assert!(!conflicts.is_empty());
    assert!(conflicts[0].contains("Incompatible"));
}

/// Test 13.4: Full-stack planning - coordinated recommendations across domains
#[test]
fn test_full_stack_planning_coordinated_recommendations() {
    let coordinator = DomainCoordinator::new();

    // Create recommendations from all three domains
    let web_rec = create_test_recommendation("web");
    let backend_rec = create_test_recommendation("backend");
    let devops_rec = create_test_recommendation("devops");

    let recommendations = vec![web_rec, backend_rec, devops_rec];

    // Coordinate full-stack recommendations
    let coordination = coordinator.coordinate_full_stack(recommendations).unwrap();

    // Verify full-stack coordination
    assert!(coordination.is_full_stack);
    assert_eq!(coordination.web_recommendations.len(), 1);
    assert_eq!(coordination.backend_recommendations.len(), 1);
    assert_eq!(coordination.devops_recommendations.len(), 1);
    assert_eq!(coordination.total_recommendations, 3);
}

/// Test 13.4: Full-stack planning - operation sequencing
#[test]
fn test_full_stack_planning_operation_sequencing() {
    use ricecoder_agents::domain::Operation;

    let coordinator = DomainCoordinator::new();

    // Create operations with different priorities
    let operations = vec![
        Operation {
            id: "deploy".to_string(),
            name: "Deploy Application".to_string(),
            priority: 3,
            dependencies: vec!["setup".to_string()],
        },
        Operation {
            id: "setup".to_string(),
            name: "Setup Infrastructure".to_string(),
            priority: 1,
            dependencies: vec![],
        },
        Operation {
            id: "configure".to_string(),
            name: "Configure Services".to_string(),
            priority: 2,
            dependencies: vec!["setup".to_string()],
        },
    ];

    // Sequence operations
    let sequenced = coordinator.sequence_operations(operations).unwrap();

    // Verify correct sequencing
    assert_eq!(sequenced[0].priority, 1);
    assert_eq!(sequenced[1].priority, 2);
    assert_eq!(sequenced[2].priority, 3);

    // Verify dependencies are respected
    assert_eq!(sequenced[0].id, "setup");
    assert_eq!(sequenced[1].id, "configure");
    assert_eq!(sequenced[2].id, "deploy");
}

/// Test 13.4: Full-stack planning - consistency validation
#[test]
fn test_full_stack_planning_consistency_validation() {
    let coordinator = DomainCoordinator::new();

    // Create valid recommendations
    let recommendations = vec![
        create_test_recommendation("web"),
        create_test_recommendation("backend"),
        create_test_recommendation("devops"),
    ];

    // Validate consistency
    assert!(coordinator.validate_consistency(&recommendations).unwrap());

    // Coordinate full-stack
    let coordination = coordinator.coordinate_full_stack(recommendations).unwrap();

    // Ensure full-stack consistency
    assert!(coordinator.ensure_full_stack_consistency(&coordination).unwrap());
}

/// Test 13.5: Custom domain support - adding new domains through configuration
#[test]
fn test_custom_domain_support_adding_domains() {
    let factory = AgentFactory::new();
    let registry = DomainRegistry::new();

    // Create configuration for a custom domain
    let config = AgentConfig {
        domain: "mobile".to_string(),
        name: "Mobile Development Agent".to_string(),
        description: "Agent for mobile development".to_string(),
        capabilities: vec![
            CapabilityConfig {
                name: "Framework Selection".to_string(),
                description: "Select mobile frameworks".to_string(),
                technologies: vec!["React Native".to_string(), "Flutter".to_string()],
            },
        ],
        best_practices: vec![],
        technology_recommendations: vec![],
        patterns: vec![],
        anti_patterns: vec![],
    };

    // Create agent from configuration
    let agent = factory.create_agent("mobile", &config).unwrap();

    // Register the custom domain agent
    registry.register_agent("mobile", agent).unwrap();

    // Verify custom domain is available
    assert!(registry.has_domain("mobile").unwrap());
    let registered_agent = registry.get_agent("mobile").unwrap();
    assert_eq!(registered_agent.domain, "mobile");
}

/// Test 13.5: Custom domain support - auto-discovery of new domains
#[test]
fn test_custom_domain_support_auto_discovery() {
    let registry = DomainRegistry::new();

    // Register multiple domains including custom ones
    registry.register_agent("web", create_test_agent("web")).unwrap();
    registry.register_agent("backend", create_test_agent("backend")).unwrap();
    registry.register_agent("devops", create_test_agent("devops")).unwrap();
    registry.register_agent("custom-mobile", create_test_agent("custom-mobile")).unwrap();
    registry.register_agent("custom-data-science", create_test_agent("custom-data-science")).unwrap();

    // Discover all domains
    let domains = registry.discover_domains().unwrap();

    // Verify all domains are discovered
    assert_eq!(domains.len(), 5);
    assert!(domains.contains(&"web".to_string()));
    assert!(domains.contains(&"backend".to_string()));
    assert!(domains.contains(&"devops".to_string()));
    assert!(domains.contains(&"custom-mobile".to_string()));
    assert!(domains.contains(&"custom-data-science".to_string()));
}

/// Test 13.5: Custom domain support - creating agents for custom domains
#[test]
fn test_custom_domain_support_creating_agents() {
    let factory = AgentFactory::new();

    // Create configurations for multiple custom domains
    let mobile_config = AgentConfig {
        domain: "mobile".to_string(),
        name: "Mobile Agent".to_string(),
        description: "Mobile development".to_string(),
        capabilities: vec![
            CapabilityConfig {
                name: "Framework Selection".to_string(),
                description: "Select frameworks".to_string(),
                technologies: vec!["React Native".to_string()],
            },
        ],
        best_practices: vec![],
        technology_recommendations: vec![],
        patterns: vec![],
        anti_patterns: vec![],
    };

    let data_science_config = AgentConfig {
        domain: "data-science".to_string(),
        name: "Data Science Agent".to_string(),
        description: "Data science development".to_string(),
        capabilities: vec![
            CapabilityConfig {
                name: "Framework Selection".to_string(),
                description: "Select frameworks".to_string(),
                technologies: vec!["TensorFlow".to_string(), "PyTorch".to_string()],
            },
        ],
        best_practices: vec![],
        technology_recommendations: vec![],
        patterns: vec![],
        anti_patterns: vec![],
    };

    // Create agents from configurations
    let mobile_agent = factory.create_agent("mobile", &mobile_config).unwrap();
    let data_science_agent = factory.create_agent("data-science", &data_science_config).unwrap();

    // Verify agents are created correctly
    assert_eq!(mobile_agent.domain, "mobile");
    assert_eq!(mobile_agent.id, "mobile-agent");
    assert_eq!(mobile_agent.capabilities.len(), 1);

    assert_eq!(data_science_agent.domain, "data-science");
    assert_eq!(data_science_agent.id, "data-science-agent");
    assert_eq!(data_science_agent.capabilities.len(), 1);
}

/// Integration test: Full workflow with multiple agents and coordination
#[test]
fn test_full_integration_workflow() {
    let registry = DomainRegistry::new();
    let knowledge_base = KnowledgeBase::new();
    let context_manager = SharedContextManager::new();
    let coordinator = DomainCoordinator::new();

    // Step 1: Register agents
    registry.register_agent("web", create_test_agent("web")).unwrap();
    registry.register_agent("backend", create_test_agent("backend")).unwrap();
    registry.register_agent("devops", create_test_agent("devops")).unwrap();

    // Step 2: Set up shared context
    context_manager.set_project_type("full-stack-app").unwrap();
    context_manager.add_technology("React").unwrap();
    context_manager.add_technology("Node.js").unwrap();
    context_manager.add_technology("Docker").unwrap();

    // Step 3: Add knowledge to knowledge base
    knowledge_base.add_best_practice("web", BestPractice {
        title: "Component-Based Architecture".to_string(),
        description: "Use components".to_string(),
        domain: "web".to_string(),
        technologies: vec!["React".to_string()],
        implementation: "Break UI into components".to_string(),
    }).unwrap();

    knowledge_base.add_tech_recommendation("backend", TechRecommendation {
        technology: "Node.js".to_string(),
        domain: "backend".to_string(),
        use_cases: vec!["APIs".to_string()],
        pros: vec!["JavaScript ecosystem".to_string()],
        cons: vec!["Single-threaded".to_string()],
        alternatives: vec!["Python".to_string()],
    }).unwrap();

    // Step 4: Create and route request
    let request = DomainRequest {
        id: "req-1".to_string(),
        domains: vec!["web".to_string(), "backend".to_string(), "devops".to_string()],
        content: "Help me build a full-stack application".to_string(),
        context: HashMap::new(),
    };

    let routed_domains = coordinator.route_request(&request).unwrap();
    assert_eq!(routed_domains.len(), 3);

    // Step 5: Collect recommendations from agents
    let recommendations = vec![
        create_test_recommendation("web"),
        create_test_recommendation("backend"),
        create_test_recommendation("devops"),
    ];

    // Step 6: Store recommendations in context
    context_manager.store_agent_recommendations("web-agent", vec![recommendations[0].clone()]).unwrap();
    context_manager.store_agent_recommendations("backend-agent", vec![recommendations[1].clone()]).unwrap();
    context_manager.store_agent_recommendations("devops-agent", vec![recommendations[2].clone()]).unwrap();

    // Step 7: Coordinate responses
    let coordinated = coordinator.coordinate_responses(recommendations).unwrap();
    assert_eq!(coordinated.domain_count, 3);

    // Step 8: Validate full-stack coordination
    let coordination = coordinator.coordinate_full_stack(coordinated.recommendations).unwrap();
    assert!(coordination.is_full_stack);
    assert!(coordinator.ensure_full_stack_consistency(&coordination).unwrap());

    // Step 9: Verify all agents are available
    for domain in &routed_domains {
        let agent = registry.get_agent(domain).unwrap();
        assert_eq!(agent.domain, *domain);
    }

    // Step 10: Verify shared context is accessible
    let project_type = context_manager.get_project_type().unwrap();
    assert_eq!(project_type, "full-stack-app");

    let all_recs = context_manager.get_all_recommendations().unwrap();
    assert_eq!(all_recs.len(), 3);
}

/// Integration test: Multi-domain coordination with knowledge base
#[test]
fn test_multi_domain_coordination_with_knowledge() {
    let registry = DomainRegistry::new();
    let knowledge_base = KnowledgeBase::new();
    let coordinator = DomainCoordinator::new();

    // Register agents
    registry.register_agent("web", create_test_agent("web")).unwrap();
    registry.register_agent("backend", create_test_agent("backend")).unwrap();

    // Add knowledge for each domain
    knowledge_base.add_best_practice("web", BestPractice {
        title: "Responsive Design".to_string(),
        description: "Design for all screen sizes".to_string(),
        domain: "web".to_string(),
        technologies: vec!["CSS".to_string()],
        implementation: "Use media queries".to_string(),
    }).unwrap();

    knowledge_base.add_best_practice("backend", BestPractice {
        title: "API Versioning".to_string(),
        description: "Version your APIs".to_string(),
        domain: "backend".to_string(),
        technologies: vec!["REST".to_string()],
        implementation: "Use URL versioning".to_string(),
    }).unwrap();

    // Get knowledge for each domain
    let web_practices = knowledge_base.get_best_practices("web").unwrap();
    let backend_practices = knowledge_base.get_best_practices("backend").unwrap();

    assert_eq!(web_practices.len(), 1);
    assert_eq!(backend_practices.len(), 1);

    // Create recommendations based on knowledge
    let recommendations = vec![
        create_test_recommendation("web"),
        create_test_recommendation("backend"),
    ];

    // Coordinate recommendations
    let coordinated = coordinator.coordinate_responses(recommendations).unwrap();
    assert_eq!(coordinated.domain_count, 2);
}

/// Integration test: Custom domain workflow
#[test]
fn test_custom_domain_integration_workflow() {
    let factory = AgentFactory::new();
    let registry = DomainRegistry::new();
    let coordinator = DomainCoordinator::new();

    // Create custom domain configuration
    let custom_config = AgentConfig {
        domain: "custom-mobile".to_string(),
        name: "Custom Mobile Agent".to_string(),
        description: "Custom mobile development".to_string(),
        capabilities: vec![
            CapabilityConfig {
                name: "Framework Selection".to_string(),
                description: "Select mobile frameworks".to_string(),
                technologies: vec!["React Native".to_string(), "Flutter".to_string()],
            },
        ],
        best_practices: vec![],
        technology_recommendations: vec![],
        patterns: vec![],
        anti_patterns: vec![],
    };

    // Create agent from custom configuration
    let custom_agent = factory.create_agent("custom-mobile", &custom_config).unwrap();

    // Register custom agent
    registry.register_agent("custom-mobile", custom_agent).unwrap();

    // Create request for custom domain
    let request = DomainRequest {
        id: "req-custom".to_string(),
        domains: vec!["custom-mobile".to_string()],
        content: "Help me with mobile development".to_string(),
        context: HashMap::new(),
    };

    // Route request to custom domain
    let routed = coordinator.route_request(&request).unwrap();
    assert_eq!(routed.len(), 1);
    assert_eq!(routed[0], "custom-mobile");

    // Verify custom agent is available
    let agent = registry.get_agent("custom-mobile").unwrap();
    assert_eq!(agent.domain, "custom-mobile");
    assert_eq!(agent.capabilities.len(), 1);
}
