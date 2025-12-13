# ricecoder-agents

**Purpose**: Multi-agent framework providing specialized task execution, coordination, and intelligent workflow automation for RiceCoder

## Overview

`ricecoder-agents` implements a sophisticated multi-agent framework that enables specialized task execution, intelligent coordination, and automated workflow management. It provides a composable architecture for building complex AI-driven workflows with proper error handling and async-first design.

## Features

- Composable agent architecture
- Async-first design for non-blocking I/O
- Specialized agents for different tasks
- Workflow orchestration
- Error transparency and explicit error handling

## Architecture

### Responsibilities
- Agent lifecycle management and coordination
- Task execution orchestration and scheduling
- Workflow state management and persistence
- Inter-agent communication and data flow
- Error handling and recovery mechanisms

### Dependencies
- **Async Runtime**: `tokio` for concurrent agent operations
- **Storage**: `ricecoder-storage` for agent state persistence
- **Serialization**: `serde` for agent configuration and data
- **Providers**: `ricecoder-providers` for AI capabilities

### Integration Points
- **Workflows**: Agents execute workflow steps with specialized capabilities
- **Sessions**: Agent interactions integrated into conversation sessions
- **Storage**: Agent state and configuration persistence
- **Providers**: AI providers power agent intelligence

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-agents = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_agents::{AgentManager, CodeReviewAgent, TestingAgent};

// Create agent manager
let manager = AgentManager::new().await?;

// Register specialized agents
manager.register_agent(Box::new(CodeReviewAgent::new())).await?;
manager.register_agent(Box::new(TestingAgent::new())).await?;

// Execute task with appropriate agent
let result = manager.execute_task("review-code", &context).await?;
println!("Agent result: {:?}", result);
```

## API Reference

### Key Types

- **`AgentManager`**: Central agent coordination and management
- **`Agent`**: Trait defining agent interface and capabilities
- **`TaskContext`**: Context information for agent task execution
- **`AgentResult`**: Result structure for agent task completion

### Key Functions

- **`register_agent()`**: Register a new agent with the manager
- **`execute_task()`**: Execute a task using the most appropriate agent
- **`get_agent_for_task()`**: Find the best agent for a specific task type

## Error Handling

```rust
use ricecoder_agents::AgentError;

match manager.execute_task("complex-task", &context).await {
    Ok(result) => println!("Task completed: {:?}", result),
    Err(AgentError::NoSuitableAgent) => eprintln!("No agent available for this task"),
    Err(AgentError::TaskExecutionFailed(msg)) => eprintln!("Task failed: {}", msg),
    Err(AgentError::AgentTimeout) => eprintln!("Agent execution timed out"),
}
```

## Testing

Run comprehensive agent tests:

```bash
# Run all tests
cargo test -p ricecoder-agents

# Run property tests for agent behavior
cargo test -p ricecoder-agents property

# Test agent coordination
cargo test -p ricecoder-agents coordination

# Test workflow integration
cargo test -p ricecoder-agents workflow
```

Key test areas:
- Agent task execution correctness
- Inter-agent coordination and communication
- Error handling and recovery scenarios
- Performance under concurrent load

## Performance

- **Task Execution**: Variable based on agent complexity (100ms - 30s)
- **Agent Coordination**: < 50ms for agent selection and task routing
- **Concurrent Tasks**: Safe execution of multiple agents simultaneously
- **Memory Usage**: Efficient agent state management with cleanup
- **Scalability**: Support for multiple agent instances and load balancing

## Contributing

When working with `ricecoder-agents`:

1. **Agent Specialization**: Keep agents focused on specific domains and capabilities
2. **Async Design**: Ensure all agent operations are properly async and non-blocking
3. **Error Transparency**: Provide clear error messages and recovery options
4. **Testing**: Thoroughly test agent behavior with various inputs and scenarios
5. **Documentation**: Document agent capabilities, limitations, and use cases

## License

MIT
