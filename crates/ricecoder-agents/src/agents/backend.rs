//! Backend Development Agent
//!
//! This module provides a specialized agent for backend development tasks,
//! including API design pattern recommendations, architecture guidance, database
//! design recommendations, scalability guidance, security pattern recommendations,
//! and observability setup guidance.

use crate::agents::Agent;
use crate::domain::{DomainAgent, DomainCapability, DomainKnowledge, TechRecommendation};
use crate::error::Result;
use crate::models::{AgentInput, AgentOutput, Finding, Severity, TaskType};
use async_trait::async_trait;
use uuid::Uuid;

/// Backend Development Agent
///
/// A specialized agent for backend development that provides recommendations for:
/// - API design patterns (REST, GraphQL, gRPC)
/// - Architecture guidance (microservices, monolithic, serverless)
/// - Database design (relational, NoSQL, graph)
/// - Scalability guidance (caching, load balancing, horizontal scaling)
/// - Security patterns (auth, authorization, data protection)
/// - Observability setup (logging, metrics, tracing)
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::agents::BackendAgent;
///
/// let agent = BackendAgent::new();
/// assert_eq!(agent.id(), "backend-agent");
/// assert_eq!(agent.domain(), "backend");
/// ```
#[derive(Debug, Clone)]
pub struct BackendAgent {
    domain_agent: DomainAgent,
}

impl BackendAgent {
    /// Create a new Backend Development Agent
    ///
    /// Initializes the agent with backend-specific capabilities and knowledge.
    ///
    /// # Returns
    ///
    /// A new `BackendAgent` instance
    pub fn new() -> Self {
        let capabilities = vec![
            DomainCapability {
                name: "API Design".to_string(),
                description: "Recommendations for REST, GraphQL, or gRPC API design patterns"
                    .to_string(),
                technologies: vec!["REST".to_string(), "GraphQL".to_string(), "gRPC".to_string()],
                patterns: vec![],
            },
            DomainCapability {
                name: "Architecture Guidance".to_string(),
                description:
                    "Recommendations for microservices, monolithic, or serverless architectures"
                        .to_string(),
                technologies: vec![
                    "Microservices".to_string(),
                    "Monolithic".to_string(),
                    "Serverless".to_string(),
                ],
                patterns: vec![],
            },
            DomainCapability {
                name: "Database Design".to_string(),
                description:
                    "Recommendations for relational, NoSQL, or graph database selection and schema design"
                        .to_string(),
                technologies: vec![
                    "PostgreSQL".to_string(),
                    "MongoDB".to_string(),
                    "Neo4j".to_string(),
                ],
                patterns: vec![],
            },
            DomainCapability {
                name: "Scalability".to_string(),
                description:
                    "Recommendations for caching, load balancing, and horizontal scaling strategies"
                        .to_string(),
                technologies: vec![
                    "Redis".to_string(),
                    "Memcached".to_string(),
                    "Load Balancers".to_string(),
                ],
                patterns: vec![],
            },
            DomainCapability {
                name: "Security".to_string(),
                description:
                    "Recommendations for authentication, authorization, and data protection patterns"
                        .to_string(),
                technologies: vec![
                    "OAuth 2.0".to_string(),
                    "JWT".to_string(),
                    "TLS".to_string(),
                ],
                patterns: vec![],
            },
            DomainCapability {
                name: "Observability".to_string(),
                description: "Recommendations for logging, metrics, and tracing implementation"
                    .to_string(),
                technologies: vec![
                    "ELK Stack".to_string(),
                    "Prometheus".to_string(),
                    "Jaeger".to_string(),
                ],
                patterns: vec![],
            },
        ];

        let knowledge = DomainKnowledge {
            best_practices: vec![],
            technology_recommendations: vec![
                TechRecommendation {
                    technology: "REST".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Web APIs".to_string(),
                        "Mobile backends".to_string(),
                        "Simple CRUD operations".to_string(),
                    ],
                    pros: vec![
                        "Simple and well-understood".to_string(),
                        "Stateless design".to_string(),
                        "Excellent caching support".to_string(),
                    ],
                    cons: vec![
                        "Over-fetching and under-fetching".to_string(),
                        "Versioning complexity".to_string(),
                        "Multiple requests for related data".to_string(),
                    ],
                    alternatives: vec!["GraphQL".to_string(), "gRPC".to_string()],
                },
                TechRecommendation {
                    technology: "GraphQL".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Complex data queries".to_string(),
                        "Mobile-first applications".to_string(),
                        "Real-time data".to_string(),
                    ],
                    pros: vec![
                        "Precise data fetching".to_string(),
                        "Single endpoint".to_string(),
                        "Strong typing".to_string(),
                    ],
                    cons: vec![
                        "Steep learning curve".to_string(),
                        "Query complexity".to_string(),
                        "Caching challenges".to_string(),
                    ],
                    alternatives: vec!["REST".to_string(), "gRPC".to_string()],
                },
                TechRecommendation {
                    technology: "gRPC".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Microservices communication".to_string(),
                        "High-performance systems".to_string(),
                        "Real-time streaming".to_string(),
                    ],
                    pros: vec![
                        "High performance".to_string(),
                        "Streaming support".to_string(),
                        "Language-agnostic".to_string(),
                    ],
                    cons: vec![
                        "Steep learning curve".to_string(),
                        "Browser support limitations".to_string(),
                        "Debugging complexity".to_string(),
                    ],
                    alternatives: vec!["REST".to_string(), "GraphQL".to_string()],
                },
                TechRecommendation {
                    technology: "PostgreSQL".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Relational data".to_string(),
                        "ACID transactions".to_string(),
                        "Complex queries".to_string(),
                    ],
                    pros: vec![
                        "Reliable".to_string(),
                        "Feature-rich".to_string(),
                        "Open source".to_string(),
                    ],
                    cons: vec![
                        "Vertical scaling limitations".to_string(),
                        "Complex setup".to_string(),
                    ],
                    alternatives: vec!["MySQL".to_string(), "MariaDB".to_string()],
                },
                TechRecommendation {
                    technology: "MongoDB".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Document storage".to_string(),
                        "Flexible schema".to_string(),
                        "Rapid development".to_string(),
                    ],
                    pros: vec![
                        "Flexible schema".to_string(),
                        "Horizontal scaling".to_string(),
                        "Easy to use".to_string(),
                    ],
                    cons: vec![
                        "No ACID transactions".to_string(),
                        "Higher memory usage".to_string(),
                    ],
                    alternatives: vec!["CouchDB".to_string(), "Firebase".to_string()],
                },
                TechRecommendation {
                    technology: "Neo4j".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Graph data".to_string(),
                        "Relationship queries".to_string(),
                        "Recommendation engines".to_string(),
                    ],
                    pros: vec![
                        "Powerful graph queries".to_string(),
                        "Relationship performance".to_string(),
                        "Intuitive modeling".to_string(),
                    ],
                    cons: vec![
                        "Specialized use case".to_string(),
                        "Smaller ecosystem".to_string(),
                    ],
                    alternatives: vec!["ArangoDB".to_string(), "TigerGraph".to_string()],
                },
                TechRecommendation {
                    technology: "Redis".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Caching".to_string(),
                        "Session storage".to_string(),
                        "Real-time analytics".to_string(),
                    ],
                    pros: vec![
                        "Lightning fast".to_string(),
                        "Versatile data structures".to_string(),
                        "Pub/Sub support".to_string(),
                    ],
                    cons: vec![
                        "In-memory only".to_string(),
                        "Data persistence complexity".to_string(),
                    ],
                    alternatives: vec!["Memcached".to_string(), "Hazelcast".to_string()],
                },
                TechRecommendation {
                    technology: "Memcached".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Simple caching".to_string(),
                        "Session storage".to_string(),
                        "Object caching".to_string(),
                    ],
                    pros: vec![
                        "Simple and lightweight".to_string(),
                        "Fast".to_string(),
                        "Distributed".to_string(),
                    ],
                    cons: vec![
                        "Limited data structures".to_string(),
                        "No persistence".to_string(),
                    ],
                    alternatives: vec!["Redis".to_string(), "Hazelcast".to_string()],
                },
                TechRecommendation {
                    technology: "Microservices".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Large applications".to_string(),
                        "Independent scaling".to_string(),
                        "Team autonomy".to_string(),
                    ],
                    pros: vec![
                        "Independent scaling".to_string(),
                        "Technology flexibility".to_string(),
                        "Team autonomy".to_string(),
                    ],
                    cons: vec![
                        "Operational complexity".to_string(),
                        "Network latency".to_string(),
                        "Data consistency challenges".to_string(),
                    ],
                    alternatives: vec!["Monolithic".to_string(), "Serverless".to_string()],
                },
                TechRecommendation {
                    technology: "Monolithic".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Small to medium applications".to_string(),
                        "Simple deployments".to_string(),
                        "Rapid development".to_string(),
                    ],
                    pros: vec![
                        "Simple deployment".to_string(),
                        "Easier debugging".to_string(),
                        "Better performance".to_string(),
                    ],
                    cons: vec![
                        "Scaling limitations".to_string(),
                        "Technology lock-in".to_string(),
                        "Deployment coupling".to_string(),
                    ],
                    alternatives: vec!["Microservices".to_string(), "Serverless".to_string()],
                },
                TechRecommendation {
                    technology: "Serverless".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Event-driven workloads".to_string(),
                        "Sporadic traffic".to_string(),
                        "Rapid prototyping".to_string(),
                    ],
                    pros: vec![
                        "No infrastructure management".to_string(),
                        "Auto-scaling".to_string(),
                        "Pay-per-use".to_string(),
                    ],
                    cons: vec![
                        "Cold start latency".to_string(),
                        "Vendor lock-in".to_string(),
                        "Debugging complexity".to_string(),
                    ],
                    alternatives: vec!["Microservices".to_string(), "Monolithic".to_string()],
                },
                TechRecommendation {
                    technology: "OAuth 2.0".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Third-party authentication".to_string(),
                        "Delegated access".to_string(),
                        "Social login".to_string(),
                    ],
                    pros: vec![
                        "Industry standard".to_string(),
                        "Secure delegation".to_string(),
                        "Wide adoption".to_string(),
                    ],
                    cons: vec![
                        "Complex implementation".to_string(),
                        "Token management".to_string(),
                    ],
                    alternatives: vec!["OpenID Connect".to_string(), "SAML".to_string()],
                },
                TechRecommendation {
                    technology: "JWT".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Stateless authentication".to_string(),
                        "API authentication".to_string(),
                        "Token-based auth".to_string(),
                    ],
                    pros: vec![
                        "Stateless".to_string(),
                        "Self-contained".to_string(),
                        "Cross-domain support".to_string(),
                    ],
                    cons: vec![
                        "Token revocation challenges".to_string(),
                        "Token size".to_string(),
                    ],
                    alternatives: vec!["Session tokens".to_string(), "OAuth 2.0".to_string()],
                },
                TechRecommendation {
                    technology: "TLS".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Secure communication".to_string(),
                        "Data encryption".to_string(),
                        "HTTPS".to_string(),
                    ],
                    pros: vec![
                        "Industry standard".to_string(),
                        "Wide support".to_string(),
                        "Proven security".to_string(),
                    ],
                    cons: vec![
                        "Performance overhead".to_string(),
                        "Certificate management".to_string(),
                    ],
                    alternatives: vec!["SSL".to_string()],
                },
                TechRecommendation {
                    technology: "ELK Stack".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Log aggregation".to_string(),
                        "Log analysis".to_string(),
                        "Centralized logging".to_string(),
                    ],
                    pros: vec![
                        "Powerful search".to_string(),
                        "Real-time analysis".to_string(),
                        "Open source".to_string(),
                    ],
                    cons: vec![
                        "Resource intensive".to_string(),
                        "Complex setup".to_string(),
                    ],
                    alternatives: vec!["Splunk".to_string(), "Datadog".to_string()],
                },
                TechRecommendation {
                    technology: "Prometheus".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Metrics collection".to_string(),
                        "System monitoring".to_string(),
                        "Alerting".to_string(),
                    ],
                    pros: vec![
                        "Time-series database".to_string(),
                        "Powerful queries".to_string(),
                        "Open source".to_string(),
                    ],
                    cons: vec![
                        "Pull-based model".to_string(),
                        "Limited long-term storage".to_string(),
                    ],
                    alternatives: vec!["Grafana".to_string(), "InfluxDB".to_string()],
                },
                TechRecommendation {
                    technology: "Jaeger".to_string(),
                    domain: "backend".to_string(),
                    use_cases: vec![
                        "Distributed tracing".to_string(),
                        "Request tracing".to_string(),
                        "Performance analysis".to_string(),
                    ],
                    pros: vec![
                        "Distributed tracing".to_string(),
                        "OpenTelemetry support".to_string(),
                        "Open source".to_string(),
                    ],
                    cons: vec![
                        "Complex setup".to_string(),
                        "Storage requirements".to_string(),
                    ],
                    alternatives: vec!["Zipkin".to_string(), "Datadog".to_string()],
                },
            ],
            patterns: vec![],
            anti_patterns: vec![],
        };

        let domain_agent = DomainAgent {
            id: "backend-agent".to_string(),
            domain: "backend".to_string(),
            capabilities,
            knowledge,
        };

        BackendAgent { domain_agent }
    }

    /// Get the domain of this agent
    pub fn domain(&self) -> &str {
        &self.domain_agent.domain
    }

    /// Get the capabilities of this agent
    pub fn capabilities(&self) -> &[DomainCapability] {
        &self.domain_agent.capabilities
    }

    /// Get the knowledge of this agent
    pub fn knowledge(&self) -> &DomainKnowledge {
        &self.domain_agent.knowledge
    }

    /// Get the underlying domain agent
    pub fn domain_agent(&self) -> &DomainAgent {
        &self.domain_agent
    }
}

impl Default for BackendAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for BackendAgent {
    fn id(&self) -> &str {
        &self.domain_agent.id
    }

    fn name(&self) -> &str {
        "Backend Development Agent"
    }

    fn description(&self) -> &str {
        "Specialized agent for backend development with expertise in API design, architecture, database design, scalability, security, and observability"
    }

    fn supports(&self, task_type: TaskType) -> bool {
        // Backend agent supports code review and other backend-related tasks
        matches!(
            task_type,
            TaskType::CodeReview
                | TaskType::Refactoring
                | TaskType::Documentation
                | TaskType::SecurityAnalysis
        )
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // For now, return a basic output
        // This will be enhanced with actual backend-specific logic
        let mut output = AgentOutput::default();

        // Add findings based on the task type
        match input.task.task_type {
            TaskType::CodeReview => {
                output.findings.push(Finding {
                    id: Uuid::new_v4().to_string(),
                    severity: Severity::Info,
                    category: "backend-best-practices".to_string(),
                    message: "Backend development best practices applied".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
            _ => {}
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_agent_creation() {
        let agent = BackendAgent::new();
        assert_eq!(agent.id(), "backend-agent");
        assert_eq!(agent.domain(), "backend");
        assert_eq!(agent.name(), "Backend Development Agent");
    }

    #[test]
    fn test_backend_agent_capabilities() {
        let agent = BackendAgent::new();
        let capabilities = agent.capabilities();
        assert_eq!(capabilities.len(), 6);

        // Check first capability
        assert_eq!(capabilities[0].name, "API Design");
        assert_eq!(capabilities[0].technologies.len(), 3);
        assert!(capabilities[0]
            .technologies
            .contains(&"REST".to_string()));
        assert!(capabilities[0]
            .technologies
            .contains(&"GraphQL".to_string()));
        assert!(capabilities[0]
            .technologies
            .contains(&"gRPC".to_string()));
    }

    #[test]
    fn test_backend_agent_knowledge() {
        let agent = BackendAgent::new();
        let knowledge = agent.knowledge();
        assert!(!knowledge.technology_recommendations.is_empty());
        assert_eq!(knowledge.technology_recommendations.len(), 17);
    }

    #[test]
    fn test_backend_agent_technology_recommendations() {
        let agent = BackendAgent::new();
        let knowledge = agent.knowledge();

        // Check REST recommendation
        let rest_rec = knowledge
            .technology_recommendations
            .iter()
            .find(|r| r.technology == "REST")
            .expect("REST recommendation not found");

        assert_eq!(rest_rec.domain, "backend");
        assert!(!rest_rec.use_cases.is_empty());
        assert!(!rest_rec.pros.is_empty());
        assert!(!rest_rec.cons.is_empty());
        assert!(!rest_rec.alternatives.is_empty());
    }

    #[test]
    fn test_backend_agent_supports_task_types() {
        let agent = BackendAgent::new();
        assert!(agent.supports(TaskType::CodeReview));
        assert!(agent.supports(TaskType::Refactoring));
        assert!(agent.supports(TaskType::Documentation));
        assert!(agent.supports(TaskType::SecurityAnalysis));
        assert!(!agent.supports(TaskType::TestGeneration));
    }

    #[test]
    fn test_backend_agent_default() {
        let agent1 = BackendAgent::new();
        let agent2 = BackendAgent::default();
        assert_eq!(agent1.id(), agent2.id());
        assert_eq!(agent1.domain(), agent2.domain());
    }

    #[test]
    fn test_backend_agent_clone() {
        let agent1 = BackendAgent::new();
        let agent2 = agent1.clone();
        assert_eq!(agent1.id(), agent2.id());
        assert_eq!(agent1.domain(), agent2.domain());
        assert_eq!(agent1.capabilities().len(), agent2.capabilities().len());
    }

    #[test]
    fn test_backend_agent_all_api_patterns_present() {
        let agent = BackendAgent::new();
        let knowledge = agent.knowledge();

        let api_patterns = vec!["REST", "GraphQL", "gRPC"];
        for pattern in api_patterns {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == pattern);
            assert!(found, "API pattern {} not found in recommendations", pattern);
        }
    }

    #[test]
    fn test_backend_agent_all_databases_present() {
        let agent = BackendAgent::new();
        let knowledge = agent.knowledge();

        let databases = vec!["PostgreSQL", "MongoDB", "Neo4j"];
        for db in databases {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == db);
            assert!(found, "Database {} not found in recommendations", db);
        }
    }

    #[test]
    fn test_backend_agent_all_architectures_present() {
        let agent = BackendAgent::new();
        let knowledge = agent.knowledge();

        let architectures = vec!["Microservices", "Monolithic", "Serverless"];
        for arch in architectures {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == arch);
            assert!(found, "Architecture {} not found in recommendations", arch);
        }
    }

    #[test]
    fn test_backend_agent_all_caching_solutions_present() {
        let agent = BackendAgent::new();
        let knowledge = agent.knowledge();

        let caching = vec!["Redis", "Memcached"];
        for cache in caching {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == cache);
            assert!(found, "Caching solution {} not found in recommendations", cache);
        }
    }

    #[test]
    fn test_backend_agent_all_security_technologies_present() {
        let agent = BackendAgent::new();
        let knowledge = agent.knowledge();

        let security = vec!["OAuth 2.0", "JWT", "TLS"];
        for tech in security {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == tech);
            assert!(found, "Security technology {} not found in recommendations", tech);
        }
    }

    #[test]
    fn test_backend_agent_all_observability_tools_present() {
        let agent = BackendAgent::new();
        let knowledge = agent.knowledge();

        let observability = vec!["ELK Stack", "Prometheus", "Jaeger"];
        for tool in observability {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == tool);
            assert!(found, "Observability tool {} not found in recommendations", tool);
        }
    }

    #[test]
    fn test_backend_agent_rest_alternatives() {
        let agent = BackendAgent::new();
        let knowledge = agent.knowledge();

        let rest_rec = knowledge
            .technology_recommendations
            .iter()
            .find(|r| r.technology == "REST")
            .expect("REST recommendation not found");

        assert!(rest_rec.alternatives.contains(&"GraphQL".to_string()));
        assert!(rest_rec.alternatives.contains(&"gRPC".to_string()));
    }

    #[test]
    fn test_backend_agent_postgresql_alternatives() {
        let agent = BackendAgent::new();
        let knowledge = agent.knowledge();

        let pg_rec = knowledge
            .technology_recommendations
            .iter()
            .find(|r| r.technology == "PostgreSQL")
            .expect("PostgreSQL recommendation not found");

        assert!(pg_rec.alternatives.contains(&"MySQL".to_string()));
        assert!(pg_rec.alternatives.contains(&"MariaDB".to_string()));
    }

    #[test]
    fn test_backend_agent_redis_alternatives() {
        let agent = BackendAgent::new();
        let knowledge = agent.knowledge();

        let redis_rec = knowledge
            .technology_recommendations
            .iter()
            .find(|r| r.technology == "Redis")
            .expect("Redis recommendation not found");

        assert!(redis_rec.alternatives.contains(&"Memcached".to_string()));
        assert!(redis_rec.alternatives.contains(&"Hazelcast".to_string()));
    }

    #[tokio::test]
    async fn test_backend_agent_execute() {
        use crate::models::{
            AgentConfig, AgentTask, ProjectContext, TaskOptions, TaskScope, TaskTarget,
        };
        use std::path::PathBuf;

        let agent = BackendAgent::new();
        let input = AgentInput {
            task: AgentTask {
                id: "task-1".to_string(),
                task_type: TaskType::CodeReview,
                target: TaskTarget {
                    files: vec![PathBuf::from("api.rs")],
                    scope: TaskScope::File,
                },
                options: TaskOptions::default(),
            },
            context: ProjectContext {
                name: "backend-project".to_string(),
                root: PathBuf::from("/tmp/backend-project"),
            },
            config: AgentConfig::default(),
        };

        let result = agent.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.findings.is_empty());
    }
}
