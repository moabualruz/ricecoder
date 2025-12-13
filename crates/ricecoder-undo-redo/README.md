# ricecoder-undo-redo

**Purpose**: Comprehensive undo/redo system providing change tracking, history management, and rollback capabilities for RiceCoder operations

## Overview

`ricecoder-undo-redo` implements a sophisticated undo/redo system that tracks all changes, maintains operation history, and enables reliable rollback of file operations and code generation. It provides granular change tracking with validation, persistence, and recovery mechanisms.

## Features

- **Change Tracking**: Comprehensive tracking of all file and code modifications
- **Undo/Redo Operations**: Multi-level undo and redo with history navigation
- **Checkpoint System**: Named checkpoints for important operation states
- **History Management**: Persistent operation history with metadata
- **Change Validation**: Validation of changes before application
- **Rollback Support**: Safe rollback to previous states
- **Storage Integration**: Persistent history storage and recovery
- **Performance Optimized**: Efficient change tracking for large operations

## Architecture

### Responsibilities
- Change capture and validation for all operations
- History management and persistence
- Undo/redo execution and state restoration
- Checkpoint creation and management
- Performance optimization for large change sets
- Error recovery and data integrity

### Dependencies
- **Storage**: `ricecoder-storage` for history persistence
- **Async Runtime**: `tokio` for concurrent operations
- **Serialization**: `serde` for change serialization
- **File System**: Standard library for file operations

### Integration Points
- **Files**: Tracks all file operations for undo/redo
- **Sessions**: Session-level change history and rollback
- **Commands**: Command execution tracking and reversal
- **TUI**: UI controls for undo/redo operations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-undo-redo = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_undo_redo::{HistoryManager, Change, ChangeType};

// Create history manager
let mut history = HistoryManager::new();

// Record a change
let change = Change {
    change_type: ChangeType::FileModification,
    description: "Modified main.rs".to_string(),
    data: change_data,
    ..Default::default()
};

history.record_change(change).await?;

// Undo last change
history.undo().await?;

// Redo undone change
history.redo().await?;
```

### Checkpoint System

```rust
use ricecoder_undo_redo::checkpoint::CheckpointManager;

// Create checkpoint manager
let mut checkpoints = CheckpointManager::new();

// Create named checkpoint
checkpoints.create_checkpoint("before-refactor", "Refactoring user module").await?;

// Perform operations...

// Rollback to checkpoint
checkpoints.rollback_to_checkpoint("before-refactor").await?;
```

### Change Validation

```rust
use ricecoder_undo_redo::models::ChangeValidator;

// Create change validator
let validator = ChangeValidator::new();

// Validate change before application
match validator.validate_change(&change).await {
    Ok(()) => apply_change(change).await?,
    Err(validation_errors) => {
        for error in validation_errors {
            eprintln!("Validation error: {}", error);
        }
    }
}
```

## Configuration

Undo/redo system configuration via YAML:

```yaml
undo_redo:
  # History settings
  history:
    max_entries: 1000
    retention_days: 30
    compress_old_entries: true

  # Checkpoint settings
  checkpoints:
    max_checkpoints: 50
    auto_checkpoint_interval: 300  # 5 minutes
    checkpoint_on_major_changes: true

  # Validation settings
  validation:
    enable_pre_validation: true
    strict_mode: false
    allow_dangerous_changes: false

  # Storage settings
  storage:
    persistent_history: true
    backup_history: true
    max_storage_size_mb: 100
```

## API Reference

### Key Types

- **`HistoryManager`**: Main undo/redo history management
- **`Change`**: Individual change representation
- **`CheckpointManager`**: Named checkpoint system
- **`ChangeValidator`**: Change validation before application
- **`HistoryStore`**: Persistent history storage

### Key Functions

- **`record_change()`**: Record a change in history
- **`undo()`**: Undo the last change
- **`redo()`**: Redo the previously undone change
- **`create_checkpoint()`**: Create a named checkpoint
- **`rollback_to_checkpoint()`**: Rollback to a specific checkpoint

## Error Handling

```rust
use ricecoder_undo_redo::UndoRedoError;

match history.undo().await {
    Ok(()) => println!("Change undone successfully"),
    Err(UndoRedoError::NoChangesToUndo) => eprintln!("No changes to undo"),
    Err(UndoRedoError::ChangeValidationFailed(msg)) => eprintln!("Validation failed: {}", msg),
    Err(UndoRedoError::StorageError(msg)) => eprintln!("Storage error: {}", msg),
}
```

## Testing

Run comprehensive undo/redo tests:

```bash
# Run all tests
cargo test -p ricecoder-undo-redo

# Run property tests for change tracking
cargo test -p ricecoder-undo-redo property

# Test history management
cargo test -p ricecoder-undo-redo history

# Test checkpoint system
cargo test -p ricecoder-undo-redo checkpoint
```

Key test areas:
- Change recording and replay accuracy
- History navigation correctness
- Checkpoint creation and rollback
- Validation rule enforcement
- Storage persistence and recovery

## Performance

- **Change Recording**: < 10ms per change
- **Undo/Redo Operations**: < 50ms for typical changes
- **History Navigation**: < 20ms for history browsing
- **Checkpoint Creation**: < 100ms for state snapshots
- **Validation**: < 5ms per change validation

## Contributing

When working with `ricecoder-undo-redo`:

1. **Data Integrity**: Ensure all operations maintain data consistency
2. **Performance**: Optimize for large change histories
3. **Validation**: Implement comprehensive change validation
4. **Error Recovery**: Provide clear rollback paths for failures
5. **Testing**: Test complex undo/redo scenarios thoroughly

## License

MIT
