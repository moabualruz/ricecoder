# ricecoder-storage

**Purpose**: Pure storage infrastructure providing configuration management, session persistence, caching, and knowledge base storage for RiceCoder.

## Overview

`ricecoder-storage` provides the foundational storage and configuration management infrastructure for RiceCoder. This crate serves as the central point for configuration loading, data persistence, session management, and cross-cutting concerns like caching. It has **no business logic dependencies** and focuses solely on storage and persistence concerns.

## Key Types

### Configuration
| Type | Description |
|------|-------------|
| `Config` | Main configuration structure (providers, defaults, governance, TUI) |
| `ConfigLoader` | Multi-source configuration loading with validation |
| `ConfigMerger` | Hierarchical configuration merging (CLI > env > project > user > defaults) |
| `HotReloadManager` | Watch configuration files for changes |
| `ProvidersConfig` | API keys, endpoints, default provider settings |
| `TuiConfig` | Theme, animations, mouse support, accessibility |
| `CliArgs` | Command-line argument configuration |
| `EnvOverrides` | Environment variable substitution (`${VAR_NAME}`) |

### Session Management
| Type | Description |
|------|-------------|
| `SessionManager` | Create, load, save, share, delete sessions |
| `SessionData` | Session metadata (id, name, owner, timestamps, tags) |
| `SessionState` | Working directory, active files, variables, command history |
| `SessionPermissions` | Sharing permissions (view, execute, modify, share) |

### Storage Managers
| Type | Description |
|------|-------------|
| `StorageManager` (trait) | Storage operations interface |
| `PathResolver` | Cross-platform path resolution |
| `GlobalStore` | Global knowledge base (`~/Documents/.ricecoder/`) |
| `ProjectStore` | Project-local knowledge base (`./.agent/`) |

### Caching
| Type | Description |
|------|-------------|
| `CacheManager` | File-based caching with TTL |
| `CacheEntry<T>` | Cached data with metadata |
| `CacheInvalidationStrategy` | TTL, version-based, or manual invalidation |
| `ConfigCache` | Configuration-specific caching |

### Types & Enums
| Type | Description |
|------|-------------|
| `StorageConfig` | Storage paths and mode configuration |
| `StorageMode` | GlobalOnly, ProjectOnly, Merged |
| `ResourceType` | Template, Standard, Spec, Governance, Boilerplate, Rule, etc. |
| `ConfigFormat` | Yaml, Toml, Json, Jsonc |
| `DocumentFormat` | Yaml, Markdown |
| `StorageState` | Available, Unavailable, ReadOnly |
| `StorageError` | Comprehensive error types with context |

## Usage Examples

### Configuration Loading

```rust
use ricecoder_storage::{ConfigLoader, Config};

// Load configuration from all sources
let loader = ConfigLoader::new();
let config: Config = loader.load()?;

// Access typed configuration
println!("Default provider: {:?}", config.providers.default_provider);
println!("Theme: {}", config.tui.theme);
println!("Vim mode: {}", config.tui.vim_mode);
```

### Session Management

```rust
use ricecoder_storage::{SessionManager, SessionPermissions};
use std::path::PathBuf;

// Create session manager
let manager = SessionManager::new(PathBuf::from("~/.ricecoder/sessions"));

// Create a new session
let session = manager.create_session("My Session".to_string(), "user@example.com".to_string())?;

// Add command to history
manager.add_command_to_history(&session.id, "rice chat".to_string())?;

// Share session with permissions
let permissions = SessionPermissions {
    can_view: true,
    can_execute: false,
    can_modify: false,
    can_share: false,
    allowed_users: vec![],
};
manager.share_session(&session.id, permissions)?;

// List user sessions
let sessions = manager.list_user_sessions("user@example.com")?;
```

### Global & Project Storage

```rust
use ricecoder_storage::{GlobalStore, ProjectStore, ResourceType};

// Global store (~/.ricecoder/ or ~/Documents/.ricecoder/)
let global_store = GlobalStore::with_default_path()?;
global_store.initialize()?;

// Store a template
global_store.store_resource(
    ResourceType::Template,
    "rust-cli.yaml",
    b"name: Rust CLI\nversion: 1.0"
)?;

// Project store (./.agent/)
let project_store = ProjectStore::with_default_path();
project_store.initialize()?;

// Store project-specific spec
project_store.store_resource(
    ResourceType::Spec,
    "feature.yaml",
    b"name: New Feature"
)?;

// List resources
let templates = global_store.list_resources(ResourceType::Template)?;
```

### Caching

```rust
use ricecoder_storage::{CacheManager, CacheInvalidationStrategy};
use std::path::PathBuf;

// Create cache manager
let cache = CacheManager::new(PathBuf::from("~/.ricecoder/cache")).await?;

// Store with TTL (1 hour)
cache.set("analysis_result", &data, CacheInvalidationStrategy::Ttl(3600)).await?;

// Retrieve
if let Some(cached) = cache.get::<AnalysisResult>("analysis_result").await? {
    println!("Cache hit!");
}
```

### Path Resolution

```rust
use ricecoder_storage::PathResolver;
use std::path::Path;

// Resolve global storage path (respects RICECODER_HOME env var)
let global_path = PathResolver::resolve_global_path()?;

// Resolve user storage path (~/.ricecoder/)
let user_path = PathResolver::resolve_user_path()?;

// Expand ~ in paths
let expanded = PathResolver::expand_home(Path::new("~/Documents/code"))?;
```

## Configuration Hierarchy

Configuration is loaded from multiple sources in priority order:

1. **CLI Flags** - Highest priority, command-line overrides
2. **Environment Variables** - `${VAR_NAME}` substitution
3. **Project Config** - `.agent/config.jsonc` (project-specific)
4. **User Config** - `~/.ricecoder/config.jsonc` (user preferences)
5. **Built-in Defaults** - Lowest priority, sensible defaults

## Configuration Format

RiceCoder uses JSONC (JSON with Comments) for configuration:

```jsonc
{
  // Provider Configuration
  "providers": {
    "api_keys": {
      "anthropic": "${ANTHROPIC_API_KEY}",
      "openai": "${OPENAI_API_KEY}"
    },
    "endpoints": {
      "ollama": "http://localhost:11434"
    },
    "default_provider": "anthropic"
  },

  // Default Model Settings
  "defaults": {
    "model": "claude-3-sonnet",
    "temperature": 0.7,
    "max_tokens": 4096
  },

  // TUI Configuration
  "tui": {
    "theme": "tokyo-night",
    "animations": true,
    "mouse": true,
    "vim_mode": false,
    "accessibility": {
      "screen_reader_enabled": false,
      "high_contrast_mode": false,
      "disable_animations": false,
      "focus_indicator_intensity": 3
    }
  }
}
```

## Directory Structure

### Global Storage (`~/Documents/.ricecoder/` or `~/.ricecoder/`)
```
.ricecoder/
├── templates/       # Code generation templates
├── standards/       # Coding standards and guidelines
├── specs/           # Specification documents
├── Governance/      # Governance documents (project rules)
├── boilerplates/    # Boilerplate projects
├── rules/           # Learned rules from learning system
├── sessions/        # Session persistence
├── cache/           # Cached analysis and config
└── config.jsonc     # User configuration
```

### Project Storage (`./.agent/`)
```
.agent/
├── templates/       # Project-specific templates
├── standards/       # Project-specific standards
├── specs/           # Project specifications
├── Governance/      # Project governance rules
├── rules/           # Project-specific learned rules
├── history/         # Command history
├── cache/           # Project cache
└── config.jsonc     # Project configuration
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `serde` | Serialization/deserialization |
| `serde_json` | JSON parsing |
| `serde_yaml` | YAML parsing |
| `toml` | TOML parsing |
| `thiserror` | Error handling |
| `tracing` | Logging and diagnostics |
| `dirs` | Cross-platform directory resolution |
| `tokio` | Async runtime |
| `notify` | File watching for hot reload |
| `jsonc-parser` | JSONC (JSON with comments) parsing |
| `jsonschema` | Configuration validation |
| `chrono` | Timestamp handling |

## Dependents

| Crate | Usage |
|-------|-------|
| `ricecoder-providers` | Provider configuration and API key storage |

## Architecture

`ricecoder-storage` serves as the **pure infrastructure layer** that other crates depend on:

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│  (ricecoder-cli, ricecoder-tui, ricecoder-mcp)              │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Business Logic Layer                      │
│  (ricecoder-providers, ricecoder-agents, ricecoder-specs)   │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                  RICECODER-STORAGE                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐│
│  │   Config    │ │   Session   │ │   Storage (Global/      ││
│  │   Loading   │ │   Manager   │ │   Project Stores)       ││
│  └─────────────┘ └─────────────┘ └─────────────────────────┘│
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐│
│  │   Caching   │ │   Path      │ │   Theme Storage         ││
│  │   Manager   │ │   Resolver  │ │                         ││
│  └─────────────┘ └─────────────┘ └─────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    File System / OS                          │
└─────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **No Business Logic** - Pure storage and persistence concerns only
2. **Cross-Platform** - Works on Windows, macOS, and Linux
3. **Environment Aware** - Respects `RICECODER_HOME` and other env vars
4. **Hierarchical Config** - Project overrides user overrides defaults
5. **Hot Reload** - Configuration changes applied without restart
6. **Type-Safe** - Strongly typed configuration structures

## Contributing

When working with `ricecoder-storage`:

1. **Keep it infrastructure** - Focus on storage and configuration concerns
2. **Maintain compatibility** - Changes affect all dependent crates
3. **Test thoroughly** - Storage failures affect the entire application
4. **Document formats** - Keep data formats and configuration schemas documented
5. **No business logic** - Do not add domain-specific logic

## License

MIT
