# RiceCoder Security

Security utilities and cryptographic operations for RiceCoder with enterprise compliance support.

## Features

- **API Key Encryption**: Secure encryption/decryption of API keys with AES-256-GCM
- **Customer-Managed Keys**: SOC 2 compliant customer-managed encryption infrastructure
- **Input Validation**: Comprehensive input sanitization and validation
- **Audit Logging**: Structured security event logging with tamper-proof trails
- **Access Control**: RBAC and ABAC (Attribute-Based Access Control) systems
- **OAuth 2.0 / OpenID Connect**: Secure token management with enterprise identity providers
- **Security Monitoring**: Real-time threat detection and anomaly analysis
- **Compliance Management**: SOC 2, GDPR, and HIPAA compliance infrastructure
- **Data Privacy**: Right to erasure, data portability, and privacy-preserving analytics
- **Compliance Reporting**: Automated compliance report generation
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
use ricecoder_security::{AccessControl, Permission, Principal, AttributeBasedAccessControl, AbacPolicy, AbacRule, AttributeCondition, AbacEffect};

let mut ac = AccessControl::new();

// Create a user principal
let user = Principal {
    id: "user123".to_string(),
    roles: vec!["user".to_string()],
    attributes: HashMap::new(),
};

// Check RBAC permissions
assert!(ac.has_permission(&user, &Permission::Read));
assert!(!ac.has_permission(&user, &Permission::Admin));

// ABAC example
let mut abac = AttributeBasedAccessControl::new();

let policy = AbacPolicy {
    name: "developer_access".to_string(),
    description: "Allow developers to access dev resources".to_string(),
    rules: vec![AbacRule {
        subject_attributes: HashMap::from([
            ("department".to_string(), AttributeCondition::Equals("engineering".to_string())),
        ]),
        resource_attributes: HashMap::from([
            ("environment".to_string(), AttributeCondition::Equals("development".to_string())),
        ]),
        action: "read".to_string(),
        effect: AbacEffect::Allow,
    }],
};

abac.add_policy(policy);

let subject_attrs = HashMap::from([
    ("department".to_string(), "engineering".to_string()),
]);
let resource_attrs = HashMap::from([
    ("environment".to_string(), "development".to_string()),
]);

assert!(matches!(abac.evaluate_access(&subject_attrs, &resource_attrs, "read"), AbacEffect::Allow));
```

### Compliance Management

```rust
use ricecoder_security::{ComplianceManager, DataErasure, DataPortability, PrivacyAnalytics};
use std::sync::Arc;

let storage = Arc::new(MemoryAuditStorage::new());
let audit_logger = Arc::new(AuditLogger::new(storage));
let compliance_manager = ComplianceManager::new(audit_logger);

// Request data erasure (GDPR right to erasure)
let erasure_request_id = compliance_manager
    .request_data_erasure("user123", ErasureReason::UserRequest)
    .await?;

// Process the erasure request
compliance_manager.process_data_erasure(&erasure_request_id).await?;

// Request data portability (GDPR right to portability)
let portability_request_id = compliance_manager
    .request_data_portability("user123", vec![DataType::PersonalInfo], ExportFormat::Json)
    .await?;

// Generate compliance report
let reporter = ComplianceReporter::new(audit_logger);
let report = reporter.generate_soc2_report(start_date, end_date).await?;
```

### OAuth 2.0 / OpenID Connect

```rust
use ricecoder_security::{TokenManager, OAuthProvider, OidcProvider};
use std::sync::Arc;

let storage = Arc::new(MemoryAuditStorage::new());
let audit_logger = Arc::new(AuditLogger::new(storage));
let mut token_manager = TokenManager::new(audit_logger);

// Register OAuth 2.0 provider
let oauth_config = OAuthProvider {
    name: "github".to_string(),
    client_id: "your-client-id".to_string(),
    client_secret: "your-client-secret".to_string(),
    auth_url: "https://github.com/login/oauth/authorize".to_string(),
    token_url: "https://github.com/login/oauth/access_token".to_string(),
    redirect_url: "http://localhost:8080/callback".to_string(),
    scopes: vec!["user:email".to_string()],
};
token_manager.register_oauth_provider(oauth_config)?;

// Register OIDC provider
let oidc_config = OidcProvider {
    name: "google".to_string(),
    issuer_url: "https://accounts.google.com".to_string(),
    client_id: "your-client-id".to_string(),
    client_secret: "your-client-secret".to_string(),
    redirect_url: "http://localhost:8080/callback".to_string(),
    scopes: vec!["openid".to_string(), "email".to_string()],
};
token_manager.register_oidc_provider(oidc_config).await?;

// Generate auth URL
let (auth_url, csrf_token, pkce_challenge) = token_manager.generate_oauth_auth_url("github", &["user:email".to_string()])?;

// Complete token exchange after user authorization
let token_id = token_manager.complete_oauth_exchange("github", "auth_code", pkce_verifier, "user123").await?;
```

### Security Monitoring

```rust
use ricecoder_security::{SecurityMonitor, SecurityEvent, SecurityEventType, ThreatLevel, MonitorConfig};
use std::sync::Arc;

let config = MonitorConfig::default();
let storage = Arc::new(MemoryAuditStorage::new());
let audit_logger = Arc::new(AuditLogger::new(storage));
let monitor = SecurityMonitor::new(config, audit_logger);

// Record a security event
let event = SecurityEvent {
    id: "event-123".to_string(),
    event_type: SecurityEventType::FailedLogin,
    timestamp: Utc::now(),
    source_ip: Some("192.168.1.100".to_string()),
    user_id: Some("user123".to_string()),
    session_id: None,
    resource: "login".to_string(),
    details: serde_json::json!({"attempt_count": 3}),
    threat_level: ThreatLevel::Medium,
    mitigated: false,
};

monitor.record_event(event).await?;

// Get security metrics
let metrics = monitor.get_security_metrics().await?;
println!("Security metrics: {}", metrics);

// Analyze for anomalies
let anomalies = monitor.analyze_anomalies().await?;
for alert in anomalies {
    println!("Alert: {} - {}", alert.title, alert.description);
}
```

### Customer-Managed Encryption

```rust
use ricecoder_security::CustomerKeyManager;

let customer_key_manager = CustomerKeyManager::new("master-password")?;

// Generate a customer key
let customer_key = customer_key_manager.generate_customer_key()?;

// Encrypt data with customer key
let encrypted = customer_key_manager
    .encrypt_with_customer_key("sensitive data", &customer_key)?;

// Decrypt data with customer key
let decrypted = customer_key_manager
    .decrypt_with_customer_key(&encrypted, &customer_key)?;
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

## Compliance Features

### SOC 2 Type II
- Customer-managed encryption keys
- Comprehensive audit logging with tamper-proof trails
- Automated compliance reporting
- Security controls for availability, processing integrity, confidentiality, and privacy

### GDPR/HIPAA
- Right to erasure (data deletion)
- Right to data portability (data export)
- Consent management and tracking
- Privacy-preserving analytics with opt-in requirements
- 90-day log retention with automated cleanup

### Privacy Analytics
- Opt-in analytics collection
- Data minimization and anonymization
- Configurable retention policies
- Privacy by design principles

## Dependencies

- `aes-gcm`: AES-256-GCM encryption
- `argon2`: Password hashing for key derivation
- `base64`: Base64 encoding/decoding
- `regex`: Pattern matching for input validation
- `serde`: Serialization
- `tokio`: Async runtime
- `uuid`: Unique identifiers
- `chrono`: Timestamps
- `oauth2`: OAuth 2.0 client library
- `openidconnect`: OpenID Connect client library
- `url`: URL parsing and manipulation
- `reqwest`: HTTP client for OAuth flows

## License

MIT OR Apache-2.0