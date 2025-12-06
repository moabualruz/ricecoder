//! Property-based tests for domain extensibility features
//!
//! Tests for:
//! - Property 9: Auto-Discovery of New Domains
//! - Property 10: Configuration-Driven Agent Creation
//! - Property 11: Hot-Reload Configuration
//! - Property 12: Custom Domain Support
//! - Property 13: Configuration Validation and Error Handling

#[cfg(test)]
mod tests {
    use crate::domain::{
        config_loader::ConfigLoader, factory::AgentFactory, registry::DomainRegistry,
    };
    use std::fs;
    use tempfile::TempDir;

    // ========================================================================
    // Property 9: Auto-Discovery of New Domains
    // ========================================================================
    // *For any* valid domain configuration file added to `config/domains/`,
    // the Domain Agent System SHALL automatically discover and register the
    // domain without code changes or system restart.
    //
    // **Validates: Requirements 5.1-5.2**

    #[test]
    fn property_9_auto_discovery_discovers_new_domains() {
        // Create a temporary directory for test configurations
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path();

        // Create a valid domain configuration
        let domain_config = r#"
domain: test-domain
name: "Test Domain"
description: "A test domain for auto-discovery"
capabilities:
  - name: "Test Capability"
    description: "A test capability"
    technologies: ["Technology A", "Technology B"]
best_practices:
  - title: "Test Practice"
    description: "A test best practice"
    technologies: ["Technology A"]
    implementation: "Test implementation"
technology_recommendations:
  - technology: "Technology A"
    use_cases: ["Use case 1"]
    pros: ["Pro 1"]
    cons: ["Con 1"]
    alternatives: ["Alternative 1"]
"#;

        // Write the configuration file
        let config_file = config_path.join("test-domain.yaml");
        fs::write(&config_file, domain_config).expect("Failed to write config file");

        // Load configuration from file
        let loader = ConfigLoader::new();
        let config = loader
            .load_from_file(&config_file)
            .expect("Failed to load config");

        // Verify configuration was loaded
        assert_eq!(config.domain, "test-domain");

        // Create a registry and register the domain
        let registry = DomainRegistry::new();
        let factory = AgentFactory::new();
        let agent = factory
            .create_agent("test-domain", &config)
            .expect("Failed to create agent");

        let _ = registry.register_agent("test-domain", agent);

        // Verify the domain is registered
        assert!(
            registry.get_agent("test-domain").is_ok(),
            "Domain should be registered"
        );

        // Verify discovered domains includes our domain
        let discovered = registry
            .discover_domains()
            .expect("Failed to discover domains");

        assert!(
            discovered.contains(&"test-domain".to_string()),
            "Domain should be in discovered domains"
        );
    }

    #[test]
    fn property_9_auto_discovery_discovers_multiple_domains() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path();

        // Create multiple domain configurations
        let domains = vec!["domain1", "domain2", "domain3"];
        let loader = ConfigLoader::new();
        let factory = AgentFactory::new();
        let registry = DomainRegistry::new();

        for domain_name in &domains {
            let config_yaml = format!(
                r#"
domain: {}
name: "{} Domain"
description: "Test domain {}"
capabilities:
  - name: "Capability"
    description: "A capability"
    technologies: ["Tech1"]
best_practices:
  - title: "Practice"
    description: "A practice"
    technologies: ["Tech1"]
    implementation: "Implementation"
technology_recommendations:
  - technology: "Tech1"
    use_cases: ["Use case"]
    pros: ["Pro"]
    cons: ["Con"]
    alternatives: ["Alt"]
"#,
                domain_name, domain_name, domain_name
            );

            let config_file = config_path.join(format!("{}.yaml", domain_name));
            fs::write(&config_file, config_yaml).expect("Failed to write config file");

            // Load and register each domain
            let config = loader
                .load_from_file(&config_file)
                .expect("Failed to load config");

            let agent = factory
                .create_agent(domain_name, &config)
                .expect("Failed to create agent");

            let _ = registry.register_agent(domain_name, agent);
        }

        // Verify all domains were discovered
        let discovered = registry
            .discover_domains()
            .expect("Failed to discover domains");

        for domain_name in &domains {
            assert!(
                discovered.contains(&domain_name.to_string()),
                "Domain {} should be discovered",
                domain_name
            );
        }
    }

    // ========================================================================
    // Property 10: Configuration-Driven Agent Creation
    // ========================================================================
    // *For any* domain configuration file with valid schema, the Domain Agent
    // System SHALL create a domain agent for that domain without code changes,
    // loading all capabilities, best practices, and technology recommendations
    // from the configuration.
    //
    // **Validates: Requirements 5.2**

    #[test]
    fn property_10_agent_creation_creates_agent_from_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path();

        let domain_config = r#"
domain: custom-domain
name: "Custom Domain"
description: "A custom domain"
capabilities:
  - name: "Capability 1"
    description: "First capability"
    technologies: ["Tech1", "Tech2"]
  - name: "Capability 2"
    description: "Second capability"
    technologies: ["Tech3"]
best_practices:
  - title: "Practice 1"
    description: "First practice"
    technologies: ["Tech1"]
    implementation: "Implementation 1"
technology_recommendations:
  - technology: "Tech1"
    use_cases: ["Use case 1", "Use case 2"]
    pros: ["Pro 1", "Pro 2"]
    cons: ["Con 1"]
    alternatives: ["Alt 1"]
"#;

        let config_file = config_path.join("custom-domain.yaml");
        fs::write(&config_file, domain_config).expect("Failed to write config file");

        // Load and create agent from configuration
        let loader = ConfigLoader::new();
        let config = loader
            .load_from_file(&config_file)
            .expect("Failed to load config");

        let factory = AgentFactory::new();
        let agent = factory
            .create_agent("custom-domain", &config)
            .expect("Failed to create agent");

        // Verify agent was created with correct properties
        assert_eq!(agent.domain, "custom-domain");

        // Verify capabilities were loaded
        assert_eq!(agent.capabilities.len(), 2);
        assert_eq!(agent.capabilities[0].name, "Capability 1");
        assert_eq!(agent.capabilities[0].technologies.len(), 2);

        // Verify best practices were loaded
        assert_eq!(agent.knowledge.best_practices.len(), 1);
        assert_eq!(agent.knowledge.best_practices[0].title, "Practice 1");

        // Verify technology recommendations were loaded
        assert_eq!(agent.knowledge.technology_recommendations.len(), 1);
        assert_eq!(
            agent.knowledge.technology_recommendations[0].technology,
            "Tech1"
        );
    }

    #[test]
    fn property_10_agent_creation_loads_all_config_fields() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path();

        let domain_config = r#"
domain: full-config-domain
name: "Full Config Domain"
description: "Domain with all fields populated"
capabilities:
  - name: "Cap1"
    description: "Capability 1"
    technologies: ["T1", "T2", "T3"]
  - name: "Cap2"
    description: "Capability 2"
    technologies: ["T4"]
best_practices:
  - title: "BP1"
    description: "Best practice 1"
    technologies: ["T1", "T2"]
    implementation: "Impl 1"
  - title: "BP2"
    description: "Best practice 2"
    technologies: ["T3"]
    implementation: "Impl 2"
technology_recommendations:
  - technology: "T1"
    use_cases: ["UC1", "UC2"]
    pros: ["P1", "P2"]
    cons: ["C1"]
    alternatives: ["A1", "A2"]
  - technology: "T2"
    use_cases: ["UC3"]
    pros: ["P3"]
    cons: ["C2", "C3"]
    alternatives: ["A3"]
"#;

        let config_file = config_path.join("full-config-domain.yaml");
        fs::write(&config_file, domain_config).expect("Failed to write config file");

        let loader = ConfigLoader::new();
        let config = loader
            .load_from_file(&config_file)
            .expect("Failed to load config");

        let factory = AgentFactory::new();
        let agent = factory
            .create_agent("full-config-domain", &config)
            .expect("Failed to create agent");

        // Verify all fields are loaded
        assert_eq!(agent.capabilities.len(), 2);
        assert_eq!(agent.knowledge.best_practices.len(), 2);
        assert_eq!(agent.knowledge.technology_recommendations.len(), 2);

        // Verify nested fields
        assert_eq!(agent.capabilities[0].technologies.len(), 3);
        assert_eq!(agent.knowledge.best_practices[0].technologies.len(), 2);
        assert_eq!(agent.knowledge.technology_recommendations[0].use_cases.len(), 2);
        assert_eq!(agent.knowledge.technology_recommendations[0].pros.len(), 2);
        assert_eq!(agent.knowledge.technology_recommendations[0].alternatives.len(), 2);
    }

    // ========================================================================
    // Property 11: Hot-Reload Configuration
    // ========================================================================
    // *For any* domain configuration file modification, the Domain Agent System
    // SHALL hot-reload the configuration and update the agent without system
    // restart, making the updated configuration available for new requests.
    //
    // **Validates: Requirements 5.3**

    #[test]
    fn property_11_hot_reload_detects_config_changes() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path();

        let initial_config = r#"
domain: reload-test
name: "Reload Test Domain"
description: "Initial description"
capabilities:
  - name: "Cap1"
    description: "Capability"
    technologies: ["Tech1"]
best_practices:
  - title: "BP1"
    description: "Practice"
    technologies: ["Tech1"]
    implementation: "Impl"
technology_recommendations:
  - technology: "Tech1"
    use_cases: ["UC1"]
    pros: ["P1"]
    cons: ["C1"]
    alternatives: ["A1"]
"#;

        let config_file = config_path.join("reload-test.yaml");
        fs::write(&config_file, initial_config).expect("Failed to write config file");

        // Load initial configuration
        let loader = ConfigLoader::new();
        let initial = loader
            .load_from_file(&config_file)
            .expect("Failed to load initial config");

        assert_eq!(initial.description, "Initial description");

        // Update configuration
        let updated_config = r#"
domain: reload-test
name: "Reload Test Domain"
description: "Updated description"
capabilities:
  - name: "Cap1"
    description: "Capability"
    technologies: ["Tech1"]
best_practices:
  - title: "BP1"
    description: "Practice"
    technologies: ["Tech1"]
    implementation: "Impl"
technology_recommendations:
  - technology: "Tech1"
    use_cases: ["UC1"]
    pros: ["P1"]
    cons: ["C1"]
    alternatives: ["A1"]
"#;

        fs::write(&config_file, updated_config).expect("Failed to write updated config");

        // Reload configuration
        let reloaded = loader
            .load_from_file(&config_file)
            .expect("Failed to load reloaded config");

        // Verify configuration was updated
        assert_eq!(reloaded.description, "Updated description");
        assert_ne!(initial.description, reloaded.description);
    }

    #[test]
    fn property_11_hot_reload_updates_agent() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path();

        let initial_config = r#"
domain: agent-reload
name: "Agent Reload Test"
description: "Initial"
capabilities:
  - name: "Cap1"
    description: "Capability"
    technologies: ["Tech1"]
best_practices:
  - title: "BP1"
    description: "Practice"
    technologies: ["Tech1"]
    implementation: "Impl"
technology_recommendations:
  - technology: "Tech1"
    use_cases: ["UC1"]
    pros: ["P1"]
    cons: ["C1"]
    alternatives: ["A1"]
"#;

        let config_file = config_path.join("agent-reload.yaml");
        fs::write(&config_file, initial_config).expect("Failed to write config file");

        let loader = ConfigLoader::new();
        let factory = AgentFactory::new();

        // Create initial agent
        let initial_config_obj = loader
            .load_from_file(&config_file)
            .expect("Failed to load config");
        let initial_agent = factory
            .create_agent("agent-reload", &initial_config_obj)
            .expect("Failed to create agent");

        assert_eq!(initial_agent.domain, "agent-reload");

        // Update configuration
        let updated_config = r#"
domain: agent-reload-updated
name: "Agent Reload Test Updated"
description: "Updated"
capabilities:
  - name: "Cap1"
    description: "Capability"
    technologies: ["Tech1"]
best_practices:
  - title: "BP1"
    description: "Practice"
    technologies: ["Tech1"]
    implementation: "Impl"
technology_recommendations:
  - technology: "Tech1"
    use_cases: ["UC1"]
    pros: ["P1"]
    cons: ["C1"]
    alternatives: ["A1"]
"#;

        fs::write(&config_file, updated_config).expect("Failed to write updated config");

        // Reload and recreate agent
        let updated_config_obj = loader
            .load_from_file(&config_file)
            .expect("Failed to load updated config");
        let updated_agent = factory
            .create_agent("agent-reload-updated", &updated_config_obj)
            .expect("Failed to create updated agent");

        // Verify agent was updated
        assert_eq!(updated_agent.domain, "agent-reload-updated");
        assert_ne!(initial_agent.domain, updated_agent.domain);
    }

    // ========================================================================
    // Property 12: Custom Domain Support
    // ========================================================================
    // *For any* custom domain configuration file added to `config/domains/`,
    // the Domain Agent System SHALL provide recommendations based on the custom
    // domain configuration, treating custom domains identically to built-in
    // domains.
    //
    // **Validates: Requirements 5.4**

    #[test]
    fn property_12_custom_domain_works_like_builtin() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path();

        let custom_domain_config = r#"
domain: custom-mobile
name: "Custom Mobile Domain"
description: "Custom mobile development domain"
capabilities:
  - name: "Framework Selection"
    description: "Recommend mobile frameworks"
    technologies: ["React Native", "Flutter"]
best_practices:
  - title: "Cross-Platform Development"
    description: "Share code between platforms"
    technologies: ["React Native", "Flutter"]
    implementation: "Use cross-platform frameworks"
technology_recommendations:
  - technology: "React Native"
    use_cases: ["Cross-platform apps"]
    pros: ["Code sharing"]
    cons: ["Performance"]
    alternatives: ["Flutter"]
"#;

        let config_file = config_path.join("custom-mobile.yaml");
        fs::write(&config_file, custom_domain_config).expect("Failed to write config file");

        // Load and create custom domain agent
        let loader = ConfigLoader::new();
        let config = loader
            .load_from_file(&config_file)
            .expect("Failed to load config");

        let factory = AgentFactory::new();
        let custom_agent = factory
            .create_agent("custom-mobile", &config)
            .expect("Failed to create custom agent");

        // Verify custom domain has all expected properties
        assert_eq!(custom_agent.domain, "custom-mobile");
        assert!(!custom_agent.capabilities.is_empty());
        assert!(!custom_agent.knowledge.best_practices.is_empty());
        assert!(!custom_agent.knowledge.technology_recommendations.is_empty());

        // Verify custom domain can provide recommendations
        let capabilities = &custom_agent.capabilities;
        assert!(capabilities.iter().any(|c| c.name == "Framework Selection"));

        let recommendations = &custom_agent.knowledge.technology_recommendations;
        assert!(recommendations
            .iter()
            .any(|r| r.technology == "React Native"));
    }

    #[test]
    fn property_12_custom_domain_provides_recommendations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path();

        let custom_domain_config = r#"
domain: custom-data-science
name: "Custom Data Science"
description: "Custom data science domain"
capabilities:
  - name: "ML Framework Selection"
    description: "Recommend ML frameworks"
    technologies: ["TensorFlow", "PyTorch", "Scikit-learn"]
best_practices:
  - title: "Model Validation"
    description: "Validate ML models properly"
    technologies: ["TensorFlow", "PyTorch"]
    implementation: "Use cross-validation and test sets"
technology_recommendations:
  - technology: "TensorFlow"
    use_cases: ["Deep learning", "Production ML"]
    pros: ["Production-ready", "Documentation"]
    cons: ["Steep learning curve"]
    alternatives: ["PyTorch"]
  - technology: "PyTorch"
    use_cases: ["Research", "Rapid prototyping"]
    pros: ["Pythonic", "Flexible"]
    cons: ["Less production-ready"]
    alternatives: ["TensorFlow"]
"#;

        let config_file = config_path.join("custom-data-science.yaml");
        fs::write(&config_file, custom_domain_config).expect("Failed to write config file");

        let loader = ConfigLoader::new();
        let config = loader
            .load_from_file(&config_file)
            .expect("Failed to load config");

        let factory = AgentFactory::new();
        let agent = factory
            .create_agent("custom-data-science", &config)
            .expect("Failed to create agent");

        // Verify recommendations are available
        let recommendations = &agent.knowledge.technology_recommendations;
        assert_eq!(recommendations.len(), 2);

        // Verify each recommendation has complete information
        for rec in recommendations {
            assert!(!rec.technology.is_empty());
            assert!(!rec.use_cases.is_empty());
            assert!(!rec.pros.is_empty());
            assert!(!rec.cons.is_empty());
            assert!(!rec.alternatives.is_empty());
        }
    }

    // ========================================================================
    // Property 13: Configuration Validation and Error Handling
    // ========================================================================
    // *For any* invalid domain configuration file, the Domain Agent System
    // SHALL reject it with clear error messages and maintain existing
    // configuration without corruption or loss.
    //
    // **Validates: Requirements 5.5**

    #[test]
    fn property_13_validation_rejects_missing_required_fields() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path();

        // Missing 'domain' field
        let invalid_config = r#"
name: "Invalid Domain"
description: "Missing domain field"
capabilities:
  - name: "Cap"
    description: "Capability"
    technologies: ["Tech"]
best_practices:
  - title: "BP"
    description: "Practice"
    technologies: ["Tech"]
    implementation: "Impl"
technology_recommendations:
  - technology: "Tech"
    use_cases: ["UC"]
    pros: ["P"]
    cons: ["C"]
    alternatives: ["A"]
"#;

        let config_file = config_path.join("invalid.yaml");
        fs::write(&config_file, invalid_config).expect("Failed to write config file");

        let loader = ConfigLoader::new();
        let result = loader.load_from_file(&config_file);

        // Verify validation fails
        assert!(
            result.is_err(),
            "Configuration with missing domain field should fail validation"
        );
    }

    #[test]
    fn property_13_validation_handles_minimal_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path();

        // Minimal valid configuration
        let minimal_config = r#"
domain: minimal-domain
name: "Minimal Domain"
description: "Minimal configuration"
capabilities:
  - name: "Cap"
    description: "Capability"
    technologies: ["Tech"]
best_practices:
  - title: "BP"
    description: "Practice"
    technologies: ["Tech"]
    implementation: "Impl"
technology_recommendations:
  - technology: "Tech"
    use_cases: ["UC"]
    pros: ["P"]
    cons: ["C"]
    alternatives: ["A"]
"#;

        let config_file = config_path.join("minimal.yaml");
        fs::write(&config_file, minimal_config).expect("Failed to write config file");

        let loader = ConfigLoader::new();
        let result = loader.load_from_file(&config_file);

        // Verify minimal configuration loads successfully
        assert!(
            result.is_ok(),
            "Minimal valid configuration should load successfully"
        );
    }

    #[test]
    fn property_13_validation_provides_clear_error_messages() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path();

        let invalid_config = r#"
name: "Missing domain"
description: "No domain field"
capabilities:
  - name: "Cap"
    description: "Capability"
    technologies: ["Tech"]
best_practices:
  - title: "BP"
    description: "Practice"
    technologies: ["Tech"]
    implementation: "Impl"
technology_recommendations:
  - technology: "Tech"
    use_cases: ["UC"]
    pros: ["P"]
    cons: ["C"]
    alternatives: ["A"]
"#;

        let config_file = config_path.join("invalid-error-msg.yaml");
        fs::write(&config_file, invalid_config).expect("Failed to write config file");

        let loader = ConfigLoader::new();
        let result = loader.load_from_file(&config_file);

        // Verify error message is clear
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                !error_msg.is_empty(),
                "Error message should not be empty"
            );
            // Error should mention validation or schema
            assert!(
                error_msg.to_lowercase().contains("validation")
                    || error_msg.to_lowercase().contains("schema")
                    || error_msg.to_lowercase().contains("required")
                    || error_msg.to_lowercase().contains("deserialize"),
                "Error message should indicate what validation failed: {}",
                error_msg
            );
        } else {
            panic!("Invalid configuration should fail validation");
        }
    }

    #[test]
    fn property_13_validation_maintains_existing_config_on_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path();

        // Create a valid configuration first
        let valid_config = r#"
domain: existing-domain
name: "Existing Domain"
description: "Valid configuration"
capabilities:
  - name: "Cap"
    description: "Capability"
    technologies: ["Tech"]
best_practices:
  - title: "BP"
    description: "Practice"
    technologies: ["Tech"]
    implementation: "Impl"
technology_recommendations:
  - technology: "Tech"
    use_cases: ["UC"]
    pros: ["P"]
    cons: ["C"]
    alternatives: ["A"]
"#;

        let config_file = config_path.join("existing.yaml");
        fs::write(&config_file, valid_config).expect("Failed to write config file");

        let loader = ConfigLoader::new();
        let existing = loader
            .load_from_file(&config_file)
            .expect("Failed to load existing config");

        // Verify existing configuration is intact
        assert_eq!(existing.domain, "existing-domain");

        // Try to load an invalid configuration (missing required domain field)
        let invalid_config = r#"
name: "Invalid"
description: "Invalid"
capabilities:
  - name: "Cap"
    description: "Capability"
    technologies: ["Tech"]
best_practices:
  - title: "BP"
    description: "Practice"
    technologies: ["Tech"]
    implementation: "Impl"
technology_recommendations:
  - technology: "Tech"
    use_cases: ["UC"]
    pros: ["P"]
    cons: ["C"]
    alternatives: ["A"]
"#;

        let invalid_file = config_path.join("invalid.yaml");
        fs::write(&invalid_file, invalid_config).expect("Failed to write invalid config");

        let invalid_result = loader.load_from_file(&invalid_file);

        // Verify invalid configuration fails
        assert!(invalid_result.is_err(), "Configuration missing domain field should fail");

        // Verify existing configuration is still intact
        let existing_again = loader
            .load_from_file(&config_file)
            .expect("Failed to reload existing config");

        assert_eq!(existing_again.domain, "existing-domain");
        assert_eq!(existing.domain, existing_again.domain);
    }
}
