# Automated Release Process

This document describes the automated release process for RiceCoder, implementing semantic versioning with enterprise release management.

## Overview

The release process consists of:
- Automated semantic versioning from conventional commits
- Breaking change detection and highlighting
- Enterprise compliance and security validation
- Multi-platform binary builds
- Staged rollouts and pre-releases
- Automated package manager updates

## Workflows

### Automated Release (`automated-release.yml`)
Triggers on pushes to `main` branch or manual dispatch.

**Features:**
- Detects conventional commits since last release
- Calculates next semantic version (major.minor.patch)
- Supports pre-releases (alpha, beta, rc)
- Creates and pushes version tags
- Updates repository version files post-release

**Version Calculation:**
- `feat:` commits → minor version bump
- `fix:` commits → patch version bump
- `BREAKING CHANGE:` → major version bump
- Pre-release suffixes: `-alpha.1`, `-beta.1`, `-rc.1`

### Release (`release.yml`)
Triggers on version tags (`v*.*.*`) or manual dispatch.

**Jobs:**
1. **Release Validation**: Tests, security audit, semver checks, breaking change detection
2. **Enterprise Release Validation**: SOC 2 compliance, security validation
3. **Prepare Release**: Generate changelog with breaking change highlights
4. **Build Release**: Cross-platform binary compilation
5. **Create Release**: GitHub release with artifacts
6. **Staged Rollout**: Validation for staging/pre-release environments
7. **Update Package Managers**: Homebrew, Scoop, Winget, npm manifests
8. **Publish**: Crates.io and npm publishing
9. **Update Version**: Repository version updates

## Environments

- **Production**: Full release with publishing
- **Staging**: Staged rollout validation without publishing

## Conventional Commits

Commits must follow [Conventional Commits](https://conventionalcommits.org/) format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New features (minor bump)
- `fix`: Bug fixes (patch bump)
- `BREAKING CHANGE`: Breaking changes (major bump)
- `docs`, `style`, `refactor`, `perf`, `test`, `chore`

## Breaking Changes

- Automatically detected in commit messages and footers
- Highlighted in changelog with 🚨 warning
- Validated via `cargo-semver-checks`

## Enterprise Features

### Compliance Checks
- SOC 2 Type II audit logging verification
- GDPR data protection validation
- HIPAA compliance (if healthcare features present)
- Security vulnerability scanning
- License compliance validation

### Security Validation
- `cargo-audit` for vulnerabilities
- `cargo-deny` for license compliance
- Unsafe code review
- Hardcoded secrets detection
- Penetration testing patterns

### Staged Rollouts
- Environment-based deployments
- Pre-release validation
- Gradual rollout capabilities
- Rollback procedures

## Manual Release

Use workflow dispatch with:
- `version`: Target version (e.g., 1.2.3)
- `prerelease`: Mark as pre-release
- `environment`: Target environment (staging/production)

## Automation

- Changelog generation via `git-cliff`
- Version bumping from conventional commits
- Multi-platform CI/CD via GitHub Actions
- Security scanning integration
- Package manager automation

## Validation

All releases undergo:
- Unit and integration tests
- Performance regression checks
- Security audits
- Compliance validation
- Cross-platform testing
- Binary execution verification

## Artifacts

- Platform-specific binaries (Linux, macOS, Windows)
- Checksums and signatures
- Changelog and release notes
- Compliance reports
- Security scan results