# RiceCoder DI Container

## DDD Layer: Infrastructure

**Layer**: Cross-cutting Infrastructure  
**Purpose**: Dependency injection and service wiring  
**Dependencies**: All crates (as service factories)  
**Dependents**: All application entry points (CLI, TUI, MCP)

---

A thread-safe dependency injection container for RiceCoder that implements the service locator pattern with TypeId-based registration.

## Features

- **Service Locator Pattern**: TypeId-based service registration and resolution
- **Multiple Lifetimes**: Singleton, transient, and scoped service management
- **Thread Safety**: RwLock-based concurrent access to services
- **Builder Pattern**: Fluent API for container configuration
- **Health Checks**: Optional service health monitoring
- **Feature Gates**: Optional service registration based on Cargo features
- **Macros**: Convenience macros for service registration and resolution

## Quick Start

```rust
use ricecoder_di::create_application_container;

// Create container with core services
let container = create_application_container().unwrap();

// Resolve services
let session_manager = container.resolve::<ricecoder_sessions::SessionManager>().unwrap();
let provider_manager = container.resolve::<ricecoder_providers::provider::manager::ProviderManager>().unwrap();
```

## Service Lifetimes

### Singleton Services

Services that are created once and reused throughout the application lifetime.

```rust
use ricecoder_di::{DIContainer, register_service};

let container = DIContainer::new();

// Register singleton service
container.register(|_| {
    let service = std::sync::Arc::new(MyService::new());
    Ok(service)
}).unwrap();

// Or use the macro
register_service!(container, MyService, |_| Ok(std::sync::Arc::new(MyService::new())));
```

### Transient Services

Services that are created each time they are requested.

```rust
container.register_transient(|_| {
    let service = std::sync::Arc::new(MyTransientService::new());
    Ok(service)
}).unwrap();
```

### Scoped Services

Services that are created once per scope and reused within that scope.

```rust
use ricecoder_di::ServiceScope;

container.register_scoped(|_| {
    let service = std::sync::Arc::new(MyScopedService::new());
    Ok(service)
}).unwrap();

let scope = ServiceScope::new();
let service1 = container.resolve_with_scope(Some(&scope)).unwrap();
let service2 = container.resolve_with_scope(Some(&scope)).unwrap();
// service1 and service2 are the same instance within this scope
```

## Builder Pattern

Use the builder pattern for fluent container configuration:

```rust
use ricecoder_di::{DIContainerBuilder, DIContainerBuilderExt};

let container = DIContainerBuilder::new()
    .register_infrastructure_services()?
    .register_use_cases()?
    .register_storage_services()?
    .register_research_services()?
    .build()
    .unwrap();
```

## Application Builders

Pre-configured container builders for different deployment scenarios:

```rust
use ricecoder_di::{
    create_application_container,
    create_cli_container,
    create_tui_container,
    create_development_container,
    create_test_container,
    create_configured_container,
    ContainerConfig
};

// Full application container
let full_container = create_application_container()?;

// CLI-only container (minimal services)
let cli_container = create_cli_container()?;

// TUI-only container (UI-focused services)
let tui_container = create_tui_container()?;

// Development container (with debugging services)
let dev_container = create_development_container()?;

// Test container (minimal for testing)
let test_container = create_test_container()?;

// Configuration-driven container
let config = ContainerConfig {
    enable_storage: true,
    enable_github: true,
    enable_parsers: true,
    ..Default::default()
};
let configured_container = create_configured_container(&config)?;
```

## Feature-Gated Services

Services can be conditionally registered based on Cargo features:

```toml
[dependencies]
ricecoder-di = { version = "0.1", features = ["storage", "research", "mcp"] }
```

Available features:
- `storage` - File and memory storage services
- `research` - Code analysis and semantic indexing
- `workflows` - Workflow execution and management
- `execution` - Command execution services
- `mcp` - Model Context Protocol services
- `tools` - Tool registry and execution
- `config` - Configuration management
- `activity-log` - Activity logging and monitoring
- `orchestration` - Workspace orchestration services
- `specs` - Specification management
- `undo-redo` - Undo/redo functionality
- `vcs` - Version control integration
- `permissions` - Permission management and checking
- `security` - Security services (access control, encryption)
- `cache` - Caching services and strategies
- `domain` - Domain services and repositories
- `learning` - Machine learning and pattern recognition
- `industry` - Industry-specific services (auth, compliance)
- `safety` - Safety monitoring and risk assessment
- `files` - File management and watching
- `themes` - Theme management and loading
- `images` - Image processing and analysis
- `completion` - Code completion services
- `lsp` - Language server protocol services
- `modes` - Mode management services
- `commands` - Command management services
- `hooks` - Hook system services
- `keybinds` - Keyboard binding services
- `teams` - Team management services
- `refactoring` - Code refactoring services
- `parsers` - AST parsing and syntax tree analysis
- `generation` - Code generation and templating
- `github` - GitHub API integration and management
- `domain-agents` - Domain-specific AI agents
- `local-models` - Local model management (Ollama)
- `cli` - Command-line interface services
- `full` - All features enabled

## Lifecycle Management

Services can implement lifecycle hooks for initialization and cleanup:

```rust
use ricecoder_di::{Lifecycle, LifecycleManager};
use async_trait::async_trait;

#[async_trait]
impl Lifecycle for MyService {
    async fn initialize(&self) -> ricecoder_di::DIResult<()> {
        // Perform startup initialization
        self.connect_to_database().await?;
        self.start_background_tasks();
        Ok(())
    }

    async fn cleanup(&self) -> ricecoder_di::DIResult<()> {
        // Perform shutdown cleanup
        self.disconnect_from_database().await?;
        self.stop_background_tasks();
        Ok(())
    }
}

// Register services with lifecycle management
let mut lifecycle_manager = LifecycleManager::new();
let service = std::sync::Arc::new(MyService::new());
lifecycle_manager.register_service(service.clone());

// Initialize all services
lifecycle_manager.initialize_all().await?;

// Application runs...

// Cleanup all services during shutdown
lifecycle_manager.cleanup_all().await?;
```

## Health Checks

Services can implement health checks for monitoring:

```rust
use ricecoder_di::{HealthCheck, HealthStatus};
use async_trait::async_trait;

#[async_trait]
impl HealthCheck for MyService {
    async fn health_check(&self) -> ricecoder_di::DIResult<HealthStatus> {
        // Perform health check logic
        if self.is_healthy() {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unhealthy("Service is not responding".to_string()))
        }
    }
}

// Register with health check
container.register_with_health_check(
    |_| Ok(std::sync::Arc::new(MyService::new())),
    |service| async move {
        service.health_check().await
    }
).unwrap();

// Check all services
let health_results = container.health_check_all().unwrap();
for (service_name, status) in health_results {
    println!("{}: {:?}", service_name, status);
}
```

## Best Practices

### 1. Use Appropriate Lifetimes

- **Singleton**: For stateless services, shared resources, and expensive-to-create objects
- **Transient**: For lightweight, stateless services that may hold different state per usage
- **Scoped**: For services that need isolation between different operations or users

### 2. Thread Safety

All services should be `Send + Sync` since they will be accessed concurrently:

```rust
#[derive(Clone)]
struct MyService {
    // Use Arc for shared ownership
    data: std::sync::Arc<std::sync::RwLock<Data>>,
}
```

### 3. Error Handling

Handle service resolution errors gracefully:

```rust
match container.resolve::<MyService>() {
    Ok(service) => {
        // Use service
    }
    Err(ricecoder_di::DIError::ServiceNotRegistered { .. }) => {
        // Handle missing service
    }
    Err(e) => {
        // Handle other errors
    }
}
```

### 4. Dependency Injection in Constructors

Use factory functions that take the container as a parameter:

```rust
container.register(|container| {
    let dependency = container.resolve::<DependencyService>()?;
    let service = std::sync::Arc::new(MyService::new(dependency));
    Ok(service)
}).unwrap();
```

### 5. Feature Flags for Optional Dependencies

Use feature flags to make services optional:

```rust
#[cfg(feature = "storage")]
container.register(|_| {
    let storage = std::sync::Arc::new(FileStorage::new());
    Ok(storage)
}).unwrap();
```

### 6. Testing

Test services in isolation and with different configurations:

```rust
#[test]
fn test_service_registration() {
    let container = DIContainer::new();
    container.register(|_| Ok(std::sync::Arc::new(TestService::new()))).unwrap();

    let service = container.resolve::<TestService>().unwrap();
    assert!(service.is_ready());
}
```

## Architecture

The DI container follows clean architecture principles:

- **Domain Layer**: Business logic and entities
- **Application Layer**: Use cases and application services (registered in DI)
- **Infrastructure Layer**: External interfaces and implementations

Services are registered at application startup and resolved as needed, enabling loose coupling and testability.

## Performance

- Singleton services are cached after first resolution
- RwLock provides concurrent read access with exclusive write access for registration
- Scoped services minimize memory usage by isolating instances per scope
- Benchmarks are available in `benches/di_benchmarks.rs`

## Testing

Comprehensive test coverage includes:

- **Unit Tests**: Core functionality and edge cases
- **Integration Tests**: Cross-service interactions and concurrent access
- **Property-Based Tests**: Complex scenarios and invariants
- **Performance Benchmarks**: Memory usage and resolution speed

Run tests with:

```bash
cargo test
cargo test --features full  # Test with all optional services
cargo bench  # Run performance benchmarks
```

## Recent Changes

### SRP Refactoring (December 2024)

**ServiceProvider and Factory-Return Pattern**: Introduced cleaner DI patterns following industry best practices.

**New Patterns**:
- **ServiceProvider trait**: Unified interface for service creation and lifecycle
- **Factory-return pattern**: Factories return concrete types, container handles wrapping in `Arc`
- **Centralized HTTP client**: Single `reqwest::Client` instance configured and shared via DI

**Changes**:
- Registration now uses `Arc::new(T)` internally, factories return `T` directly
- HTTP client registered as singleton in container
- All providers and services receive injected dependencies
- Cleaner separation between factory logic and lifetime management

**Migration**:
```rust
// Old pattern (manual Arc wrapping)
container.register(|_| {
    Ok(Arc::new(MyService::new()))
})?;

// New pattern (factory returns bare type)
container.register(|_| {
    Ok(MyService::new())  // Container wraps in Arc
})?;
```

**Benefits**:
- Reduced boilerplate in registration code
- Consistent lifetime management
- Easier to refactor service constructors
- Better alignment with industry DI patterns

## Examples

See `src/usage.rs` for detailed usage examples and patterns.