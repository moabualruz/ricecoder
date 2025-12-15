use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ricecoder_mcp::{
    ToolMarshaler,
    transport::{MCPMessage, MCPRequest, MCPResponse, MCPNotification},
    protocol_validation::MCPProtocolValidator,
    permissions::MCPPermissionManager,
    registry::ToolRegistry,
    metadata::{ToolMetadata, ToolSource},
};
use serde_json::json;
use std::sync::Arc;

fn benchmark_marshal_input(c: &mut Criterion) {
    let marshaler = ToolMarshaler::new();
    let input = black_box(json!({"param1": "value1", "param2": 42}));

    c.bench_function("marshal_input", |b| {
        b.iter(|| marshaler.marshal_input(&input))
    });
}

fn benchmark_unmarshal_output(c: &mut Criterion) {
    let marshaler = ToolMarshaler::new();
    let output = black_box(json!({"result": "success"}));

    c.bench_function("unmarshal_output", |b| {
        b.iter(|| marshaler.unmarshal_output(&output))
    });
}

fn benchmark_message_validation(c: &mut Criterion) {
    let validator = MCPProtocolValidator::new();
    let message = black_box(MCPMessage::Request(MCPRequest {
        id: "bench-test".to_string(),
        method: "test.benchmark".to_string(),
        params: json!({
            "data": "x".repeat(1000), // 1KB payload
            "nested": {
                "array": vec![1; 100],
                "object": {
                    "key1": "value1",
                    "key2": 42,
                    "key3": true
                }
            }
        }),
    }));

    c.bench_function("message_validation", |b| {
        b.iter(|| validator.validate_message(&message))
    });
}

fn benchmark_permission_check(c: &mut Criterion) {
    let permission_manager = MCPPermissionManager::new();

    // Setup some rules
    for i in 0..10 {
        let rule = ricecoder_mcp::permissions::PermissionRule {
            pattern: format!("tool-{}-*", i),
            level: ricecoder_mcp::permissions::PermissionLevelConfig::Allow,
            agent_id: Some(format!("agent-{}", i)),
        };
        permission_manager.add_global_rule(rule).unwrap();
    }

    let tool_name = black_box("tool-5-action".to_string());
    let agent_id = black_box(Some("agent-5".to_string()));

    c.bench_function("permission_check", |b| {
        b.iter(|| permission_manager.check_permission(&tool_name, agent_id.as_deref()))
    });
}

fn benchmark_tool_registry_lookup(c: &mut Criterion) {
    let mut registry = ToolRegistry::new();

    // Setup tools
    for i in 0..1000 {
        let tool = ToolMetadata {
            id: format!("bench-tool-{}", i),
            name: format!("Benchmark Tool {}", i),
            description: "Tool for benchmarking".to_string(),
            category: "benchmark".to_string(),
            parameters: vec![],
            return_type: "string".to_string(),
            source: ToolSource::Custom,
            server_id: None,
        };
        registry.register_tool(tool).unwrap();
    }

    let tool_id = black_box("bench-tool-500".to_string());

    c.bench_function("tool_registry_lookup", |b| {
        b.iter(|| registry.get_tool(&tool_id))
    });
}

fn benchmark_json_serialization(c: &mut Criterion) {
    let message = black_box(MCPMessage::Request(MCPRequest {
        id: "bench-serialize".to_string(),
        method: "test.serialization".to_string(),
        params: json!({
            "data": "x".repeat(1000),
            "numbers": vec![1; 100],
            "nested": {
                "deep": {
                    "structure": {
                        "with": "multiple",
                        "levels": "of nesting"
                    }
                }
            }
        }),
    }));

    c.bench_function("json_serialization", |b| {
        b.iter(|| serde_json::to_string(&message))
    });
}

fn benchmark_json_deserialization(c: &mut Criterion) {
    let json_str = black_box(r#"{
        "type": "request",
        "data": {
            "id": "bench-deserialize",
            "method": "test.deserialization",
            "params": {
                "data": "x",
                "numbers": [1, 2, 3],
                "nested": {"key": "value"}
            }
        }
    }"#.to_string());

    c.bench_function("json_deserialization", |b| {
        b.iter(|| serde_json::from_str::<MCPMessage>(&json_str))
    });
}

criterion_group!(
    benches,
    benchmark_marshal_input,
    benchmark_unmarshal_output,
    benchmark_message_validation,
    benchmark_permission_check,
    benchmark_tool_registry_lookup,
    benchmark_json_serialization,
    benchmark_json_deserialization
);
criterion_main!(benches);
