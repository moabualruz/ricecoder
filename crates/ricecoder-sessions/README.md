# ricecoder-sessions

Session Management for RiceCoder - Business Logic Layer

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

### 🔗 Integration Points:
- **Storage**: Uses `ricecoder-storage` for persistence
- **TUI**: Provides interfaces for UI integration (but doesn't depend on TUI)
- **Providers**: Coordinates with AI providers for message processing

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

## Key Components

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

## Contributing

When working with `ricecoder-sessions`:

1. **Keep business logic here**: Session management belongs in this crate
2. **Use interfaces for UI**: Don't depend on TUI crates
3. **Test thoroughly**: Session operations are critical for user data
4. **Document data formats**: Keep storage formats well-documented

## License

MIT
