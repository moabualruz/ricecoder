//! Integration tests for ricecoder-industry enterprise features

use ricecoder_industry::*;
use ricecoder_industry::connections::ConnectionConfig;
use ricecoder_industry::compliance::SecurityRule;
use ricecoder_industry::providers::EnterpriseProviderConfig;
use ricecoder_industry::tools::{ToolCapability, ToolMetadata, ConfigRequirement, ConfigValueType};
use std::collections::HashMap;
use tokio::test;

#[test]
async fn test_oauth_client_flow() {
    let client = OAuthClient::new();

    // Test registering a flow
    let config = OAuthConfig::new(
        "test-client-id".to_string(),
        "test-client-secret".to_string(),
        "https://example.com/oauth/authorize".to_string(),
        "https://example.com/oauth/token".to_string(),
        "http://localhost:8080/callback".to_string(),
    )
    .with_scopes(vec!["read".to_string(), "write".to_string()]);

    assert!(client.register_flow("test-provider".to_string(), config).await.is_ok());

    // Test getting authorization URL (this would normally require a real server)
    // For testing, we just verify the flow is registered
    // Note: OAuthClient doesn't have a get_provider method, flows are internal
}

#[test]
async fn test_connection_manager() {
    let manager = ConnectionManager::new();

    // Create a test connection config
    let config = ConnectionConfig {
        id: "test-connection".to_string(),
        tool: "github".to_string(),
        name: "Test GitHub".to_string(),
        base_url: "https://api.github.com".to_string(),
        auth_method: connections::AuthMethod::ApiKey {
            header_name: "Authorization".to_string(),
            key: "token test-token".to_string(),
        },
        timeout_seconds: 30,
        rate_limit: None,
        config: HashMap::new(),
    };

    let connector = connections::ToolConnector::new(config);
    let connection: Box<dyn connections::ToolConnection> = Box::new(connector);

    // Test adding connection
    assert!(manager.add_connection(connection).await.is_ok());

    // Test listing connections
    let connections = manager.list_connections().await;
    assert_eq!(connections.len(), 1);
    assert_eq!(connections[0], "test-connection");

    // Test taking connection (removes it from manager)
    assert!(manager.take_connection("test-connection").await.is_some());
    assert!(manager.take_connection("non-existent").await.is_none());
}

#[test]
async fn test_audit_logger() {
    let logger = AuditLogger::new(100);

    // Test logging a success event
    assert!(logger
        .log_success(
            "test-user".to_string(),
            "test-action".to_string(),
            "test-resource".to_string(),
            HashMap::new(),
            "127.0.0.1".to_string(),
            Some("session-123".to_string()),
        )
        .await
        .is_ok());

    // Test logging a failure event
    assert!(logger
        .log_failure(
            "test-user".to_string(),
            "test-action".to_string(),
            "test-resource".to_string(),
            "Test error".to_string(),
            HashMap::new(),
            "127.0.0.1".to_string(),
            Some("session-123".to_string()),
        )
        .await
        .is_ok());

    // Test getting entries
    let entries = logger.get_entries(None, None, None, None).await;
    assert_eq!(entries.len(), 2);

    // Test filtering by actor
    let user_entries = logger.get_entries(Some("test-user"), None, None, None).await;
    assert_eq!(user_entries.len(), 2);

    let other_entries = logger.get_entries(Some("other-user"), None, None, None).await;
    assert_eq!(other_entries.len(), 0);

    // Test getting stats
    let stats = logger.get_stats().await;
    assert_eq!(stats.total_entries, 2);
    assert_eq!(stats.success_count, 1);
    assert_eq!(stats.failure_count, 1);
}

#[test]
async fn test_security_validator() {
    let validator = SecurityValidator::new();

    // Add a security rule
    let rule = SecurityRule {
        id: "test-rule".to_string(),
        name: "Test Required Field".to_string(),
        description: "Test rule for required fields".to_string(),
        severity: compliance::SecuritySeverity::High,
        condition: compliance::SecurityCondition::RequiredField {
            field: "api_key".to_string(),
        },
        actions: vec![compliance::SecurityAction::Block],
    };

    assert!(validator.add_rule(rule).await.is_ok());

    // Test validation with missing required field
    let invalid_data = serde_json::json!({
        "name": "test"
    });

    let violations = validator.validate(&invalid_data).await.unwrap();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule_id, "test-rule");

    // Test validation with required field present
    let valid_data = serde_json::json!({
        "name": "test",
        "api_key": "secret-key"
    });

    let violations = validator.validate(&valid_data).await.unwrap();
    assert_eq!(violations.len(), 0);
}

#[test]
async fn test_compliance_manager() {
    let audit_logger = AuditLogger::new(100);
    let security_validator = SecurityValidator::new();
    let compliance_manager = ComplianceManager::new(audit_logger, security_validator);

    // Add a security rule
    let rule = SecurityRule {
        id: "compliance-rule".to_string(),
        name: "Compliance Test".to_string(),
        description: "Test compliance rule".to_string(),
        severity: compliance::SecuritySeverity::Medium,
        condition: compliance::SecurityCondition::RequiredField {
            field: "user_id".to_string(),
        },
        actions: vec![compliance::SecurityAction::Log],
    };

    compliance_manager.security_validator.add_rule(rule).await.unwrap();

    // Test validation and logging
    let valid_data = serde_json::json!({
        "user_id": "user123",
        "action": "test"
    });

    let violations = compliance_manager
        .validate_and_log(
            "test-user".to_string(),
            "test-action".to_string(),
            "test-resource".to_string(),
            &valid_data,
            "127.0.0.1".to_string(),
            None,
        )
        .await
        .unwrap();

    assert_eq!(violations.len(), 0);

    // Test invalid data
    let invalid_data = serde_json::json!({
        "action": "test"
    });

    let violations = compliance_manager
        .validate_and_log(
            "test-user".to_string(),
            "test-action".to_string(),
            "test-resource".to_string(),
            &invalid_data,
            "127.0.0.1".to_string(),
            None,
        )
        .await
        .unwrap();

    assert_eq!(violations.len(), 1);

    // Test getting compliance summary
    let summary = compliance_manager.get_compliance_summary().await;
    assert_eq!(summary.total_checks, 0); // No explicit checks run yet
}

#[test]
async fn test_provider_manager() {
    let manager = ProviderManager::new();

    // Create a test provider config
    let config = EnterpriseProviderConfig {
        name: "test-provider".to_string(),
        provider_type: providers::ProviderType::OpenAI,
        base_url: "https://api.openai.com".to_string(),
        api_version: "v1".to_string(),
        auth_config: providers::ProviderAuthConfig {
            method: connections::AuthMethod::ApiKey {
                header_name: "Authorization".to_string(),
                key: "Bearer test-key".to_string(),
            },
            api_key: Some("test-key".to_string()),
            client_id: None,
            client_secret: None,
            tenant_id: None,
            organization_id: None,
        },
        rate_limit: providers::RateLimitConfig {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            max_concurrent: 10,
            burst_limit: 20,
        },
        enterprise_features: vec![providers::EnterpriseFeature::AuditLogging],
        custom_config: HashMap::new(),
    };

    let provider = providers::GenericEnterpriseProvider::new(config);
    let provider: Box<dyn providers::EnterpriseProvider> = Box::new(provider);

    // Test adding provider
    assert!(manager.add_provider(provider).await.is_ok());

    // Test listing providers
    let providers = manager.list_providers().await;
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0], "test-provider");

    // Test taking provider (removes it from manager)
    assert!(manager.take_provider("test-provider").await.is_some());
    assert!(manager.take_provider("non-existent").await.is_none());
}

#[test]
async fn test_tool_registry() {
    let registry = ToolRegistry::new();

    // Create a mock tool for testing
    struct MockTool {
        name: String,
        description: String,
    }

    #[async_trait::async_trait]
    impl IndustryTool for MockTool {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn capabilities(&self) -> Vec<ToolCapability> {
            vec![ToolCapability::CodeAnalysis, ToolCapability::CodeGeneration]
        }

        async fn execute(&self, _params: HashMap<String, serde_json::Value>) -> IndustryResult<serde_json::Value> {
            Ok(serde_json::json!({"result": "success"}))
        }

        fn config_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            })
        }

        fn validate_params(&self, params: &HashMap<String, serde_json::Value>) -> IndustryResult<()> {
            if !params.contains_key("input") {
                return Err(IndustryError::ConfigError {
                    field: "input".to_string(),
                    message: "Input parameter is required".to_string(),
                });
            }
            Ok(())
        }
    }

    let tool = MockTool {
        name: "test-tool".to_string(),
        description: "A test tool".to_string(),
    };

    let metadata = ToolMetadata {
        name: "test-tool".to_string(),
        version: "1.0.0".to_string(),
        description: "A test tool for integration testing".to_string(),
        author: "Test Author".to_string(),
        capabilities: vec![ToolCapability::CodeAnalysis, ToolCapability::CodeGeneration],
        config_requirements: vec![ConfigRequirement {
            key: "api_key".to_string(),
            value_type: ConfigValueType::String,
            required: true,
            default_value: None,
            description: "API key for the tool".to_string(),
        }],
        dependencies: vec![],
    };

    // Test registering tool
    assert!(registry.register_tool(Box::new(tool), metadata).await.is_ok());

    // Test listing tools
    let tools = registry.list_tools().await;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0], "test-tool");

    // Test listing by capability
    let analysis_tools = registry.list_tools_by_capability(&ToolCapability::CodeAnalysis).await;
    assert_eq!(analysis_tools.len(), 1);

    // Test getting metadata
    let meta = registry.get_metadata("test-tool").await.unwrap();
    assert_eq!(meta.name, "test-tool");
    assert_eq!(meta.capabilities.len(), 2);

    // Test executing tool
    let params = HashMap::from([
        ("input".to_string(), serde_json::json!("test input"))
    ]);

    let result = registry.execute_tool("test-tool", params).await.unwrap();
    assert!(result.success);
    assert_eq!(result.data.unwrap()["result"], "success");

    // Test parameter validation
    let invalid_params = HashMap::new();
    let result = registry.execute_tool("test-tool", invalid_params).await;
    assert!(result.is_err());
}

#[test]
async fn test_config_validation() {
    let registry = ToolRegistry::new();

    // Create and register a tool with config requirements
    struct ConfigTestTool;

    #[async_trait::async_trait]
    impl IndustryTool for ConfigTestTool {
        fn name(&self) -> &str {
            "config-test-tool"
        }

        fn description(&self) -> &str {
            "Tool for testing configuration validation"
        }

        fn capabilities(&self) -> Vec<ToolCapability> {
            vec![ToolCapability::Testing]
        }

        async fn execute(&self, _params: HashMap<String, serde_json::Value>) -> IndustryResult<serde_json::Value> {
            Ok(serde_json::json!({"status": "ok"}))
        }

        fn config_schema(&self) -> serde_json::Value {
            serde_json::json!({})
        }

        fn validate_params(&self, _params: &HashMap<String, serde_json::Value>) -> IndustryResult<()> {
            Ok(())
        }
    }

    let metadata = ToolMetadata {
        name: "config-test-tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Configuration validation test tool".to_string(),
        author: "Test".to_string(),
        capabilities: vec![ToolCapability::Testing],
        config_requirements: vec![
            ConfigRequirement {
                key: "api_key".to_string(),
                value_type: ConfigValueType::String,
                required: true,
                default_value: None,
                description: "Required API key".to_string(),
            },
            ConfigRequirement {
                key: "timeout".to_string(),
                value_type: ConfigValueType::Integer,
                required: false,
                default_value: Some(serde_json::json!(30)),
                description: "Optional timeout".to_string(),
            },
        ],
        dependencies: vec![],
    };

    registry.register_tool(Box::new(ConfigTestTool), metadata).await.unwrap();

    // Test valid configuration
    let valid_config = HashMap::from([
        ("api_key".to_string(), serde_json::json!("test-key")),
        ("timeout".to_string(), serde_json::json!(60)),
    ]);

    let errors = registry.validate_tool_config("config-test-tool", &valid_config).await.unwrap();
    assert_eq!(errors.len(), 0);

    // Test missing required field
    let invalid_config = HashMap::from([
        ("timeout".to_string(), serde_json::json!(60)),
    ]);

    let errors = registry.validate_tool_config("config-test-tool", &invalid_config).await.unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field, "api_key");

    // Test invalid type
    let invalid_type_config = HashMap::from([
        ("api_key".to_string(), serde_json::json!("test-key")),
        ("timeout".to_string(), serde_json::json!("not-a-number")),
    ]);

    let errors = registry.validate_tool_config("config-test-tool", &invalid_type_config).await.unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field, "timeout");
    assert!(errors[0].message.contains("invalid type"));
}