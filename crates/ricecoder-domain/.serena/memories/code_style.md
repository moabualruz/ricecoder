# Code Style and Conventions

## Rust Conventions
- Follows standard Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Uses `rustfmt` for automatic formatting
- Enforces `#![forbid(unsafe_code)]` - no unsafe code allowed
- Comprehensive documentation with `//!` module docs and `///` item docs

## Domain-Driven Design Patterns
- **Entities**: Encapsulate business logic with identity (Project, Session, etc.)
- **Value Objects**: Immutable objects for domain concepts (ProjectId, ProgrammingLanguage, etc.)
- **Domain Services**: Business logic not belonging to entities (ProjectService, SessionService)
- **Repository Interfaces**: Contracts for data persistence (defined in domain, implemented in infrastructure)
- **Specification Pattern**: For complex business rules and queries

## Error Handling
- All domain operations return `DomainResult<T>` = `Result<T, DomainError>`
- Domain-specific error types with actionable messages
- Error context maintained through call stacks
- Validation errors for business rule violations

## Validation and Invariants
- Constructors validate invariants
- Methods enforce business rules
- Value objects ensure immutability and validity
- Business rule validators for complex logic

## Testing Approach
- Unit tests for all public APIs
- Property-based tests using `proptest` for edge cases
- Business rule validation tests
- Integration tests for service interactions

## Performance Considerations
- Minimize allocations in hot paths
- Use efficient data structures
- Cache expensive computations
- Profile performance-critical operations