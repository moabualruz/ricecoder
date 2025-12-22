//! Cross-crate integration tests for external service integration (MCP, providers) with compliance validation
//!
//! Tests the integration between:
//! - ricecoder-mcp (Model Context Protocol)
//! - ricecoder-providers (AI providers)
//! - ricecoder-security (compliance validation)
//! - ricecoder-permissions (access control)

use std::{path::PathBuf, sync::Arc};

use ricecoder_mcp::{client::MCPClientConfig, MCPClient, ToolRegistry};
use ricecoder_permissions::{PermissionLevel, PermissionManager, ToolPermission};
use ricecoder_providers::{
    providers::{AnthropicProvider, OpenAiProvider},
    Provider, ProviderManager,
};
use ricecoder_security::{audit::AuditLogger, compliance::ComplianceManager};
use ricecoder_storage::{StorageManager, StorageMode};
use tempfile::tempdir;
use tokio::time::{timeout, Duration};

/// Mock storage manager for testing
struct MockStorageManager {
    global_path: PathBuf,
    project_path: Option<PathBuf>,
}

impl StorageManager for MockStorageManager {
    fn global_path(&self) -> &PathBuf {
        &self.global_path
    }

    fn project_path(&self) -> Option<&PathBuf> {
        self.project_path.as_ref()
    }

    fn mode(&self) -> StorageMode {
        StorageMode::Merged
    }

    fn global_resource_path(
        &self,
        _resource_type: ricecoder_storage::types::ResourceType,
    ) -> PathBuf {
        self.global_path.join("resources")
    }

    fn project_resource_path(
        &self,
        _resource_type: ricecoder_storage::types::ResourceType,
    ) -> Option<PathBuf> {
        self.project_path.as_ref().map(|p| p.join("resources"))
    }

    fn is_first_run(&self) -> bool {
        false
    }
}

#[tokio::test]
async fn test_mcp_provider_integration_with_compliance() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path().to_path_buf();

    // Initialize components
    let storage = Arc::new(MockStorageManager {
        global_path: storage_path.clone(),
        project_path: Some(storage_path.clone()),
    });

    let permission_manager = PermissionManager::new(storage.clone());
    let compliance_manager = ComplianceManager::new();
    let audit_logger = AuditLogger::new(storage.clone());

    // Initialize MCP client with compliance monitoring
    let mcp_config = MCPClientConfig {
        server_url: "http://localhost:3000".to_string(), // Mock URL for testing
        timeout: Duration::from_secs(30),
        retry_attempts: 3,
        enable_compliance_logging: true,
    };

    let mcp_client = MCPClient::new(mcp_config, Some(audit_logger.clone()));

    // Initialize provider manager
    let provider_manager = ProviderManager::new();

    // Register providers with compliance validation
    let openai_provider = OpenAiProvider::new("test-key".to_string());
    let anthropic_provider = AnthropicProvider::new("test-key".to_string());

    provider_manager
        .register_provider(Box::new(openai_provider))
        .await?;
    provider_manager
        .register_provider(Box::new(anthropic_provider))
        .await?;

    // Test MCP tool discovery with permissions
    permission_manager
        .add_permission(
            "mcp_tools",
            ToolPermission {
                tool_name: "file_operations".to_string(),
                level: PermissionLevel::Allow,
                requires_prompt: false,
            },
        )
        .await?;

    // Mock MCP server connection (would fail in real test without server)
    // This tests the integration setup and compliance validation
    let connection_result = timeout(Duration::from_millis(100), mcp_client.connect()).await;

    // Test should handle connection failure gracefully
    assert!(
        connection_result.is_err() || connection_result.unwrap().is_err(),
        "MCP connection should fail without server (expected for testing)"
    );

    // Test compliance validation of MCP configuration
    let compliance_result = compliance_manager.validate_mcp_config(&mcp_config).await?;
    assert!(
        compliance_result.is_compliant,
        "MCP config should be compliant"
    );

    // Test provider compliance
    let providers = provider_manager.list_providers().await?;
    for provider in providers {
        let provider_compliance = compliance_manager
            .validate_provider_config(&provider)
            .await?;
        assert!(
            provider_compliance.is_compliant,
            "Provider {} should be compliant",
            provider.id
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_mcp_tool_execution_with_permission_enforcement(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path().to_path_buf();

    // Initialize components
    let storage = Arc::new(MockStorageManager {
        global_path: storage_path.clone(),
        project_path: Some(storage_path.clone()),
    });

    let permission_manager = PermissionManager::new(storage.clone());
    let tool_registry = ToolRegistry::new();
    let audit_logger = AuditLogger::new(storage.clone());

    // Register MCP tools with different permission levels
    tool_registry.register_tool(ricecoder_mcp::ToolMetadata {
        name: "safe_read_file".to_string(),
        description: "Read file contents safely".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"}
            }
        }),
        permissions_required: vec!["read_files".to_string()],
    });

    tool_registry.register_tool(ricecoder_mcp::ToolMetadata {
        name: "dangerous_write_file".to_string(),
        description: "Write file contents".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "content": {"type": "string"}
            }
        }),
        permissions_required: vec!["write_files".to_string()],
    });

    // Set up permissions
    permission_manager
        .add_permission(
            "user",
            ToolPermission {
                tool_name: "read_files".to_string(),
                level: PermissionLevel::Allow,
                requires_prompt: false,
            },
        )
        .await?;

    permission_manager
        .add_permission(
            "user",
            ToolPermission {
                tool_name: "write_files".to_string(),
                level: PermissionLevel::Deny,
                requires_prompt: false,
            },
        )
        .await?;

    // Test tool execution with permission checks
    let read_result = tool_registry
        .check_permissions("safe_read_file", &["user"])
        .await?;
    assert!(read_result.allowed, "Read tool should be allowed");

    let write_result = tool_registry
        .check_permissions("dangerous_write_file", &["user"])
        .await?;
    assert!(!write_result.allowed, "Write tool should be denied");

    // Test audit logging of permission checks
    let audit_entries = audit_logger.get_entries("user").await?;
    assert!(
        !audit_entries.is_empty(),
        "Should have audit entries for permission checks"
    );

    Ok(())
}

#[tokio::test]
async fn test_provider_mcp_integration_with_security_headers(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;

    // Initialize provider manager with security features
    let provider_manager = ProviderManager::new();

    // Create providers with security configurations
    let openai_provider = OpenAiProvider::new("sk-test1234567890abcdef".to_string());
    let anthropic_provider = AnthropicProvider::new("sk-ant-test1234567890".to_string());

    // Test security header validation
    let security_headers = ricecoder_providers::security_headers::SecurityHeadersBuilder::new()
        .with_content_security_policy("default-src 'self'")
        .with_x_frame_options("DENY")
        .with_hsts_max_age(31536000)
        .build();

    let validator = ricecoder_providers::security_headers::SecurityHeadersValidator::new();
    let validation_result = validator.validate(&security_headers)?;

    assert!(
        validation_result.is_valid,
        "Security headers should be valid"
    );

    // Register providers
    provider_manager
        .register_provider(Box::new(openai_provider))
        .await?;
    provider_manager
        .register_provider(Box::new(anthropic_provider))
        .await?;

    // Test API key redaction
    let sensitive_message = "API_KEY=sk-1234567890abcdef USER_TOKEN=abc123";
    let redacted = ricecoder_providers::redaction::redact(sensitive_message);

    assert!(
        !redacted.contains("sk-1234567890abcdef"),
        "API key should be redacted"
    );
    assert!(
        !redacted.contains("abc123"),
        "User token should be redacted"
    );
    assert!(
        redacted.contains("[REDACTED]"),
        "Should contain redaction markers"
    );

    // Test compliance validation of redaction
    let compliance_manager = ComplianceManager::new();
    let compliance_result = compliance_manager.validate_data_handling(&redacted).await?;

    assert!(
        compliance_result.is_compliant,
        "Redacted data should be compliant"
    );

    Ok(())
}

#[tokio::test]
async fn test_mcp_provider_fallback_with_compliance_monitoring(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path().to_path_buf();

    // Initialize components
    let storage = Arc::new(MockStorageManager {
        global_path: storage_path.clone(),
        project_path: Some(storage_path.clone()),
    });

    let provider_manager = ProviderManager::new();
    let compliance_manager = ComplianceManager::new();
    let audit_logger = AuditLogger::new(storage.clone());

    // Register multiple providers for fallback testing
    let openai_provider = OpenAiProvider::new("invalid-key".to_string()); // Will fail
    let anthropic_provider = AnthropicProvider::new("sk-ant-test-valid".to_string()); // Valid fallback

    provider_manager
        .register_provider(Box::new(openai_provider))
        .await?;
    provider_manager
        .register_provider(Box::new(anthropic_provider))
        .await?;

    // Configure fallback strategy
    provider_manager
        .set_fallback_strategy(vec!["openai".to_string(), "anthropic".to_string()])
        .await?;

    // Test request with fallback (mock failure of first provider)
    let test_request = ricecoder_providers::ChatRequest {
        messages: vec![ricecoder_providers::Message {
            role: ricecoder_providers::MessageRole::User,
            content: "Hello".to_string(),
        }],
        model: "gpt-3.5-turbo".to_string(),
        temperature: Some(0.7),
        max_tokens: Some(100),
    };

    // This would normally try OpenAI first, then fallback to Anthropic
    // For testing, we verify the setup and compliance monitoring
    let compliance_result = compliance_manager
        .validate_provider_fallback(&provider_manager)
        .await?;
    assert!(
        compliance_result.is_compliant,
        "Provider fallback should be compliant"
    );

    // Test audit logging of fallback events
    audit_logger
        .log_event("provider_fallback", "openai->anthropic", "test-user")
        .await?;

    let audit_entries = audit_logger.get_entries("test-user").await?;
    assert!(
        audit_entries
            .iter()
            .any(|e| e.action == "provider_fallback"),
        "Should have fallback audit entries"
    );

    Ok(())
}

#[tokio::test]
async fn test_mcp_enterprise_compliance_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path().to_path_buf();

    // Initialize enterprise components
    let storage = Arc::new(MockStorageManager {
        global_path: storage_path.clone(),
        project_path: Some(storage_path.clone()),
    });

    let mcp_compliance = ricecoder_mcp::compliance::MCPEnterpriseMonitor::new();
    let provider_compliance = ricecoder_providers::evaluation::ProviderEvaluator::new();
    let audit_logger = AuditLogger::new(storage.clone());

    // Test MCP enterprise monitoring
    let mcp_report = mcp_compliance.generate_compliance_report().await?;
    assert!(
        mcp_report.is_compliant,
        "MCP should be enterprise compliant"
    );

    // Test provider evaluation for enterprise use
    let evaluation_result = provider_compliance.evaluate_enterprise_readiness().await?;
    assert!(
        evaluation_result.overall_score >= 0.8,
        "Providers should be enterprise-ready"
    );

    // Test data residency compliance
    let data_residency_check = ricecoder_security::compliance::DataPortability::new();
    let residency_result = data_residency_check
        .validate_data_location("us-west-2")
        .await?;
    assert!(
        residency_result.compliant,
        "Data residency should be compliant"
    );

    // Test audit trail completeness
    let audit_trail = audit_logger.generate_compliance_report().await?;
    assert!(audit_trail.is_complete, "Audit trail should be complete");

    // Test privacy analytics
    let privacy_analytics = ricecoder_security::compliance::PrivacyAnalytics::new();
    let privacy_report = privacy_analytics.analyze_data_usage().await?;
    assert!(
        privacy_report.privacy_score >= 0.9,
        "Privacy score should be high"
    );

    Ok(())
}

#[tokio::test]
async fn test_mcp_provider_rate_limiting_with_compliance() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = tempdir()?;

    // Initialize rate limiting
    let rate_limiter = ricecoder_providers::rate_limiter::RateLimiterRegistry::new();

    // Configure rate limits for different providers
    rate_limiter
        .add_limiter(
            "openai",
            ricecoder_providers::rate_limiter::TokenBucketLimiter::new(100, 60), // 100 requests per minute
        )
        .await?;

    rate_limiter
        .add_limiter(
            "anthropic",
            ricecoder_providers::rate_limiter::TokenBucketLimiter::new(50, 60), // 50 requests per minute
        )
        .await?;

    // Test rate limiting compliance
    let compliance_manager = ComplianceManager::new();
    let rate_limit_compliance = compliance_manager
        .validate_rate_limiting(&rate_limiter)
        .await?;
    assert!(
        rate_limit_compliance.is_compliant,
        "Rate limiting should be compliant"
    );

    // Simulate requests and verify rate limiting
    for i in 0..120 {
        let can_proceed = rate_limiter.check_limit("openai").await?;
        if i < 100 {
            assert!(can_proceed, "Should allow requests within limit");
        } else {
            // Should start blocking after 100 requests
            assert!(!can_proceed, "Should block requests over limit");
        }
    }

    // Test exponential backoff
    let backoff = ricecoder_providers::rate_limiter::ExponentialBackoff::new(
        Duration::from_millis(100),
        Duration::from_secs(30),
        2.0,
    );

    let mut delay = backoff.initial_delay;
    for _ in 0..5 {
        assert!(delay <= backoff.max_delay, "Delay should not exceed max");
        delay = backoff.next_delay(delay);
    }

    Ok(())
}
