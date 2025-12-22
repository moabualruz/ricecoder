//! Web Development Agent
//!
//! This module provides a specialized agent for web development tasks,
//! including frontend framework recommendations, styling guidance, build tool
//! recommendations, testing strategies, performance optimization, and deployment guidance.

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    agents::Agent,
    domain::{DomainAgent, DomainCapability, DomainKnowledge, TechRecommendation},
    error::Result,
    models::{AgentInput, AgentOutput, Finding, Severity, TaskType},
};

/// Web Development Agent
///
/// A specialized agent for web development that provides recommendations for:
/// - Frontend frameworks (React, Vue, Angular)
/// - Styling solutions (CSS, Tailwind CSS, styled-components)
/// - Build tools (Vite, Webpack)
/// - Testing frameworks (Jest, Vitest, Playwright)
/// - Performance optimization
/// - Deployment patterns
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::agents::WebAgent;
///
/// let agent = WebAgent::new();
/// assert_eq!(agent.id(), "web-agent");
/// assert_eq!(agent.domain(), "web");
/// ```
#[derive(Debug, Clone)]
pub struct WebAgent {
    domain_agent: DomainAgent,
}

impl WebAgent {
    /// Create a new Web Development Agent
    ///
    /// Initializes the agent with web-specific capabilities and knowledge.
    ///
    /// # Returns
    ///
    /// A new `WebAgent` instance
    pub fn new() -> Self {
        let capabilities = vec![
            DomainCapability {
                name: "Frontend Framework Selection".to_string(),
                description: "Recommend frontend frameworks based on project needs".to_string(),
                technologies: vec![
                    "React".to_string(),
                    "Vue".to_string(),
                    "Angular".to_string(),
                ],
                patterns: vec![],
            },
            DomainCapability {
                name: "Styling Guidance".to_string(),
                description: "Provide expertise on CSS, Tailwind CSS, styled-components"
                    .to_string(),
                technologies: vec![
                    "CSS".to_string(),
                    "Tailwind CSS".to_string(),
                    "styled-components".to_string(),
                ],
                patterns: vec![],
            },
            DomainCapability {
                name: "Build Configuration".to_string(),
                description: "Guidance on Vite or Webpack configuration".to_string(),
                technologies: vec!["Vite".to_string(), "Webpack".to_string()],
                patterns: vec![],
            },
            DomainCapability {
                name: "Testing Strategy".to_string(),
                description: "Recommendations for web testing frameworks".to_string(),
                technologies: vec![
                    "Jest".to_string(),
                    "Vitest".to_string(),
                    "Playwright".to_string(),
                ],
                patterns: vec![],
            },
            DomainCapability {
                name: "Performance Optimization".to_string(),
                description: "Specific optimization recommendations for web applications"
                    .to_string(),
                technologies: vec![
                    "React".to_string(),
                    "Vue".to_string(),
                    "Angular".to_string(),
                ],
                patterns: vec![],
            },
            DomainCapability {
                name: "Deployment Guidance".to_string(),
                description: "Recommendations for web deployment patterns and platforms"
                    .to_string(),
                technologies: vec![
                    "Vercel".to_string(),
                    "Netlify".to_string(),
                    "AWS".to_string(),
                    "Docker".to_string(),
                ],
                patterns: vec![],
            },
        ];

        let knowledge = DomainKnowledge {
            best_practices: vec![],
            technology_recommendations: vec![
                TechRecommendation {
                    technology: "React".to_string(),
                    domain: "web".to_string(),
                    use_cases: vec![
                        "Single Page Applications".to_string(),
                        "Complex UIs".to_string(),
                        "Real-time applications".to_string(),
                    ],
                    pros: vec![
                        "Large ecosystem".to_string(),
                        "Strong community".to_string(),
                        "Excellent tooling".to_string(),
                    ],
                    cons: vec![
                        "Steep learning curve".to_string(),
                        "JSX syntax".to_string(),
                        "Frequent updates".to_string(),
                    ],
                    alternatives: vec!["Vue".to_string(), "Angular".to_string()],
                },
                TechRecommendation {
                    technology: "Vue".to_string(),
                    domain: "web".to_string(),
                    use_cases: vec![
                        "Progressive enhancement".to_string(),
                        "Single Page Applications".to_string(),
                        "Rapid prototyping".to_string(),
                    ],
                    pros: vec![
                        "Easy to learn".to_string(),
                        "Excellent documentation".to_string(),
                        "Flexible".to_string(),
                    ],
                    cons: vec![
                        "Smaller ecosystem".to_string(),
                        "Less enterprise adoption".to_string(),
                    ],
                    alternatives: vec!["React".to_string(), "Angular".to_string()],
                },
                TechRecommendation {
                    technology: "Angular".to_string(),
                    domain: "web".to_string(),
                    use_cases: vec![
                        "Large-scale applications".to_string(),
                        "Enterprise projects".to_string(),
                        "Complex applications".to_string(),
                    ],
                    pros: vec![
                        "Full-featured framework".to_string(),
                        "Strong typing with TypeScript".to_string(),
                        "Enterprise support".to_string(),
                    ],
                    cons: vec![
                        "Steep learning curve".to_string(),
                        "Verbose syntax".to_string(),
                        "Heavier bundle size".to_string(),
                    ],
                    alternatives: vec!["React".to_string(), "Vue".to_string()],
                },
                TechRecommendation {
                    technology: "Vite".to_string(),
                    domain: "web".to_string(),
                    use_cases: vec![
                        "Modern web development".to_string(),
                        "Fast development experience".to_string(),
                    ],
                    pros: vec![
                        "Lightning fast HMR".to_string(),
                        "Optimized build".to_string(),
                        "Native ES modules".to_string(),
                    ],
                    cons: vec![
                        "Newer tool".to_string(),
                        "Less mature than Webpack".to_string(),
                    ],
                    alternatives: vec!["Webpack".to_string(), "Parcel".to_string()],
                },
                TechRecommendation {
                    technology: "Webpack".to_string(),
                    domain: "web".to_string(),
                    use_cases: vec![
                        "Complex bundling scenarios".to_string(),
                        "Legacy projects".to_string(),
                    ],
                    pros: vec![
                        "Mature and stable".to_string(),
                        "Highly configurable".to_string(),
                        "Large ecosystem".to_string(),
                    ],
                    cons: vec![
                        "Complex configuration".to_string(),
                        "Slower build times".to_string(),
                    ],
                    alternatives: vec!["Vite".to_string(), "Parcel".to_string()],
                },
                TechRecommendation {
                    technology: "Jest".to_string(),
                    domain: "web".to_string(),
                    use_cases: vec![
                        "Unit testing".to_string(),
                        "Integration testing".to_string(),
                    ],
                    pros: vec![
                        "Zero configuration".to_string(),
                        "Great documentation".to_string(),
                        "Snapshot testing".to_string(),
                    ],
                    cons: vec![
                        "Slower than alternatives".to_string(),
                        "Higher memory usage".to_string(),
                    ],
                    alternatives: vec!["Vitest".to_string(), "Mocha".to_string()],
                },
                TechRecommendation {
                    technology: "Vitest".to_string(),
                    domain: "web".to_string(),
                    use_cases: vec![
                        "Unit testing".to_string(),
                        "Fast test execution".to_string(),
                    ],
                    pros: vec![
                        "Lightning fast".to_string(),
                        "Vite integration".to_string(),
                        "Jest-compatible API".to_string(),
                    ],
                    cons: vec!["Newer tool".to_string(), "Smaller ecosystem".to_string()],
                    alternatives: vec!["Jest".to_string(), "Mocha".to_string()],
                },
                TechRecommendation {
                    technology: "Playwright".to_string(),
                    domain: "web".to_string(),
                    use_cases: vec![
                        "End-to-end testing".to_string(),
                        "Cross-browser testing".to_string(),
                    ],
                    pros: vec![
                        "Cross-browser support".to_string(),
                        "Reliable automation".to_string(),
                        "Great debugging tools".to_string(),
                    ],
                    cons: vec![
                        "Heavier than Cypress".to_string(),
                        "Steeper learning curve".to_string(),
                    ],
                    alternatives: vec!["Cypress".to_string(), "Selenium".to_string()],
                },
                TechRecommendation {
                    technology: "Tailwind CSS".to_string(),
                    domain: "web".to_string(),
                    use_cases: vec![
                        "Rapid UI development".to_string(),
                        "Utility-first styling".to_string(),
                    ],
                    pros: vec![
                        "Rapid development".to_string(),
                        "Consistent design".to_string(),
                        "Small bundle size".to_string(),
                    ],
                    cons: vec![
                        "Learning curve".to_string(),
                        "HTML markup verbosity".to_string(),
                    ],
                    alternatives: vec!["Bootstrap".to_string(), "styled-components".to_string()],
                },
                TechRecommendation {
                    technology: "styled-components".to_string(),
                    domain: "web".to_string(),
                    use_cases: vec![
                        "Component-scoped styling".to_string(),
                        "Dynamic styling".to_string(),
                    ],
                    pros: vec![
                        "Component scoping".to_string(),
                        "Dynamic styling".to_string(),
                        "No class name conflicts".to_string(),
                    ],
                    cons: vec![
                        "Runtime overhead".to_string(),
                        "Larger bundle size".to_string(),
                    ],
                    alternatives: vec!["Tailwind CSS".to_string(), "CSS Modules".to_string()],
                },
            ],
            patterns: vec![],
            anti_patterns: vec![],
        };

        let domain_agent = DomainAgent {
            id: "web-agent".to_string(),
            domain: "web".to_string(),
            capabilities,
            knowledge,
        };

        WebAgent { domain_agent }
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

impl Default for WebAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for WebAgent {
    fn id(&self) -> &str {
        &self.domain_agent.id
    }

    fn name(&self) -> &str {
        "Web Development Agent"
    }

    fn description(&self) -> &str {
        "Specialized agent for web development with expertise in frontend frameworks, styling, build tools, testing, performance optimization, and deployment"
    }

    fn supports(&self, task_type: TaskType) -> bool {
        // Web agent supports code review and other web-related tasks
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
        // This will be enhanced with actual web-specific logic
        let mut output = AgentOutput::default();

        // Add findings based on the task type
        match input.task.task_type {
            TaskType::CodeReview => {
                output.findings.push(Finding {
                    id: Uuid::new_v4().to_string(),
                    severity: Severity::Info,
                    category: "web-best-practices".to_string(),
                    message: "Web development best practices applied".to_string(),
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
    fn test_web_agent_creation() {
        let agent = WebAgent::new();
        assert_eq!(agent.id(), "web-agent");
        assert_eq!(agent.domain(), "web");
        assert_eq!(agent.name(), "Web Development Agent");
    }

    #[test]
    fn test_web_agent_capabilities() {
        let agent = WebAgent::new();
        let capabilities = agent.capabilities();
        assert_eq!(capabilities.len(), 6);

        // Check first capability
        assert_eq!(capabilities[0].name, "Frontend Framework Selection");
        assert_eq!(capabilities[0].technologies.len(), 3);
        assert!(capabilities[0].technologies.contains(&"React".to_string()));
        assert!(capabilities[0].technologies.contains(&"Vue".to_string()));
        assert!(capabilities[0]
            .technologies
            .contains(&"Angular".to_string()));
    }

    #[test]
    fn test_web_agent_knowledge() {
        let agent = WebAgent::new();
        let knowledge = agent.knowledge();
        assert!(!knowledge.technology_recommendations.is_empty());
        assert_eq!(knowledge.technology_recommendations.len(), 10);
    }

    #[test]
    fn test_web_agent_technology_recommendations() {
        let agent = WebAgent::new();
        let knowledge = agent.knowledge();

        // Check React recommendation
        let react_rec = knowledge
            .technology_recommendations
            .iter()
            .find(|r| r.technology == "React")
            .expect("React recommendation not found");

        assert_eq!(react_rec.domain, "web");
        assert!(!react_rec.use_cases.is_empty());
        assert!(!react_rec.pros.is_empty());
        assert!(!react_rec.cons.is_empty());
        assert!(!react_rec.alternatives.is_empty());
    }

    #[test]
    fn test_web_agent_supports_task_types() {
        let agent = WebAgent::new();
        assert!(agent.supports(TaskType::CodeReview));
        assert!(agent.supports(TaskType::Refactoring));
        assert!(agent.supports(TaskType::Documentation));
        assert!(agent.supports(TaskType::SecurityAnalysis));
        assert!(!agent.supports(TaskType::TestGeneration));
    }

    #[test]
    fn test_web_agent_default() {
        let agent1 = WebAgent::new();
        let agent2 = WebAgent::default();
        assert_eq!(agent1.id(), agent2.id());
        assert_eq!(agent1.domain(), agent2.domain());
    }

    #[test]
    fn test_web_agent_clone() {
        let agent1 = WebAgent::new();
        let agent2 = agent1.clone();
        assert_eq!(agent1.id(), agent2.id());
        assert_eq!(agent1.domain(), agent2.domain());
        assert_eq!(agent1.capabilities().len(), agent2.capabilities().len());
    }

    #[test]
    fn test_web_agent_all_frameworks_present() {
        let agent = WebAgent::new();
        let knowledge = agent.knowledge();

        let frameworks = vec!["React", "Vue", "Angular"];
        for framework in frameworks {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == framework);
            assert!(
                found,
                "Framework {} not found in recommendations",
                framework
            );
        }
    }

    #[test]
    fn test_web_agent_all_build_tools_present() {
        let agent = WebAgent::new();
        let knowledge = agent.knowledge();

        let build_tools = vec!["Vite", "Webpack"];
        for tool in build_tools {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == tool);
            assert!(found, "Build tool {} not found in recommendations", tool);
        }
    }

    #[test]
    fn test_web_agent_all_testing_frameworks_present() {
        let agent = WebAgent::new();
        let knowledge = agent.knowledge();

        let testing_frameworks = vec!["Jest", "Vitest", "Playwright"];
        for framework in testing_frameworks {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == framework);
            assert!(
                found,
                "Testing framework {} not found in recommendations",
                framework
            );
        }
    }

    #[test]
    fn test_web_agent_styling_solutions_present() {
        let agent = WebAgent::new();
        let knowledge = agent.knowledge();

        let styling_solutions = vec!["Tailwind CSS", "styled-components"];
        for solution in styling_solutions {
            let found = knowledge
                .technology_recommendations
                .iter()
                .any(|r| r.technology == solution);
            assert!(
                found,
                "Styling solution {} not found in recommendations",
                solution
            );
        }
    }

    #[tokio::test]
    async fn test_web_agent_execute() {
        use std::path::PathBuf;

        use crate::models::{
            AgentConfig, AgentTask, ProjectContext, TaskOptions, TaskScope, TaskTarget,
        };

        let agent = WebAgent::new();
        let input = AgentInput {
            task: AgentTask {
                id: "task-1".to_string(),
                task_type: TaskType::CodeReview,
                target: TaskTarget {
                    files: vec![PathBuf::from("test.tsx")],
                    scope: TaskScope::File,
                },
                options: TaskOptions::default(),
            },
            context: ProjectContext {
                name: "web-project".to_string(),
                root: PathBuf::from("/tmp/web-project"),
            },
            config: AgentConfig::default(),
        };

        let result = agent.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.findings.is_empty());
    }

    #[test]
    fn test_web_agent_react_alternatives() {
        let agent = WebAgent::new();
        let knowledge = agent.knowledge();

        let react_rec = knowledge
            .technology_recommendations
            .iter()
            .find(|r| r.technology == "React")
            .expect("React recommendation not found");

        assert!(react_rec.alternatives.contains(&"Vue".to_string()));
        assert!(react_rec.alternatives.contains(&"Angular".to_string()));
    }

    #[test]
    fn test_web_agent_vite_alternatives() {
        let agent = WebAgent::new();
        let knowledge = agent.knowledge();

        let vite_rec = knowledge
            .technology_recommendations
            .iter()
            .find(|r| r.technology == "Vite")
            .expect("Vite recommendation not found");

        assert!(vite_rec.alternatives.contains(&"Webpack".to_string()));
        assert!(vite_rec.alternatives.contains(&"Parcel".to_string()));
    }

    #[test]
    fn test_web_agent_jest_alternatives() {
        let agent = WebAgent::new();
        let knowledge = agent.knowledge();

        let jest_rec = knowledge
            .technology_recommendations
            .iter()
            .find(|r| r.technology == "Jest")
            .expect("Jest recommendation not found");

        assert!(jest_rec.alternatives.contains(&"Vitest".to_string()));
        assert!(jest_rec.alternatives.contains(&"Mocha".to_string()));
    }
}
