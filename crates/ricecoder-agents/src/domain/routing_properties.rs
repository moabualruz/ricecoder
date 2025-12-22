//! Property-based tests for domain agent routing
//!
//! **Feature: ricecoder-domain-agents, Property 1: Web Agent Domain Routing**
//! **Feature: ricecoder-domain-agents, Property 2: Backend Agent Domain Routing**
//! **Feature: ricecoder-domain-agents, Property 3: DevOps Agent Domain Routing**
//! **Validates: Requirements 1.1-1.6, 2.1-2.6, 3.1-3.6**

#[cfg(test)]
mod tests {
    use crate::domain::{
        factory::{AgentConfig, CapabilityConfig, TechRecommendationConfig},
        AgentFactory, DomainRegistry,
    };

    /// Helper function to create a test agent configuration
    fn create_agent_config(
        domain: &str,
        name: &str,
        capabilities: Vec<(&str, Vec<&str>)>,
        tech_recommendations: Vec<(&str, Vec<&str>)>,
    ) -> AgentConfig {
        AgentConfig {
            domain: domain.to_string(),
            name: name.to_string(),
            description: format!("Agent for {} development", domain),
            capabilities: capabilities
                .into_iter()
                .map(|(cap_name, techs)| CapabilityConfig {
                    name: cap_name.to_string(),
                    description: format!("Capability: {}", cap_name),
                    technologies: techs.into_iter().map(|t| t.to_string()).collect(),
                })
                .collect(),
            best_practices: vec![],
            technology_recommendations: tech_recommendations
                .into_iter()
                .map(|(tech, use_cases)| TechRecommendationConfig {
                    technology: tech.to_string(),
                    use_cases: use_cases.into_iter().map(|u| u.to_string()).collect(),
                    pros: vec!["Pro 1".to_string()],
                    cons: vec!["Con 1".to_string()],
                    alternatives: vec![],
                })
                .collect(),
            patterns: vec![],
            anti_patterns: vec![],
        }
    }

    /// Property 1: Web Agent Domain Routing
    /// For any web development request, the web agent SHALL be routed to and provide
    /// recommendations for web technologies (React, Vue, Angular, CSS, Tailwind, Vite, Webpack, Jest, Vitest, Playwright).
    ///
    /// This property tests that:
    /// 1. Web agent is registered and discoverable
    /// 2. Web agent provides recommendations for web technologies
    /// 3. Web agent capabilities include frontend frameworks, styling, build tools, testing
    #[test]
    fn property_web_agent_domain_routing_registration() {
        let registry = DomainRegistry::new();
        let factory = AgentFactory::new();

        // Create web agent configuration
        let web_config = create_agent_config(
            "web",
            "Web Development Agent",
            vec![
                (
                    "Frontend Framework Selection",
                    vec!["React", "Vue", "Angular"],
                ),
                (
                    "Styling Guidance",
                    vec!["CSS", "Tailwind CSS", "styled-components"],
                ),
                ("Build Configuration", vec!["Vite", "Webpack"]),
                ("Testing Strategy", vec!["Jest", "Vitest", "Playwright"]),
            ],
            vec![
                ("React", vec!["Single Page Applications", "Complex UIs"]),
                ("Vue", vec!["Progressive enhancement", "Rapid prototyping"]),
                ("Angular", vec!["Enterprise applications"]),
                ("Vite", vec!["Modern web development"]),
                ("Webpack", vec!["Complex bundling scenarios"]),
                ("Jest", vec!["JavaScript testing"]),
                ("Vitest", vec!["Vite projects"]),
                ("Playwright", vec!["E2E testing"]),
            ],
        );

        // Create web agent
        let web_agent = factory.create_agent("web", &web_config).unwrap();

        // Register web agent
        registry.register_agent("web", web_agent.clone()).unwrap();

        // Property 1: Web agent is registered
        assert!(registry.has_domain("web").unwrap());

        // Property 2: Web agent is discoverable
        let domains = registry.discover_domains().unwrap();
        assert!(domains.contains(&"web".to_string()));

        // Property 3: Web agent can be retrieved
        let retrieved_agent = registry.get_agent("web").unwrap();
        assert_eq!(retrieved_agent.domain, "web");
        assert_eq!(retrieved_agent.id, "web-agent");

        // Property 4: Web agent has correct capabilities
        let capabilities = registry.list_capabilities("web").unwrap();
        assert_eq!(capabilities.len(), 4);

        let capability_names: Vec<_> = capabilities.iter().map(|c| &c.name).collect();
        assert!(capability_names.contains(&&"Frontend Framework Selection".to_string()));
        assert!(capability_names.contains(&&"Styling Guidance".to_string()));
        assert!(capability_names.contains(&&"Build Configuration".to_string()));
        assert!(capability_names.contains(&&"Testing Strategy".to_string()));

        // Property 5: Web agent has web technologies
        let web_techs: Vec<_> = capabilities.iter().flat_map(|c| &c.technologies).collect();
        assert!(web_techs.contains(&&"React".to_string()));
        assert!(web_techs.contains(&&"Vue".to_string()));
        assert!(web_techs.contains(&&"Angular".to_string()));
        assert!(web_techs.contains(&&"Vite".to_string()));
        assert!(web_techs.contains(&&"Webpack".to_string()));
        assert!(web_techs.contains(&&"Jest".to_string()));
        assert!(web_techs.contains(&&"Vitest".to_string()));
        assert!(web_techs.contains(&&"Playwright".to_string()));

        // Property 6: Web agent has technology recommendations
        let tech_recs = &web_agent.knowledge.technology_recommendations;
        assert!(!tech_recs.is_empty());

        let tech_names: Vec<_> = tech_recs.iter().map(|t| &t.technology).collect();
        assert!(tech_names.contains(&&"React".to_string()));
        assert!(tech_names.contains(&&"Vue".to_string()));
        assert!(tech_names.contains(&&"Vite".to_string()));
        assert!(tech_names.contains(&&"Jest".to_string()));
    }

    /// Property 1: Web Agent Domain Routing (Multiple Registrations)
    /// For any web development request, the web agent SHALL remain routable
    /// even after multiple registrations
    #[test]
    fn property_web_agent_domain_routing_multiple_registrations() {
        let registry = DomainRegistry::new();
        let factory = AgentFactory::new();

        let web_config = create_agent_config(
            "web",
            "Web Development Agent",
            vec![("Frontend Framework Selection", vec!["React", "Vue"])],
            vec![("React", vec!["SPAs"])],
        );

        // Register web agent multiple times
        for _ in 0..3 {
            let web_agent = factory.create_agent("web", &web_config).unwrap();
            registry.register_agent("web", web_agent).unwrap();
        }

        // Property: Web agent should still be routable
        assert!(registry.has_domain("web").unwrap());
        let agent = registry.get_agent("web").unwrap();
        assert_eq!(agent.domain, "web");
    }

    /// Property 2: Backend Agent Domain Routing
    /// For any backend development request, the backend agent SHALL be routed to and provide
    /// recommendations for backend technologies (REST, GraphQL, gRPC, microservices, monolithic,
    /// serverless, PostgreSQL, MongoDB, Redis, OAuth, JWT).
    ///
    /// This property tests that:
    /// 1. Backend agent is registered and discoverable
    /// 2. Backend agent provides recommendations for backend technologies
    /// 3. Backend agent capabilities include API design, architecture, database, scalability, security, observability
    #[test]
    fn property_backend_agent_domain_routing_registration() {
        let registry = DomainRegistry::new();
        let factory = AgentFactory::new();

        // Create backend agent configuration
        let backend_config = create_agent_config(
            "backend",
            "Backend Development Agent",
            vec![
                ("API Design", vec!["REST", "GraphQL", "gRPC"]),
                (
                    "Architecture Guidance",
                    vec!["Microservices", "Monolithic", "Serverless"],
                ),
                ("Database Design", vec!["PostgreSQL", "MongoDB", "Neo4j"]),
                ("Scalability", vec!["Redis", "Memcached", "Load Balancers"]),
                ("Security", vec!["OAuth 2.0", "JWT", "TLS"]),
                ("Observability", vec!["ELK Stack", "Prometheus", "Jaeger"]),
            ],
            vec![
                ("REST", vec!["Web APIs", "Microservices"]),
                ("GraphQL", vec!["Flexible APIs", "Real-time data"]),
                ("gRPC", vec!["High-performance APIs"]),
                ("PostgreSQL", vec!["Relational data", "ACID transactions"]),
                ("MongoDB", vec!["Document storage", "Flexible schema"]),
                ("Redis", vec!["Caching", "Session storage"]),
                ("OAuth 2.0", vec!["Authentication"]),
                ("JWT", vec!["Token-based auth"]),
            ],
        );

        // Create backend agent
        let backend_agent = factory.create_agent("backend", &backend_config).unwrap();

        // Register backend agent
        registry
            .register_agent("backend", backend_agent.clone())
            .unwrap();

        // Property 1: Backend agent is registered
        assert!(registry.has_domain("backend").unwrap());

        // Property 2: Backend agent is discoverable
        let domains = registry.discover_domains().unwrap();
        assert!(domains.contains(&"backend".to_string()));

        // Property 3: Backend agent can be retrieved
        let retrieved_agent = registry.get_agent("backend").unwrap();
        assert_eq!(retrieved_agent.domain, "backend");
        assert_eq!(retrieved_agent.id, "backend-agent");

        // Property 4: Backend agent has correct capabilities
        let capabilities = registry.list_capabilities("backend").unwrap();
        assert_eq!(capabilities.len(), 6);

        let capability_names: Vec<_> = capabilities.iter().map(|c| &c.name).collect();
        assert!(capability_names.contains(&&"API Design".to_string()));
        assert!(capability_names.contains(&&"Architecture Guidance".to_string()));
        assert!(capability_names.contains(&&"Database Design".to_string()));
        assert!(capability_names.contains(&&"Scalability".to_string()));
        assert!(capability_names.contains(&&"Security".to_string()));
        assert!(capability_names.contains(&&"Observability".to_string()));

        // Property 5: Backend agent has backend technologies
        let backend_techs: Vec<_> = capabilities.iter().flat_map(|c| &c.technologies).collect();
        assert!(backend_techs.contains(&&"REST".to_string()));
        assert!(backend_techs.contains(&&"GraphQL".to_string()));
        assert!(backend_techs.contains(&&"gRPC".to_string()));
        assert!(backend_techs.contains(&&"PostgreSQL".to_string()));
        assert!(backend_techs.contains(&&"MongoDB".to_string()));
        assert!(backend_techs.contains(&&"Redis".to_string()));
        assert!(backend_techs.contains(&&"OAuth 2.0".to_string()));
        assert!(backend_techs.contains(&&"JWT".to_string()));

        // Property 6: Backend agent has technology recommendations
        let tech_recs = &backend_agent.knowledge.technology_recommendations;
        assert!(!tech_recs.is_empty());

        let tech_names: Vec<_> = tech_recs.iter().map(|t| &t.technology).collect();
        assert!(tech_names.contains(&&"REST".to_string()));
        assert!(tech_names.contains(&&"GraphQL".to_string()));
        assert!(tech_names.contains(&&"PostgreSQL".to_string()));
        assert!(tech_names.contains(&&"MongoDB".to_string()));
    }

    /// Property 2: Backend Agent Domain Routing (Multiple Registrations)
    /// For any backend development request, the backend agent SHALL remain routable
    /// even after multiple registrations
    #[test]
    fn property_backend_agent_domain_routing_multiple_registrations() {
        let registry = DomainRegistry::new();
        let factory = AgentFactory::new();

        let backend_config = create_agent_config(
            "backend",
            "Backend Development Agent",
            vec![("API Design", vec!["REST", "GraphQL"])],
            vec![("REST", vec!["Web APIs"])],
        );

        // Register backend agent multiple times
        for _ in 0..3 {
            let backend_agent = factory.create_agent("backend", &backend_config).unwrap();
            registry.register_agent("backend", backend_agent).unwrap();
        }

        // Property: Backend agent should still be routable
        assert!(registry.has_domain("backend").unwrap());
        let agent = registry.get_agent("backend").unwrap();
        assert_eq!(agent.domain, "backend");
    }

    /// Property 3: DevOps Agent Domain Routing
    /// For any DevOps request, the DevOps agent SHALL be routed to and provide
    /// recommendations for DevOps technologies (GitHub Actions, GitLab CI, Jenkins, Terraform,
    /// CloudFormation, Ansible, Docker, Kubernetes, Prometheus, Grafana).
    ///
    /// This property tests that:
    /// 1. DevOps agent is registered and discoverable
    /// 2. DevOps agent provides recommendations for DevOps technologies
    /// 3. DevOps agent capabilities include CI/CD, IaC, containerization, observability, security scanning, auto-scaling
    #[test]
    fn property_devops_agent_domain_routing_registration() {
        let registry = DomainRegistry::new();
        let factory = AgentFactory::new();

        // Create DevOps agent configuration
        let devops_config = create_agent_config(
            "devops",
            "DevOps Agent",
            vec![
                (
                    "CI/CD Pipeline Setup",
                    vec!["GitHub Actions", "GitLab CI", "Jenkins"],
                ),
                (
                    "Infrastructure as Code",
                    vec!["Terraform", "CloudFormation", "Ansible"],
                ),
                ("Containerization", vec!["Docker", "Kubernetes"]),
                (
                    "Observability Infrastructure",
                    vec!["Prometheus", "Grafana", "ELK Stack"],
                ),
                ("Security Scanning", vec!["Trivy", "Snyk", "SonarQube"]),
                (
                    "Auto-Scaling",
                    vec!["Kubernetes", "AWS Auto Scaling", "Docker Swarm"],
                ),
            ],
            vec![
                ("GitHub Actions", vec!["CI/CD pipelines"]),
                ("GitLab CI", vec!["CI/CD pipelines"]),
                ("Jenkins", vec!["CI/CD pipelines"]),
                ("Terraform", vec!["Infrastructure as Code"]),
                ("CloudFormation", vec!["Infrastructure as Code"]),
                ("Ansible", vec!["Infrastructure as Code"]),
                ("Docker", vec!["Containerization"]),
                ("Kubernetes", vec!["Container orchestration"]),
                ("Prometheus", vec!["Monitoring"]),
                ("Grafana", vec!["Visualization"]),
            ],
        );

        // Create DevOps agent
        let devops_agent = factory.create_agent("devops", &devops_config).unwrap();

        // Register DevOps agent
        registry
            .register_agent("devops", devops_agent.clone())
            .unwrap();

        // Property 1: DevOps agent is registered
        assert!(registry.has_domain("devops").unwrap());

        // Property 2: DevOps agent is discoverable
        let domains = registry.discover_domains().unwrap();
        assert!(domains.contains(&"devops".to_string()));

        // Property 3: DevOps agent can be retrieved
        let retrieved_agent = registry.get_agent("devops").unwrap();
        assert_eq!(retrieved_agent.domain, "devops");
        assert_eq!(retrieved_agent.id, "devops-agent");

        // Property 4: DevOps agent has correct capabilities
        let capabilities = registry.list_capabilities("devops").unwrap();
        assert_eq!(capabilities.len(), 6);

        let capability_names: Vec<_> = capabilities.iter().map(|c| &c.name).collect();
        assert!(capability_names.contains(&&"CI/CD Pipeline Setup".to_string()));
        assert!(capability_names.contains(&&"Infrastructure as Code".to_string()));
        assert!(capability_names.contains(&&"Containerization".to_string()));
        assert!(capability_names.contains(&&"Observability Infrastructure".to_string()));
        assert!(capability_names.contains(&&"Security Scanning".to_string()));
        assert!(capability_names.contains(&&"Auto-Scaling".to_string()));

        // Property 5: DevOps agent has DevOps technologies
        let devops_techs: Vec<_> = capabilities.iter().flat_map(|c| &c.technologies).collect();
        assert!(devops_techs.contains(&&"GitHub Actions".to_string()));
        assert!(devops_techs.contains(&&"GitLab CI".to_string()));
        assert!(devops_techs.contains(&&"Jenkins".to_string()));
        assert!(devops_techs.contains(&&"Terraform".to_string()));
        assert!(devops_techs.contains(&&"CloudFormation".to_string()));
        assert!(devops_techs.contains(&&"Ansible".to_string()));
        assert!(devops_techs.contains(&&"Docker".to_string()));
        assert!(devops_techs.contains(&&"Kubernetes".to_string()));
        assert!(devops_techs.contains(&&"Prometheus".to_string()));
        assert!(devops_techs.contains(&&"Grafana".to_string()));

        // Property 6: DevOps agent has technology recommendations
        let tech_recs = &devops_agent.knowledge.technology_recommendations;
        assert!(!tech_recs.is_empty());

        let tech_names: Vec<_> = tech_recs.iter().map(|t| &t.technology).collect();
        assert!(tech_names.contains(&&"GitHub Actions".to_string()));
        assert!(tech_names.contains(&&"Terraform".to_string()));
        assert!(tech_names.contains(&&"Docker".to_string()));
        assert!(tech_names.contains(&&"Kubernetes".to_string()));
    }

    /// Property 3: DevOps Agent Domain Routing (Multiple Registrations)
    /// For any DevOps request, the DevOps agent SHALL remain routable
    /// even after multiple registrations
    #[test]
    fn property_devops_agent_domain_routing_multiple_registrations() {
        let registry = DomainRegistry::new();
        let factory = AgentFactory::new();

        let devops_config = create_agent_config(
            "devops",
            "DevOps Agent",
            vec![("CI/CD Pipeline Setup", vec!["GitHub Actions", "Jenkins"])],
            vec![("GitHub Actions", vec!["CI/CD"])],
        );

        // Register DevOps agent multiple times
        for _ in 0..3 {
            let devops_agent = factory.create_agent("devops", &devops_config).unwrap();
            registry.register_agent("devops", devops_agent).unwrap();
        }

        // Property: DevOps agent should still be routable
        assert!(registry.has_domain("devops").unwrap());
        let agent = registry.get_agent("devops").unwrap();
        assert_eq!(agent.domain, "devops");
    }

    /// Property: All Three Agents Can Be Registered and Routed
    /// For any combination of web, backend, and DevOps agents, all three SHALL be
    /// registered and routable independently
    #[test]
    fn property_all_agents_routing_independence() {
        let registry = DomainRegistry::new();
        let factory = AgentFactory::new();

        // Create all three agents
        let web_config = create_agent_config(
            "web",
            "Web Agent",
            vec![("Frontend", vec!["React"])],
            vec![("React", vec!["SPAs"])],
        );

        let backend_config = create_agent_config(
            "backend",
            "Backend Agent",
            vec![("API Design", vec!["REST"])],
            vec![("REST", vec!["APIs"])],
        );

        let devops_config = create_agent_config(
            "devops",
            "DevOps Agent",
            vec![("CI/CD", vec!["GitHub Actions"])],
            vec![("GitHub Actions", vec!["CI/CD"])],
        );

        // Create and register all agents
        let web_agent = factory.create_agent("web", &web_config).unwrap();
        let backend_agent = factory.create_agent("backend", &backend_config).unwrap();
        let devops_agent = factory.create_agent("devops", &devops_config).unwrap();

        registry.register_agent("web", web_agent).unwrap();
        registry.register_agent("backend", backend_agent).unwrap();
        registry.register_agent("devops", devops_agent).unwrap();

        // Property 1: All three domains are registered
        assert!(registry.has_domain("web").unwrap());
        assert!(registry.has_domain("backend").unwrap());
        assert!(registry.has_domain("devops").unwrap());

        // Property 2: All three domains are discoverable
        let domains = registry.discover_domains().unwrap();
        assert_eq!(domains.len(), 3);
        assert!(domains.contains(&"web".to_string()));
        assert!(domains.contains(&"backend".to_string()));
        assert!(domains.contains(&"devops".to_string()));

        // Property 3: Each agent can be retrieved independently
        let web_agent = registry.get_agent("web").unwrap();
        let backend_agent = registry.get_agent("backend").unwrap();
        let devops_agent = registry.get_agent("devops").unwrap();

        assert_eq!(web_agent.domain, "web");
        assert_eq!(backend_agent.domain, "backend");
        assert_eq!(devops_agent.domain, "devops");

        // Property 4: Each agent has correct ID
        assert_eq!(web_agent.id, "web-agent");
        assert_eq!(backend_agent.id, "backend-agent");
        assert_eq!(devops_agent.id, "devops-agent");

        // Property 5: Each agent has correct capabilities
        let web_caps = registry.list_capabilities("web").unwrap();
        let backend_caps = registry.list_capabilities("backend").unwrap();
        let devops_caps = registry.list_capabilities("devops").unwrap();

        assert_eq!(web_caps.len(), 1);
        assert_eq!(backend_caps.len(), 1);
        assert_eq!(devops_caps.len(), 1);

        // Property 6: Capabilities are domain-specific
        assert_eq!(web_caps[0].name, "Frontend");
        assert_eq!(backend_caps[0].name, "API Design");
        assert_eq!(devops_caps[0].name, "CI/CD");
    }

    /// Property: Domain Count Accuracy
    /// For any number of registered agents, the domain count SHALL be accurate
    #[test]
    fn property_domain_count_accuracy() {
        let registry = DomainRegistry::new();
        let factory = AgentFactory::new();

        // Create test configurations
        let domains = vec!["web", "backend", "devops"];

        // Register agents one by one and verify count
        for (idx, domain) in domains.iter().enumerate() {
            let config = create_agent_config(
                domain,
                &format!("{} Agent", domain),
                vec![("Capability", vec!["Tech"])],
                vec![("Tech", vec!["Use case"])],
            );

            let agent = factory.create_agent(domain, &config).unwrap();
            registry.register_agent(domain, agent).unwrap();

            // Property: Count should match number of registered agents
            assert_eq!(
                registry.domain_count().unwrap(),
                idx + 1,
                "Domain count should be {} after registering {} agents",
                idx + 1,
                idx + 1
            );
        }

        // Property: Final count should be 3
        assert_eq!(registry.domain_count().unwrap(), 3);
    }
}
