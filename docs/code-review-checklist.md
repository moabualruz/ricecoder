# RiceCoder Code Review Checklist

Use this checklist when reviewing code contributions to RiceCoder to ensure quality, maintainability, security, and architectural compliance.

## 1. RiceCoder Architecture Compliance

### Hexagonal Architecture
- [ ] Domain logic is in the domain layer (entities, value objects, domain services)
- [ ] Application logic is in the application layer (use cases, commands, queries)
- [ ] Infrastructure concerns are in the infrastructure layer (database, external APIs, MCP)
- [ ] Interface/adapter code is in the interfaces layer (controllers, CLI, TUI, event listeners)
- [ ] Dependencies point inward (infrastructure → application → domain)
- [ ] No domain layer dependencies on outer layers
- [ ] Ports (interfaces) are defined in domain/application layers
- [ ] Adapters implement ports in infrastructure/interfaces layers
- [ ] No business logic leaking into adapters

### RiceCoder-Specific Architecture
- [ ] MCP (Model Context Protocol) integration follows established patterns
- [ ] Session management uses proper persistence and sharing mechanisms
- [ ] Provider ecosystem integration is abstracted through interfaces
- [ ] Dependency injection container is used for service wiring
- [ ] Configuration management follows hierarchical pattern
- [ ] Logging and audit trails are properly implemented
- [ ] Security controls are integrated at appropriate layers

## 2. Domain Layer Review

### Core Entities
- [ ] Project, File, AnalysisResult entities have clear identity and lifecycle
- [ ] Session entity manages state transitions properly
- [ ] Provider entities handle configuration and health monitoring
- [ ] MCP entities (Server, Tool, Transport) follow protocol specifications
- [ ] Value objects are immutable and validate in constructor
- [ ] Domain logic is encapsulated in entities/value objects (not services)
- [ ] Domain services only for operations spanning multiple entities
- [ ] Domain events are emitted for significant state changes
- [ ] No infrastructure dependencies in domain layer
- [ ] Aggregates enforce business invariants
- [ ] Aggregate boundaries are clear and appropriate
- [ ] Repository interfaces are defined (not implementations)

### Business Rules
- [ ] Code analysis rules are properly validated
- [ ] Session sharing policies are enforced
- [ ] Provider selection logic follows business requirements
- [ ] MCP tool execution respects security constraints
- [ ] Audit logging captures all business-significant events

## 3. Application Layer Review

### Use Cases
- [ ] Use cases have single responsibility
- [ ] Use cases orchestrate domain logic (not implement it)
- [ ] Commands represent write operations (code generation, session creation)
- [ ] Queries represent read operations (analysis, session retrieval)
- [ ] DTOs are used for input/output (not domain entities)
- [ ] Transaction boundaries are clear for multi-step operations
- [ ] Error handling follows application-specific patterns
- [ ] No business logic in use cases (delegated to domain)
- [ ] Dependencies are injected via constructor (not instantiated)

### RiceCoder-Specific Use Cases
- [ ] Code generation use cases validate inputs and handle conflicts
- [ ] Session management use cases handle persistence and sharing
- [ ] Provider management use cases handle failover and optimization
- [ ] MCP integration use cases handle tool execution and results
- [ ] Analysis use cases handle different analysis types (patterns, LSP, etc.)

## 4. Infrastructure Layer Review

### Data Persistence
- [ ] Repository implementations map between domain and persistence
- [ ] Database queries are optimized (no N+1 problems)
- [ ] Session storage uses encryption for sensitive data
- [ ] Configuration storage follows hierarchical patterns
- [ ] Migration scripts are tested and reversible

### External Integrations
- [ ] AI provider clients handle rate limits and errors gracefully
- [ ] MCP server connections implement retry logic and health monitoring
- [ ] LSP server integrations handle protocol compliance
- [ ] External API clients use circuit breakers for resilience
- [ ] Configuration is externalized (not hardcoded)
- [ ] Secrets are not committed to version control
- [ ] Connection pooling is configured properly for performance

### RiceCoder-Specific Infrastructure
- [ ] MCP transport implementations (stdio, HTTP, SSE) are robust
- [ ] Provider failover mechanisms work correctly
- [ ] Session sharing uses secure, expirable URLs
- [ ] Audit logging is tamper-proof and comprehensive
- [ ] Performance monitoring is integrated

## 5. Interface Layer Review

### CLI/TUI Interfaces
- [ ] CLI commands are thin and delegate to use cases
- [ ] TUI components follow established patterns
- [ ] No business logic in interface code
- [ ] Request validation is performed at boundaries
- [ ] Response formatting is consistent
- [ ] Error messages are user-friendly
- [ ] Progress indicators are shown for long operations
- [ ] Keyboard shortcuts follow conventions

### API Interfaces (if applicable)
- [ ] RESTful conventions are followed
- [ ] HTTP methods are used correctly
- [ ] Status codes are appropriate
- [ ] Request/response formats are consistent
- [ ] Authentication/authorization is enforced
- [ ] Rate limiting is implemented
- [ ] API documentation is current

### RiceCoder-Specific Interfaces
- [ ] Session commands handle multi-session scenarios
- [ ] Provider commands show health and performance metrics
- [ ] MCP commands display server and tool status
- [ ] Code generation commands handle file conflicts
- [ ] Analysis commands show progress and results

## 6. Dependency Injection & Service Wiring

- [ ] Dependencies are injected via constructor
- [ ] No service locator pattern (anti-pattern)
- [ ] Dependencies are interfaces (not concrete classes)
- [ ] Dependency lifetimes are appropriate (singleton, scoped, transient)
- [ ] Circular dependencies are avoided
- [ ] DI container configuration is maintainable
- [ ] Dependencies are testable (mockable)
- [ ] Service registration follows conventions

## 7. Testing Coverage

### Unit Tests
- [ ] Unit tests exist for domain logic
- [ ] Unit tests exist for use cases
- [ ] Unit tests exist for infrastructure adapters
- [ ] Unit tests exist for interface components
- [ ] Tests are independent (no shared state)
- [ ] Tests are deterministic (no flaky tests)
- [ ] Test names are descriptive and follow conventions
- [ ] Tests follow AAA pattern (Arrange, Act, Assert)
- [ ] Edge cases are tested
- [ ] Error scenarios are tested
- [ ] Test coverage meets minimum threshold (80%)

### Integration Tests
- [ ] Integration tests exist for repository implementations
- [ ] Integration tests exist for external API clients
- [ ] Integration tests exist for MCP server interactions
- [ ] Integration tests exist for provider integrations
- [ ] End-to-end tests cover complete user workflows
- [ ] Tests use test databases and mock external services
- [ ] Integration tests are isolated and repeatable

### Property-Based Tests
- [ ] Property tests exist for complex domain logic
- [ ] Property tests exist for parsing and validation logic
- [ ] Property tests use appropriate generators
- [ ] Shrinking is enabled for failing test cases

### RiceCoder-Specific Tests
- [ ] MCP protocol compliance is tested
- [ ] Session persistence and sharing is tested
- [ ] Provider failover scenarios are tested
- [ ] Code generation conflict resolution is tested
- [ ] Performance regression tests exist

## 8. Error Handling & Resilience

- [ ] Errors are caught and handled appropriately
- [ ] Custom error types are used for domain errors
- [ ] Error messages are clear and actionable
- [ ] Sensitive information is not exposed in errors
- [ ] Errors are logged with appropriate context and correlation IDs
- [ ] Stack traces are logged for debugging (not exposed to users)
- [ ] User-facing errors are user-friendly
- [ ] Error handling is consistent across codebase
- [ ] Retry logic is implemented for transient failures
- [ ] Circuit breakers protect against cascading failures
- [ ] Fallback mechanisms exist for critical operations
- [ ] Graceful degradation is implemented

## 9. Logging & Observability

- [ ] Logging is structured (JSON format preferred)
- [ ] Log levels are appropriate (debug, info, warn, error)
- [ ] Sensitive data is not logged (passwords, tokens, API keys, PII)
- [ ] Request/response logging includes correlation IDs
- [ ] Performance metrics are logged
- [ ] Business events are logged for audit trails
- [ ] Errors are logged with full context
- [ ] Logging is consistent across codebase
- [ ] Log verbosity is configurable per environment
- [ ] Log aggregation and monitoring is considered

## 10. Documentation

### Code Documentation
- [ ] Public APIs have comprehensive doc comments
- [ ] Complex logic has explanatory comments
- [ ] Function parameters and return values are documented
- [ ] Error conditions are documented
- [ ] Examples are provided where helpful
- [ ] Code is self-documenting (clear names, simple logic)

### User Documentation
- [ ] README is up to date with new features
- [ ] CLI commands are documented
- [ ] Configuration options are documented
- [ ] Troubleshooting guides exist for common issues
- [ ] API documentation is current (if applicable)
- [ ] Changelog is updated with user-facing changes

### Architecture Documentation
- [ ] Architecture decisions are documented (ADRs)
- [ ] System design is documented
- [ ] Component interactions are documented
- [ ] Deployment and operational procedures are documented

## 11. Security Considerations

### Authentication & Authorization
- [ ] Authentication is required for protected operations
- [ ] Authorization checks are implemented
- [ ] Session management is secure
- [ ] API keys are handled securely
- [ ] OAuth flows are implemented correctly (if used)

### Data Protection
- [ ] Sensitive data is encrypted at rest
- [ ] Data is encrypted in transit (TLS 1.2+)
- [ ] Input validation prevents injection attacks
- [ ] Output encoding prevents XSS
- [ ] CSRF protection is implemented (if applicable)
- [ ] Secrets are not committed to version control
- [ ] Secure defaults are used

### RiceCoder-Specific Security
- [ ] MCP server authentication is verified
- [ ] Provider API keys are encrypted
- [ ] Session sharing URLs are secure and expirable
- [ ] Audit logs are tamper-proof
- [ ] Code generation doesn't execute malicious code
- [ ] File operations are sandboxed

## 12. Performance Considerations

### Code Performance
- [ ] Database queries are optimized
- [ ] Indexes are used appropriately for query performance
- [ ] N+1 query problems are avoided
- [ ] Caching is used where appropriate
- [ ] Lazy loading is used for large datasets
- [ ] Pagination is implemented for lists
- [ ] Background jobs are used for long-running tasks
- [ ] Resource pooling is configured
- [ ] Memory leaks are avoided
- [ ] Algorithms are efficient (time/space complexity)

### RiceCoder-Specific Performance
- [ ] Startup time meets targets (< 3 seconds)
- [ ] Response time meets targets (< 500ms for typical operations)
- [ ] Memory usage meets targets (< 300MB for typical sessions)
- [ ] Large project analysis is incremental
- [ ] Concurrent sessions are supported (up to 10+)
- [ ] MCP tool execution is optimized
- [ ] Provider switching is fast
- [ ] Code generation handles large files efficiently

## 13. Compliance & Regulatory

### SOC 2 / Enterprise Compliance
- [ ] Audit logging captures all significant events
- [ ] Data encryption meets compliance requirements
- [ ] Access controls are implemented
- [ ] Security monitoring is in place
- [ ] Incident response procedures exist

### GDPR / Privacy
- [ ] Personal data handling is documented
- [ ] Data retention policies are defined
- [ ] Right to erasure is implemented
- [ ] Privacy-preserving defaults are used
- [ ] Data processing consent is managed

### HIPAA (if handling PHI)
- [ ] PHI is encrypted at rest (AES-256)
- [ ] PHI is encrypted in transit (TLS 1.2+)
- [ ] Access to PHI is logged (audit trail)
- [ ] Authentication is required for PHI access
- [ ] Authorization checks are in place
- [ ] Automatic session timeout is implemented
- [ ] Minimum necessary standard is enforced
- [ ] PHI is not logged or exposed in errors
- [ ] Audit logs have 6-year retention
- [ ] Breach detection mechanisms are in place

## 14. Code Quality Standards

### Rust-Specific Quality
- [ ] Code follows Rust API Guidelines
- [ ] `cargo fmt` formatting is applied
- [ ] `cargo clippy` warnings are addressed
- [ ] Unsafe code is justified and minimal
- [ ] Memory safety is ensured
- [ ] Borrow checker rules are followed
- [ ] Error handling uses Result/Option appropriately
- [ ] Performance is considered (zero-cost abstractions)

### General Code Quality
- [ ] Code is readable and understandable
- [ ] Functions/methods have single responsibility
- [ ] Classes/structs have single responsibility
- [ ] Abstractions are appropriate (not over-engineered)
- [ ] Code is maintainable and extensible
- [ ] Code is testable
- [ ] Coupling is low, cohesion is high
- [ ] SOLID principles are followed
- [ ] Design patterns are used appropriately

### Naming & Style
- [ ] Names are descriptive and follow conventions
- [ ] Constants are used instead of magic numbers
- [ ] Functions are appropriately sized
- [ ] Comments explain complex logic
- [ ] Dead code is removed
- [ ] TODOs are tracked or resolved

## 15. Configuration & Environment

- [ ] Configuration is externalized
- [ ] Environment variables are used for secrets
- [ ] Default values are sensible
- [ ] Configuration is validated on startup
- [ ] Different configs exist for different environments
- [ ] No hardcoded URLs or credentials
- [ ] Feature flags are used appropriately
- [ ] Configuration changes don't require code changes

## 16. Version Control & Git

- [ ] Commit messages are clear and descriptive
- [ ] Commits are atomic (single logical change)
- [ ] No merge conflicts
- [ ] No commented-out code
- [ ] No debug code or console.logs in production code
- [ ] Branch naming follows conventions
- [ ] PR description is clear and complete
- [ ] Related issues are linked
- [ ] Changes are properly tested

## 17. Deployment & Operations

- [ ] Code is deployable independently
- [ ] Rollback procedures exist
- [ ] Database migrations are safe
- [ ] Configuration changes are backward compatible
- [ ] Monitoring and alerting are considered
- [ ] Logs are structured for operational use
- [ ] Health checks are implemented
- [ ] Graceful shutdown is implemented

## 18. Accessibility & Usability

- [ ] TUI interface is keyboard navigable
- [ ] Color schemes consider color blindness
- [ ] Error messages are clear and actionable
- [ ] Progress indicators are provided
- [ ] Help text is available and useful
- [ ] Default behaviors are intuitive

## Review Process Checklist

### Before Starting Review
- [ ] Understand the requirements and context
- [ ] Review related issues and discussions
- [ ] Check if changes align with project goals
- [ ] Ensure you have domain knowledge or ask for clarification

### During Review
- [ ] Review code thoroughly (don't rush)
- [ ] Test the changes locally if needed
- [ ] Check automated test results
- [ ] Verify documentation is updated
- [ ] Consider edge cases and error scenarios
- [ ] Check for security implications
- [ ] Verify performance impact
- [ ] Ensure compliance requirements are met

### Providing Feedback
- [ ] Be constructive and respectful
- [ ] Explain reasoning for suggestions
- [ ] Provide specific examples
- [ ] Suggest improvements, not just problems
- [ ] Acknowledge good work
- [ ] Ask questions if unclear
- [ ] Prioritize critical issues

### After Review
- [ ] Ensure all concerns are addressed
- [ ] Re-review if significant changes are made
- [ ] Approve when satisfied
- [ ] Provide final feedback
- [ ] Merge when ready

## Automated Checks

All pull requests must pass these automated checks:

- [ ] **Build**: `cargo build --release` succeeds
- [ ] **Tests**: `cargo test` passes with ≥80% coverage
- [ ] **Linting**: `cargo clippy` passes with zero warnings
- [ ] **Formatting**: `cargo fmt --check` passes
- [ ] **Security**: `cargo audit` passes
- [ ] **Performance**: Benchmark regression check passes
- [ ] **License**: License compatibility check passes

## Final Approval Criteria

- [ ] All automated checks pass
- [ ] Code review checklist is complete
- [ ] Security review is complete (if needed)
- [ ] Performance impact is acceptable
- [ ] Documentation is updated
- [ ] Tests are comprehensive
- [ ] No breaking changes without justification
- [ ] Deployment plan is clear

---

**Review Guidelines**:
- Not all items apply to every PR - use judgment
- Focus on important issues for the specific change
- Be constructive and help improve the code
- Consider the bigger picture and long-term maintainability
- Automate what can be automated (linting, formatting, testing)
- Keep reviews timely (respond within 24 hours)
- Prioritize security, correctness, and maintainability