# ricecoder-sessions

**Purpose**: Comprehensive session management and conversation handling for RiceCoder with persistence, sharing, and background processing

## Overview

`ricecoder-sessions` provides comprehensive session management functionality that was extracted from the TUI during the architectural refactoring. This crate handles all session-related business logic independently of the user interface.

## Features

- **Session CRUD Operations**: Create, read, update, delete sessions
- **Message Management**: Handle conversation messages with metadata
- **Persistence**: Save/load sessions to/from storage
- **Token Tracking**: Monitor token usage and limits
- **Session Compaction**: Reduce context size for long conversations
- **Export Functionality**: Export sessions to Markdown and other formats
- **Undo/Redo**: Message-level undo/redo operations
- **Background Processing**: Handle async operations and agents

## Architecture

After the TUI isolation refactoring, session management was moved from `ricecoder-tui` to this dedicated crate:

### ✅ Responsibilities:
- Session lifecycle management
- Message storage and retrieval
- Token counting and limits
- Session serialization/deserialization
- Background agent coordination
- Session sharing and export

### Dependencies
- **Async Runtime**: `tokio` for concurrent operations
- **Serialization**: `serde` for session persistence
- **Time Handling**: `chrono` for timestamps and scheduling
- **Storage**: `ricecoder-storage` for data persistence
- **UUID**: `uuid` for unique identifiers

### Integration Points
- **Storage**: Uses `ricecoder-storage` for session persistence and metadata
- **TUI**: Provides session interfaces for terminal UI (dependency injection)
- **Providers**: Coordinates with AI providers for message processing and token tracking
- **Background Agents**: Manages async operations and agent execution
- **Sharing**: Integrates with session sharing and export functionality

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-sessions = "0.1"
```

## Usage

```rust
use ricecoder_sessions::{SessionManager, Session};

let mut manager = SessionManager::new();

// Create a new session
let session_id = manager.create_session().await?;

// Add messages
let mut message = Message::new(MessageRole::User);
message.add_text("Hello, world!");
manager.add_message(&session_id, message).await?;

// Get session data
let session = manager.get_session(&session_id).await?;
```

## Configuration

Session behavior is configured via YAML:

```yaml
sessions:
  # Storage settings
  storage:
    max_sessions: 1000
    auto_save: true
    backup_interval_minutes: 30

  # Token management
  tokens:
    max_context_tokens: 100000
    compaction_threshold: 0.8
    reserve_tokens: 1000

  # Background processing
  background:
    max_concurrent_agents: 5
    agent_timeout_seconds: 300
    retry_attempts: 3

  # Sharing settings
  sharing:
    default_expiration_hours: 24
    max_shared_sessions: 50
    require_authentication: false
```

## API Reference

- **`SessionManager`**: Main entry point for session operations
- **`Session`**: Represents a conversation session
- **`Message`**: Individual messages with role, content, and metadata
- **`TokenUsage`**: Tracks token consumption
- **`SessionCompactor`**: Handles context size management

## Data Model

```rust
pub struct Session {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: SessionMetadata,
}

pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub parts: Vec<MessagePart>,
    pub timestamp: DateTime<Utc>,
    pub metadata: MessageMetadata,
}
```

## Session Storage

Sessions are stored in the following structure:
```
~/.ricecoder/sessions/
├── index.json              # Session index
├── {session-id}/
│   ├── metadata.json       # Session metadata
│   ├── messages.json       # Session messages
│   └── snapshots/          # Undo/redo snapshots
```

## Integration

`ricecoder-sessions` is designed to work with other RiceCoder components:

- **CLI Integration**: Main application coordinates TUI and sessions
- **Provider Integration**: Sessions trigger AI provider calls
- **Storage Integration**: Sessions persist via storage layer
- **TUI Integration**: Sessions provide data to UI (no direct dependency)

## Error Handling

```rust
use ricecoder_sessions::SessionError;

match manager.create_session().await {
    Ok(session_id) => println!("Created session: {}", session_id),
    Err(SessionError::StorageError(msg)) => eprintln!("Storage error: {}", msg),
    Err(SessionError::ValidationError(msg)) => eprintln!("Validation error: {}", msg),
    Err(SessionError::NotFound(id)) => eprintln!("Session not found: {}", id),
}
```

## Testing

Run comprehensive session tests:

```bash
# Run all tests
cargo test -p ricecoder-sessions

# Run property tests for session correctness
cargo test -p ricecoder-sessions property

# Test persistence and recovery
cargo test -p ricecoder-sessions persistence

# Test sharing functionality
cargo test -p ricecoder-sessions sharing
```

Key test areas:
- Session lifecycle operations
- Message ordering and persistence
- Token tracking accuracy
- Background agent execution
- Session sharing and export

## Performance

- **Session Creation**: < 10ms for new sessions
- **Message Addition**: < 5ms per message with persistence
- **Session Loading**: < 50ms for typical sessions (< 100 messages)
- **Token Tracking**: Minimal overhead (< 1ms per message)
- **Concurrent Access**: Safe for multiple concurrent operations
- **Memory**: Efficient storage with optional compaction

## Contributing

When working with `ricecoder-sessions`:

1. **Keep business logic here**: Session management belongs in this crate
2. **Use interfaces for UI**: Don't depend on TUI crates
3. **Test thoroughly**: Session operations are critical for user data
4. **Document data formats**: Keep storage formats well-documented

## License

MIT
