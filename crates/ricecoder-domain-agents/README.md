# ricecoder-domain-agents

**Purpose**: Domain-specific agents providing specialized language and framework support for frontend, backend, DevOps, and cloud development in RiceCoder

## DDD Layer

**Application** - Provides domain-specialized agent implementations as application services.

## Overview

`ricecoder-domain-agents` provides specialized AI agents for different development domains including frontend frameworks, backend technologies, DevOps practices, data engineering, mobile development, and cloud architecture. Each agent is trained and optimized for its specific domain, providing expert-level assistance and code generation.

## Features

- **Frontend Agents**: Specialized agents for React, Vue, Angular, and modern frontend frameworks
- **Backend Agents**: Domain experts for Node.js, Python, Go, Rust, and enterprise backend systems
- **DevOps Agents**: Infrastructure automation, CI/CD pipeline management, and cloud deployment expertise
- **Data Engineering**: ETL pipeline design, database optimization, and big data processing knowledge
- **Mobile Development**: iOS, Android, React Native, and Flutter application development support
- **Cloud Architecture**: AWS, Azure, GCP service integration and cloud-native application design

## Architecture

### Responsibilities
- Domain-specific agent management and specialization
- Context-aware code generation for different frameworks
- Best practices enforcement for each technology stack
- Integration with development workflows and tools
- Performance optimization for domain-specific tasks

### Dependencies
- **Providers**: `ricecoder-providers` for AI capabilities
- **Storage**: `ricecoder-storage` for agent configurations
- **Async Runtime**: `tokio` for concurrent agent operations

### Integration Points
- **Agents Framework**: Extends the base agent framework with domain specialization
- **Workflows**: Domain agents participate in specialized development workflows
- **Sessions**: Context-aware assistance based on project technology stack

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-domain-agents = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_domain_agents::{DomainAgentManager, FrontendAgent, BackendAgent};

// Create domain agent manager
let manager = DomainAgentManager::new();

// Register specialized agents
manager.register_agent(Box::new(FrontendAgent::new("react")))?;
manager.register_agent(Box::new(BackendAgent::new("rust")))?;

// Get agent for specific task
let agent = manager.get_agent_for_domain("frontend", "react-component")?;

// Execute domain-specific task
let result = agent.execute_task("create-login-form", &context).await?;
println!("Generated component: {}", result.code);
```

## API Reference

### Key Types

- **`DomainAgentManager`**: Central domain agent coordination
- **`FrontendAgent`**: React, Vue, Angular specialized agent
- **`BackendAgent`**: Node.js, Python, Go, Rust backend agent
- **`DevOpsAgent`**: Infrastructure and deployment agent

### Key Functions

- **`register_agent()`**: Register a domain-specific agent
- **`get_agent_for_domain()`**: Get appropriate agent for technology stack
- **`execute_task()`**: Execute domain-specific development task

## Error Handling

```rust
use ricecoder_domain_agents::DomainAgentError;

match agent.execute_task("create-component", &context).await {
    Ok(result) => println!("Component created: {}", result.path),
    Err(DomainAgentError::UnsupportedFramework) => eprintln!("Framework not supported"),
    Err(DomainAgentError::TaskExecutionFailed(msg)) => eprintln!("Task failed: {}", msg),
}
```

## Testing

Run domain agent tests:

```bash
# Run all tests
cargo test -p ricecoder-domain-agents

# Test specific domain agents
cargo test -p ricecoder-domain-agents frontend
cargo test -p ricecoder-domain-agents backend
```

## Performance

- **Task Execution**: Variable based on complexity (500ms - 10s)
- **Agent Selection**: < 50ms for domain matching
- **Concurrent Tasks**: Support for multiple domain agents
- **Memory Usage**: Efficient domain-specific model loading

## Contributing

When working with `ricecoder-domain-agents`:

1. **Domain Expertise**: Ensure agents have deep knowledge of their technology stack
2. **Best Practices**: Implement current best practices for each framework
3. **Testing**: Test agents with real-world scenarios for each domain
4. **Documentation**: Document supported frameworks and limitations

## License

MIT
