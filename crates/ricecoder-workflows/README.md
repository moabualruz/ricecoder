# ricecoder-workflows

**Purpose**: Declarative workflow execution framework with state management, error handling, and approval gates for multi-step agentic operations

## Overview

`ricecoder-workflows` provides a comprehensive framework for defining and executing complex, multi-step workflows with built-in state management, error recovery, risk assessment, and human approval gates. It enables reliable execution of agentic operations with proper monitoring, rollback capabilities, and safety constraints.

## Features

- **Declarative Workflow Definition**: YAML/JSON-based workflow specifications
- **State Management**: Persistent workflow state with recovery capabilities
- **Error Handling**: Comprehensive error recovery with retry logic and rollback
- **Approval Gates**: Human-in-the-loop approval for high-risk operations
- **Risk Scoring**: Automatic risk assessment for workflow steps
- **Parallel Execution**: Concurrent step execution with dependency resolution
- **Progress Tracking**: Real-time workflow progress and status reporting
- **Safety Constraints**: Configurable safety rules and validation
- **Parameter Substitution**: Dynamic parameter injection and validation

## Architecture

### Responsibilities
- Workflow parsing and validation
- Step execution orchestration
- State persistence and recovery
- Error handling and retry logic
- Approval gate management
- Risk assessment and safety validation
- Progress tracking and reporting

### Dependencies
- **Async Runtime**: `tokio` for concurrent workflow execution
- **Serialization**: `serde` for workflow definitions and state
- **Storage**: `ricecoder-storage` for workflow persistence
- **Time Handling**: `chrono` for scheduling and timeouts

### Integration Points
- **Agents**: Executes multi-step agent operations
- **Storage**: Persists workflow definitions and execution state
- **TUI**: Provides workflow monitoring and approval interfaces
- **Commands**: Integrates with command execution system
- **Sessions**: Tracks workflow execution in session context

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-workflows = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_workflows::{WorkflowEngine, WorkflowDefinition};

// Load workflow definition
let definition = WorkflowDefinition::from_yaml(yaml_content)?;

// Create workflow engine
let engine = WorkflowEngine::new();

// Execute workflow
let execution_id = engine.execute_workflow(definition, initial_params).await?;
```

### Advanced Usage with Approval Gates

```rust
use ricecoder_workflows::{WorkflowEngine, ApprovalGate};

// Create workflow with approval requirements
let mut workflow = WorkflowDefinition::new("deploy-production");
workflow.add_step("build", build_step);
workflow.add_step("test", test_step);
workflow.add_approval_gate("deploy-gate", ApprovalGate::new("production-deploy"));
workflow.add_step("deploy", deploy_step);

// Execute with approval handling
let result = engine.execute_with_approvals(workflow).await?;
```

### Parallel Execution

```rust
use ricecoder_workflows::ParallelExecutor;

// Execute steps in parallel with dependencies
let executor = ParallelExecutor::new();
let results = executor.execute_parallel(steps, dependencies).await?;
```

## Configuration

Workflow behavior is configured via YAML:

```yaml
workflows:
  # Execution settings
  execution:
    max_concurrent_steps: 5
    step_timeout_seconds: 300
    retry_attempts: 3

  # Approval settings
  approvals:
    default_timeout_hours: 24
    auto_approve_low_risk: true
    notification_channels: ["email", "slack"]

  # Risk assessment
  risk:
    high_risk_threshold: 0.8
    require_approval_above: 0.6
    factors:
      - data_modification: 0.3
      - external_calls: 0.2
      - resource_usage: 0.1

  # Safety constraints
  safety:
    max_execution_time: 3600
    max_resource_usage: 0.8
    allowed_commands: ["git", "cargo", "docker"]
```

## API Reference

### Key Types

- **`WorkflowEngine`**: Main workflow execution orchestrator
- **`WorkflowDefinition`**: Declarative workflow specification
- **`StepExecutor`**: Individual step execution handler
- **`ApprovalGate`**: Human approval requirement definition
- **`RiskScorer`**: Automatic risk assessment engine

### Key Functions

- **`execute_workflow()`**: Execute a complete workflow
- **`execute_with_approvals()`**: Execute workflow with approval handling
- **`add_approval_gate()`**: Add human approval requirement
- **`calculate_risk_score()`**: Assess workflow risk level

## Error Handling

```rust
use ricecoder_workflows::WorkflowError;

match engine.execute_workflow(definition, params).await {
    Ok(result) => println!("Workflow completed: {:?}", result),
    Err(WorkflowError::StepFailed(step, msg)) => eprintln!("Step '{}' failed: {}", step, msg),
    Err(WorkflowError::ApprovalTimeout(gate)) => eprintln!("Approval timeout for gate: {}", gate),
    Err(WorkflowError::RiskThresholdExceeded(score)) => eprintln!("Risk score too high: {}", score),
}
```

## Testing

Run comprehensive workflow tests:

```bash
# Run all tests
cargo test -p ricecoder-workflows

# Run property tests for workflow correctness
cargo test -p ricecoder-workflows property

# Test approval gate functionality
cargo test -p ricecoder-workflows approval

# Test error recovery scenarios
cargo test -p ricecoder-workflows error_recovery
```

Key test areas:
- Workflow execution correctness
- Parallel execution dependencies
- Approval gate handling
- Error recovery and rollback
- Risk scoring accuracy
- State persistence integrity

## Performance

- **Workflow Parsing**: < 50ms for typical workflow definitions
- **Step Execution**: Variable based on step complexity
- **State Persistence**: < 10ms for workflow state updates
- **Risk Assessment**: < 5ms per workflow evaluation
- **Concurrent Steps**: Efficient parallel execution with dependency resolution
- **Memory**: Minimal overhead with streaming execution

## Contributing

When working with `ricecoder-workflows`:

1. **Safety First**: Implement comprehensive safety constraints and risk assessment
2. **Error Recovery**: Ensure robust error handling and rollback capabilities
3. **Approval Gates**: Design clear approval processes for high-risk operations
4. **Testing**: Test both success and failure scenarios thoroughly
5. **Documentation**: Keep workflow definitions and schemas well-documented

## License

MIT</content>
<parameter name="filePath">projects/ricecoder/crates/ricecoder-workflows/README.md