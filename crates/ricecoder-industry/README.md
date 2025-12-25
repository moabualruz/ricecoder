# ricecoder-industry

Enterprise integrations and OAuth authentication for RiceCoder.

## DDD Layer

**Layer**: Domain (Enterprise Domain Models)

### Responsibilities

- Industry-specific domain models
- Enterprise OAuth integration (SAML, OIDC)
- Corporate identity provider support
- Enterprise compliance patterns
- Industry vertical configurations

### SOLID Analysis

| Principle | Score | Notes |
|-----------|-------|-------|
| SRP | ✅ | Clear separation of industry models and auth providers |
| OCP | ✅ | Extensible via new industry verticals and OAuth providers |
| LSP | ✅ | Consistent authentication interfaces |
| ISP | ✅ | Segregated concerns (auth, compliance, industry) |
| DIP | ✅ | Depends on security and domain abstractions |

**Score**: 5/5

### Integration Points

| Component | Direction | Purpose |
|-----------|-----------|---------|
| ricecoder-security | Depends on | Security primitives |
| ricecoder-domain | Depends on | Core domain models |
| ricecoder-api | Used by | Enterprise auth endpoints |

## Features

- **OAuth Providers**: Google, Microsoft, GitHub, Okta, Auth0
- **Enterprise SSO**: SAML 2.0, OpenID Connect
- **Industry Verticals**: Healthcare, Finance, Legal, Government
- **Compliance**: HIPAA, SOC 2, GDPR patterns

## Usage

```rust
use ricecoder_industry::{OAuthProvider, EnterpriseAuth};

let auth = EnterpriseAuth::new(OAuthProvider::Okta, config)?;
let token = auth.authenticate(credentials).await?;
```

## Recent Changes

### SRP Refactoring (December 2024)

**EnterpriseProvider Trait Split**: Separated enterprise provider concerns following Interface Segregation Principle.

**New Traits**:
- `AuthenticationProvider`: OAuth/SAML authentication operations
- `ComplianceProvider`: Industry compliance checks (HIPAA, SOC 2, GDPR)
- `ConnectionProvider`: Enterprise connection management

**Changes**:
- `EnterpriseAuth` now implements 3 focused traits
- Cleaner separation between auth, compliance, and connection concerns
- Easier to implement custom enterprise providers
- No breaking changes to high-level API

**Migration**: Use specific traits for focused dependencies. Existing `EnterpriseAuth` API remains stable.

## License

MIT
