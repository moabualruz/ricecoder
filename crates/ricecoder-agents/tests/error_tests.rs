//! Unit tests for AgentError

use ricecoder_agents::error::*;

#[test]
fn test_agent_error_not_found() {
    let error = AgentError::not_found("test-agent");
    assert!(matches!(error, AgentError::NotFound(_)));
    assert_eq!(error.to_string(), "Agent not found: test-agent");
}

#[test]
fn test_agent_error_execution_failed() {
    let error = AgentError::execution_failed("execution error");
    assert!(matches!(error, AgentError::ExecutionFailed(_)));
    assert_eq!(error.to_string(), "Agent execution failed: execution error");
}

#[test]
fn test_agent_error_timeout() {
    let error = AgentError::timeout(5000);
    assert!(matches!(error, AgentError::Timeout(_)));
    assert_eq!(error.to_string(), "Agent timeout after 5000ms");
}

#[test]
fn test_agent_error_config_error() {
    let error = AgentError::config_error("invalid config");
    assert!(matches!(error, AgentError::ConfigError(_)));
    assert_eq!(error.to_string(), "Configuration error: invalid config");
}

#[test]
fn test_agent_error_path_error() {
    let error = AgentError::path_error("path not found");
    assert!(matches!(error, AgentError::PathError(_)));
    assert_eq!(error.to_string(), "Path resolution error: path not found");
}

#[test]
fn test_agent_error_provider_error() {
    let error = AgentError::provider_error("provider unavailable");
    assert!(matches!(error, AgentError::ProviderError(_)));
    assert_eq!(error.to_string(), "Provider error: provider unavailable");
}

#[test]
fn test_agent_error_invalid_input() {
    let error = AgentError::invalid_input("invalid data");
    assert!(matches!(error, AgentError::InvalidInput(_)));
    assert_eq!(error.to_string(), "Invalid input: invalid data");
}

#[test]
fn test_agent_error_internal() {
    let error = AgentError::internal("internal error");
    assert!(matches!(error, AgentError::Internal(_)));
    assert_eq!(error.to_string(), "Internal error: internal error");
}

#[test]
fn test_agent_error_serialization_error() {
    let error = AgentError::SerializationError("invalid json".to_string());
    assert!(matches!(error, AgentError::SerializationError(_)));
}

#[test]
fn test_agent_error_clone() {
    let error = AgentError::not_found("test-agent");
    let cloned = error.clone();
    assert_eq!(error.to_string(), cloned.to_string());
}

#[test]
fn test_error_display_trait() {
    let error = AgentError::not_found("agent-1");
    let display_string = format!("{}", error);
    assert_eq!(display_string, "Agent not found: agent-1");
}

#[test]
fn test_error_debug_trait() {
    let error = AgentError::timeout(1000);
    let debug_string = format!("{:?}", error);
    assert!(debug_string.contains("Timeout"));
}

#[test]
fn test_result_type_ok() {
    let result: Result<i32> = Ok(42);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_result_type_err() {
    let result: Result<i32> = Err(AgentError::not_found("test"));
    assert!(result.is_err());
}
