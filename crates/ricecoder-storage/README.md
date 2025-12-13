# ricecoder-storage

**Purpose**: Configuration and storage infrastructure providing hierarchical settings management, data persistence, and caching for RiceCoder

## Overview

`ricecoder-storage` provides the foundational storage and configuration management infrastructure for RiceCoder. This crate serves as the central point for configuration loading, data persistence, and cross-cutting concerns like caching and preferences.

## Features

- **Configuration Management**: YAML/JSONC configuration with environment variable overrides
- **Project/User Settings**: Hierarchical configuration (project > user > defaults)
- **Hot Reload**: Configuration changes applied without restart
- **Type-Safe Config**: Strongly typed configuration structures
- **Caching**: High-performance caching with TTL and invalidation
- **Preferences**: User preference persistence
- **File Watching**: Configuration file change detection
- **Data Persistence**: Structured data storage and retrieval

## Architecture

`ricecoder-storage` serves as the infrastructure layer that other crates depend on:

### ✅ Infrastructure Responsibilities:
- Configuration loading and validation
- Data persistence and retrieval
- Caching and performance optimization
- File watching and hot reload
- User preferences management
- Path resolution and environment handling

### 🔗 Integration Points:
- **All Crates**: Every RiceCoder crate can use storage for configuration
- **TUI**: Provides UI configuration and theme storage
- **Business Logic**: Sessions, providers, etc. use storage for persistence
- **CLI**: Main application manages storage lifecycle

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-storage = "0.1"
```

## Usage

### Configuration Loading

```rust
use ricecoder_storage::config::{Config, ConfigLoader};

// Load configuration with hot reload
let loader = ConfigLoader::new();
let config = loader.load().await?;

// Access typed configuration
println!("Theme: {}", config.ui.theme.name);
println!("Provider: {}", config.providers.default);
```

### Data Persistence

```rust
use ricecoder_storage::{StorageManager, StorageConfig};

// Create storage manager
let storage = StorageManager::new(StorageConfig::default()).await?;

// Store and retrieve data
storage.store("session_data", &session).await?;
let retrieved: Session = storage.retrieve("session_data").await?;
```

### Caching

```rust
use ricecoder_storage::cache::{CacheManager, CacheInvalidationStrategy};

// Create cache with TTL
let cache = CacheManager::new(cache_dir).await?;
cache.set("key", data, CacheInvalidationStrategy::Ttl(3600)).await?;
let cached_data = cache.get("key").await?;
```

## Configuration Hierarchy

Configuration is loaded from multiple sources in priority order:

1. **CLI Flags**: Highest priority, command-line overrides
2. **Environment Variables**: `${VAR_NAME}` substitution
3. **Project Config**: `.ricecoder/config.jsonc` (project-specific)
4. **User Config**: `~/.ricecoder/config.jsonc` (user preferences)
5. **Built-in Defaults**: Lowest priority, sensible defaults

## Configuration Format

RiceCoder uses JSONC (JSON with Comments) for configuration:

```jsonc
{
  // UI Configuration
  "ui": {
    "theme": "tokyo-night",
    "vim_mode": true,
    "sidebar_width": 30
  },

  // Provider Configuration
  "providers": {
    "anthropic": {
      "api_key": "${ANTHROPIC_API_KEY}",
      "default_model": "claude-3-sonnet-20240229"
    }
  },

  // Session Configuration
  "sessions": {
    "auto_save": true,
    "max_tokens": 100000
  }
}
```

## Key Components

- **`ConfigLoader`**: Main configuration loading and management
- **`StorageManager`**: Data persistence and retrieval
- **`CacheManager`**: High-performance caching
- **`PreferencesManager`**: User preference handling
- **`FileWatcher`**: Configuration file change detection

## Integration

`ricecoder-storage` is used by all RiceCoder components:

- **TUI**: Loads UI configuration and themes
- **Sessions**: Persists conversation data
- **Providers**: Stores API keys and provider settings
- **CLI**: Manages global application configuration
- **All Crates**: Access configuration and caching services

## Contributing

When working with `ricecoder-storage`:

1. **Keep it infrastructure**: Focus on storage and configuration concerns
2. **Maintain compatibility**: Changes affect all crates
3. **Test thoroughly**: Storage failures affect the entire application
4. **Document formats**: Keep data formats and configuration schemas documented

## License

MIT
