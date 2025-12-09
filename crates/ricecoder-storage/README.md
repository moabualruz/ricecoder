# ricecoder-storage

Storage and configuration management for RiceCoder.

## Features

- YAML-based configuration
- Project and user-level settings
- Environment variable overrides
- Type-safe configuration loading
- Hot-reload support

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-storage = "0.1"
```

## Usage

```rust
use ricecoder_storage::config::Config;

// Load configuration
let config = Config::load()?;

// Access settings
println!("Theme: {}", config.theme);
```

## Configuration

Configuration is loaded from (in priority order):

1. Environment variables
2. Project config: `.ricecoder/config.yaml`
3. User config: `~/.ricecoder/config.yaml`
4. Built-in defaults

## License

MIT
