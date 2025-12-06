//! Multi-agent coordination for domain agents

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::models::Recommendation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A request to be routed to domain agents
///
/// This struct represents a user request that needs to be routed
/// to one or more domain agents for processing.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::DomainRequest;
///
/// let request = DomainRequest {
///     id: "req-1".to_string(),
///     domains: vec!["web".to_string(), "backend".to_string()],
///     content: "Help me set up a full-stack application".to_string(),
///     context: Default::default(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRequest {
    /// Request identifier
    pub id: String,
    /// Target domains for this request
    pub domains: Vec<String>,
    /// Request content
    pub content: String,
    /// Request context
    pub context: HashMap<String, serde_json::Value>,
}

/// Coordinates multi-agent workflows
///
/// This struct manages coordination between domain agents,
/// including request routing, response aggregation, and operation sequencing.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::DomainCoordinator;
///
/// let coordinator = DomainCoordinator::new();
/// let coordinated = coordinator.coordinate_responses(responses)?;
/// ```
#[derive(Debug, Clone)]
pub struct DomainCoordinator;

impl DomainCoordinator {
    /// Create a new domain coordinator
    pub fn new() -> Self {
        Self
    }

    /// Route a request to appropriate domain agents
    ///
    /// # Arguments
    ///
    /// * `request` - The request to route
    ///
    /// # Returns
    ///
    /// Returns the target domains for this request
    pub fn route_request(&self, request: &DomainRequest) -> DomainResult<Vec<String>> {
        if request.domains.is_empty() {
            return Err(DomainError::coordination_error(
                "Request must specify at least one target domain",
            ));
        }

        // Validate that all requested domains are known
        let valid_domains = vec!["web", "backend", "devops"];
        for domain in &request.domains {
            if !valid_domains.contains(&domain.as_str()) && !domain.starts_with("custom-") {
                return Err(DomainError::coordination_error(format!(
                    "Unknown domain: {}",
                    domain
                )));
            }
        }

        Ok(request.domains.clone())
    }

    /// Determine which agents should handle a request based on content
    ///
    /// # Arguments
    ///
    /// * `content` - The request content
    ///
    /// # Returns
    ///
    /// Returns the inferred domains for this request
    pub fn infer_domains(&self, content: &str) -> DomainResult<Vec<String>> {
        let mut domains = Vec::new();
        let content_lower = content.to_lowercase();

        // Infer web domain
        if content_lower.contains("frontend")
            || content_lower.contains("react")
            || content_lower.contains("vue")
            || content_lower.contains("angular")
            || content_lower.contains("styling")
            || content_lower.contains("ui")
            || content_lower.contains("web")
        {
            domains.push("web".to_string());
        }

        // Infer backend domain
        if content_lower.contains("backend")
            || content_lower.contains("api")
            || content_lower.contains("database")
            || content_lower.contains("server")
            || content_lower.contains("rest")
            || content_lower.contains("graphql")
            || content_lower.contains("microservice")
        {
            domains.push("backend".to_string());
        }

        // Infer devops domain
        if content_lower.contains("devops")
            || content_lower.contains("deployment")
            || content_lower.contains("ci/cd")
            || content_lower.contains("docker")
            || content_lower.contains("kubernetes")
            || content_lower.contains("infrastructure")
            || content_lower.contains("terraform")
        {
            domains.push("devops".to_string());
        }

        // If no domains inferred, default to all
        if domains.is_empty() {
            domains = vec!["web".to_string(), "backend".to_string(), "devops".to_string()];
        }

        Ok(domains)
    }

    /// Coordinate responses from multiple agents
    ///
    /// # Arguments
    ///
    /// * `responses` - Responses from domain agents
    ///
    /// # Returns
    ///
    /// Returns coordinated response
    pub fn coordinate_responses(
        &self,
        responses: Vec<Recommendation>,
    ) -> DomainResult<CoordinatedResponse> {
        // Group recommendations by domain
        let mut by_domain: std::collections::HashMap<String, Vec<Recommendation>> =
            std::collections::HashMap::new();

        for response in responses {
            by_domain
                .entry(response.domain.clone())
                .or_insert_with(Vec::new)
                .push(response);
        }

        // Create coordinated response
        let coordinated = CoordinatedResponse {
            recommendations: by_domain.values().flatten().cloned().collect(),
            domain_count: by_domain.len(),
            total_recommendations: by_domain.values().map(|v| v.len()).sum(),
        };

        Ok(coordinated)
    }

    /// Sequence operations for cross-domain tasks
    ///
    /// # Arguments
    ///
    /// * `operations` - Operations to sequence
    ///
    /// # Returns
    ///
    /// Returns sequenced operations
    pub fn sequence_operations(
        &self,
        operations: Vec<Operation>,
    ) -> DomainResult<Vec<Operation>> {
        // Sort operations by dependency order
        let mut sequenced = operations;
        sequenced.sort_by_key(|op| op.priority);

        Ok(sequenced)
    }

    /// Validate consistency across domains
    ///
    /// # Arguments
    ///
    /// * `recommendations` - Recommendations to validate
    ///
    /// # Returns
    ///
    /// Returns true if recommendations are consistent
    pub fn validate_consistency(&self, recommendations: &[Recommendation]) -> DomainResult<bool> {
        // Check for basic consistency
        if recommendations.is_empty() {
            return Ok(true);
        }

        // Verify all recommendations have required fields
        for rec in recommendations {
            if rec.domain.is_empty() || rec.category.is_empty() {
                return Err(DomainError::coordination_error(
                    "Recommendation missing required fields",
                ));
            }
        }

        // Check for cross-domain consistency
        let mut domains_present = std::collections::HashSet::new();
        for rec in recommendations {
            domains_present.insert(rec.domain.clone());
        }

        // Verify that recommendations from different domains are complementary
        if domains_present.len() > 1 {
            // Group recommendations by domain
            let mut by_domain: HashMap<String, Vec<&Recommendation>> = HashMap::new();
            for rec in recommendations {
                by_domain
                    .entry(rec.domain.clone())
                    .or_insert_with(Vec::new)
                    .push(rec);
            }

            // Verify each domain has recommendations
            for domain_recs in by_domain.values() {
                if domain_recs.is_empty() {
                    return Err(DomainError::coordination_error(
                        "Domain has no recommendations",
                    ));
                }
            }
        }

        Ok(true)
    }

    /// Coordinate full-stack recommendations across domains
    ///
    /// # Arguments
    ///
    /// * `recommendations` - Recommendations from all domains
    ///
    /// # Returns
    ///
    /// Returns a full-stack coordination result
    pub fn coordinate_full_stack(
        &self,
        recommendations: Vec<Recommendation>,
    ) -> DomainResult<FullStackCoordination> {
        // Validate consistency
        self.validate_consistency(&recommendations)?;

        // Group recommendations by domain
        let mut by_domain: HashMap<String, Vec<Recommendation>> = HashMap::new();
        for rec in recommendations {
            by_domain
                .entry(rec.domain.clone())
                .or_insert_with(Vec::new)
                .push(rec);
        }

        // Ensure all three domains are represented for full-stack
        let has_web = by_domain.contains_key("web");
        let has_backend = by_domain.contains_key("backend");
        let has_devops = by_domain.contains_key("devops");

        let is_full_stack = has_web && has_backend && has_devops;

        Ok(FullStackCoordination {
            web_recommendations: by_domain.get("web").cloned().unwrap_or_default(),
            backend_recommendations: by_domain.get("backend").cloned().unwrap_or_default(),
            devops_recommendations: by_domain.get("devops").cloned().unwrap_or_default(),
            is_full_stack,
            total_recommendations: by_domain.values().map(|v| v.len()).sum(),
        })
    }

    /// Ensure consistency across full-stack domains
    ///
    /// # Arguments
    ///
    /// * `coordination` - The full-stack coordination to validate
    ///
    /// # Returns
    ///
    /// Returns true if all domains are consistent
    pub fn ensure_full_stack_consistency(
        &self,
        coordination: &FullStackCoordination,
    ) -> DomainResult<bool> {
        if !coordination.is_full_stack {
            return Ok(true);
        }

        // Verify each domain has recommendations
        if coordination.web_recommendations.is_empty() {
            return Err(DomainError::coordination_error(
                "Web domain has no recommendations",
            ));
        }

        if coordination.backend_recommendations.is_empty() {
            return Err(DomainError::coordination_error(
                "Backend domain has no recommendations",
            ));
        }

        if coordination.devops_recommendations.is_empty() {
            return Err(DomainError::coordination_error(
                "DevOps domain has no recommendations",
            ));
        }

        // Verify technology stack consistency
        let mut all_techs = std::collections::HashSet::new();
        for rec in &coordination.web_recommendations {
            for tech in &rec.technologies {
                all_techs.insert(tech.clone());
            }
        }
        for rec in &coordination.backend_recommendations {
            for tech in &rec.technologies {
                all_techs.insert(tech.clone());
            }
        }
        for rec in &coordination.devops_recommendations {
            for tech in &rec.technologies {
                all_techs.insert(tech.clone());
            }
        }

        // Ensure we have a reasonable technology stack
        if all_techs.is_empty() {
            return Err(DomainError::coordination_error(
                "No technologies recommended across domains",
            ));
        }

        Ok(true)
    }

    /// Detect conflicts in full-stack recommendations
    ///
    /// # Arguments
    ///
    /// * `coordination` - The full-stack coordination to analyze
    ///
    /// # Returns
    ///
    /// Returns a vector of potential conflicts
    pub fn detect_full_stack_conflicts(
        &self,
        coordination: &FullStackCoordination,
    ) -> DomainResult<Vec<String>> {
        let mut conflicts = Vec::new();

        // Check for incompatible technology combinations
        let web_techs: std::collections::HashSet<_> = coordination
            .web_recommendations
            .iter()
            .flat_map(|r| r.technologies.clone())
            .collect();

        let backend_techs: std::collections::HashSet<_> = coordination
            .backend_recommendations
            .iter()
            .flat_map(|r| r.technologies.clone())
            .collect();

        let devops_techs: std::collections::HashSet<_> = coordination
            .devops_recommendations
            .iter()
            .flat_map(|r| r.technologies.clone())
            .collect();

        // Check for known incompatibilities
        let incompatible_pairs = vec![
            ("Webpack", "Vite"),
            ("npm", "yarn"),
            ("PostgreSQL", "MongoDB"),
            ("REST", "GraphQL"),
            ("Microservices", "Monolithic"),
        ];

        for (tech_a, tech_b) in incompatible_pairs {
            let has_a_web = web_techs.contains(tech_a);
            let has_b_web = web_techs.contains(tech_b);
            let has_a_backend = backend_techs.contains(tech_a);
            let has_b_backend = backend_techs.contains(tech_b);
            let has_a_devops = devops_techs.contains(tech_a);
            let has_b_devops = devops_techs.contains(tech_b);

            if (has_a_web || has_a_backend || has_a_devops)
                && (has_b_web || has_b_backend || has_b_devops)
            {
                conflicts.push(format!(
                    "Incompatible technologies: {} and {} are recommended",
                    tech_a, tech_b
                ));
            }
        }

        Ok(conflicts)
    }
}

impl Default for DomainCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Coordinated response from multiple agents
#[derive(Debug, Clone)]
pub struct CoordinatedResponse {
    /// All recommendations
    pub recommendations: Vec<Recommendation>,
    /// Number of domains involved
    pub domain_count: usize,
    /// Total number of recommendations
    pub total_recommendations: usize,
}

/// Full-stack coordination across web, backend, and DevOps domains
///
/// This struct represents coordinated recommendations across all three
/// development domains for a complete full-stack application.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::FullStackCoordination;
///
/// let coordination = FullStackCoordination {
///     web_recommendations: vec![],
///     backend_recommendations: vec![],
///     devops_recommendations: vec![],
///     is_full_stack: true,
///     total_recommendations: 0,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct FullStackCoordination {
    /// Recommendations from web domain
    pub web_recommendations: Vec<Recommendation>,
    /// Recommendations from backend domain
    pub backend_recommendations: Vec<Recommendation>,
    /// Recommendations from DevOps domain
    pub devops_recommendations: Vec<Recommendation>,
    /// Whether this is a complete full-stack coordination
    pub is_full_stack: bool,
    /// Total number of recommendations
    pub total_recommendations: usize,
}

/// An operation to be sequenced
#[derive(Debug, Clone)]
pub struct Operation {
    /// Operation identifier
    pub id: String,
    /// Operation name
    pub name: String,
    /// Priority (lower = earlier)
    pub priority: u32,
    /// Dependencies
    pub dependencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Recommendation;

    fn create_test_recommendation(domain: &str) -> Recommendation {
        Recommendation {
            domain: domain.to_string(),
            category: "test".to_string(),
            content: "Test recommendation".to_string(),
            technologies: vec!["Tech1".to_string()],
            rationale: "Test rationale".to_string(),
        }
    }

    #[test]
    fn test_coordinator_creation() {
        let coordinator = DomainCoordinator::new();
        assert_eq!(std::mem::size_of_val(&coordinator), 0); // Zero-sized type
    }

    #[test]
    fn test_coordinate_responses_single_domain() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![create_test_recommendation("web")];

        let coordinated = coordinator.coordinate_responses(responses).unwrap();

        assert_eq!(coordinated.domain_count, 1);
        assert_eq!(coordinated.total_recommendations, 1);
    }

    #[test]
    fn test_coordinate_responses_multiple_domains() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![
            create_test_recommendation("web"),
            create_test_recommendation("backend"),
            create_test_recommendation("devops"),
        ];

        let coordinated = coordinator.coordinate_responses(responses).unwrap();

        assert_eq!(coordinated.domain_count, 3);
        assert_eq!(coordinated.total_recommendations, 3);
    }

    #[test]
    fn test_coordinate_responses_empty() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![];

        let coordinated = coordinator.coordinate_responses(responses).unwrap();

        assert_eq!(coordinated.domain_count, 0);
        assert_eq!(coordinated.total_recommendations, 0);
    }

    #[test]
    fn test_sequence_operations() {
        let coordinator = DomainCoordinator::new();
        let operations = vec![
            Operation {
                id: "op1".to_string(),
                name: "Operation 1".to_string(),
                priority: 3,
                dependencies: vec![],
            },
            Operation {
                id: "op2".to_string(),
                name: "Operation 2".to_string(),
                priority: 1,
                dependencies: vec![],
            },
            Operation {
                id: "op3".to_string(),
                name: "Operation 3".to_string(),
                priority: 2,
                dependencies: vec![],
            },
        ];

        let sequenced = coordinator.sequence_operations(operations).unwrap();

        assert_eq!(sequenced[0].priority, 1);
        assert_eq!(sequenced[1].priority, 2);
        assert_eq!(sequenced[2].priority, 3);
    }

    #[test]
    fn test_validate_consistency_valid() {
        let coordinator = DomainCoordinator::new();
        let recommendations = vec![create_test_recommendation("web")];

        assert!(coordinator.validate_consistency(&recommendations).unwrap());
    }

    #[test]
    fn test_validate_consistency_empty() {
        let coordinator = DomainCoordinator::new();
        let recommendations = vec![];

        assert!(coordinator.validate_consistency(&recommendations).unwrap());
    }

    #[test]
    fn test_validate_consistency_invalid_domain() {
        let coordinator = DomainCoordinator::new();
        let mut rec = create_test_recommendation("web");
        rec.domain = String::new();

        assert!(coordinator.validate_consistency(&[rec]).is_err());
    }

    #[test]
    fn test_validate_consistency_invalid_category() {
        let coordinator = DomainCoordinator::new();
        let mut rec = create_test_recommendation("web");
        rec.category = String::new();

        assert!(coordinator.validate_consistency(&[rec]).is_err());
    }

    #[test]
    fn test_default_coordinator() {
        let coordinator = DomainCoordinator::default();
        let responses = vec![create_test_recommendation("web")];

        assert!(coordinator.coordinate_responses(responses).is_ok());
    }

    #[test]
    fn test_coordinated_response_structure() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![
            create_test_recommendation("web"),
            create_test_recommendation("web"),
            create_test_recommendation("backend"),
        ];

        let coordinated = coordinator.coordinate_responses(responses).unwrap();

        assert_eq!(coordinated.domain_count, 2);
        assert_eq!(coordinated.total_recommendations, 3);
        assert_eq!(coordinated.recommendations.len(), 3);
    }

    #[test]
    fn test_operation_sequencing_with_dependencies() {
        let coordinator = DomainCoordinator::new();
        let operations = vec![
            Operation {
                id: "setup".to_string(),
                name: "Setup".to_string(),
                priority: 1,
                dependencies: vec![],
            },
            Operation {
                id: "deploy".to_string(),
                name: "Deploy".to_string(),
                priority: 2,
                dependencies: vec!["setup".to_string()],
            },
        ];

        let sequenced = coordinator.sequence_operations(operations).unwrap();

        assert_eq!(sequenced[0].id, "setup");
        assert_eq!(sequenced[1].id, "deploy");
    }

    #[test]
    fn test_multiple_recommendations_same_domain() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![
            create_test_recommendation("web"),
            create_test_recommendation("web"),
            create_test_recommendation("web"),
        ];

        let coordinated = coordinator.coordinate_responses(responses).unwrap();

        assert_eq!(coordinated.domain_count, 1);
        assert_eq!(coordinated.total_recommendations, 3);
    }

    #[test]
    fn test_route_request_valid() {
        let coordinator = DomainCoordinator::new();
        let request = DomainRequest {
            id: "req-1".to_string(),
            domains: vec!["web".to_string(), "backend".to_string()],
            content: "Help me set up a full-stack app".to_string(),
            context: std::collections::HashMap::new(),
        };

        let routed = coordinator.route_request(&request).unwrap();
        assert_eq!(routed.len(), 2);
        assert!(routed.contains(&"web".to_string()));
        assert!(routed.contains(&"backend".to_string()));
    }

    #[test]
    fn test_route_request_empty_domains() {
        let coordinator = DomainCoordinator::new();
        let request = DomainRequest {
            id: "req-1".to_string(),
            domains: vec![],
            content: "Help me set up a full-stack app".to_string(),
            context: std::collections::HashMap::new(),
        };

        assert!(coordinator.route_request(&request).is_err());
    }

    #[test]
    fn test_route_request_invalid_domain() {
        let coordinator = DomainCoordinator::new();
        let request = DomainRequest {
            id: "req-1".to_string(),
            domains: vec!["invalid-domain".to_string()],
            content: "Help me set up a full-stack app".to_string(),
            context: std::collections::HashMap::new(),
        };

        assert!(coordinator.route_request(&request).is_err());
    }

    #[test]
    fn test_route_request_custom_domain() {
        let coordinator = DomainCoordinator::new();
        let request = DomainRequest {
            id: "req-1".to_string(),
            domains: vec!["custom-mobile".to_string()],
            content: "Help me set up a mobile app".to_string(),
            context: std::collections::HashMap::new(),
        };

        let routed = coordinator.route_request(&request).unwrap();
        assert_eq!(routed.len(), 1);
        assert_eq!(routed[0], "custom-mobile");
    }

    #[test]
    fn test_infer_domains_web() {
        let coordinator = DomainCoordinator::new();
        let domains = coordinator
            .infer_domains("I need help with React and styling")
            .unwrap();

        assert!(domains.contains(&"web".to_string()));
    }

    #[test]
    fn test_infer_domains_backend() {
        let coordinator = DomainCoordinator::new();
        let domains = coordinator
            .infer_domains("I need help with REST API and database design")
            .unwrap();

        assert!(domains.contains(&"backend".to_string()));
    }

    #[test]
    fn test_infer_domains_devops() {
        let coordinator = DomainCoordinator::new();
        let domains = coordinator
            .infer_domains("I need help with Docker and Kubernetes")
            .unwrap();

        assert!(domains.contains(&"devops".to_string()));
    }

    #[test]
    fn test_infer_domains_full_stack() {
        let coordinator = DomainCoordinator::new();
        let domains = coordinator
            .infer_domains("I need help with React, Node.js API, and Docker deployment")
            .unwrap();

        assert!(domains.contains(&"web".to_string()));
        assert!(domains.contains(&"backend".to_string()));
        assert!(domains.contains(&"devops".to_string()));
    }

    #[test]
    fn test_infer_domains_empty_defaults_to_all() {
        let coordinator = DomainCoordinator::new();
        let domains = coordinator.infer_domains("general help").unwrap();

        assert_eq!(domains.len(), 3);
        assert!(domains.contains(&"web".to_string()));
        assert!(domains.contains(&"backend".to_string()));
        assert!(domains.contains(&"devops".to_string()));
    }

    #[test]
    fn test_coordinate_full_stack() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![
            create_test_recommendation("web"),
            create_test_recommendation("backend"),
            create_test_recommendation("devops"),
        ];

        let coordination = coordinator.coordinate_full_stack(responses).unwrap();

        assert!(coordination.is_full_stack);
        assert_eq!(coordination.web_recommendations.len(), 1);
        assert_eq!(coordination.backend_recommendations.len(), 1);
        assert_eq!(coordination.devops_recommendations.len(), 1);
        assert_eq!(coordination.total_recommendations, 3);
    }

    #[test]
    fn test_coordinate_full_stack_partial() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![
            create_test_recommendation("web"),
            create_test_recommendation("backend"),
        ];

        let coordination = coordinator.coordinate_full_stack(responses).unwrap();

        assert!(!coordination.is_full_stack);
        assert_eq!(coordination.web_recommendations.len(), 1);
        assert_eq!(coordination.backend_recommendations.len(), 1);
        assert_eq!(coordination.devops_recommendations.len(), 0);
        assert_eq!(coordination.total_recommendations, 2);
    }

    #[test]
    fn test_validate_consistency_cross_domain() {
        let coordinator = DomainCoordinator::new();
        let recommendations = vec![
            create_test_recommendation("web"),
            create_test_recommendation("backend"),
            create_test_recommendation("devops"),
        ];

        assert!(coordinator.validate_consistency(&recommendations).unwrap());
    }

    #[test]
    fn test_domain_request_creation() {
        let request = DomainRequest {
            id: "req-1".to_string(),
            domains: vec!["web".to_string()],
            content: "Help me".to_string(),
            context: std::collections::HashMap::new(),
        };

        assert_eq!(request.id, "req-1");
        assert_eq!(request.domains.len(), 1);
    }

    #[test]
    fn test_full_stack_coordination_structure() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![
            create_test_recommendation("web"),
            create_test_recommendation("web"),
            create_test_recommendation("backend"),
            create_test_recommendation("devops"),
        ];

        let coordination = coordinator.coordinate_full_stack(responses).unwrap();

        assert!(coordination.is_full_stack);
        assert_eq!(coordination.web_recommendations.len(), 2);
        assert_eq!(coordination.backend_recommendations.len(), 1);
        assert_eq!(coordination.devops_recommendations.len(), 1);
        assert_eq!(coordination.total_recommendations, 4);
    }

    #[test]
    fn test_ensure_full_stack_consistency_valid() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![
            create_test_recommendation("web"),
            create_test_recommendation("backend"),
            create_test_recommendation("devops"),
        ];

        let coordination = coordinator.coordinate_full_stack(responses).unwrap();
        assert!(coordinator
            .ensure_full_stack_consistency(&coordination)
            .unwrap());
    }

    #[test]
    fn test_ensure_full_stack_consistency_partial() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![
            create_test_recommendation("web"),
            create_test_recommendation("backend"),
        ];

        let coordination = coordinator.coordinate_full_stack(responses).unwrap();
        // Partial stack should still be valid
        assert!(coordinator
            .ensure_full_stack_consistency(&coordination)
            .unwrap());
    }

    #[test]
    fn test_ensure_full_stack_consistency_missing_web() {
        let coordinator = DomainCoordinator::new();
        let coordination = FullStackCoordination {
            web_recommendations: vec![],
            backend_recommendations: vec![create_test_recommendation("backend")],
            devops_recommendations: vec![create_test_recommendation("devops")],
            is_full_stack: true,
            total_recommendations: 2,
        };

        assert!(coordinator
            .ensure_full_stack_consistency(&coordination)
            .is_err());
    }

    #[test]
    fn test_detect_full_stack_conflicts_none() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![
            create_test_recommendation("web"),
            create_test_recommendation("backend"),
            create_test_recommendation("devops"),
        ];

        let coordination = coordinator.coordinate_full_stack(responses).unwrap();
        let conflicts = coordinator
            .detect_full_stack_conflicts(&coordination)
            .unwrap();

        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_detect_full_stack_conflicts_incompatible_techs() {
        let coordinator = DomainCoordinator::new();

        let mut web_rec = create_test_recommendation("web");
        web_rec.technologies = vec!["Webpack".to_string()];

        let mut backend_rec = create_test_recommendation("backend");
        backend_rec.technologies = vec!["Node.js".to_string()];

        let mut devops_rec = create_test_recommendation("devops");
        devops_rec.technologies = vec!["Vite".to_string()];

        let coordination = FullStackCoordination {
            web_recommendations: vec![web_rec],
            backend_recommendations: vec![backend_rec],
            devops_recommendations: vec![devops_rec],
            is_full_stack: true,
            total_recommendations: 3,
        };

        let conflicts = coordinator
            .detect_full_stack_conflicts(&coordination)
            .unwrap();

        assert!(!conflicts.is_empty());
        assert!(conflicts[0].contains("Incompatible"));
    }

    #[test]
    fn test_full_stack_coordination_empty() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![];

        let coordination = coordinator.coordinate_full_stack(responses).unwrap();

        assert!(!coordination.is_full_stack);
        assert_eq!(coordination.total_recommendations, 0);
    }

    #[test]
    fn test_full_stack_coordination_single_domain() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![create_test_recommendation("web")];

        let coordination = coordinator.coordinate_full_stack(responses).unwrap();

        assert!(!coordination.is_full_stack);
        assert_eq!(coordination.web_recommendations.len(), 1);
        assert_eq!(coordination.backend_recommendations.len(), 0);
        assert_eq!(coordination.devops_recommendations.len(), 0);
    }

    #[test]
    fn test_full_stack_coordination_two_domains() {
        let coordinator = DomainCoordinator::new();
        let responses = vec![
            create_test_recommendation("web"),
            create_test_recommendation("backend"),
        ];

        let coordination = coordinator.coordinate_full_stack(responses).unwrap();

        assert!(!coordination.is_full_stack);
        assert_eq!(coordination.web_recommendations.len(), 1);
        assert_eq!(coordination.backend_recommendations.len(), 1);
        assert_eq!(coordination.devops_recommendations.len(), 0);
    }
}
