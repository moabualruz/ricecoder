# ricecoder-activity-log

Structured activity logging and audit trails for RiceCoder.

## DDD Layer

**Layer**: Infrastructure

### Responsibilities

- Structured activity event logging
- Audit trail generation and storage
- Session activity tracking
- Compliance-ready logging formats
- Event monitoring and alerting

### SOLID Analysis

| Principle | Score | Notes |
|-----------|-------|-------|
| SRP | ✅ | Each component handles single concern (Logger, Storage, Events) |
| OCP | ✅ | Extensible via event types and storage backends |
| LSP | ✅ | Consistent logging interfaces |
| ISP | ✅ | Segregated concerns (events, storage, monitoring) |
| DIP | ✅ | Depends on session abstractions |

**Score**: 5/5

### Integration Points

| Component | Direction | Purpose |
|-----------|-----------|---------|
| ricecoder-sessions | Depends on | Session context for activity tracking |
| ricecoder-security | Used by | Security audit logging |
| ricecoder-api | Used by | API request logging |

## Features

- **Event Types**: User actions, system events, errors, security events
- **Storage Backends**: File, database, streaming
- **Monitoring**: Real-time event monitoring
- **Compliance**: SOC 2, GDPR-ready audit trails

## Usage

```rust
use ricecoder_activity_log::{ActivityLogger, Event};

let logger = ActivityLogger::new(config)?;
logger.log_event(Event::user_action("code_review", metadata)).await?;
```

## License

MIT
