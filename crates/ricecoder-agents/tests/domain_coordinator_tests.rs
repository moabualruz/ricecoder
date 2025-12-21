//! Unit tests for domain coordinator

use ricecoder_agents::domain::coordinator::*;

#[test]
fn test_coordinator_creation() {
    let coordinator = DomainCoordinator::new();
    assert_eq!(std::mem::size_of_val(&coordinator), 0); // Zero-sized type
}

#[test]
fn test_route_request() {
    let coordinator = DomainCoordinator::new();

    let request = DomainRequest {
        id: "test-request".to_string(),
        domains: vec!["web-development".to_string()],
        content: "Build a web application".to_string(),
        context: std::collections::HashMap::new(),
    };

    let result = coordinator.route_request(&request);
    assert!(result.is_ok());

    let domains = result.unwrap();
    assert!(!domains.is_empty());
    assert!(domains.contains(&"web-development".to_string()));
}

#[test]
fn test_route_request_empty_domains() {
    let coordinator = DomainCoordinator::new();

    let request = DomainRequest {
        id: "test-request".to_string(),
        domains: vec![],
        content: "Build an application".to_string(),
        context: std::collections::HashMap::new(),
    };

    let result = coordinator.route_request(&request);
    assert!(result.is_err()); // Should fail with empty domains
}

#[test]
fn test_domain_request_formatting() {
    let request = DomainRequest {
        id: "test-request".to_string(),
        domains: vec!["web".to_string(), "api".to_string()],
        content: "Build something".to_string(),
        context: std::collections::HashMap::new(),
    };

    let formatted = format!("{}", request);
    assert!(formatted.contains("test-request"));
    assert!(formatted.contains("Build something"));
}

#[test]
fn test_domain_request_debug_formatting() {
    let request = DomainRequest {
        id: "test-request".to_string(),
        domains: vec!["web".to_string()],
        content: "Test content".to_string(),
        context: std::collections::HashMap::new(),
    };

    let debug_str = format!("{:?}", request);
    assert!(debug_str.contains("DomainRequest"));
    assert!(debug_str.contains("test-request"));
}

#[test]
fn test_coordinator_creation() {
    let coordinator = DomainCoordinator::new();
    assert_eq!(std::mem::size_of_val(&coordinator), 0); // Zero-sized type
}

#[test]
fn test_domain_analysis() {
    let coordinator = DomainCoordinator::new();

    let analysis = coordinator.analyze_domain("web-development");
    assert!(analysis.is_ok());

    let result = analysis.unwrap();
    assert!(!result.recommendations.is_empty());
    assert!(!result.insights.is_empty());
}

#[test]
fn test_domain_analysis_unknown_domain() {
    let coordinator = DomainCoordinator::new();

    let analysis = coordinator.analyze_domain("unknown-domain");
    assert!(analysis.is_ok()); // Should handle unknown domains gracefully

    let result = analysis.unwrap();
    assert!(result.recommendations.is_empty());
}

#[test]
fn test_technology_recommendations() {
    let coordinator = DomainCoordinator::new();

    let recommendations = coordinator.recommend_technologies("web-development", &["frontend"]);
    assert!(recommendations.is_ok());

    let result = recommendations.unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_technology_recommendations_unknown_domain() {
    let coordinator = DomainCoordinator::new();

    let recommendations = coordinator.recommend_technologies("unknown", &["test"]);
    assert!(recommendations.is_ok());

    let result = recommendations.unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_domain_patterns() {
    let coordinator = DomainCoordinator::new();

    let patterns = coordinator.get_domain_patterns("web-development");
    assert!(patterns.is_ok());

    let result = patterns.unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_domain_patterns_unknown_domain() {
    let coordinator = DomainCoordinator::new();

    let patterns = coordinator.get_domain_patterns("unknown");
    assert!(patterns.is_ok());

    let result = patterns.unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_best_practices() {
    let coordinator = DomainCoordinator::new();

    let practices = coordinator.get_best_practices("web-development");
    assert!(practices.is_ok());

    let result = practices.unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_best_practices_unknown_domain() {
    let coordinator = DomainCoordinator::new();

    let practices = coordinator.get_best_practices("unknown");
    assert!(practices.is_ok());

    let result = practices.unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_architecture_patterns() {
    let coordinator = DomainCoordinator::new();

    let patterns = coordinator.suggest_architecture_patterns("web-development", &["scalability"]);
    assert!(patterns.is_ok());

    let result = patterns.unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_architecture_patterns_unknown_domain() {
    let coordinator = DomainCoordinator::new();

    let patterns = coordinator.suggest_architecture_patterns("unknown", &["test"]);
    assert!(patterns.is_ok());

    let result = patterns.unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_integration_strategies() {
    let coordinator = DomainCoordinator::new();

    let strategies = coordinator.get_integration_strategies("web-development", &["api"]);
    assert!(strategies.is_ok());

    let result = strategies.unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_integration_strategies_unknown_domain() {
    let coordinator = DomainCoordinator::new();

    let strategies = coordinator.get_integration_strategies("unknown", &["test"]);
    assert!(strategies.is_ok());

    let result = strategies.unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_domain_expertise_assessment() {
    let coordinator = DomainCoordinator::new();

    let assessment =
        coordinator.assess_domain_expertise("web-development", &["react", "typescript"]);
    assert!(assessment.is_ok());

    let result = assessment.unwrap();
    assert!(result.score >= 0.0 && result.score <= 1.0);
}

#[test]
fn test_domain_expertise_assessment_unknown_domain() {
    let coordinator = DomainCoordinator::new();

    let assessment = coordinator.assess_domain_expertise("unknown", &["test"]);
    assert!(assessment.is_ok());

    let result = assessment.unwrap();
    assert_eq!(result.score, 0.0); // Unknown domain should have 0 expertise
}

#[test]
fn test_learning_path() {
    let coordinator = DomainCoordinator::new();

    let path = coordinator.generate_learning_path("web-development", "beginner");
    assert!(path.is_ok());

    let result = path.unwrap();
    assert!(!result.topics.is_empty());
}

#[test]
fn test_learning_path_unknown_domain() {
    let coordinator = DomainCoordinator::new();

    let path = coordinator.generate_learning_path("unknown", "beginner");
    assert!(path.is_ok());

    let result = path.unwrap();
    assert!(result.topics.is_empty());
}

#[test]
fn test_code_review_guidelines() {
    let coordinator = DomainCoordinator::new();

    let guidelines = coordinator.get_code_review_guidelines("web-development");
    assert!(guidelines.is_ok());

    let result = guidelines.unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_code_review_guidelines_unknown_domain() {
    let coordinator = DomainCoordinator::new();

    let guidelines = coordinator.get_code_review_guidelines("unknown");
    assert!(guidelines.is_ok());

    let result = guidelines.unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_performance_optimization_tips() {
    let coordinator = DomainCoordinator::new();

    let tips = coordinator.get_performance_optimization_tips("web-development");
    assert!(tips.is_ok());

    let result = tips.unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_performance_optimization_tips_unknown_domain() {
    let coordinator = DomainCoordinator::new();

    let tips = coordinator.get_performance_optimization_tips("unknown");
    assert!(tips.is_ok());

    let result = tips.unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_security_considerations() {
    let coordinator = DomainCoordinator::new();

    let considerations = coordinator.get_security_considerations("web-development");
    assert!(considerations.is_ok());

    let result = considerations.unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_security_considerations_unknown_domain() {
    let coordinator = DomainCoordinator::new();

    let considerations = coordinator.get_security_considerations("unknown");
    assert!(considerations.is_ok());

    let result = considerations.unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_domain_recommendation_formatting() {
    let recommendation = create_test_recommendation("test-domain");

    let formatted = format!("{}", recommendation);
    assert!(formatted.contains("test-domain"));
    assert!(formatted.contains("Test recommendation"));
}

#[test]
fn test_domain_recommendation_debug_formatting() {
    let recommendation = create_test_recommendation("test-domain");

    let debug_str = format!("{:?}", recommendation);
    assert!(debug_str.contains("Recommendation"));
    assert!(debug_str.contains("test-domain"));
}

#[test]
fn test_domain_analysis_result_formatting() {
    let coordinator = DomainCoordinator::new();
    let analysis = coordinator.analyze_domain("web-development").unwrap();

    let formatted = format!("{}", analysis);
    assert!(formatted.contains("Domain Analysis"));
    assert!(formatted.contains("web-development"));
}

#[test]
fn test_domain_analysis_result_debug_formatting() {
    let coordinator = DomainCoordinator::new();
    let analysis = coordinator.analyze_domain("web-development").unwrap();

    let debug_str = format!("{:?}", analysis);
    assert!(debug_str.contains("DomainAnalysisResult"));
}

#[test]
fn test_technology_recommendation_formatting() {
    let coordinator = DomainCoordinator::new();
    let recommendations = coordinator
        .recommend_technologies("web-development", &["frontend"])
        .unwrap();

    if !recommendations.is_empty() {
        let formatted = format!("{}", recommendations[0]);
        assert!(!formatted.is_empty());
    }
}

#[test]
fn test_domain_pattern_formatting() {
    let coordinator = DomainCoordinator::new();
    let patterns = coordinator.get_domain_patterns("web-development").unwrap();

    if !patterns.is_empty() {
        let formatted = format!("{}", patterns[0]);
        assert!(!formatted.is_empty());
    }
}

#[test]
fn test_best_practice_formatting() {
    let coordinator = DomainCoordinator::new();
    let practices = coordinator.get_best_practices("web-development").unwrap();

    if !practices.is_empty() {
        let formatted = format!("{}", practices[0]);
        assert!(!formatted.is_empty());
    }
}

#[test]
fn test_architecture_pattern_formatting() {
    let coordinator = DomainCoordinator::new();
    let patterns = coordinator
        .suggest_architecture_patterns("web-development", &["scalability"])
        .unwrap();

    if !patterns.is_empty() {
        let formatted = format!("{}", patterns[0]);
        assert!(!formatted.is_empty());
    }
}

#[test]
fn test_integration_strategy_formatting() {
    let coordinator = DomainCoordinator::new();
    let strategies = coordinator
        .get_integration_strategies("web-development", &["api"])
        .unwrap();

    if !strategies.is_empty() {
        let formatted = format!("{}", strategies[0]);
        assert!(!formatted.is_empty());
    }
}

#[test]
fn test_expertise_assessment_formatting() {
    let coordinator = DomainCoordinator::new();
    let assessment = coordinator
        .assess_domain_expertise("web-development", &["react"])
        .unwrap();

    let formatted = format!("{}", assessment);
    assert!(formatted.contains("Expertise Assessment"));
    assert!(formatted.contains("web-development"));
}

#[test]
fn test_learning_path_formatting() {
    let coordinator = DomainCoordinator::new();
    let path = coordinator
        .generate_learning_path("web-development", "beginner")
        .unwrap();

    let formatted = format!("{}", path);
    assert!(formatted.contains("Learning Path"));
    assert!(formatted.contains("web-development"));
}

#[test]
fn test_code_review_guideline_formatting() {
    let coordinator = DomainCoordinator::new();
    let guidelines = coordinator
        .get_code_review_guidelines("web-development")
        .unwrap();

    if !guidelines.is_empty() {
        let formatted = format!("{}", guidelines[0]);
        assert!(!formatted.is_empty());
    }
}

#[test]
fn test_performance_tip_formatting() {
    let coordinator = DomainCoordinator::new();
    let tips = coordinator
        .get_performance_optimization_tips("web-development")
        .unwrap();

    if !tips.is_empty() {
        let formatted = format!("{}", tips[0]);
        assert!(!formatted.is_empty());
    }
}

#[test]
fn test_security_consideration_formatting() {
    let coordinator = DomainCoordinator::new();
    let considerations = coordinator
        .get_security_considerations("web-development")
        .unwrap();

    if !considerations.is_empty() {
        let formatted = format!("{}", considerations[0]);
        assert!(!formatted.is_empty());
    }
}
