//! Property-based tests for domain knowledge relevance
//!
//! **Feature: ricecoder-domain-agents, Property 4: Knowledge Application Relevance**
//! **Validates: Requirements 1.2-1.6, 2.2-2.6, 3.2-3.6**

#[cfg(test)]
mod tests {
    use crate::domain::{KnowledgeBase, AgentFactory};
    use crate::domain::factory::{
        AgentConfig, CapabilityConfig, BestPracticeConfig, TechRecommendationConfig,
        PatternConfig, AntiPatternConfig,
    };

    /// Helper function to create a test agent configuration with knowledge
    fn create_agent_config_with_knowledge(
        domain: &str,
        name: &str,
        capabilities: Vec<(&str, Vec<&str>)>,
        tech_recommendations: Vec<(&str, Vec<&str>)>,
        best_practices: Vec<(&str, Vec<&str>)>,
        patterns: Vec<(&str, Vec<&str>)>,
        anti_patterns: Vec<(&str, &str)>,
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
            best_practices: best_practices
                .into_iter()
                .map(|(title, techs)| BestPracticeConfig {
                    title: title.to_string(),
                    description: format!("Best practice: {}", title),
                    technologies: techs.into_iter().map(|t| t.to_string()).collect(),
                    implementation: "Implementation guidance".to_string(),
                })
                .collect(),
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
            patterns: patterns
                .into_iter()
                .map(|(name, techs)| PatternConfig {
                    name: name.to_string(),
                    description: format!("Pattern: {}", name),
                    technologies: techs.into_iter().map(|t| t.to_string()).collect(),
                    use_cases: vec!["Use case 1".to_string()],
                })
                .collect(),
            anti_patterns: anti_patterns
                .into_iter()
                .map(|(name, reason)| AntiPatternConfig {
                    name: name.to_string(),
                    description: format!("Anti-pattern: {}", name),
                    why_avoid: reason.to_string(),
                    better_alternative: "Better approach".to_string(),
                })
                .collect(),
        }
    }

    /// Property 4: Knowledge Application Relevance - Web Domain
    /// For any web domain agent, all recommendations SHALL be relevant to web development
    /// and include appropriate web technologies (React, Vue, Angular, CSS, Tailwind, Vite, Webpack, Jest, Vitest, Playwright).
    ///
    /// This property tests that:
    /// 1. Web agent knowledge contains only web-relevant technologies
    /// 2. Best practices are applicable to web development
    /// 3. Technology recommendations include web technologies
    /// 4. Patterns are relevant to web development
    /// 5. Anti-patterns are relevant to web development
    #[test]
    fn property_knowledge_relevance_web_domain() {
        let factory = AgentFactory::new();
        let kb = KnowledgeBase::new();

        // Create web agent configuration with comprehensive knowledge
        let web_config = create_agent_config_with_knowledge(
            "web",
            "Web Development Agent",
            vec![
                ("Frontend Framework Selection", vec!["React", "Vue", "Angular"]),
                ("Styling Guidance", vec!["CSS", "Tailwind CSS", "styled-components"]),
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
            vec![
                ("Component-Based Architecture", vec!["React", "Vue"]),
                ("Performance Optimization", vec!["React", "Vue", "Angular"]),
            ],
            vec![
                ("MVC Pattern", vec!["React", "Vue"]),
                ("Singleton Pattern", vec!["JavaScript"]),
            ],
            vec![
                ("God Component", "Component that does too much"),
                ("Tight Coupling", "Components tightly coupled"),
            ],
        );

        // Create web agent
        let web_agent = factory.create_agent("web", &web_config).unwrap();

        // Load knowledge into knowledge base
        for practice in &web_agent.knowledge.best_practices {
            kb.add_best_practice("web", practice.clone()).unwrap();
        }

        for recommendation in &web_agent.knowledge.technology_recommendations {
            kb.add_tech_recommendation("web", recommendation.clone()).unwrap();
        }

        for pattern in &web_agent.knowledge.patterns {
            kb.add_pattern("web", pattern.clone()).unwrap();
        }

        for anti_pattern in &web_agent.knowledge.anti_patterns {
            kb.add_anti_pattern("web", anti_pattern.clone()).unwrap();
        }

        // Property 1: All best practices are for web domain
        let practices = kb.get_best_practices("web").unwrap();
        assert!(!practices.is_empty());
        for practice in &practices {
            assert_eq!(practice.domain, "web");
            // All technologies should be web-related
            for tech in &practice.technologies {
                assert!(
                    vec!["React", "Vue", "Angular", "CSS", "Tailwind CSS", "styled-components"]
                        .contains(&tech.as_str()),
                    "Technology {} is not web-related",
                    tech
                );
            }
        }

        // Property 2: All technology recommendations are for web domain
        let tech_recs = kb.get_tech_recommendations("web").unwrap();
        assert!(!tech_recs.is_empty());
        for rec in &tech_recs {
            assert_eq!(rec.domain, "web");
            // All technologies should be web-related
            assert!(
                vec!["React", "Vue", "Angular", "Vite", "Webpack", "Jest", "Vitest", "Playwright"]
                    .contains(&rec.technology.as_str()),
                "Technology {} is not web-related",
                rec.technology
            );
            // All recommendations should have use cases
            assert!(!rec.use_cases.is_empty());
            // All recommendations should have pros and cons
            assert!(!rec.pros.is_empty());
            assert!(!rec.cons.is_empty());
        }

        // Property 3: All patterns are for web domain
        let patterns = kb.get_patterns("web").unwrap();
        assert!(!patterns.is_empty());
        for pattern in &patterns {
            assert_eq!(pattern.domain, "web");
            // All technologies should be web-related
            for tech in &pattern.technologies {
                assert!(
                    vec!["React", "Vue", "Angular", "JavaScript"]
                        .contains(&tech.as_str()),
                    "Technology {} is not web-related",
                    tech
                );
            }
        }

        // Property 4: All anti-patterns are for web domain
        let anti_patterns = kb.get_anti_patterns("web").unwrap();
        assert!(!anti_patterns.is_empty());
        for anti_pattern in &anti_patterns {
            assert_eq!(anti_pattern.domain, "web");
            // Anti-patterns should have rationale
            assert!(!anti_pattern.why_avoid.is_empty());
            // Anti-patterns should have alternatives
            assert!(!anti_pattern.better_alternative.is_empty());
        }

        // Property 5: Web technologies are present in recommendations
        let tech_names: Vec<_> = tech_recs.iter().map(|t| &t.technology).collect();
        assert!(tech_names.contains(&&"React".to_string()));
        assert!(tech_names.contains(&&"Vue".to_string()));
        assert!(tech_names.contains(&&"Vite".to_string()));
        assert!(tech_names.contains(&&"Jest".to_string()));
    }

    /// Property 4: Knowledge Application Relevance - Backend Domain
    /// For any backend domain agent, all recommendations SHALL be relevant to backend development
    /// and include appropriate backend technologies (REST, GraphQL, gRPC, PostgreSQL, MongoDB, Redis, OAuth, JWT).
    ///
    /// This property tests that:
    /// 1. Backend agent knowledge contains only backend-relevant technologies
    /// 2. Best practices are applicable to backend development
    /// 3. Technology recommendations include backend technologies
    /// 4. Patterns are relevant to backend development
    /// 5. Anti-patterns are relevant to backend development
    #[test]
    fn property_knowledge_relevance_backend_domain() {
        let factory = AgentFactory::new();
        let kb = KnowledgeBase::new();

        // Create backend agent configuration with comprehensive knowledge
        let backend_config = create_agent_config_with_knowledge(
            "backend",
            "Backend Development Agent",
            vec![
                ("API Design", vec!["REST", "GraphQL", "gRPC"]),
                ("Architecture Guidance", vec!["Microservices", "Monolithic", "Serverless"]),
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
            vec![
                ("API Versioning", vec!["REST", "GraphQL"]),
                ("Database Indexing", vec!["PostgreSQL", "MongoDB"]),
            ],
            vec![
                ("MVC Pattern", vec!["Django", "Rails"]),
                ("Repository Pattern", vec!["Java", "C#"]),
            ],
            vec![
                ("God Object", "Class that does too much"),
                ("Tight Coupling", "Services tightly coupled"),
            ],
        );

        // Create backend agent
        let backend_agent = factory.create_agent("backend", &backend_config).unwrap();

        // Load knowledge into knowledge base
        for practice in &backend_agent.knowledge.best_practices {
            kb.add_best_practice("backend", practice.clone()).unwrap();
        }

        for recommendation in &backend_agent.knowledge.technology_recommendations {
            kb.add_tech_recommendation("backend", recommendation.clone()).unwrap();
        }

        for pattern in &backend_agent.knowledge.patterns {
            kb.add_pattern("backend", pattern.clone()).unwrap();
        }

        for anti_pattern in &backend_agent.knowledge.anti_patterns {
            kb.add_anti_pattern("backend", anti_pattern.clone()).unwrap();
        }

        // Property 1: All best practices are for backend domain
        let practices = kb.get_best_practices("backend").unwrap();
        assert!(!practices.is_empty());
        for practice in &practices {
            assert_eq!(practice.domain, "backend");
            // All technologies should be backend-related
            for tech in &practice.technologies {
                assert!(
                    vec!["REST", "GraphQL", "PostgreSQL", "MongoDB"]
                        .contains(&tech.as_str()),
                    "Technology {} is not backend-related",
                    tech
                );
            }
        }

        // Property 2: All technology recommendations are for backend domain
        let tech_recs = kb.get_tech_recommendations("backend").unwrap();
        assert!(!tech_recs.is_empty());
        for rec in &tech_recs {
            assert_eq!(rec.domain, "backend");
            // All technologies should be backend-related
            assert!(
                vec!["REST", "GraphQL", "gRPC", "PostgreSQL", "MongoDB", "Redis", "OAuth 2.0", "JWT"]
                    .contains(&rec.technology.as_str()),
                "Technology {} is not backend-related",
                rec.technology
            );
            // All recommendations should have use cases
            assert!(!rec.use_cases.is_empty());
            // All recommendations should have pros and cons
            assert!(!rec.pros.is_empty());
            assert!(!rec.cons.is_empty());
        }

        // Property 3: All patterns are for backend domain
        let patterns = kb.get_patterns("backend").unwrap();
        assert!(!patterns.is_empty());
        for pattern in &patterns {
            assert_eq!(pattern.domain, "backend");
            // All technologies should be backend-related
            for tech in &pattern.technologies {
                assert!(
                    vec!["Django", "Rails", "Java", "C#"]
                        .contains(&tech.as_str()),
                    "Technology {} is not backend-related",
                    tech
                );
            }
        }

        // Property 4: All anti-patterns are for backend domain
        let anti_patterns = kb.get_anti_patterns("backend").unwrap();
        assert!(!anti_patterns.is_empty());
        for anti_pattern in &anti_patterns {
            assert_eq!(anti_pattern.domain, "backend");
            // Anti-patterns should have rationale
            assert!(!anti_pattern.why_avoid.is_empty());
            // Anti-patterns should have alternatives
            assert!(!anti_pattern.better_alternative.is_empty());
        }

        // Property 5: Backend technologies are present in recommendations
        let tech_names: Vec<_> = tech_recs.iter().map(|t| &t.technology).collect();
        assert!(tech_names.contains(&&"REST".to_string()));
        assert!(tech_names.contains(&&"GraphQL".to_string()));
        assert!(tech_names.contains(&&"PostgreSQL".to_string()));
        assert!(tech_names.contains(&&"MongoDB".to_string()));
    }

    /// Property 4: Knowledge Application Relevance - DevOps Domain
    /// For any DevOps domain agent, all recommendations SHALL be relevant to DevOps
    /// and include appropriate DevOps technologies (GitHub Actions, GitLab CI, Jenkins, Terraform,
    /// CloudFormation, Ansible, Docker, Kubernetes, Prometheus, Grafana).
    ///
    /// This property tests that:
    /// 1. DevOps agent knowledge contains only DevOps-relevant technologies
    /// 2. Best practices are applicable to DevOps
    /// 3. Technology recommendations include DevOps technologies
    /// 4. Patterns are relevant to DevOps
    /// 5. Anti-patterns are relevant to DevOps
    #[test]
    fn property_knowledge_relevance_devops_domain() {
        let factory = AgentFactory::new();
        let kb = KnowledgeBase::new();

        // Create DevOps agent configuration with comprehensive knowledge
        let devops_config = create_agent_config_with_knowledge(
            "devops",
            "DevOps Agent",
            vec![
                ("CI/CD Pipeline Setup", vec!["GitHub Actions", "GitLab CI", "Jenkins"]),
                ("Infrastructure as Code", vec!["Terraform", "CloudFormation", "Ansible"]),
                ("Containerization", vec!["Docker", "Kubernetes"]),
                ("Observability Infrastructure", vec!["Prometheus", "Grafana", "ELK Stack"]),
                ("Security Scanning", vec!["Trivy", "Snyk", "SonarQube"]),
                ("Auto-Scaling", vec!["Kubernetes", "AWS Auto Scaling", "Docker Swarm"]),
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
            vec![
                ("Infrastructure as Code", vec!["Terraform", "CloudFormation"]),
                ("Monitoring and Alerting", vec!["Prometheus", "Grafana"]),
            ],
            vec![
                ("Blue-Green Deployment", vec!["Kubernetes", "Docker"]),
                ("Canary Deployment", vec!["Kubernetes"]),
            ],
            vec![
                ("Manual Infrastructure", "Infrastructure managed manually"),
                ("No Monitoring", "No monitoring or alerting"),
            ],
        );

        // Create DevOps agent
        let devops_agent = factory.create_agent("devops", &devops_config).unwrap();

        // Load knowledge into knowledge base
        for practice in &devops_agent.knowledge.best_practices {
            kb.add_best_practice("devops", practice.clone()).unwrap();
        }

        for recommendation in &devops_agent.knowledge.technology_recommendations {
            kb.add_tech_recommendation("devops", recommendation.clone()).unwrap();
        }

        for pattern in &devops_agent.knowledge.patterns {
            kb.add_pattern("devops", pattern.clone()).unwrap();
        }

        for anti_pattern in &devops_agent.knowledge.anti_patterns {
            kb.add_anti_pattern("devops", anti_pattern.clone()).unwrap();
        }

        // Property 1: All best practices are for DevOps domain
        let practices = kb.get_best_practices("devops").unwrap();
        assert!(!practices.is_empty());
        for practice in &practices {
            assert_eq!(practice.domain, "devops");
            // All technologies should be DevOps-related
            for tech in &practice.technologies {
                assert!(
                    vec!["Terraform", "CloudFormation", "Prometheus", "Grafana"]
                        .contains(&tech.as_str()),
                    "Technology {} is not DevOps-related",
                    tech
                );
            }
        }

        // Property 2: All technology recommendations are for DevOps domain
        let tech_recs = kb.get_tech_recommendations("devops").unwrap();
        assert!(!tech_recs.is_empty());
        for rec in &tech_recs {
            assert_eq!(rec.domain, "devops");
            // All technologies should be DevOps-related
            assert!(
                vec!["GitHub Actions", "GitLab CI", "Jenkins", "Terraform", "CloudFormation", "Ansible", "Docker", "Kubernetes", "Prometheus", "Grafana"]
                    .contains(&rec.technology.as_str()),
                "Technology {} is not DevOps-related",
                rec.technology
            );
            // All recommendations should have use cases
            assert!(!rec.use_cases.is_empty());
            // All recommendations should have pros and cons
            assert!(!rec.pros.is_empty());
            assert!(!rec.cons.is_empty());
        }

        // Property 3: All patterns are for DevOps domain
        let patterns = kb.get_patterns("devops").unwrap();
        assert!(!patterns.is_empty());
        for pattern in &patterns {
            assert_eq!(pattern.domain, "devops");
            // All technologies should be DevOps-related
            for tech in &pattern.technologies {
                assert!(
                    vec!["Kubernetes", "Docker"]
                        .contains(&tech.as_str()),
                    "Technology {} is not DevOps-related",
                    tech
                );
            }
        }

        // Property 4: All anti-patterns are for DevOps domain
        let anti_patterns = kb.get_anti_patterns("devops").unwrap();
        assert!(!anti_patterns.is_empty());
        for anti_pattern in &anti_patterns {
            assert_eq!(anti_pattern.domain, "devops");
            // Anti-patterns should have rationale
            assert!(!anti_pattern.why_avoid.is_empty());
            // Anti-patterns should have alternatives
            assert!(!anti_pattern.better_alternative.is_empty());
        }

        // Property 5: DevOps technologies are present in recommendations
        let tech_names: Vec<_> = tech_recs.iter().map(|t| &t.technology).collect();
        assert!(tech_names.contains(&&"GitHub Actions".to_string()));
        assert!(tech_names.contains(&&"Terraform".to_string()));
        assert!(tech_names.contains(&&"Docker".to_string()));
        assert!(tech_names.contains(&&"Kubernetes".to_string()));
    }

    /// Property 4: Knowledge Retrieval Accuracy
    /// For any domain with loaded knowledge, retrieving knowledge by domain SHALL return
    /// only knowledge for that domain
    #[test]
    fn property_knowledge_retrieval_accuracy() {
        let factory = AgentFactory::new();
        let kb = KnowledgeBase::new();

        // Create and load knowledge for multiple domains
        let domains = vec!["web", "backend", "devops"];

        for domain in &domains {
            let config = create_agent_config_with_knowledge(
                domain,
                &format!("{} Agent", domain),
                vec![("Capability", vec!["Tech"])],
                vec![("Tech", vec!["Use case"])],
                vec![("Practice", vec!["Tech"])],
                vec![("Pattern", vec!["Tech"])],
                vec![("Anti-pattern", "Reason")],
            );

            let agent = factory.create_agent(domain, &config).unwrap();

            for practice in &agent.knowledge.best_practices {
                kb.add_best_practice(domain, practice.clone()).unwrap();
            }

            for recommendation in &agent.knowledge.technology_recommendations {
                kb.add_tech_recommendation(domain, recommendation.clone()).unwrap();
            }

            for pattern in &agent.knowledge.patterns {
                kb.add_pattern(domain, pattern.clone()).unwrap();
            }

            for anti_pattern in &agent.knowledge.anti_patterns {
                kb.add_anti_pattern(domain, anti_pattern.clone()).unwrap();
            }
        }

        // Property: Each domain should only return its own knowledge
        for domain in &domains {
            let practices = kb.get_best_practices(domain).unwrap();
            for practice in &practices {
                assert_eq!(practice.domain, *domain);
            }

            let tech_recs = kb.get_tech_recommendations(domain).unwrap();
            for rec in &tech_recs {
                assert_eq!(rec.domain, *domain);
            }

            let patterns = kb.get_patterns(domain).unwrap();
            for pattern in &patterns {
                assert_eq!(pattern.domain, *domain);
            }

            let anti_patterns = kb.get_anti_patterns(domain).unwrap();
            for anti_pattern in &anti_patterns {
                assert_eq!(anti_pattern.domain, *domain);
            }
        }
    }

    /// Property 4: Technology Recommendation Completeness
    /// For any technology recommendation, it SHALL include use cases, pros, and cons
    #[test]
    fn property_technology_recommendation_completeness() {
        let factory = AgentFactory::new();
        let kb = KnowledgeBase::new();

        // Create agent with technology recommendations
        let config = create_agent_config_with_knowledge(
            "web",
            "Web Agent",
            vec![("Framework", vec!["React", "Vue"])],
            vec![
                ("React", vec!["SPAs", "Complex UIs"]),
                ("Vue", vec!["Progressive enhancement"]),
            ],
            vec![],
            vec![],
            vec![],
        );

        let agent = factory.create_agent("web", &config).unwrap();

        for recommendation in &agent.knowledge.technology_recommendations {
            kb.add_tech_recommendation("web", recommendation.clone()).unwrap();
        }

        // Property: All recommendations should be complete
        let tech_recs = kb.get_tech_recommendations("web").unwrap();
        for rec in &tech_recs {
            // Must have use cases
            assert!(!rec.use_cases.is_empty(), "Technology {} has no use cases", rec.technology);
            // Must have pros
            assert!(!rec.pros.is_empty(), "Technology {} has no pros", rec.technology);
            // Must have cons
            assert!(!rec.cons.is_empty(), "Technology {} has no cons", rec.technology);
        }
    }

    /// Property 4: Best Practice Applicability
    /// For any best practice, it SHALL be applicable to at least one technology
    #[test]
    fn property_best_practice_applicability() {
        let factory = AgentFactory::new();
        let kb = KnowledgeBase::new();

        // Create agent with best practices
        let config = create_agent_config_with_knowledge(
            "web",
            "Web Agent",
            vec![("Framework", vec!["React"])],
            vec![],
            vec![
                ("Component-Based Architecture", vec!["React", "Vue"]),
                ("Performance Optimization", vec!["React"]),
            ],
            vec![],
            vec![],
        );

        let agent = factory.create_agent("web", &config).unwrap();

        for practice in &agent.knowledge.best_practices {
            kb.add_best_practice("web", practice.clone()).unwrap();
        }

        // Property: All best practices should be applicable to at least one technology
        let practices = kb.get_best_practices("web").unwrap();
        for practice in &practices {
            assert!(
                !practice.technologies.is_empty(),
                "Best practice {} has no applicable technologies",
                practice.title
            );
        }
    }

    /// Property 4: Anti-Pattern Guidance
    /// For any anti-pattern, it SHALL include why to avoid it and a better alternative
    #[test]
    fn property_anti_pattern_guidance() {
        let factory = AgentFactory::new();
        let kb = KnowledgeBase::new();

        // Create agent with anti-patterns
        let config = create_agent_config_with_knowledge(
            "web",
            "Web Agent",
            vec![("Framework", vec!["React"])],
            vec![],
            vec![],
            vec![],
            vec![
                ("God Component", "Component does too much"),
                ("Tight Coupling", "Components are tightly coupled"),
            ],
        );

        let agent = factory.create_agent("web", &config).unwrap();

        for anti_pattern in &agent.knowledge.anti_patterns {
            kb.add_anti_pattern("web", anti_pattern.clone()).unwrap();
        }

        // Property: All anti-patterns should have guidance
        let anti_patterns = kb.get_anti_patterns("web").unwrap();
        for anti_pattern in &anti_patterns {
            assert!(
                !anti_pattern.why_avoid.is_empty(),
                "Anti-pattern {} has no guidance on why to avoid it",
                anti_pattern.name
            );
            assert!(
                !anti_pattern.better_alternative.is_empty(),
                "Anti-pattern {} has no better alternative",
                anti_pattern.name
            );
        }
    }
}
