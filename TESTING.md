# RiceCoder Testing Policy

> **Version:** 1.0  
> **Last Updated:** 2025-12-25  
> **Traceability:** Requirement R2 (Test Suite Reconstruction)

## Overview

This document establishes the testing policy for all RiceCoder crates. It defines where tests live, how they are organized, coverage expectations, and the migration plan for existing inline tests.

---

## Test Organization Policy

### Core Rule

**All tests MUST be placed in `tests/` directories, NOT inline `#[cfg(test)]` modules.**

### Directory Structure

Each crate follows this structure:

```
crates/{crate-name}/
├── src/
│   └── lib.rs              # No #[cfg(test)] modules here
├── tests/
│   ├── unit/               # Unit tests (isolated, fast)
│   │   ├── mod.rs          # Module re-exports
│   │   └── {module}_test.rs
│   ├── integration/        # Integration tests (cross-module, I/O)
│   │   ├── mod.rs
│   │   └── {feature}_test.rs
│   └── fixtures/           # Test data and mocks
│       ├── sample.json
│       └── mock_data.rs
└── Cargo.toml
```

### Naming Conventions

| Type | File Pattern | Example |
|------|--------------|---------|
| Unit test | `{module}_test.rs` | `parser_test.rs` |
| Integration test | `{feature}_test.rs` | `api_integration_test.rs` |
| Property test | `{module}_properties.rs` | `validation_properties.rs` |
| Fixture data | `{name}.json`, `{name}.yaml` | `sample_config.yaml` |
| Mock implementations | `mock_{trait}.rs` | `mock_repository.rs` |

---

## Coverage Targets by Layer

Coverage requirements are based on DDD layer classification:

| Layer | Target | Rationale |
|-------|--------|-----------|
| **Domain** | ≥90% | Core business logic; highest risk if bugs exist |
| **Application** | ≥85% | Use case orchestration; critical for correctness |
| **Infrastructure** | ≥80% | External I/O; integration tests cover edge cases |
| **Presentation** | ≥75% | UI/CLI; some paths hard to test deterministically |

### Test Types by Layer

| Layer | Required Test Types |
|-------|---------------------|
| Domain | Unit tests, property tests, contract tests, invariant tests |
| Application | Use case tests with mocked repositories, service integration |
| Infrastructure | Integration tests, I/O mocking, external API mocking |
| Presentation | Widget tests, CLI command tests, keybinding tests |

---

## Anti-Patterns (DO NOT DO)

### 1. Inline Test Modules

```rust
// ❌ WRONG - Tests inside src/
// src/parser.rs
pub fn parse(input: &str) -> Result<Ast, Error> { ... }

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_valid() { ... }
}
```

**Why this is bad:**
- Tests mixed with production code
- Harder to discover and run selectively
- Clutters source files
- Inconsistent with Rust community best practices for libraries

### 2. Tests Depending on File System Paths

```rust
// ❌ WRONG - Hardcoded paths
#[test]
fn test_read_file() {
    let content = fs::read_to_string("/home/user/test.txt").unwrap();
}
```

**Why this is bad:**
- Fails on different machines
- Non-deterministic
- Requires manual setup

### 3. Tests Without Assertions

```rust
// ❌ WRONG - No assertions
#[test]
fn test_something() {
    let result = do_something();
    // No assert! - test always passes
}
```

### 4. Large Integration Tests Without Isolation

```rust
// ❌ WRONG - Shared mutable state
static mut COUNTER: i32 = 0;

#[test]
fn test_one() {
    unsafe { COUNTER += 1; }
}

#[test]
fn test_two() {
    unsafe { assert_eq!(COUNTER, 0); } // Flaky!
}
```

---

## Correct Patterns (DO THIS)

### 1. Separate Test Files

```rust
// ✅ CORRECT - Tests in tests/ directory
// tests/unit/parser_test.rs
use ricecoder_parser::parse;

#[test]
fn parse_valid_input_returns_ast() {
    let result = parse("valid input");
    assert!(result.is_ok());
}

#[test]
fn parse_invalid_input_returns_error() {
    let result = parse("{{invalid");
    assert!(result.is_err());
}
```

### 2. Fixture-Based Testing

```rust
// ✅ CORRECT - Use fixtures
// tests/fixtures/mod.rs
pub fn sample_config() -> Config {
    serde_json::from_str(include_str!("sample_config.json")).unwrap()
}

// tests/unit/config_test.rs
use crate::fixtures::sample_config;

#[test]
fn config_loads_correctly() {
    let config = sample_config();
    assert_eq!(config.name, "test");
}
```

### 3. Property-Based Testing (Domain Layer)

```rust
// ✅ CORRECT - Property tests for domain
// tests/unit/amount_properties.rs
use proptest::prelude::*;
use ricecoder_domain::Amount;

proptest! {
    #[test]
    fn amount_add_is_commutative(a in 0i64..1000, b in 0i64..1000) {
        let sum1 = Amount::new(a) + Amount::new(b);
        let sum2 = Amount::new(b) + Amount::new(a);
        prop_assert_eq!(sum1, sum2);
    }
}
```

### 4. Mocked Repository Testing (Application Layer)

```rust
// ✅ CORRECT - Mock repositories for use cases
// tests/unit/create_session_test.rs
use ricecoder_application::CreateSessionUseCase;
use crate::fixtures::MockSessionRepository;

#[test]
fn create_session_stores_in_repository() {
    let mock_repo = MockSessionRepository::new();
    let use_case = CreateSessionUseCase::new(mock_repo.clone());
    
    let result = use_case.execute("test-session");
    
    assert!(result.is_ok());
    assert!(mock_repo.contains("test-session"));
}
```

---

## Migration Plan for Inline Tests

### Phase 1: Inventory (Week 1)

1. Run inventory script to find all inline `#[cfg(test)]` modules:
   ```bash
   rg "#\[cfg\(test\)\]" --type rust crates/ -l > inline_tests.txt
   ```

2. Categorize by crate and priority (Domain > Application > Infrastructure > Presentation)

3. Create migration tracking issue

### Phase 2: Domain Layer Migration (Week 2)

1. For each Domain crate with inline tests:
   - Create `tests/unit/` directory
   - Move test functions to `{module}_test.rs`
   - Update imports to use `use crate_name::*` instead of `use super::*`
   - Remove `#[cfg(test)]` module from source
   - Run `cargo test` to verify

### Phase 3: Application Layer Migration (Week 3)

1. Same process as Phase 2
2. Additional: Create mock implementations in `tests/fixtures/`

### Phase 4: Infrastructure + Presentation (Week 4)

1. Same process
2. Additional: Move integration tests to `tests/integration/`

### Phase 5: Verification (Week 5)

1. Run full `cargo test --workspace`
2. Verify zero inline `#[cfg(test)]` modules remain:
   ```bash
   rg "#\[cfg\(test\)\]" --type rust crates/*/src/ -c
   # Should return 0 matches or only documented exceptions
   ```

---

## Exceptions Process

Some cases may justify inline tests. These require **explicit documentation**.

### When Exceptions May Be Granted

1. **Macro testing**: Macros that generate code may need inline tests to verify expansion
2. **Compile-time assertions**: `const` assertions or compile-fail tests
3. **Private API testing**: Testing private functions that cannot be accessed from `tests/`

### Exception Documentation Requirements

Each exception MUST be documented in the crate's `README.md`:

```markdown
## Test Exceptions

### `src/macros.rs` - Inline tests

**Reason:** Macro hygiene testing requires inline context to verify expansion.

**Approved by:** [Name]  
**Date:** YYYY-MM-DD  
**Review date:** YYYY-MM-DD (annual review required)
```

### Exception Registry

All exceptions are tracked in `wiki/Testing-Strategy-and-Policy.md`:

| Crate | File | Reason | Approved | Review Date |
|-------|------|--------|----------|-------------|
| ricecoder-macros | src/derive.rs | Macro expansion testing | 2025-12-25 | 2026-12-25 |

---

## Test Execution

### Local Development

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p ricecoder-domain

# Run with verbose output
cargo test --workspace -- --nocapture

# Run single test
cargo test -p ricecoder-domain test_name
```

### CI Pipeline

```yaml
# Required CI checks
- cargo build --workspace
- cargo clippy --workspace -- -D warnings
- cargo test --workspace
- cargo audit
```

### Performance Requirements

| Metric | Target | Current |
|--------|--------|---------|
| `cargo test --workspace` | ≤10 minutes | TBD |
| Individual test timeout | ≤60 seconds | N/A |
| Flaky test rate | 0% | TBD |

---

## Test Utilities

### Recommended Crates

| Crate | Purpose |
|-------|---------|
| `proptest` | Property-based testing |
| `mockall` | Mock generation |
| `tempfile` | Temporary file fixtures |
| `assert_matches` | Pattern matching assertions |
| `tokio::test` | Async test runtime |

### Shared Test Utilities

Create `ricecoder-test-utils` crate for shared testing infrastructure:

```
crates/ricecoder-test-utils/
├── src/
│   ├── lib.rs
│   ├── fixtures.rs      # Common test data
│   ├── mocks.rs         # Shared mock implementations
│   └── assertions.rs    # Custom assertion macros
└── Cargo.toml
```

---

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-25 | Initial policy document |

---

## References

- **Requirement R2**: Test Suite Reconstruction (requirements.md)
- **Design Component 2**: Test Infrastructure (design.md)
- **Wiki**: Testing-Strategy-and-Policy.md (to be created)
