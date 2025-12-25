# ricecoder-files

**Purpose**: Safe file operations with atomic writes, backups, rollback support, and comprehensive audit logging for RiceCoder

## DDD Layer

**Infrastructure** - Implements file system abstractions for the RiceCoder ecosystem.

## Overview

`ricecoder-files` provides a comprehensive file management system designed for safe, reliable file operations in development environments. It features atomic writes, automatic backups, conflict resolution, transaction support, and complete audit logging to ensure data integrity and recoverability.

## Features

- **Atomic Writes**: File operations that are either completely successful or completely rolled back
- **Automatic Backups**: Configurable backup creation before file modifications
- **Transaction Support**: Multi-file operations with ACID-like properties
- **Conflict Resolution**: Intelligent detection and resolution of file conflicts
- **Audit Logging**: Complete audit trail of all file operations
- **Git Integration**: Seamless integration with Git for version control
- **File Watching**: Real-time monitoring of file system changes (using `notify` crate)
- **Content Verification**: Integrity checking via SHA-256 hashing
- **Rollback Support**: Ability to undo operations and restore previous states

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        FileManager                               │
│           (Central coordinator for all file operations)          │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   SafeWriter    │  │ TransactionMgr  │  │  BackupManager  │  │
│  │  (Atomic ops)   │  │  (ACID multi-   │  │  (Retention &   │  │
│  │                 │  │   file ops)     │  │   restoration)  │  │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘  │
│           │                    │                    │           │
│  ┌────────┴────────┐  ┌────────┴────────┐  ┌────────┴────────┐  │
│  │ConflictResolver │  │    DiffEngine   │  │ContentVerifier  │  │
│  │(Skip/Overwrite/ │  │  (unified diff  │  │  (SHA-256 hash  │  │
│  │     Merge)      │  │   generation)   │  │   integrity)    │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│                                                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   AuditLogger   │  │ GitIntegration  │  │   FileWatcher   │  │
│  │  (JSON audit    │  │  (git2 bindings │  │  (notify crate  │  │
│  │     trail)      │  │   for VCS)      │  │   debouncing)   │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│                                                                   │
└───────────────────────────┬─────────────────────────────────────┘
                            │ implements
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                   ricecoder-domain Ports                         │
│    FileReader  │  FileWriter  │  FileManager (trait)            │
└─────────────────────────────────────────────────────────────────┘
```

### Core Components

| Component | Responsibility | Key Types |
|-----------|----------------|-----------|
| `FileManager` | Central coordinator, orchestrates all operations | `FileManager` |
| `SafeWriter` | Atomic writes with temp file + rename pattern | `SafeWriter` |
| `TransactionManager` | Multi-file ACID transactions | `TransactionManager`, `FileTransaction` |
| `BackupManager` | Timestamped backups with retention policy | `BackupManager`, `BackupMetadata` |
| `ConflictResolver` | Detect and resolve file conflicts | `ConflictResolver`, `ConflictResolution` |
| `ContentVerifier` | SHA-256 hash verification | `ContentVerifier` |
| `DiffEngine` | Unified diff generation (via `similar` crate) | `DiffEngine`, `FileDiff`, `DiffHunk` |
| `AuditLogger` | Persistent JSON audit trail | `AuditLogger`, `AuditEntry` |
| `GitIntegration` | Git status, staging, commits (via `git2`) | `GitIntegration`, `GitStatus` |
| `FileWatcher` | File system monitoring (via `notify`) | `FileWatcher`, `FileChangeBatch` |
| `FileSystemRepository` | Domain port implementation | `FileSystemRepository` |

### SOLID Compliance

| Principle | Implementation |
|-----------|----------------|
| **SRP** | Each component has single responsibility (e.g., `BackupManager` only handles backups) |
| **OCP** | Extensible via `ConflictResolution` enum strategies |
| **LSP** | `FileSystemRepository` implements domain traits substitutably |
| **ISP** | Domain traits split: `FileReader`, `FileWriter`, `FileManager` |
| **DIP** | Depends on `ricecoder-domain` abstractions, not concretions |

### Dependencies

#### Internal (RiceCoder Crates)
- `ricecoder-domain`: Port interfaces (`FileReader`, `FileWriter`, `FileManager` traits)

#### External Libraries
- `tokio`: Async runtime for concurrent file operations
- `git2`: libgit2 bindings for Git integration
- `similar`: Fast text diffing (Myers algorithm)
- `notify`: Cross-platform file system notifications
- `sha2`: SHA-256 hashing for content verification
- `serde`/`serde_json`: Serialization for audit logs and metadata
- `chrono`: Timestamp handling
- `uuid`: Transaction and backup IDs
- `thiserror`: Error type derivation
- `tracing`: Structured logging
- `async-trait`: Async trait support

### Integration Points
- **All Crates**: Provides file operations for the entire RiceCoder ecosystem
- **Domain Layer**: Implements `FileRepository` port via `FileSystemRepository`
- **Sessions**: Manages session file operations with rollback
- **Commands**: Safe file operations for command execution
- **TUI**: File picker and file management interfaces

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-files = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_files::{FileManager, SafeWriter};

// Create file manager
let manager = FileManager::new().await?;

// Safe file write with automatic backup
manager.write_file("config.toml", content, true).await?;
```

### Transaction Support

```rust
use ricecoder_files::{TransactionManager, FileTransaction};

// Create transaction manager
let tx_manager = TransactionManager::new();

// Start transaction
let mut transaction = FileTransaction::new("update-configs");

// Add operations to transaction
transaction.add_operation(FileOperation::Write {
    path: "config.toml".into(),
    content: new_config,
    backup: true,
});

// Execute transaction (all or nothing)
tx_manager.execute_transaction(transaction).await?;
```

### Backup and Rollback

```rust
use ricecoder_files::BackupManager;

// Create backup manager
let backup_manager = BackupManager::new(backup_dir);

// Create backup before modification
let backup_id = backup_manager.create_backup("important.txt").await?;

// Modify file...

// Rollback if needed
backup_manager.restore_backup(&backup_id).await?;
```

### File Watching

```rust
use ricecoder_files::{FileWatcher, WatcherConfig};

// Create file watcher
let config = WatcherConfig {
    debounce_ms: 100,
    watch_paths: vec!["src/".into(), "tests/".into()],
    ..Default::default()
};

let mut watcher = FileWatcher::new(config).await?;

// Watch for changes
while let Some(batch) = watcher.next_batch().await {
    for change in batch.changes {
        println!("File changed: {:?}", change.path);
    }
}
```

## Configuration

File operations configuration via YAML:

```yaml
files:
  # Backup settings
  backup:
    enabled: true
    directory: "~/.ricecoder/backups"
    retention_days: 30
    max_backups_per_file: 10

  # Transaction settings
  transactions:
    timeout_seconds: 300
    max_concurrent: 5
    auto_rollback_on_failure: true

  # Audit settings
  audit:
    enabled: true
    log_directory: "~/.ricecoder/audit"
    max_log_size_mb: 100
    retention_days: 365

  # Watcher settings
  watcher:
    debounce_ms: 100
    max_batch_size: 100
    ignored_patterns: ["*.tmp", "*.log"]
```

## API Reference

### Key Types

- **`FileManager`**: Main file operations coordinator
- **`SafeWriter`**: Atomic file writing with backup support
- **`TransactionManager`**: Multi-file transaction coordination
- **`BackupManager`**: Backup creation and restoration
- **`FileWatcher`**: File system change monitoring

### Key Functions

- **`write_file()`**: Safe file writing with optional backup
- **`read_file()`**: File reading with integrity checking
- **`execute_transaction()`**: Execute multi-file transaction
- **`create_backup()`**: Create backup of file or directory
- **`watch_files()`**: Monitor file system changes

## Error Handling

```rust
use ricecoder_files::FileError;

match manager.write_file(path, content, true).await {
    Ok(()) => println!("File written successfully"),
    Err(FileError::BackupFailed(msg)) => eprintln!("Backup failed: {}", msg),
    Err(FileError::WriteFailed(msg)) => eprintln!("Write failed: {}", msg),
    Err(FileError::PermissionDenied) => eprintln!("Permission denied"),
}
```

## Testing

Run comprehensive file operation tests:

```bash
# Run all tests
cargo test -p ricecoder-files

# Run property tests for file operations
cargo test -p ricecoder-files property

# Test transaction safety
cargo test -p ricecoder-files transaction

# Test backup integrity
cargo test -p ricecoder-files backup
```

Key test areas:
- Atomic write correctness
- Transaction rollback integrity
- Backup restoration accuracy
- Conflict resolution effectiveness
- Audit logging completeness

## Performance

- **File Writes**: < 10ms for small files with backup
- **Transaction Execution**: < 100ms for typical multi-file operations
- **Backup Creation**: < 50ms for average-sized files
- **File Watching**: < 5ms debounce for change detection
- **Audit Logging**: Minimal overhead (< 1ms per operation)

## Contributing

When working with `ricecoder-files`:

1. **Safety First**: Ensure all operations maintain data integrity
2. **Atomicity**: Operations should be atomic or provide clear rollback paths
3. **Auditability**: All file changes must be logged for compliance
4. **Performance**: Optimize for common file sizes and operations
5. **Testing**: Test both success and failure scenarios thoroughly

## Test Coverage

The crate has comprehensive test coverage with 12 dedicated test files:

| Test File | Coverage Area |
|-----------|---------------|
| `atomic_write_properties.rs` | Property tests for atomic write correctness |
| `audit_trail_integration.rs` | Audit logging integration tests |
| `audit_trail_properties.rs` | Property tests for audit trail consistency |
| `backup_integrity_properties.rs` | Property tests for backup hash integrity |
| `backup_restoration_properties.rs` | Property tests for backup restoration |
| `conflict_detection_properties.rs` | Property tests for conflict detection |
| `conflict_resolution_integration.rs` | Integration tests for conflict resolution |
| `content_verification_properties.rs` | Property tests for SHA-256 verification |
| `diff_round_trip_properties.rs` | Property tests for diff reversibility |
| `git_operations_integration.rs` | Git integration tests |
| `multi_file_transactions_integration.rs` | Transaction integration tests |
| `transaction_rollback_properties.rs` | Property tests for rollback correctness |

Run tests:
```bash
cargo test -p ricecoder-files
```

## License

MIT