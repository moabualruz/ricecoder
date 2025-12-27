# Slash Command Integration Guide

## Overview

The `TaskTool::execute_slash_command()` method enables execution of markdown-based slash commands as agent tasks.

## How It Works

### 1. Command Detection (Already Implemented)
Location: `ricecoder-tui/src/tui/prompt/handler.rs:459-464`

```rust
// Slash command
else if input.starts_with('/') {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let command = parts[0].trim_start_matches('/').to_string();
    let args = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
    self.pending_events.push(PromptEvent::CommandSubmit { command, args });
}
```

### 2. Command Execution (NEW - Ready to Use)
Location: `ricecoder-agents/src/tools/task.rs:321-415`

```rust
/// Execute a slash command from markdown config
pub async fn execute_slash_command(
    &self,
    command_name: &str,              // e.g., "commit" (no "/" prefix)
    args: Option<&str>,              // User's arguments after the command
    context: TaskExecutionContext,   // Execution context with session info
) -> Result<TaskResult>
```

### 3. Integration Point (TODO - TUI Layer)

The TUI needs to handle `PromptEvent::CommandSubmit` events:

```rust
// Pseudocode for TUI event loop
match prompt_event {
    PromptEvent::CommandSubmit { command, args } => {
        // Check if this is a markdown command
        if let Ok(task_tool) = app_context.get_task_tool() {
            // Build execution context
            let context = TaskExecutionContext {
                session_id: current_session_id.clone(),
                message_id: generate_message_id(),
                model: current_model_config.clone(),
                metadata_callback: Arc::new(|meta| { /* handle metadata */ }),
                abort_rx: Some(abort_receiver.clone()),
            };
            
            // Execute command as agent task
            let result = task_tool.execute_slash_command(
                &command,
                Some(&args),
                context
            ).await?;
            
            // Display result in chat
            display_agent_response(result);
        }
    }
    // ... other events
}
```

## Example Flow

User types: `/commit add error handling`

1. **Prompt Handler** detects "/" → emits `PromptEvent::CommandSubmit { command: "commit", args: "add error handling" }`
2. **TUI Event Loop** receives event → calls `task_tool.execute_slash_command("commit", Some("add error handling"), context)`
3. **TaskTool** loads `config/commands/commit.md`:
   ```markdown
   ---
   description: git commit and push
   model: opencode/glm-4.6
   subtask: true
   ---
   
   commit and push
   
   make sure it includes a prefix like docs:, tui:, core:...
   ```
4. **TaskTool** builds prompt: `"add error handling\n\ncommit and push\n\nmake sure it includes a prefix..."`
5. **TaskTool** executes as task with "general" agent
6. **Agent** performs git commit with proper message formatting
7. **Result** returned to TUI → displayed in chat

## Command Structure

Markdown commands support these frontmatter fields:

- `description`: Human-readable command description
- `model`: Optional model override (format: "provider/model")
- `subtask`: Boolean - if true, runs as background subtask
- `argument-hint`: Optional hint shown in command palette

The markdown body becomes the agent prompt, prefixed with user args.

## Next Steps

To fully wire this up, the TUI needs to:

1. Find where `PromptEvent`s are consumed (likely in main app loop or prompt component)
2. Add handler for `PromptEvent::CommandSubmit`
3. Call `TaskTool::execute_slash_command()` with appropriate context
4. Display results in chat UI

## Status

- ✅ Command loading (`CommandLoader`)
- ✅ Command detection (prompt handler)
- ✅ Command execution method (`TaskTool::execute_slash_command`)
- ❌ TUI event handler (needs implementation)
- ❌ UI result display (needs implementation)

The backend infrastructure is complete. Only the TUI event loop integration remains.
