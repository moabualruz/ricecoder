# ricecoder-execution

**Purpose**: Execution planning and management system with test integration, approval gates, and rollback capabilities for RiceCoder workflows

## Overview

`ricecoder-execution` provides a comprehensive execution framework for managing complex, multi-step operations with built-in testing, risk assessment, user approval gates, and automatic rollback capabilities. It enables safe execution of workflows with proper validation, progress tracking, and recovery mechanisms.

## Features

- **Execution Plans**: Structured execution planning with dependency management
- **Test Integration**: Automatic test running before and after execution steps
- **Approval Gates**: Human-in-the-loop approval for high-risk operations
- **Rollback Support**: Automatic and manual rollback for failed operations
- **Risk Assessment**: Execution risk scoring and safety validation
- **Progress Tracking**: Real-time execution progress and status reporting
- **Multiple Execution Modes**: Automatic, step-by-step, and dry-run modes
- **File Operations**: Safe file creation, modification, and deletion with backups

## Architecture

### Responsibilities
- Execution plan creation and validation
- Step-by-step execution orchestration
- Test integration and validation
- Approval gate management and user interaction
- Risk assessment and safety checking
- Rollback planning and execution
- Progress tracking and status reporting

### Dependencies
- **Async Runtime**: `tokio` for concurrent execution
- **File System**: `std::fs` and file operation utilities
- **Process Management**: Command execution and process handling
- **Storage**: `ricecoder-storage` for execution state persistence

### Integration Points
- **Workflows**: Executes workflow steps with proper sequencing
- **TUI**: Provides execution monitoring and approval interfaces
- **Storage**: Persists execution plans and rollback information
- **Commands**: Integrates with command execution system
- **Testing**: Runs tests as part of execution validation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-execution = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_execution::{ExecutionManager, ExecutionPlan};

// Create execution manager
let manager = ExecutionManager::new();

// Load execution plan
let plan = ExecutionPlan::from_yaml(plan_yaml)?;

// Execute plan
let result = manager.execute_plan(plan).await?;
```

### Advanced Usage with Approval Gates

```rust
use ricecoder_execution::{ExecutionManager, ApprovalManager};

// Create managers
let execution_manager = ExecutionManager::new();
let approval_manager = ApprovalManager::new();

// Execute with approval handling
let result = execution_manager.execute_with_approvals(plan, approval_manager).await?;
```

### Step-by-Step Execution

```rust
use ricecoder_execution::modes::StepByStepModeExecutor;

// Create step-by-step executor
let executor = StepByStepModeExecutor::new();

// Execute step by step with user confirmation
for step in plan.steps {
    let approved = executor.request_approval(&step).await?;
    if approved {
        executor.execute_step(step).await?;
    }
}
```

## Configuration

Execution behavior is configured via YAML:

```yaml
execution:
  # Risk assessment
  risk:
    high_risk_threshold: 0.8
    require_approval_above: 0.6
    factors:
      - file_modification: 0.4
      - command_execution: 0.3
      - data_changes: 0.3

  # Approval settings
  approvals:
    timeout_minutes: 60
    auto_approve_low_risk: true
    notification_enabled: true

  # Testing
  testing:
    run_tests_before: true
    run_tests_after: true
    fail_on_test_failure: true
    test_frameworks: ["cargo", "jest", "pytest"]

  # Rollback
  rollback:
    auto_rollback_on_failure: true
    backup_files: true
    max_rollback_steps: 10

  # Modes
  modes:
    default: "automatic"
    allow_step_by_step: true
    allow_dry_run: true
```

## API Reference

### Key Types

- **`ExecutionManager`**: Main execution orchestration coordinator
- **`ExecutionPlan`**: Structured execution plan with steps and dependencies
- **`ApprovalManager`**: Handles user approval requests and responses
- **`RollbackHandler`**: Manages rollback operations and recovery
- **`TestRunner`**: Integrates test execution into workflow steps

### Key Functions

- **`execute_plan()`**: Execute a complete execution plan
- **`execute_with_approvals()`**: Execute with approval gate handling
- **`request_approval()`**: Request user approval for high-risk operations
- **`rollback()`**: Rollback failed operations to safe state

## Error Handling

```rust
use ricecoder_execution::ExecutionError;

match manager.execute_plan(plan).await {
    Ok(result) => println!("Execution completed: {:?}", result),
    Err(ExecutionError::ApprovalRejected(step)) => eprintln!("Approval rejected for step: {}", step),
    Err(ExecutionError::TestFailure(results)) => eprintln!("Tests failed: {:?}", results),
    Err(ExecutionError::RollbackFailed(msg)) => eprintln!("Rollback failed: {}", msg),
}
```

## Testing

Run comprehensive execution tests:

```bash
# Run all tests
cargo test -p ricecoder-execution

# Run property tests for execution correctness
cargo test -p ricecoder-execution property

# Test approval gate functionality
cargo test -p ricecoder-execution approval

# Test rollback scenarios
cargo test -p ricecoder-execution rollback
```

Key test areas:
- Execution plan validation
- Step dependency resolution
- Approval gate handling
- Rollback correctness
- Test integration accuracy
- Risk scoring validation

## Performance

- **Plan Validation**: < 20ms for typical execution plans
- **Step Execution**: Variable based on step complexity
- **Risk Assessment**: < 5ms per execution evaluation
- **Approval Handling**: < 100ms for approval request/response
- **Rollback Operations**: < 50ms per rollback step
- **Test Integration**: Variable based on test suite size

## Contributing

When working with `ricecoder-execution`:

1. **Safety First**: Implement comprehensive risk assessment and approval gates
2. **Test Integration**: Ensure proper test running and failure handling
3. **Rollback Robustness**: Test rollback scenarios thoroughly
4. **User Experience**: Make approval processes clear and responsive
5. **Error Recovery**: Provide clear error messages and recovery options

## License

MIT