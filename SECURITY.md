# RiceCoder Security Policy

**Last Updated**: December 5, 2025

**Version**: 1.0

---

## Table of Contents

1. [Security Overview](#security-overview)
2. [Reporting Security Vulnerabilities](#reporting-security-vulnerabilities)
3. [Security Best Practices](#security-best-practices)
4. [API Key Management](#api-key-management)
5. [File Security](#file-security)
6. [Network Security](#network-security)
7. [Audit Logging](#audit-logging)
8. [Rate Limiting](#rate-limiting)
9. [Permissions System](#permissions-system)
10. [Security Updates](#security-updates)
11. [Compliance](#compliance)
12. [FAQ](#faq)

---

## Security Overview

RiceCoder is designed with security as a core principle. This document outlines the security features, best practices, and policies for using RiceCoder safely.

### Key Security Features

✅ **Secure Credential Storage**
- API keys stored in memory by default
- Environment variable support for secure loading
- OS keychain integration (planned)
- Encryption at rest (planned)

✅ **Credential Redaction**
- Automatic redaction of API keys from logs
- Redaction of sensitive information in error messages
- Support for custom redaction patterns

✅ **Safe File Operations**
- Atomic writes with temporary files
- Automatic backups before modifications
- Permission preservation
- Directory traversal prevention

✅ **Network Security**
- HTTPS/TLS by default
- Certificate validation enabled
- No HTTP fallback
- Request signing for API calls

✅ **Audit Logging**
- Centralized audit log for security events
- Tracking of API key access
- Authentication attempt logging
- Permission decision logging

✅ **Rate Limiting**
- Token bucket rate limiter
- Exponential backoff for retries
- Per-provider rate limiting
- Configurable limits

✅ **Permission System**
- Fine-grained tool access control
- Role-based access control
- User prompts for sensitive operations
- Permission configuration support

---

## Reporting Security Vulnerabilities

If you discover a security vulnerability in RiceCoder, please report it responsibly.

### Reporting Process

1. **Do NOT** create a public GitHub issue for security vulnerabilities
2. **Use GitHub Security Advisories** at https://github.com/moabualruz/ricecoder/security/advisories with:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if available)

3. **Wait** for acknowledgment (within 48 hours)
4. **Coordinate** with the security team on disclosure timeline
5. **Receive** credit in security advisory (if desired)

### Security Advisory Timeline

- **Day 1**: Vulnerability reported
- **Day 2**: Acknowledgment and initial assessment
- **Day 7**: Fix development begins
- **Day 14**: Fix completed and tested
- **Day 21**: Security advisory published
- **Day 28**: Public disclosure (if applicable)

### Supported Versions

| Version | Status | Security Updates |
|---------|--------|------------------|
| 1.0.x | Current | Yes |
| 0.4.x | Beta | Yes (critical only) |
| 0.3.x | Beta | No |
| 0.2.x | Alpha | No |
| 0.1.x | Alpha | No |

---

## Security Best Practices

### 1. API Key Management

**DO:**
- ✅ Store API keys in environment variables
- ✅ Use OS keychain for sensitive environments
- ✅ Rotate API keys regularly (every 90 days)
- ✅ Use separate keys for different environments
- ✅ Monitor API key usage in audit logs

**DON'T:**
- ❌ Hardcode API keys in configuration files
- ❌ Commit API keys to version control
- ❌ Share API keys via email or chat
- ❌ Use the same key for multiple environments
- ❌ Log API keys in debug output

### 2. Configuration Security

**DO:**
- ✅ Use configuration files for non-sensitive settings
- ✅ Restrict file permissions (chmod 600 for config files)
- ✅ Use environment variables for sensitive values
- ✅ Validate configuration on load
- ✅ Review configuration changes

**DON'T:**
- ❌ Store secrets in configuration files
- ❌ Make configuration files world-readable
- ❌ Use default credentials
- ❌ Skip configuration validation
- ❌ Commit sensitive configuration to version control

### 3. File Security

**DO:**
- ✅ Use atomic file operations
- ✅ Create backups before modifications
- ✅ Validate file permissions
- ✅ Use relative paths within project
- ✅ Enable git integration for change tracking

**DON'T:**
- ❌ Use absolute paths
- ❌ Skip backup creation
- ❌ Modify files without validation
- ❌ Allow directory traversal
- ❌ Ignore file permission errors

### 4. Logging Security

**DO:**
- ✅ Enable audit logging for security events
- ✅ Review audit logs regularly
- ✅ Rotate logs to prevent unbounded growth
- ✅ Archive logs for compliance
- ✅ Redact sensitive information from logs

**DON'T:**
- ❌ Log API keys or credentials
- ❌ Log sensitive user data
- ❌ Disable audit logging
- ❌ Store logs in world-readable locations
- ❌ Ignore suspicious log entries

### 5. Network Security

**DO:**
- ✅ Use HTTPS for all remote connections
- ✅ Verify TLS certificates
- ✅ Use secure DNS (DoH/DoT)
- ✅ Monitor network traffic
- ✅ Use VPN for sensitive operations

**DON'T:**
- ❌ Use HTTP for sensitive data
- ❌ Disable certificate validation
- ❌ Trust self-signed certificates
- ❌ Send credentials over unencrypted channels
- ❌ Use public WiFi for sensitive operations

### 6. Permission Management

**DO:**
- ✅ Use principle of least privilege
- ✅ Grant only necessary permissions
- ✅ Review permissions regularly
- ✅ Revoke unused permissions
- ✅ Audit permission changes

**DON'T:**
- ❌ Grant excessive permissions
- ❌ Use default permissions
- ❌ Forget to revoke permissions
- ❌ Share credentials across users
- ❌ Ignore permission errors

---

## API Key Management

### Setting API Keys

#### Option 1: Environment Variables (Recommended)

```bash
# Set API key in environment
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."

# Run ricecoder
rice chat
```

#### Option 2: Configuration File

```yaml
# ~/.ricecoder/config.yaml
providers:
  api_keys:
    openai: "sk-..."
    anthropic: "sk-ant-..."
```

**⚠️ WARNING**: Configuration files are stored in plain text. Use environment variables for production.

#### Option 3: OS Keychain (Planned)

```bash
# Store API key in OS keychain
rice config set-key openai "sk-..."

# Retrieve from keychain automatically
rice chat
```

### Rotating API Keys

```bash
# Rotate API key
rice config rotate-key openai

# Verify new key works
rice chat
```

### Checking API Key Status

```bash
# List configured providers
rice config list-providers

# Check if API key is available
rice config check-key openai
```

---

## File Security

### Safe File Operations

RiceCoder uses atomic file operations to ensure data safety:

1. **Temporary File**: Write to temporary file
2. **Validation**: Validate file contents
3. **Backup**: Create backup of original
4. **Atomic Rename**: Rename temporary to target
5. **Verification**: Verify file integrity

### File Permissions

RiceCoder respects file permissions:

- **Config files**: 0o600 (read/write for owner only)
- **Project files**: 0o644 (read/write for owner, read for others)
- **Directories**: 0o755 (read/write/execute for owner, read/execute for others)

### Directory Traversal Prevention

RiceCoder prevents directory traversal attacks:

```rust
// ✅ SAFE: Paths are validated
let path = path_resolver.resolve("src/main.rs")?;

// ❌ UNSAFE: Would be rejected
let path = path_resolver.resolve("../../../etc/passwd")?;
```

---

## Network Security

### HTTPS/TLS Configuration

All network communication uses HTTPS/TLS:

- **TLS Version**: 1.2 or higher
- **Certificate Validation**: Enabled
- **Cipher Suites**: Modern, secure ciphers
- **Certificate Pinning**: Planned for major providers

### Request Signing

API requests are signed with credentials:

```
Authorization: Bearer <api_key>
X-API-Key: <api_key>
```

### Rate Limiting

RiceCoder implements rate limiting to prevent abuse:

- **Default**: 10 requests/second per provider
- **Burst**: 100 requests maximum
- **Backoff**: Exponential backoff on rate limit errors

---

## Audit Logging

### Audit Log Location

```
~/.ricecoder/audit.log
```

### Audit Log Format

Each entry is a JSON object:

```json
{
  "timestamp": "2025-12-05T10:30:00+00:00",
  "event_type": "ApiKeyAccessed",
  "component": "providers",
  "actor": "system",
  "resource": "openai",
  "result": "success",
  "details": "API key accessed"
}
```

### Audit Events

| Event Type | Description |
|-----------|-------------|
| ApiKeyAccessed | API key was accessed |
| ApiKeyRotated | API key was rotated |
| AuthenticationAttempt | Authentication was attempted |
| AuthorizationDecision | Permission decision was made |
| ConfigurationLoaded | Configuration was loaded |
| FileAccessed | File was accessed |
| FileModified | File was modified |
| PermissionDenied | Permission was denied |
| RateLimitExceeded | Rate limit was exceeded |
| SecurityError | Security error occurred |

### Reviewing Audit Logs

```bash
# View recent audit events
tail -f ~/.ricecoder/audit.log

# Search for specific events
grep "ApiKeyAccessed" ~/.ricecoder/audit.log

# Parse JSON logs
cat ~/.ricecoder/audit.log | jq '.event_type'
```

---

## Rate Limiting

### Configuration

```yaml
# ~/.ricecoder/config.yaml
providers:
  rate_limits:
    openai:
      tokens_per_second: 10
      max_tokens: 100
    anthropic:
      tokens_per_second: 5
      max_tokens: 50
```

### Backoff Strategy

RiceCoder uses exponential backoff with jitter:

- **Initial Delay**: 100ms
- **Multiplier**: 2.0 (doubles each retry)
- **Max Delay**: 30 seconds
- **Jitter**: ±10% to prevent thundering herd

### Monitoring Rate Limits

```bash
# Check rate limit status
rice config check-rate-limit openai

# View rate limit history
grep "RateLimitExceeded" ~/.ricecoder/audit.log
```

---

## Permissions System

### Permission Levels

| Level | Description |
|-------|-------------|
| allow | Always allow without prompting |
| ask | Prompt user before allowing |
| deny | Always deny |

### Configuring Permissions

```yaml
# ~/.ricecoder/config.yaml
permissions:
  tools:
    read_file:
      level: ask
      description: "Read files from disk"
    write_file:
      level: ask
      description: "Write files to disk"
    execute_command:
      level: deny
      description: "Execute shell commands"
```

### Permission Prompts

```
⚠️  Permission Required

Tool: read_file
Resource: /path/to/file.txt
Description: Read files from disk

Allow? [y/n/always/never]
```

---

## Security Updates

### Checking for Updates

```bash
# Check for security updates
rice update check

# Install security updates
rice update install
```

### Update Policy

- **Critical**: Released immediately
- **High**: Released within 7 days
- **Medium**: Released within 30 days
- **Low**: Released with next version

### Dependency Updates

RiceCoder dependencies are regularly updated:

```bash
# Check for vulnerable dependencies
cargo audit

# Update dependencies
cargo update
```

---

## Compliance

### Standards

RiceCoder follows these security standards:

- **OWASP Top 10**: Addresses all major categories
- **CWE Top 25**: Addresses common weaknesses
- **NIST Cybersecurity Framework**: Implements core functions
- **Rust Security Guidelines**: Follows best practices

### Certifications

- ✅ No known vulnerabilities (as of December 5, 2025)
- ✅ Passes `cargo audit` security scan
- ✅ Follows Rust security guidelines
- ✅ Implements OWASP recommendations

### Data Protection

RiceCoder does not store user data:

- ✅ No user accounts
- ✅ No data collection
- ✅ No telemetry
- ✅ No tracking

---

## FAQ

### Q: Is my API key safe with RiceCoder?

**A**: Yes. RiceCoder stores API keys in memory by default and never logs them. Use environment variables for additional security.

### Q: Can RiceCoder execute arbitrary code?

**A**: No. RiceCoder generates code but requires explicit user approval before writing files. Generated code is never executed automatically.

### Q: How do I know if my API key was compromised?

**A**: Check the audit log for suspicious API key access:
```bash
grep "ApiKeyAccessed" ~/.ricecoder/audit.log
```

### Q: What if I accidentally commit my API key?

**A**: Immediately rotate the key:
```bash
rice config rotate-key <provider>
```

### Q: Can RiceCoder access files outside my project?

**A**: No. RiceCoder is restricted to the current project directory and global configuration directory.

### Q: How do I report a security issue?

**A**: Use GitHub Security Advisories at https://github.com/moabualruz/ricecoder/security/advisories with details. Do not create public GitHub issues for security vulnerabilities.

### Q: Is RiceCoder suitable for production use?

**A**: RiceCoder is currently in Beta (v0.1.4). Production use is not recommended until v1.0.0 is released.

### Q: How often are security updates released?

**A**: Critical security updates are released immediately. Other updates follow the standard release schedule.

### Q: Can I disable security features?

**A**: No. Security features cannot be disabled. This is intentional to protect users.

### Q: How do I verify RiceCoder's security?

**A**: Review the security audit at `SECURITY_AUDIT_PHASE4.md` and check the source code on GitHub.

---

## Contact

For security questions or concerns:

- **GitHub Security Advisories**: [Report Issue](https://github.com/moabualruz/ricecoder/security/advisories)
- **GitHub Issues**: [Report Bug](https://github.com/moabualruz/ricecoder/issues)
- **GitHub Discussions**: [Ask Questions](https://github.com/moabualruz/ricecoder/discussions)

---

## Changelog

### Version 1.0 (December 5, 2025)

- ✅ Initial security policy
- ✅ Comprehensive security audit
- ✅ Rate limiting implementation
- ✅ Audit logging implementation
- ✅ Security headers implementation
- ✅ Vulnerability reporting process

---

**Last Updated**: December 5, 2025

**Next Review**: After Phase 4 implementation

**Maintained by**: RiceCoder Security Team
