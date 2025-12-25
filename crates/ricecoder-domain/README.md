# RiceCoder Domain

The core domain entities and business logic for RiceCoder. This crate contains the central business rules, entities, and domain services that form the foundation of the RiceCoder system.

## Architecture

This crate follows Domain-Driven Design principles:

- **Entities**: Core business objects with identity and business logic
- **Value Objects**: Immutable objects representing business concepts
- **Domain Services**: Business logic that doesn't belong to entities
- **Repository Interfaces**: Contracts for data persistence (implemented in infrastructure)

## Core Entities

### Project
Represents a code project being analyzed by RiceCoder.

```rust
let project = Project::new(
    "my-rust-project".to_string(),
    ProgrammingLanguage::Rust,
    "/path/to/project".to_string(),
)?;
```

### CodeFile
Represents a source code file with metadata and content.

```rust
let file = CodeFile::new(
    project_id,
    "src/main.rs".to_string(),
    "fn main() {}".to_string(),
    ProgrammingLanguage::Rust,
)?;
```

### Session
Represents an AI interaction session with a specific provider and model.

```rust
let session = Session::new("openai".to_string(), "gpt-4".to_string());
```

### AnalysisResult
Contains the results of code analysis operations.

```rust
let result = AnalysisResult::new(project_id, Some(file_id), AnalysisType::Complexity);
```

### Provider
Configuration for AI providers and their available models.

```rust
let provider = Provider::new("openai".to_string(), "OpenAI".to_string(), ProviderType::OpenAI);
```

## Value Objects

### Identifiers
- `ProjectId`: UUID-based project identifier
- `SessionId`: UUID-based session identifier
- `FileId`: Path-based file identifier

### Domain Concepts
- `ProgrammingLanguage`: Supported programming languages with file extensions
- `SemanticVersion`: Version numbers for analysis results
- `ValidUrl`: Validated URL wrapper
- `MimeType`: MIME type for file content

## Domain Services

### Business Logic Services
- `ProjectService`: Project management operations
- `SessionService`: Session lifecycle management
- `ProviderService`: Provider configuration and management
- `AnalysisService`: Code analysis operations
- `FileService`: File operations and management

### Validation Services
- `BusinessRulesValidator`: Business rule validation
- `ValidationResult`: Structured validation results

## Repository Interfaces

### Data Access Contracts
- `ProjectRepository`: Project persistence
- `SessionRepository`: Session persistence
- `AnalysisRepository`: Analysis result persistence
- `FileRepository`: File persistence
- `ProviderRepository`: Provider configuration persistence

### Transaction Support
- `UnitOfWork`: Transaction management
- `Transaction`: Individual transaction operations

## Business Rules

### Project Validation
- Names must be 1-100 characters
- Only alphanumeric characters, hyphens, and underscores allowed
- Paths cannot contain ".." for security

### Session Management
- Sessions can be active, paused, or ended
- Only active sessions can be paused
- Only paused sessions can be resumed

### Analysis Validation
- Empty files generate warnings for certain analysis types
- Language-specific recommendations for analysis types

## Error Handling

All domain operations return `DomainResult<T>` which is `Result<T, DomainError>`.

### Error Types
- `InvalidProjectName`: Project name validation failures
- `InvalidFilePath`: File path validation failures
- `InvalidSessionState`: Session state transition errors
- `EntityNotFound`: Missing entities
- `BusinessRuleViolation`: Business rule violations
- `ValidationError`: General validation failures

## Testing

### Unit Tests
Comprehensive unit tests for all entities, value objects, and business logic.

### Property-Based Tests
Using `proptest` to validate business rules with generated inputs:
- Project name validation
- Semantic version parsing
- File content updates
- Session metadata handling

### Business Rule Testing
Validation of business rules and constraints:
- Project creation rules
- Session operation validation
- Analysis operation recommendations

## Dependencies

### Core Dependencies
- `serde`: Serialization/deserialization
- `thiserror`: Error handling
- `uuid`: Unique identifier generation
- `chrono`: Date/time handling
- `regex`: Regular expression validation
- `url`: URL validation
- `mime_guess`: MIME type detection

### Development Dependencies
- `proptest`: Property-based testing
- `tokio`: Async testing support

## Usage Examples

### Creating and Managing Projects
```rust
use ricecoder_domain::*;

// Create a new project
let project = Project::new(
    "my-project".to_string(),
    ProgrammingLanguage::Rust,
    "/path/to/project".to_string(),
)?;

// Update project metadata
project.add_metadata("author".to_string(), "developer".to_string());
```

### Session Management
```rust
// Create a session
let mut session = Session::new("openai".to_string(), "gpt-4".to_string());

// Associate with project
session.set_project(project_id);

// Manage session lifecycle
session.pause();
session.resume();
session.end();
```

### Analysis Operations
```rust
// Create analysis result
let mut result = AnalysisResult::new(project_id, Some(file_id), AnalysisType::Complexity);

// Complete analysis
let metrics = AnalysisMetrics {
    lines_of_code: 150,
    cyclomatic_complexity: 8.5,
    maintainability_index: 78.0,
    technical_debt_ratio: 0.15,
    execution_time_ms: 250,
};

result.complete(serde_json::json!({"score": 8.5}), metrics);
```

## Development Guidelines

### Entity Design
- Entities should encapsulate business logic
- Use value objects for immutable concepts
- Validate invariants in constructors and methods

### Error Handling
- Use domain-specific error types
- Provide actionable error messages
- Maintain error context through call stacks

### Testing Strategy
- Unit tests for all public APIs
- Property-based tests for complex logic
- Business rule validation tests
- Integration tests for service interactions

### Performance Considerations
- Minimize allocations in hot paths
- Use efficient data structures
- Cache expensive computations
- Profile performance-critical operations

## Recent Changes

### SRP Refactoring (December 2024)

**Enriched Entities**: Migrated business logic from services into domain entities following Rich Domain Model pattern. Entities now encapsulate behavior and invariants directly.

**Changes**:
- `Project`, `Session`, `CodeFile` entities enriched with methods previously in service layer
- Value objects now validate invariants at construction time
- Domain events added for state transitions
- Repository interfaces remain stable (no breaking changes)

**Migration**: Services now orchestrate entity operations rather than implementing business logic. Entity methods are self-contained and testable in isolation.

## Related Crates

- **ricecoder-storage**: Implements repository interfaces
- **ricecoder-security**: Provides security services
- **ricecoder-providers**: Manages AI provider configurations
- **ricecoder-sessions**: Implements session services
- **ricecoder-analysis**: Implements analysis services