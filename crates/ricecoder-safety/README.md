# ricecoder-safety

Enterprise-grade security constraints and risk analysis for RiceCoder.

## DDD Layer

**Layer**: Infrastructure (Security)

### Responsibilities

- Security constraint validation
- Risk analysis and scoring
- Operation safety checks
- Destructive action prevention
- Compliance validation

### SOLID Analysis

| Principle | Score | Notes |
|-----------|-------|-------|
| SRP | ✅ | Clear separation of constraints, risk analysis, validation |
| OCP | ✅ | Extensible via new constraint types and validators |
| LSP | ✅ | Consistent constraint interfaces |
| ISP | ✅ | Segregated concerns (constraints, risk, compliance) |
| DIP | ✅ | Depends on security and activity-log abstractions |

**Score**: 5/5

### Integration Points

| Component | Direction | Purpose |
|-----------|-----------|---------|
| ricecoder-security | Depends on | Security primitives |
| ricecoder-activity-log | Depends on | Audit logging for safety events |
| ricecoder-execution | Used by | Pre-execution safety checks |
| ricecoder-workflows | Used by | Workflow safety validation |

## Features

- **Constraint Types**: File operations, network access, system calls
- **Risk Scoring**: Configurable risk thresholds
- **Validation**: Pre-execution safety checks
- **Compliance**: SOC 2, HIPAA safety patterns

## Usage

```rust
use ricecoder_safety::{SafetyChecker, Constraint};

let checker = SafetyChecker::new(config)?;
let result = checker.validate_operation(operation).await?;

if result.is_safe() {
    // Proceed with operation
} else {
    // Handle constraint violation
}
```

## Risk Levels

| Level | Score | Action |
|-------|-------|--------|
| Low | 0-30 | Auto-approve |
| Medium | 31-60 | Require confirmation |
| High | 61-100 | Block or require admin approval |

## License

MIT
