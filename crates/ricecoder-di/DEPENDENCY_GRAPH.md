## Service Dependency Graph

The RiceCoder DI container organizes services in layers following clean architecture principles:

### Infrastructure Layer (Bottom)
- **Storage Services**: `StorageManager`, `FileStorage`, `MemoryStorage`
- **Cache Services**: `CacheManager`, `CacheStorage`, `CacheStrategy`
- **External APIs**: `GitHubManager`, `LocalModelManager`, `WebhookHandler`
- **File Systems**: `FileManager`, `FileWatcher`, `TransactionManager`
- **Version Control**: `VCSManager`, `GitIntegration`, `RepositoryManager`

### Domain Layer (Middle)
- **Core Entities**: `DomainService`, `Repository`, `EntityManager`
- **Business Logic**: `WorkflowEngine`, `SpecProcessor`, `CodeGenerator`
- **Parsing**: `Parser`, `LanguageRegistry`
- **Learning**: `LearningManager`, `PatternCapturer`, `RuleValidator`

### Application Layer (Top)
- **Use Cases**: `SessionLifecycleUseCase`, `ProviderSwitchingUseCase`
- **Orchestration**: `WorkspaceOrchestrator`, `DomainAgentRegistryManager`
- **User Interface**: `ThemeManager`, `CommandRouter`, `ModeManager`

### Dependency Flow
```
Infrastructure ← Domain ← Application
     ↑            ↑         ↑
   External     Business   User
  Services     Services  Services
```

### Service Registration Order
Services are registered in dependency order to ensure proper initialization:

1. **Infrastructure Services** (no dependencies)
2. **Domain Services** (depend on infrastructure)
3. **Application Services** (depend on domain + infrastructure)

### Lifecycle Management
Services can implement the `Lifecycle` trait for startup/shutdown hooks:

- **Initialization**: Database connections, background tasks, service discovery
- **Cleanup**: Resource cleanup, connection closing, task termination

### Configuration-Driven Wiring
The `ContainerConfig` struct allows runtime configuration of service registration:

```rust
let config = ContainerConfig {
    enable_github: true,
    enable_parsers: true,
    enable_generation: false,
    ..Default::default()
};
let container = create_configured_container(&config)?;
```

This enables different deployment scenarios with appropriate service sets.