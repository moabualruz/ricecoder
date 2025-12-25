# Interface Design: ISP-Compliant Trait Architecture

> **Last Updated:** 2025-12-24  
> **Status:** ✅ Complete  
> **Spec Reference:** [Task 3.3 DIP/ISP Validation](/.ai/specs/ricecoder-alpha-solid-ddd/tasks.md)

## Overview

RiceCoder follows the **Interface Segregation Principle (ISP)** by splitting large, monolithic traits into smaller, role-specific sub-traits. This document explains the design rationale and implementation patterns.

## Design Rationale

### Problem: Fat Interfaces

Before ISP refactoring, we had "fat" interfaces with 7-11 methods:

| Original Trait | Method Count | ISP Violation |
|----------------|--------------|---------------|
| `SpecificationRepository` | 7 | ✅ Violates (>5) |
| `AiProvider` | 8 | ✅ Violates (>5) |
| `FileRepository` | 11 | ✅ Violates (>5) |
| `CacheRepository` | 8 | ✅ Violates (>5) |

**Issues:**
- Clients forced to implement unused methods
- Mock implementations bloated with unnecessary stubs
- Coupling between unrelated concerns

### Solution: Role-Specific Sub-Traits

Each fat interface was split into 2-3 focused sub-traits, each with ≤5 methods:

```
FatTrait → ReaderTrait + WriterTrait + (optional) ManagerTrait
```

## Trait Splitting Patterns

### Pattern 1: SpecificationRepository

**Before:**
```rust
trait SpecificationRepository {
    fn find_by_id(...);     // Read
    fn find_by_project(...); // Read
    fn find_all(...);        // Read
    fn find_by_status(...);  // Read
    fn exists(...);          // Read
    fn save(...);            // Write
    fn delete(...);          // Write
}
```

**After:**
```rust
trait SpecificationReader: Send + Sync {
    fn find_by_id(...);      // 5 methods
    fn find_by_project(...);
    fn find_all(...);
    fn find_by_status(...);
    fn exists(...);
}

trait SpecificationWriter: Send + Sync {
    fn save(...);   // 2 methods
    fn delete(...);
}

// Blanket impl for backward compatibility
impl<T: SpecificationReader + SpecificationWriter> SpecificationRepository for T {}
```

### Pattern 2: AiProvider

**Split:**
- `AiProviderInfo` (4 methods): `id`, `name`, `models`, `default_model`
- `AiProviderChat` (4 methods): `chat`, `count_tokens`, `health_check`, `supports_capability`

**Rationale:**
- Info-only clients (UI displays) don't need chat capability
- Chat clients don't need provider metadata enumeration

### Pattern 3: FileRepository

**Split:**
- `FileReader` (5 methods): `read`, `read_string`, `exists`, `metadata`, `list_directory`
- `FileWriter` (3 methods): `write`, `write_string`, `delete`
- `FileManager` (3 methods): `create_directory`, `copy`, `move_path`

**Rationale:**
- Read-only clients (analyzers, viewers) get minimal interface
- Write clients (generators, editors) get focused mutation API
- Management clients (project setup) get structure operations

### Pattern 4: CacheRepository

**Split:**
- `CacheReader` (5 methods): `get`, `contains`, `entry_info`, `keys`, `statistics`
- `CacheWriter` (3 methods): `set`, `remove`, `clear`

**Rationale:**
- Cache lookups are hot-path; read-only interface allows optimization
- Write operations can have side effects (eviction, size limits)

## Blanket Implementation Pattern

Each split uses a **blanket implementation** for backward compatibility:

```rust
/// Parent trait composed of sub-traits
pub trait SpecificationRepository: SpecificationReader + SpecificationWriter {}

/// Blanket impl: any T implementing both sub-traits automatically implements parent
impl<T: SpecificationReader + SpecificationWriter> SpecificationRepository for T {}
```

**Benefits:**
- Existing code using `SpecificationRepository` continues to work
- New code can depend on narrower `SpecificationReader` or `SpecificationWriter`
- Single implementation satisfies all three traits

## Client Usage Examples

### Read-Only Client (ISP-Compliant)

```rust
/// This service only needs read operations
struct SpecificationQueryService<R: SpecificationReader> {
    reader: R,
}

impl<R: SpecificationReader> SpecificationQueryService<R> {
    async fn count_by_status(&self, status: SpecStatus) -> DomainResult<usize> {
        let specs = self.reader.find_by_status(status).await?;
        Ok(specs.len())
    }
}
```

### Write-Only Client (ISP-Compliant)

```rust
/// This service only needs write operations
struct SpecificationPersistenceService<W: SpecificationWriter> {
    writer: W,
}

impl<W: SpecificationWriter> SpecificationPersistenceService<W> {
    async fn save(&self, spec: &Specification) -> DomainResult<()> {
        self.writer.save(spec).await
    }
}
```

### Full Access Client (Backward Compatible)

```rust
/// This service needs both read and write
fn full_access<R: SpecificationRepository>(repo: &R) {
    // Can use all methods from both SpecificationReader and SpecificationWriter
}
```

## Method Count Summary

| Trait | Methods | ISP Status |
|-------|---------|------------|
| `SpecificationReader` | 5 | ✅ At limit |
| `SpecificationWriter` | 2 | ✅ Well under |
| `AiProviderInfo` | 4 | ✅ Under limit |
| `AiProviderChat` | 4 | ✅ Under limit |
| `FileReader` | 5 | ✅ At limit |
| `FileWriter` | 3 | ✅ Under limit |
| `FileManager` | 3 | ✅ Under limit |
| `CacheReader` | 5 | ✅ At limit |
| `CacheWriter` | 3 | ✅ Under limit |

## Testing ISP Compliance

ISP compliance is verified in `ricecoder-domain/tests/isp_tests.rs`:

1. **Compile-time verification:** Client structs using sub-traits compile successfully
2. **Method count assertions:** Each sub-trait has ≤5 methods
3. **Blanket impl verification:** Types implementing sub-traits work with parent trait bounds

## Related Files

- **Implementation:**
  - `ricecoder-domain/src/repositories.rs` - SpecificationReader/Writer
  - `ricecoder-domain/src/ports/ai.rs` - AiProviderInfo/Chat
  - `ricecoder-domain/src/ports/file.rs` - FileReader/Writer/Manager
  - `ricecoder-domain/src/ports/cache.rs` - CacheReader/Writer

- **Tests:**
  - `ricecoder-domain/tests/isp_tests.rs` - ISP compliance tests
  - `ricecoder-domain/tests/repository_contracts.rs` - Contract tests
  - `ricecoder-domain/tests/provider_contracts.rs` - Provider contract tests

## References

- [Interface Segregation Principle (Robert C. Martin)](https://en.wikipedia.org/wiki/Interface_segregation_principle)
- [SOLID Principles in Rust](https://doc.rust-lang.org/book/)
- [RiceCoder Architecture Spec](/.ai/specs/ricecoder-alpha-solid-ddd/design.md)
