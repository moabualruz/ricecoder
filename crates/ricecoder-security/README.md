# RiceCoder Security

Security utilities and cryptographic operations for RiceCoder.

## Features

- **API Key Encryption**: Secure encryption/decryption of API keys with AES-256-GCM
- **Input Validation**: Comprehensive input sanitization and validation
- **Audit Logging**: Structured security event logging with query capabilities
- **Access Control**: Role-based permission system with fine-grained controls
- **Security Testing**: Property-based tests for security invariants

## Usage

### API Key Management

```rust
use ricecoder_security::{KeyManager, Result};

let key_manager = KeyManager::new("master-password")?;
let api_key = "sk-your-secret-api-key";

// Encrypt the API key
let encrypted = key_manager.encrypt_api_key(api_key)?;

// Save to file
key_manager.save_to_file(&encrypted, "api_key.enc").await?;

// Load and decrypt later
let loaded = KeyManager::load_from_file("api_key.enc").await?;
let decrypted = key_manager.decrypt_api_key(&loaded)?;
assert_eq!(api_key, decrypted);
```

### Input Validation

```rust
use ricecoder_security::validate_input;

let user_input = "<script>alert('xss')</script>safe content";
let result = validate_input(user_input);

// Validation will detect and reject malicious input
assert!(result.is_err());
```

### Audit Logging

```rust
use ricecoder_security::{AuditLogger, AuditEvent, AuditEventType};
use std::sync::Arc;

let storage = Arc::new(MemoryAuditStorage::new());
let logger = AuditLogger::new(storage);

// Log API key access
logger.log_api_key_access(
    Some("user123".to_string()),
    Some("session456".to_string()),
    "openai"
).await?;
```

### Access Control

```rust
use ricecoder_security::{AccessControl, Permission, Principal};

let mut ac = AccessControl::new();

// Create a user principal
let user = Principal {
    id: "user123".to_string(),
    roles: vec!["user".to_string()],
    attributes: HashMap::new(),
};

// Check permissions
assert!(ac.has_permission(&user, &Permission::Read));
assert!(!ac.has_permission(&user, &Permission::Admin));
```

## Security Considerations

- API keys are encrypted using AES-256-GCM with Argon2 key derivation
- Input validation prevents common injection attacks (XSS, path traversal)
- Audit logging provides comprehensive security event tracking
- Role-based access control with principle of least privilege
- All cryptographic operations use well-vetted Rust libraries

## Testing

The crate includes comprehensive unit tests and property-based tests:

```bash
cargo test
```

Property-based tests verify security invariants across a wide range of inputs.

## Dependencies

- `aes-gcm`: AES-256-GCM encryption
- `argon2`: Password hashing for key derivation
- `base64`: Base64 encoding/decoding
- `regex`: Pattern matching for input validation
- `serde`: Serialization
- `tokio`: Async runtime
- `uuid`: Unique identifiers
- `chrono`: Timestamps

## License

MIT OR Apache-2.0