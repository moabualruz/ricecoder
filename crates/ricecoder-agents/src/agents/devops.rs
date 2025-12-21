//! DevOps Agent
//!
//! This module provides a specialized agent for DevOps tasks,
//! including CI/CD pipeline recommendations, Infrastructure as Code guidance,
//! containerization recommendations, observability infrastructure guidance,
//! security scanning setup recommendations, and auto-scaling configuration guidance.

use crate::agents::Agent;
use crate::domain::{DomainAgent, DomainCapability, DomainKnowledge, TechRecommendation};
use crate::error::Result;
use crate::models::{AgentInput, AgentOutput, Finding, Severity, TaskType};
use async_trait::async_trait;
use uuid::Uuid;

/// DevOps Agent
///
/// A specialized agent for DevOps that provides recommendations for:
/// - CI/CD pipeline setup (GitHub Actions, GitLab CI, Jenkins)
/// - Infrastructure as Code (Terraform, CloudFormation, Ansible)
/// - Containerization (Docker, Kubernetes)
/// - Observability infrastructure (monitoring, logging, alerting)
/// - Security scanning setup
/// - Auto-scaling configuration
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::agents::DevOpsAgent;
///
/// let agent = DevOpsAgent::new();
/// assert_eq!(agent.id(), "devops-agent");
/// assert_eq!(agent.domain(), "devops");
/// ```
#[derive(Debug, Clone)]
pub struct DevOpsAgent {
    domain_agent: DomainAgent,
}

impl DevOpsAgent {
    /// Create a new DevOps Agent
    ///
    /// Initializes the agent with DevOps-specific capabilities and knowledge.
    ///
    /// # Returns
    ///
    /// A new `DevOpsAgent` instance
    pub fn new() -> Self {
        let capabilities = vec![
            DomainCapability {
                name: "CI/CD Pipeline Setup".to_string(),
                description: "Recommendations for GitHub Actions, GitLab CI, or Jenkins"
                    .to_string(),
                technologies: vec![
                    "GitHub Actions".to_string(),
                    "GitLab CI".to_string(),
                    "Jenkins".to_string(),
                ],
                patterns: vec![],
            },
            DomainCapability {
                name: "Infrastructure as Code".to_string(),
                description: "Recommendations for Terraform, CloudFormation, or Ansible"
                    .to_string(),
                technologies: vec![
                    "Terraform".to_string(),
                    "CloudFormation".to_string(),
                    "Ansible".to_string(),
                ],
                patterns: vec![],
            },
            DomainCapability {
                name: "Containerization".to_string(),
                description: "Recommendations for Docker and Kubernetes configuration".to_string(),
                technologies: vec!["Docker".to_string(), "Kubernetes".to_string()],
                patterns: vec![],
            },
            DomainCapability {
                name: "Observability Infrastructure".to_string(),
                description: "Recommendations for monitoring, logging, and alerting".to_string(),
                technologies: vec![
                    "Prometheus".to_string(),
                    "Grafana".to_string(),
                    "ELK Stack".to_string(),
                ],
                patterns: vec![],
            },
            DomainCapability {
                name: "Security Scanning".to_string(),
                description: "Recommendations for vulnerability scanning and compliance"
                    .to_string(),
                technologies: vec![
                    "Trivy".to_string(),
                    "Snyk".to_string(),
                    "SonarQube".to_string(),
                ],
                patterns: vec![],
            },
            DomainCapability {
                name: "Auto-Scaling".to_string(),
                description: "Recommendations for scaling policies and resource management"
                    .to_string(),
                technologies: vec![
                    "Kubernetes".to_string(),
                    "AWS Auto Scaling".to_string(),
                    "Docker Swarm".to_string(),
                ],
                patterns: vec![],
            },
        ];

        let knowledge = DomainKnowledge {
            best_practices: vec![],
            technology_recommendations: vec![
                TechRecommendation {
                    technology: "GitHub Actions".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "GitHub-hosted CI/CD".to_string(),
                        "Automated testing".to_string(),
                        "Deployment automation".to_string(),
                    ],
                    pros: vec![
                        "Native GitHub integration".to_string(),
                        "Free for public repos".to_string(),
                        "Easy to set up".to_string(),
                    ],
                    cons: vec![
                        "Limited to GitHub".to_string(),
                        "Execution time limits".to_string(),
                    ],
                    alternatives: vec!["GitLab CI".to_string(), "Jenkins".to_string()],
                },
                TechRecommendation {
                    technology: "GitLab CI".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "GitLab-hosted CI/CD".to_string(),
                        "Self-hosted runners".to_string(),
                        "Complex pipelines".to_string(),
                    ],
                    pros: vec![
                        "Powerful pipeline features".to_string(),
                        "Self-hosted runners".to_string(),
                        "Integrated with GitLab".to_string(),
                    ],
                    cons: vec![
                        "Limited to GitLab".to_string(),
                        "Steeper learning curve".to_string(),
                    ],
                    alternatives: vec!["GitHub Actions".to_string(), "Jenkins".to_string()],
                },
                TechRecommendation {
                    technology: "Jenkins".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "Self-hosted CI/CD".to_string(),
                        "Complex pipelines".to_string(),
                        "Legacy systems".to_string(),
                    ],
                    pros: vec![
                        "Highly customizable".to_string(),
                        "Large plugin ecosystem".to_string(),
                        "Self-hosted".to_string(),
                    ],
                    cons: vec![
                        "Complex setup".to_string(),
                        "Maintenance overhead".to_string(),
                        "Older technology".to_string(),
                    ],
                    alternatives: vec!["GitHub Actions".to_string(), "GitLab CI".to_string()],
                },
                TechRecommendation {
                    technology: "Terraform".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "Infrastructure as Code".to_string(),
                        "Multi-cloud deployments".to_string(),
                        "Reproducible infrastructure".to_string(),
                    ],
                    pros: vec![
                        "Declarative syntax".to_string(),
                        "Multi-cloud support".to_string(),
                        "State management".to_string(),
                    ],
                    cons: vec![
                        "State management complexity".to_string(),
                        "Learning curve".to_string(),
                    ],
                    alternatives: vec!["CloudFormation".to_string(), "Ansible".to_string()],
                },
                TechRecommendation {
                    technology: "CloudFormation".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "AWS Infrastructure as Code".to_string(),
                        "AWS-specific deployments".to_string(),
                    ],
                    pros: vec![
                        "Native AWS integration".to_string(),
                        "No additional tools needed".to_string(),
                        "Good documentation".to_string(),
                    ],
                    cons: vec![
                        "AWS-only".to_string(),
                        "Verbose syntax".to_string(),
                        "Limited multi-cloud support".to_string(),
                    ],
                    alternatives: vec!["Terraform".to_string(), "Ansible".to_string()],
                },
                TechRecommendation {
                    technology: "Ansible".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "Configuration management".to_string(),
                        "Infrastructure automation".to_string(),
                        "Agentless automation".to_string(),
                    ],
                    pros: vec![
                        "Agentless".to_string(),
                        "Easy to learn".to_string(),
                        "Flexible".to_string(),
                    ],
                    cons: vec![
                        "Slower than alternatives".to_string(),
                        "Less suitable for IaC".to_string(),
                    ],
                    alternatives: vec!["Terraform".to_string(), "Chef".to_string()],
                },
                TechRecommendation {
                    technology: "Docker".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "Containerization".to_string(),
                        "Application packaging".to_string(),
                        "Development environments".to_string(),
                    ],
                    pros: vec![
                        "Lightweight containers".to_string(),
                        "Easy to use".to_string(),
                        "Large ecosystem".to_string(),
                    ],
                    cons: vec![
                        "Learning curve".to_string(),
                        "Security considerations".to_string(),
                    ],
                    alternatives: vec!["Podman".to_string(), "containerd".to_string()],
                },
                TechRecommendation {
                    technology: "Kubernetes".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "Container orchestration".to_string(),
                        "Microservices".to_string(),
                        "High availability".to_string(),
                    ],
                    pros: vec![
                        "Powerful orchestration".to_string(),
                        "Auto-scaling".to_string(),
                        "Self-healing".to_string(),
                    ],
                    cons: vec![
                        "Complex setup".to_string(),
                        "Steep learning curve".to_string(),
                        "Operational overhead".to_string(),
                    ],
                    alternatives: vec!["Docker Swarm".to_string(), "AWS ECS".to_string()],
                },
                TechRecommendation {
                    technology: "Prometheus".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "Metrics collection".to_string(),
                        "Monitoring".to_string(),
                        "Alerting".to_string(),
                    ],
                    pros: vec![
                        "Time-series database".to_string(),
                        "Powerful query language".to_string(),
                        "Easy to set up".to_string(),
                    ],
                    cons: vec![
                        "Limited long-term storage".to_string(),
                        "Requires additional tools".to_string(),
                    ],
                    alternatives: vec!["Grafana Loki".to_string(), "InfluxDB".to_string()],
                },
                TechRecommendation {
                    technology: "Grafana".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "Metrics visualization".to_string(),
                        "Dashboards".to_string(),
                        "Alerting".to_string(),
                    ],
                    pros: vec![
                        "Beautiful dashboards".to_string(),
                        "Multiple data sources".to_string(),
                        "Easy to use".to_string(),
                    ],
                    cons: vec![
                        "Requires data source".to_string(),
                        "Learning curve for advanced features".to_string(),
                    ],
                    alternatives: vec!["Kibana".to_string(), "Datadog".to_string()],
                },
                TechRecommendation {
                    technology: "ELK Stack".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "Log aggregation".to_string(),
                        "Log analysis".to_string(),
                        "Centralized logging".to_string(),
                    ],
                    pros: vec![
                        "Powerful search".to_string(),
                        "Scalable".to_string(),
                        "Open source".to_string(),
                    ],
                    cons: vec![
                        "Complex setup".to_string(),
                        "Resource intensive".to_string(),
                        "Operational overhead".to_string(),
                    ],
                    alternatives: vec!["Splunk".to_string(), "Datadog".to_string()],
                },
                TechRecommendation {
                    technology: "Trivy".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "Vulnerability scanning".to_string(),
                        "Container scanning".to_string(),
                        "CI/CD integration".to_string(),
                    ],
                    pros: vec![
                        "Fast scanning".to_string(),
                        "Easy to use".to_string(),
                        "Comprehensive database".to_string(),
                    ],
                    cons: vec![
                        "False positives".to_string(),
                        "Limited remediation guidance".to_string(),
                    ],
                    alternatives: vec!["Snyk".to_string(), "Grype".to_string()],
                },
                TechRecommendation {
                    technology: "Snyk".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "Vulnerability management".to_string(),
                        "Dependency scanning".to_string(),
                        "Container scanning".to_string(),
                    ],
                    pros: vec![
                        "Developer-friendly".to_string(),
                        "Remediation guidance".to_string(),
                        "Multiple scanning types".to_string(),
                    ],
                    cons: vec!["Paid service".to_string(), "Limited free tier".to_string()],
                    alternatives: vec!["Trivy".to_string(), "Dependabot".to_string()],
                },
                TechRecommendation {
                    technology: "SonarQube".to_string(),
                    domain: "devops".to_string(),
                    use_cases: vec![
                        "Code quality analysis".to_string(),
                        "Security scanning".to_string(),
                        "Technical debt tracking".to_string(),
                    ],
                    pros: vec![
                        "Comprehensive analysis".to_string(),
                        "Multiple languages".to_string(),
                        "Good reporting".to_string(),
                    ],
                    cons: vec![
                        "Complex setup".to_string(),
                        "Resource intensive".to_string(),
                        "Expensive for large teams".to_string(),
                    ],
                    alternatives: vec!["CodeClimate".to_string(), "Codacy".to_string()],
                },
            ],
            patterns: vec![],
            anti_patterns: vec![],
        };

        let domain_agent = DomainAgent {
            id: "devops-agent".to_string(),
            domain: "devops".to_string(),
            capabilities,
            knowledge,
        };

        DevOpsAgent { domain_agent }
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

impl Default for DevOpsAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for DevOpsAgent {
    fn id(&self) -> &str {
        &self.domain_agent.id
    }

    fn name(&self) -> &str {
        "DevOps Agent"
    }

    fn description(&self) -> &str {
        "Specialized agent for DevOps with expertise in CI/CD pipelines, Infrastructure as Code, containerization, observability, security scanning, and auto-scaling"
    }

    fn supports(&self, task_type: TaskType) -> bool {
        // DevOps agent supports code review and other DevOps-related tasks
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
        // This will be enhanced with actual DevOps-specific logic
        let mut output = AgentOutput::default();

        // Add findings based on the task type
        match input.task.task_type {
            TaskType::CodeReview => {
                output.findings.push(Finding {
                    id: Uuid::new_v4().to_string(),
                    severity: Severity::Info,
                    category: "devops-best-practices".to_string(),
                    message: "DevOps best practices applied".to_string(),
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
    fn test_devops_agent_creation() {
        let agent = DevOpsAgent::new();
        assert_eq!(agent.id(), "devops-agent");
        assert_eq!(agent.domain(), "devops");
        assert_eq!(agent.name(), "DevOps Agent");
    }

    #[test]
    fn test_devops_agent_capabilities() {
        let agent = DevOpsAgent::new();
        let capabilities = agent.capabilities();
        assert_eq!(capabilities.len(), 6);

        // Check first capability
        assert_eq!(capabilities[0].name, "CI/CD Pipeline Setup");
        assert_eq!(capabilities[0].technologies.len(), 3);
        assert!(capabilities[0]
            .technologies
            .contains(&"GitHub Actions".to_string()));
        assert!(capabilities[0]
            .technologies
            .contains(&"GitLab CI".to_string()));
        assert!(capabilities[0]
            .technologies
            .contains(&"Jenkins".to_string()));
    }

    #[test]
    fn test_devops_agent_knowledge() {
        let agent = DevOpsAgent::new();
        let knowledge = agent.knowledge();
        assert!(!knowledge.technology_recommendations.is_empty());
        assert_eq!(knowledge.technology_recommendations.len(), 14);
    }

    #[test]
    fn test_devops_agent_technology_recommendations() {
        let agent = DevOpsAgent::new();
        let knowledge = agent.knowledge();

        // Check Kubernetes recommendation
        let k8s_rec = knowledge
            .technology_recommendations
            .iter()
            .find(|r| r.technology == "Kubernetes")
            .expect("Kubernetes recommendation not found");

        assert_eq!(k8s_rec.domain, "devops");
        assert!(!k8s_rec.use_cases.is_empty());
        assert!(!k8s_rec.pros.is_empty());
        assert!(!k8s_rec.cons.is_empty());
        assert!(!k8s_rec.alternatives.is_empty());
    }

    #[test]
    fn test_devops_agent_supports_task_types() {
        let agent = DevOpsAgent::new();
        assert!(agent.supports(TaskType::CodeReview));
        assert!(agent.supports(TaskType::Refactoring));
        assert!(agent.supports(TaskType::Documentation));
        assert!(agent.supports(TaskType::SecurityAnalysis));
        assert!(!agent.supports(TaskType::TestGeneration));
    }

    #[test]
    fn test_devops_agent_default() {
        let agent1 = DevOpsAgent::new();
        let agent2 = DevOpsAgent::default();
        assert_eq!(agent1.id(), agent2.id());
        assert_eq!(agent1.domain(), agent2.domain());
    }

    #[test]
    fn test_devops_agent_clone() {
        let agent1 = DevOpsAgent::new();
        let agent2 = agent1.clone();
        assert_eq!(agent1.id(), agent2.id());
        assert_eq!(agent1.domain(), agent2.domain());
        assert_eq!(agent1.capabilities().len(), agent2.capabilities().len());
    }

    #[test]
    fn test_devops_agent_all_cicd_tools_present() {
        let agent = DevOpsAgent::new();
        let knowledge = agent.knowledge();

        let cicd_tools = vec!["GitHub Actions", "GitLab CI", "Jenkins"];
        for tool in cicd_tools {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == tool);
            assert!(found, "CI/CD tool {} not found in recommendations", tool);
        }
    }

    #[test]
    fn test_devops_agent_all_iac_tools_present() {
        let agent = DevOpsAgent::new();
        let knowledge = agent.knowledge();

        let iac_tools = vec!["Terraform", "CloudFormation", "Ansible"];
        for tool in iac_tools {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == tool);
            assert!(found, "IaC tool {} not found in recommendations", tool);
        }
    }

    #[test]
    fn test_devops_agent_all_container_tools_present() {
        let agent = DevOpsAgent::new();
        let knowledge = agent.knowledge();

        let container_tools = vec!["Docker", "Kubernetes"];
        for tool in container_tools {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == tool);
            assert!(
                found,
                "Container tool {} not found in recommendations",
                tool
            );
        }
    }

    #[test]
    fn test_devops_agent_all_observability_tools_present() {
        let agent = DevOpsAgent::new();
        let knowledge = agent.knowledge();

        let observability_tools = vec!["Prometheus", "Grafana", "ELK Stack"];
        for tool in observability_tools {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == tool);
            assert!(
                found,
                "Observability tool {} not found in recommendations",
                tool
            );
        }
    }

    #[test]
    fn test_devops_agent_all_security_tools_present() {
        let agent = DevOpsAgent::new();
        let knowledge = agent.knowledge();

        let security_tools = vec!["Trivy", "Snyk", "SonarQube"];
        for tool in security_tools {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == tool);
            assert!(found, "Security tool {} not found in recommendations", tool);
        }
    }

    #[test]
    fn test_devops_agent_terraform_alternatives() {
        let agent = DevOpsAgent::new();
        let knowledge = agent.knowledge();

        let terraform_rec = knowledge
            .technology_recommendations
            .iter()
            .find(|r| r.technology == "Terraform")
            .expect("Terraform recommendation not found");

        assert!(terraform_rec
            .alternatives
            .contains(&"CloudFormation".to_string()));
        assert!(terraform_rec.alternatives.contains(&"Ansible".to_string()));
    }

    #[test]
    fn test_devops_agent_kubernetes_alternatives() {
        let agent = DevOpsAgent::new();
        let knowledge = agent.knowledge();

        let k8s_rec = knowledge
            .technology_recommendations
            .iter()
            .find(|r| r.technology == "Kubernetes")
            .expect("Kubernetes recommendation not found");

        assert!(k8s_rec.alternatives.contains(&"Docker Swarm".to_string()));
        assert!(k8s_rec.alternatives.contains(&"AWS ECS".to_string()));
    }

    #[test]
    fn test_devops_agent_github_actions_alternatives() {
        let agent = DevOpsAgent::new();
        let knowledge = agent.knowledge();

        let github_rec = knowledge
            .technology_recommendations
            .iter()
            .find(|r| r.technology == "GitHub Actions")
            .expect("GitHub Actions recommendation not found");

        assert!(github_rec.alternatives.contains(&"GitLab CI".to_string()));
        assert!(github_rec.alternatives.contains(&"Jenkins".to_string()));
    }

    #[tokio::test]
    async fn test_devops_agent_execute() {
        use crate::models::{
            AgentConfig, AgentTask, ProjectContext, TaskOptions, TaskScope, TaskTarget,
        };
        use std::path::PathBuf;

        let agent = DevOpsAgent::new();
        let input = AgentInput {
            task: AgentTask {
                id: "task-1".to_string(),
                task_type: TaskType::CodeReview,
                target: TaskTarget {
                    files: vec![PathBuf::from("Dockerfile")],
                    scope: TaskScope::File,
                },
                options: TaskOptions::default(),
            },
            context: ProjectContext {
                name: "devops-project".to_string(),
                root: PathBuf::from("/tmp/devops-project"),
            },
            config: AgentConfig::default(),
        };

        let result = agent.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.findings.is_empty());
    }
}
