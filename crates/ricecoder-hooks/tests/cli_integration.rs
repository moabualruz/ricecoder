//! Integration tests for CLI and events

use ricecoder_hooks::*;

#[test]
fn test_hook_registration_and_listing() {
    let mut registry = InMemoryHookRegistry::new();

    // Create a test hook
    let hook = Hook {
        id: "test-hook-1".to_string(),
        name: "Test Hook 1".to_string(),
        description: Some("A test hook".to_string()),
        event: "file_saved".to_string(),
        action: Action::Command(CommandAction {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            timeout_ms: Some(5000),
            capture_output: true,
        }),
        enabled: true,
        tags: vec!["test".to_string()],
        metadata: serde_json::json!({}),
        condition: None,
    };

    // Register the hook
    registry.register_hook(hook).unwrap();

    // Create CLI and list hooks
    let mut cli = HookCli::new(registry);
    let result = cli.execute(HookCommand::List { format: None }).unwrap();

    assert!(result.contains("Test Hook 1"));
}

#[test]
fn test_hook_inspection() {
    let mut registry = InMemoryHookRegistry::new();

    let hook = Hook {
        id: "inspect-test".to_string(),
        name: "Inspect Test".to_string(),
        description: Some("Hook for inspection".to_string()),
        event: "test_passed".to_string(),
        action: Action::Command(CommandAction {
            command: "echo".to_string(),
            args: vec!["success".to_string()],
            timeout_ms: Some(5000),
            capture_output: true,
        }),
        enabled: true,
        tags: vec!["test".to_string()],
        metadata: serde_json::json!({}),
        condition: None,
    };

    registry.register_hook(hook).unwrap();

    let mut cli = HookCli::new(registry);
    let result = cli
        .execute(HookCommand::Inspect {
            id: "inspect-test".to_string(),
            format: None,
        })
        .unwrap();

    assert!(result.contains("Inspect Test"));
    assert!(result.contains("test_passed"));
}

#[test]
fn test_hook_enable_disable() {
    let mut registry = InMemoryHookRegistry::new();

    let hook = Hook {
        id: "toggle-test".to_string(),
        name: "Toggle Test".to_string(),
        description: None,
        event: "file_saved".to_string(),
        action: Action::Command(CommandAction {
            command: "echo".to_string(),
            args: vec![],
            timeout_ms: None,
            capture_output: false,
        }),
        enabled: true,
        tags: vec![],
        metadata: serde_json::json!({}),
        condition: None,
    };

    registry.register_hook(hook).unwrap();

    let mut cli = HookCli::new(registry);

    // Disable the hook
    let result = cli
        .execute(HookCommand::Disable {
            id: "toggle-test".to_string(),
        })
        .unwrap();

    assert!(result.contains("disabled"));

    // Enable the hook
    let result = cli
        .execute(HookCommand::Enable {
            id: "toggle-test".to_string(),
        })
        .unwrap();

    assert!(result.contains("enabled"));
}

#[test]
fn test_hook_deletion() {
    let mut registry = InMemoryHookRegistry::new();

    let hook = Hook {
        id: "delete-test".to_string(),
        name: "Delete Test".to_string(),
        description: None,
        event: "file_saved".to_string(),
        action: Action::Command(CommandAction {
            command: "echo".to_string(),
            args: vec![],
            timeout_ms: None,
            capture_output: false,
        }),
        enabled: true,
        tags: vec![],
        metadata: serde_json::json!({}),
        condition: None,
    };

    registry.register_hook(hook).unwrap();

    let mut cli = HookCli::new(registry);

    // Delete the hook
    let result = cli
        .execute(HookCommand::Delete {
            id: "delete-test".to_string(),
        })
        .unwrap();

    assert!(result.contains("deleted"));

    // Verify it's deleted
    let list_result = cli.execute(HookCommand::List { format: None }).unwrap();

    assert!(list_result.contains("No hooks found"));
}

#[test]
fn test_json_output_format() {
    let mut registry = InMemoryHookRegistry::new();

    let hook = Hook {
        id: "json-test".to_string(),
        name: "JSON Test".to_string(),
        description: Some("Test JSON output".to_string()),
        event: "generation_complete".to_string(),
        action: Action::Command(CommandAction {
            command: "echo".to_string(),
            args: vec!["json".to_string()],
            timeout_ms: Some(5000),
            capture_output: true,
        }),
        enabled: true,
        tags: vec!["json".to_string()],
        metadata: serde_json::json!({"key": "value"}),
        condition: None,
    };

    registry.register_hook(hook).unwrap();

    let mut cli = HookCli::new(registry);

    // List with JSON format
    let result = cli
        .execute(HookCommand::List {
            format: Some("json".to_string()),
        })
        .unwrap();

    // Verify it's valid JSON
    let parsed: std::result::Result<Vec<Hook>, _> = serde_json::from_str(&result);
    assert!(parsed.is_ok());

    let hooks = parsed.unwrap();
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].id, "json-test");
}

#[test]
fn test_system_event_creation() {
    let event = SystemEvent::FileSaved(FileSavedEvent {
        file_path: "/path/to/file.rs".to_string(),
        size: 1024,
        hash: "abc123".to_string(),
        timestamp: "2024-01-01T12:00:00Z".to_string(),
        language: Some("rust".to_string()),
    });

    assert_eq!(event.event_type(), "file_saved");

    let context = event.to_event_context();
    assert!(context.data.is_object());
    assert!(context.metadata.is_object());
}

#[test]
fn test_multiple_system_events() {
    let events = [
        SystemEvent::FileSaved(FileSavedEvent {
            file_path: "/path/to/file.rs".to_string(),
            size: 1024,
            hash: "abc123".to_string(),
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            language: Some("rust".to_string()),
        }),
        SystemEvent::TestPassed(TestPassedEvent {
            test_name: "test_example".to_string(),
            duration_ms: 100,
            assertions_passed: 5,
            timestamp: "2024-01-01T12:00:01Z".to_string(),
        }),
        SystemEvent::GenerationComplete(GenerationCompleteEvent {
            spec_path: "/path/to/spec.md".to_string(),
            output_dir: "/path/to/output".to_string(),
            files_generated: 3,
            duration_ms: 500,
            timestamp: "2024-01-01T12:00:02Z".to_string(),
        }),
    ];

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].event_type(), "file_saved");
    assert_eq!(events[1].event_type(), "test_passed");
    assert_eq!(events[2].event_type(), "generation_complete");
}

#[test]
fn test_custom_event() {
    let event = SystemEvent::Custom(CustomEvent {
        name: "my_custom_event".to_string(),
        data: serde_json::json!({
            "key1": "value1",
            "key2": 42,
        }),
        timestamp: "2024-01-01T12:00:00Z".to_string(),
    });

    assert_eq!(event.event_type(), "custom");

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("my_custom_event"));
    assert!(json.contains("value1"));
}

#[test]
fn test_hook_with_tool_call_action() {
    let mut registry = InMemoryHookRegistry::new();

    let hook = Hook {
        id: "tool-call-test".to_string(),
        name: "Tool Call Test".to_string(),
        description: None,
        event: "file_saved".to_string(),
        action: Action::ToolCall(ToolCallAction {
            tool_name: "formatter".to_string(),
            tool_path: "/usr/local/bin/prettier".to_string(),
            parameters: ParameterBindings {
                bindings: std::collections::HashMap::new(),
            },
            timeout_ms: Some(5000),
        }),
        enabled: true,
        tags: vec![],
        metadata: serde_json::json!({}),
        condition: None,
    };

    registry.register_hook(hook).unwrap();

    let mut cli = HookCli::new(registry);
    let result = cli
        .execute(HookCommand::Inspect {
            id: "tool-call-test".to_string(),
            format: None,
        })
        .unwrap();

    assert!(result.contains("Tool Call Test"));
    assert!(result.contains("Tool Call"));
}

#[test]
fn test_hook_with_ai_prompt_action() {
    let mut registry = InMemoryHookRegistry::new();

    let hook = Hook {
        id: "ai-prompt-test".to_string(),
        name: "AI Prompt Test".to_string(),
        description: None,
        event: "generation_complete".to_string(),
        action: Action::AiPrompt(AiPromptAction {
            prompt_template: "Review the generated code".to_string(),
            variables: std::collections::HashMap::new(),
            model: Some("gpt-4".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            stream: false,
        }),
        enabled: true,
        tags: vec![],
        metadata: serde_json::json!({}),
        condition: None,
    };

    registry.register_hook(hook).unwrap();

    let mut cli = HookCli::new(registry);
    let result = cli
        .execute(HookCommand::Inspect {
            id: "ai-prompt-test".to_string(),
            format: None,
        })
        .unwrap();

    assert!(result.contains("AI Prompt Test"));
    assert!(result.contains("AI Prompt"));
}

#[test]
fn test_hook_with_chain_action() {
    let mut registry = InMemoryHookRegistry::new();

    let hook = Hook {
        id: "chain-test".to_string(),
        name: "Chain Test".to_string(),
        description: None,
        event: "file_saved".to_string(),
        action: Action::Chain(ChainAction {
            hook_ids: vec!["hook1".to_string(), "hook2".to_string()],
            pass_output: true,
        }),
        enabled: true,
        tags: vec![],
        metadata: serde_json::json!({}),
        condition: None,
    };

    registry.register_hook(hook).unwrap();

    let mut cli = HookCli::new(registry);
    let result = cli
        .execute(HookCommand::Inspect {
            id: "chain-test".to_string(),
            format: None,
        })
        .unwrap();

    assert!(result.contains("Chain Test"));
    assert!(result.contains("Chain"));
}

#[test]
fn test_hook_with_condition() {
    let mut registry = InMemoryHookRegistry::new();

    let hook = Hook {
        id: "condition-test".to_string(),
        name: "Condition Test".to_string(),
        description: None,
        event: "file_saved".to_string(),
        action: Action::Command(CommandAction {
            command: "echo".to_string(),
            args: vec![],
            timeout_ms: None,
            capture_output: false,
        }),
        enabled: true,
        tags: vec![],
        metadata: serde_json::json!({}),
        condition: Some(Condition {
            expression: "file_path.ends_with('.rs')".to_string(),
            context_keys: vec!["file_path".to_string()],
        }),
    };

    registry.register_hook(hook).unwrap();

    let mut cli = HookCli::new(registry);
    let result = cli
        .execute(HookCommand::Inspect {
            id: "condition-test".to_string(),
            format: None,
        })
        .unwrap();

    assert!(result.contains("Condition Test"));
}

#[test]
fn test_build_success_event() {
    let event = SystemEvent::BuildSuccess(BuildSuccessEvent {
        target: "release".to_string(),
        duration_ms: 5000,
        artifacts: vec!["/path/to/binary".to_string(), "/path/to/lib.a".to_string()],
        timestamp: "2024-01-01T12:00:00Z".to_string(),
    });

    assert_eq!(event.event_type(), "build_success");
}

#[test]
fn test_deployment_complete_event() {
    let event = SystemEvent::DeploymentComplete(DeploymentCompleteEvent {
        target: "production".to_string(),
        environment: "aws".to_string(),
        duration_ms: 10000,
        timestamp: "2024-01-01T12:00:00Z".to_string(),
    });

    assert_eq!(event.event_type(), "deployment_complete");
}
