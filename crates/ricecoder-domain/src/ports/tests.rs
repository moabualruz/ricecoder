//! Tests for port interfaces and value objects

use super::*;
use std::path::PathBuf;

#[test]
fn test_chat_message_constructors() {
    let user = ChatMessage::user("Hello");
    assert_eq!(user.role, ChatRole::User);
    assert_eq!(user.content, "Hello");

    let assistant = ChatMessage::assistant("Hi there!");
    assert_eq!(assistant.role, ChatRole::Assistant);

    let system = ChatMessage::system("You are helpful.");
    assert_eq!(system.role, ChatRole::System);
}

#[test]
fn test_chat_request_builder() {
    let request = AiChatRequest::new("gpt-4", vec![ChatMessage::user("Hello")])
        .with_temperature(0.7)
        .with_max_tokens(1000)
        .with_stop(vec!["END".to_string()]);

    assert_eq!(request.model, "gpt-4");
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(1000));
    assert_eq!(request.stop, Some(vec!["END".to_string()]));
}

#[test]
fn test_health_check_result_constructors() {
    let healthy = HealthCheckResult::healthy(50);
    assert_eq!(healthy.status, ProviderHealthStatus::Healthy);
    assert_eq!(healthy.latency_ms, Some(50));
    assert!(healthy.error.is_none());

    let degraded = HealthCheckResult::degraded(2000);
    assert_eq!(degraded.status, ProviderHealthStatus::Degraded);

    let unhealthy = HealthCheckResult::unhealthy("Connection refused");
    assert_eq!(unhealthy.status, ProviderHealthStatus::Unhealthy);
    assert_eq!(unhealthy.error, Some("Connection refused".to_string()));
}

#[test]
fn test_token_usage_default() {
    let usage = TokenUsage::default();
    assert_eq!(usage.prompt_tokens, 0);
    assert_eq!(usage.completion_tokens, 0);
    assert_eq!(usage.total_tokens, 0);
}

#[test]
fn test_finish_reason_serialization() {
    let reason = FinishReason::Stop;
    let json = serde_json::to_string(&reason).unwrap();
    assert_eq!(json, "\"stop\"");

    let reason = FinishReason::ContentFilter;
    let json = serde_json::to_string(&reason).unwrap();
    assert_eq!(json, "\"content_filter\"");
}

#[test]
fn test_cache_statistics_hit_rate() {
    let stats = CacheStatistics {
        hits: 80,
        misses: 20,
        entry_count: 100,
        total_size_bytes: 1024,
        evictions: 5,
        avg_retrieval_time_ms: 1.5,
    };
    assert!((stats.hit_rate() - 80.0).abs() < f64::EPSILON);

    // Test zero case
    let empty_stats = CacheStatistics::default();
    assert!((empty_stats.hit_rate() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_file_metadata_serialization() {
    let metadata = FileMetadata {
        path: PathBuf::from("/test/file.txt"),
        size: 1024,
        is_directory: false,
        modified: None,
        created: None,
        is_readonly: false,
    };
    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("file.txt"));
    assert!(json.contains("1024"));
}

#[test]
fn test_write_result_serialization() {
    let result = WriteResult {
        path: PathBuf::from("/test/output.txt"),
        bytes_written: 512,
        backup_created: true,
        backup_path: Some(PathBuf::from("/test/output.txt.bak")),
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("output.txt"));
    assert!(json.contains("512"));
    assert!(json.contains("backup_created"));
}

#[test]
fn test_cache_entry_info_serialization() {
    let info = CacheEntryInfo {
        key: "user:123".to_string(),
        size_bytes: 256,
        created_at: chrono::Utc::now(),
        ttl: Some(std::time::Duration::from_secs(3600)),
        is_expired: false,
    };
    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("user:123"));
    assert!(json.contains("256"));
}
