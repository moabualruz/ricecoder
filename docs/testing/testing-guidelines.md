# RiceCoder Testing Guidelines

## Overview

RiceCoder follows a comprehensive testing strategy aligned with DDD/SOLID principles. This document provides guidelines for writing, organizing, and maintaining tests.

## Test Pyramid

```
        /\
       /E2E\        ← Few, slow, high confidence
      /------\
     / Integr \     ← Some, medium speed
    /----------\
   /   Unit     \   ← Many, fast, focused
  /--------------\
```

### Target Coverage

| Layer | Target | Rationale |
|-------|--------|-----------|
| Domain | ≥90% | Business logic is critical |
| Application | ≥85% | Service orchestration |
| Infrastructure | ≥80% | I/O operations tested |
| Presentation | ≥75% | User-facing behavior |

## Test Organization

### Directory Structure

```
crates/
├── ricecoder-domain/
│   └── tests/
│       ├── domain_tests.rs      # Entity and value object tests
│       ├── property_tests.rs    # Property-based tests
│       ├── isp_tests.rs         # Interface segregation tests
│       ├── provider_contracts.rs # AI provider contracts
│       └── repository_contracts.rs # Repository contracts
├── ricecoder-application/
│   └── tests/
│       └── service_tests.rs     # Service integration tests
├── ricecoder-persistence/
│   └── tests/
│       └── (in-module tests)    # Repository implementation tests
└── ricecoder-cli/
    └── tests/
        └── cli_tests.rs         # CLI integration tests
```

### Naming Conventions

```rust
// Unit tests: test_<method>_<scenario>_<expected>
#[test]
fn test_create_project_with_valid_name_succeeds() { }

#[test]
fn test_create_project_with_empty_name_fails() { }

// Property tests: proptest_<property>
proptest! {
    #[test]
    fn proptest_project_name_roundtrip(name in valid_project_name()) {
        // Property: valid names can be created
    }
}

// Integration tests: test_<workflow>
#[tokio::test]
async fn test_create_and_retrieve_project() { }
```

## Test Types

### 1. Unit Tests

Test individual functions/methods in isolation.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_project_name_validation() {
        // Arrange
        let valid_name = "my-project";
        let invalid_name = "";
        
        // Act & Assert
        assert!(Project::validate_name(valid_name).is_ok());
        assert!(Project::validate_name(invalid_name).is_err());
    }
}
```

### 2. Property-Based Tests

Verify invariants across many generated inputs.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn proptest_session_id_unique(seed in 0u64..1000u64) {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        prop_assert_ne!(id1, id2);
    }
}
```

**When to use:**
- Value object invariants
- Serialization roundtrips
- Domain rule validation

### 3. Integration Tests

Test component interactions with real implementations.

```rust
#[tokio::test]
async fn test_project_service_with_repository() {
    // Arrange
    let repo = Arc::new(InMemoryProjectRepository::new());
    let service = ProjectService::new(repo);
    
    // Act
    let project = service.create_project("test", "path").await.unwrap();
    let found = service.get_project(project.id()).await.unwrap();
    
    // Assert
    assert!(found.is_some());
}
```

### 4. Contract Tests

Verify implementations satisfy trait contracts.

```rust
/// Contract: All ProjectRepository implementations must satisfy
#[tokio::test]
async fn test_project_repository_contracts() {
    let repo = InMemoryProjectRepository::new();
    
    // Contract: save then find returns same entity
    let project = create_test_project();
    repo.save(&project).await.unwrap();
    let found = repo.find_by_id(project.id()).await.unwrap();
    assert_eq!(found.unwrap().id(), project.id());
    
    // Contract: delete removes entity
    repo.delete(project.id()).await.unwrap();
    let found = repo.find_by_id(project.id()).await.unwrap();
    assert!(found.is_none());
}
```

### 5. End-to-End Tests

Test complete user workflows.

```rust
#[tokio::test]
async fn test_complete_project_lifecycle() {
    // Setup application with real dependencies
    let app = TestApplication::new().await;
    
    // Create project
    let project = app.create_project("e2e-test", "/tmp/test").await;
    
    // Create session
    let session = app.create_session(project.id(), "openai", "gpt-4").await;
    
    // Verify state
    assert!(app.get_project(project.id()).await.is_some());
    assert!(app.get_session(session.id()).await.is_some());
    
    // Cleanup
    app.delete_project(project.id()).await;
}
```

## Mock Implementations

### Creating Mocks

```rust
use std::sync::Arc;
use async_trait::async_trait;

/// Mock implementation for testing
pub struct MockProjectRepository {
    projects: RwLock<HashMap<ProjectId, Project>>,
    save_error: Option<DomainError>,
}

impl MockProjectRepository {
    pub fn new() -> Self {
        Self {
            projects: RwLock::new(HashMap::new()),
            save_error: None,
        }
    }
    
    /// Configure mock to return error on save
    pub fn with_save_error(mut self, error: DomainError) -> Self {
        self.save_error = Some(error);
        self
    }
}

#[async_trait]
impl ProjectRepository for MockProjectRepository {
    async fn save(&self, project: &Project) -> DomainResult<()> {
        if let Some(ref error) = self.save_error {
            return Err(error.clone());
        }
        self.projects.write().insert(project.id().clone(), project.clone());
        Ok(())
    }
    // ... other methods
}
```

### Using Mocks

```rust
#[tokio::test]
async fn test_create_project_handles_repository_error() {
    // Arrange
    let mock_repo = Arc::new(
        MockProjectRepository::new()
            .with_save_error(DomainError::InternalError("DB down".into()))
    );
    let service = ProjectService::new(mock_repo);
    
    // Act
    let result = service.create_project("test", "/path").await;
    
    // Assert
    assert!(result.is_err());
}
```

## Best Practices

### DO

1. **Test behavior, not implementation**
   ```rust
   // Good: Tests behavior
   #[test]
   fn test_project_can_be_activated() {
       let project = Project::new("test", Language::Rust, "/path").unwrap();
       assert!(project.is_active());
   }
   
   // Bad: Tests implementation details
   #[test]
   fn test_project_has_created_at_field() {
       let project = Project::new("test", Language::Rust, "/path").unwrap();
       assert!(project.created_at.is_some()); // Exposes internal field
   }
   ```

2. **Use descriptive test names**
   ```rust
   // Good
   fn test_session_pause_when_active_succeeds() {}
   fn test_session_pause_when_already_paused_fails() {}
   
   // Bad
   fn test_pause() {}
   fn test_pause2() {}
   ```

3. **Follow Arrange-Act-Assert pattern**
   ```rust
   #[test]
   fn test_project_rename() {
       // Arrange
       let mut project = create_test_project("old-name");
       
       // Act
       project.rename("new-name").unwrap();
       
       // Assert
       assert_eq!(project.name(), "new-name");
   }
   ```

4. **Test edge cases**
   ```rust
   #[test]
   fn test_project_name_with_max_length() { }
   
   #[test]
   fn test_project_name_with_special_characters() { }
   
   #[test]
   fn test_project_name_with_unicode() { }
   ```

### DON'T

1. **Don't test private methods directly**
2. **Don't use `#[ignore]` without a reason**
3. **Don't write flaky tests (avoid timing dependencies)**
4. **Don't test external services in unit tests**

## Running Tests

### Basic Commands

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p ricecoder-domain

# Run specific test
cargo test test_create_project

# Run with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored
```

### Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html --output-dir coverage/

# View report
open coverage/index.html
```

### Mutation Testing

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation tests
cargo mutants -p ricecoder-domain

# Skip slow mutants
cargo mutants --timeout 60
```

## Continuous Integration

### Test Matrix

| Check | Command | Required |
|-------|---------|----------|
| Unit Tests | `cargo test --lib` | ✅ |
| Integration Tests | `cargo test --tests` | ✅ |
| Doc Tests | `cargo test --doc` | ✅ |
| Clippy | `cargo clippy -- -D warnings` | ✅ |
| Format | `cargo fmt --check` | ✅ |
| Coverage | `cargo tarpaulin` | ⚠️ (on merge) |

### Pre-commit Hooks

```bash
# .git/hooks/pre-commit
#!/bin/bash
cargo fmt --check && cargo clippy && cargo test
```

## Troubleshooting

### Common Issues

1. **Async test not running**: Add `#[tokio::test]` attribute
2. **Test timeout**: Use `#[tokio::test(start_paused = true)]` for time-dependent tests
3. **Flaky tests**: Avoid `sleep()`, use channels/signals instead

### Debugging Tests

```bash
# Run single test with output
cargo test test_name -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test test_name
```

## References

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [proptest Documentation](https://docs.rs/proptest/latest/proptest/)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [cargo-mutants](https://mutants.rs/)
