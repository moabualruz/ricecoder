# RiceCoder Updates

A comprehensive update management system for RiceCoder that provides automatic update checking, self-updating binaries, distribution analytics, rollback capabilities, staged releases, and enterprise compliance reporting.

## Features

### 🔄 Automatic Update Checking
- Background update checking with configurable intervals
- Enterprise policy controls for update management
- Channel-based releases (stable, beta, nightly)
- Minimum version requirements and compatibility checking

### 📦 Self-Updating Binaries
- Secure binary downloads with integrity validation
- SHA-256 checksum verification
- Optional cryptographic signature validation
- Automatic backup creation before updates
- Cross-platform binary management

### 📊 Distribution Analytics
- Usage tracking and analytics collection
- Enterprise usage reporting
- Performance metrics monitoring
- Security incident tracking
- Compliance reporting (SOC 2, GDPR, HIPAA)

### 🔙 Rollback Capabilities
- Version rollback to previous installations
- Backup management with configurable retention
- Rollback validation and feasibility checking
- Emergency rollback procedures

### 🎯 Staged Releases
- Gradual rollout management
- Percentage-based deployment control
- Channel-specific staging
- Rollback capabilities for staged releases

### 🏢 Enterprise Features
- Policy-based update controls
- Audit logging and compliance reporting
- Multi-organization support
- Custom update servers and proxy support

## Architecture

The crate follows a modular architecture with clear separation of concerns:

- **`checker`**: Update checking logic with policy enforcement
- **`updater`**: Binary update and installation management
- **`analytics`**: Usage tracking and enterprise reporting
- **`policy`**: Enterprise policy management and access control
- **`rollback`**: Version rollback and backup management

## Usage

### Basic Update Checking

```rust
use ricecoder_updates::{UpdateChecker, UpdatePolicy};

let policy = UpdatePolicy::default();
let checker = UpdateChecker::new(
    policy,
    "https://updates.ricecoder.com".to_string(),
    current_version,
);

// Check for updates
let result = checker.check_for_updates().await?;
if result.update_available {
    println!("Update available: {}", result.latest_version.unwrap());
}
```

### Binary Updates

```rust
use ricecoder_updates::BinaryUpdater;

let policy = UpdatePolicy::default();
let updater = BinaryUpdater::new(policy, install_path);

// Install update
let operation = updater.install_update(&release_info).await?;
match operation.status {
    UpdateStatus::Installed => println!("Update successful"),
    UpdateStatus::Failed => println!("Update failed: {:?}", operation.error_message),
    _ => {}
}
```

### Analytics Collection

```rust
use ricecoder_updates::AnalyticsCollector;

let collector = AnalyticsCollector::new(
    "https://analytics.ricecoder.com".to_string(),
    current_version,
    platform,
);

// Record usage
collector.record_usage(
    300, // duration in seconds
    vec!["lsp".to_string(), "completion".to_string()], // commands
    vec!["lsp".to_string()], // features
    0, // error count
    performance_metrics,
).await?;
```

### Rollback Management

```rust
use ricecoder_updates::RollbackManager;

let manager = RollbackManager::new(backup_dir, install_dir, 10);

// Rollback to specific version
let rollback_info = manager.rollback_to_version(&target_version).await?;

// List available backups
let backups = manager.list_backups().await?;
for backup in backups {
    println!("Backup: {} ({})", backup.version, backup.created_at);
}
```

## Configuration

### Update Policy Configuration

```rust
use ricecoder_updates::{UpdatePolicyConfig, SecurityRequirements};

let config = UpdatePolicyConfig {
    auto_update_enabled: true,
    check_interval_hours: 24,
    allowed_channels: vec![ReleaseChannel::Stable],
    require_approval: false,
    max_download_size_mb: 100,
    security_requirements: SecurityRequirements {
        require_signature: true,
        require_checksum: true,
        allowed_cas: vec!["RiceCoderCA".to_string()],
        minimum_security_level: SecuritySeverity::High,
    },
    enterprise_settings: Some(EnterpriseSettings {
        organization_id: "my-org".to_string(),
        compliance_requirements: vec!["SOC2".to_string(), "GDPR".to_string()],
        custom_update_server: None,
        proxy_settings: None,
        audit_level: "detailed".to_string(),
    }),
};

let policy = UpdatePolicy::new(config);
```

## Enterprise Features

### Compliance Reporting

```rust
use ricecoder_updates::{EnterpriseDashboard, ComplianceReporter};

let dashboard = EnterpriseDashboard::new(collector, "my-org".to_string());
let reporter = ComplianceReporter::new(dashboard);

// Generate SOC 2 compliance report
let soc2_report = reporter.generate_soc2_report(30).await?;

// Generate GDPR compliance report
let gdpr_report = reporter.generate_gdpr_report(30).await?;
```

### Staged Releases

```rust
use ricecoder_updates::StagedReleaseManager;

let manager = StagedReleaseManager::new(staging_dir);

// Stage a release
manager.stage_release(&release_info, "beta").await?;

// Update rollout percentage
manager.update_rollout("beta", 25).await?; // 25% rollout
```

## Security

The updates system implements multiple security measures:

- **Integrity Verification**: SHA-256 checksum validation for all downloads
- **Signature Validation**: Optional cryptographic signature verification
- **Policy Enforcement**: Enterprise policies control update behavior
- **Audit Logging**: All update operations are logged for compliance
- **Rollback Safety**: Automatic backups enable safe rollback procedures

## Testing

The crate includes comprehensive testing:

```bash
cargo test -p ricecoder-updates
```

Tests cover:
- Unit tests for all components
- Integration tests for update workflows
- Property-based tests for policy evaluation
- Security validation tests
- Rollback scenario testing

## Dependencies

- `reqwest`: HTTP client for update checking and downloads
- `tokio`: Async runtime for background operations
- `serde`: Serialization for configuration and data models
- `chrono`: Date/time handling
- `semver`: Version management
- `sha2`: Cryptographic hashing
- `uuid`: Unique identifier generation
- `ricecoder-security`: Security and access control
- `ricecoder-activity-log`: Audit logging

## Contributing

When contributing to the updates system:

1. Ensure all tests pass
2. Add tests for new functionality
3. Update documentation for API changes
4. Follow the existing code style and patterns
5. Consider security implications for enterprise features

## License

This crate is part of RiceCoder and follows the same licensing terms.