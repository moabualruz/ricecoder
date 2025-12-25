# ricecoder-orchestration

**Purpose**: Multi-project orchestration and workspace management providing cross-project operations and coordinated development workflows for RiceCoder

## DDD Layer

**Application** - Multi-project orchestration as an application service layer.

## Features

- **Multi-Project Management**: Coordinated operations across multiple projects and repositories
- **Workspace Orchestration**: Intelligent workspace setup and dependency management
- **Cross-Project Operations**: Bulk operations, refactoring, and analysis across project boundaries
- **Dependency Resolution**: Automatic resolution of inter-project dependencies and conflicts
- **Resource Coordination**: Efficient resource allocation and parallel processing across projects

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-orchestration = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_orchestration::{WorkspaceManager, ProjectCoordinator};

// Create workspace manager
let workspace = WorkspaceManager::new("./workspace")?;

// Add projects to workspace
workspace.add_project("api", "./projects/api")?;
workspace.add_project("web", "./projects/web")?;

// Coordinate operations across projects
let coordinator = ProjectCoordinator::new(workspace);
coordinator.run_operation("build-all").await?;
```

## Documentation

For more information, see the [documentation](https://docs.rs/ricecoder-orchestration).

## License

MIT
