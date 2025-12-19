# RiceGrep

AI-enhanced code search tool with full ripgrep compatibility and modern CLI architecture.

## Features

- **Ripgrep Compatible**: Drop-in replacement for ripgrep with identical CLI options
- **AI-Enhanced Search**: Intelligent query understanding and result reranking
- **Content Display**: Full file content viewing with syntax highlighting support
- **Answer Generation**: AI-powered answer synthesis from search results
- **Deterministic Fallback**: Always works, even without AI dependencies
- **Spelling Correction**: Automatic correction of typos in search queries
- **Language Awareness**: Programming language detection and context-aware ranking
- **Custom Ignore Files**: Support for .ricegrepignore files with .gitignore-style patterns
- **Progress Indication**: Visual progress bars and spinners for long-running operations
- **Dry Run Support**: Preview operations without making changes (--dry-run flag)
- **File Size Management**: Configurable limits on file sizes for search and indexing
- **Indexing**: Fast search acceleration for large codebases with progress feedback
- **Watch Mode**: Continuous monitoring with automatic index updates
- **Safe Replace**: Preview and execute find-replace operations safely
- **Symbol Rename**: Language-aware symbol renaming with safety checks
- **AI Assistant Integration**: Native support for Claude Code, OpenCode, Codex, Factory Droid, Cursor, Windsurf
- **MCP Server**: Model Context Protocol server for AI assistant tool integration
- **Skill System**: Structured skill definitions for assistant capability discovery
- **Plugin Marketplace**: Extensible plugin system for third-party integrations
- **Modern CLI**: Subcommand architecture with comprehensive help and completion

## Installation

```bash
cargo install --path crates/ricegrep
```

## Usage

### Modern Subcommand Architecture

RiceGrep uses a modern subcommand architecture for better organization and discoverability:

```bash
ricegrep search [OPTIONS] [PATTERN] [PATH]...    # Search for patterns
ricegrep replace [OPTIONS] OLD_SYMBOL NEW_SYMBOL FILE  # Symbol rename operations
ricegrep watch [OPTIONS] [PATH]...               # Watch mode for continuous monitoring
ricegrep mcp [OPTIONS]                           # Start MCP server for AI assistants
ricegrep install [OPTIONS] [PLUGIN]              # Install AI assistant integrations
ricegrep uninstall [OPTIONS] [PLUGIN]            # Uninstall AI assistant integrations
ricegrep --help                                  # Show help
ricegrep --version                               # Show version
```

### Basic Search
```bash
# Search for function definitions (subcommand syntax)
ricegrep search 'fn main' src/

# Traditional ripgrep-compatible syntax (still supported)
ricegrep 'fn main' src/

# Case-insensitive search
ricegrep search --ignore-case 'TODO' .

# Word-based search
ricegrep search --word-regexp 'function' .

# Custom ignore file (e.g., .ricegrepignore)
ricegrep search --ignore-file .ricegrepignore 'debug' src/

# Limit file size for performance
ricegrep search --max-file-size 1048576 'pattern' large-dir/

# Suppress progress output
ricegrep search --quiet 'pattern' .
```

### Symbol Rename Operations
```bash
# Preview symbol rename (language-aware)
ricegrep replace old_function new_function src/main.rs

# Execute symbol rename with confirmation
ricegrep replace --force old_variable new_variable lib.rs

# Dry-run to see what would be changed
ricegrep replace --dry-run old_name new_name file.rs
```

### AI-Enhanced Features
```bash
# Content display - show full file contents
ricegrep search --content 'println' src/

# Answer generation - AI-powered answers from results
ricegrep search --answer 'how does error handling work' .

# Deterministic results - disable AI reranking
ricegrep search --no-rerank 'function' .

# Natural language queries (legacy mode)
ricegrep --ai-enhanced 'find all functions that handle errors'
```

### Custom Ignore Files
```bash
# Use a custom ignore file (.ricegrepignore)
ricegrep search --ignore-file .ricegrepignore 'password' .

# .ricegrepignore supports .gitignore-style patterns
echo -e "*.log\ntemp/*\n!important.log" > .ricegrepignore
ricegrep search --ignore-file .ricegrepignore 'error' .
```

### Indexing for Performance
```bash
# Build search index (legacy mode for compatibility)
ricegrep --index-build .

# Check index status
ricegrep --index-status

# Watch mode for continuous updates
ricegrep watch .
ricegrep watch --timeout 300 src/  # Watch with 5-minute timeout
```

## AI Assistant Integration

RiceGrep integrates seamlessly with popular AI coding assistants, providing enhanced search and refactoring capabilities.

### Supported Assistants

- **Claude Code**: Plugin marketplace integration with skills system
- **OpenCode**: Plugin system with hooks and event handling
- **Codex**: Skills-based integration with MCP server support
- **Factory Droid**: Python hooks for background processing
- **Cursor**: Extension API integration
- **Windsurf**: Assistant compatibility layer

### Installation

Install RiceGrep integration for your preferred AI assistant:

```bash
# Claude Code integration
ricegrep install claude-code

# OpenCode integration
ricegrep install opencode

# Codex integration
ricegrep install codex

# Factory Droid integration
ricegrep install factory-droid

# Cursor integration
ricegrep install cursor

# Windsurf integration
ricegrep install windsurf
```

### MCP Server

RiceGrep provides an MCP (Model Context Protocol) server for AI assistants that support the protocol:

```bash
# Start MCP server (stdio mode for most assistants)
ricegrep mcp

# Start MCP server with custom settings
ricegrep mcp --host localhost --port 8080
```

The MCP server exposes two main tools:
- `search`: Semantic search with AI-enhanced ranking
- `replace_symbol`: Language-aware symbol renaming

### Skill Definitions

RiceGrep provides structured skill definitions that AI assistants can use to understand available capabilities:

```bash
# Export skills as JSON for assistant integration
ricegrep export-skills --format json

# Export skills as YAML
ricegrep export-skills --format yaml
```

Available skills:
- **ricegrep-search**: Semantic search with natural language queries
- **ricegrep-replace**: Safe symbol renaming with language awareness

### Safe Replace Operations
```bash
# Preview changes (dry-run)
ricegrep search 'old_name' --replace 'new_name' --dry-run file.rs

# Preview changes (legacy --preview flag)
ricegrep search 'old_name' --replace 'new_name' --preview file.rs

# Execute changes
ricegrep search 'old_name' --replace 'new_name' --force file.rs
```

## Configuration

RiceGrep supports cascading configuration:
- Command-line options (highest priority)
- Environment variables with `RICEGREP_` prefix
- TOML configuration file (`.ricegrep.toml`)

### Example Configuration (.ricegrep.toml)
```toml
# AI settings
ai_enabled = true
confidence_threshold = 0.7

# Search settings
max_results = 100
follow_symlinks = true

# Output settings
color = "auto"
line_numbers = true
```

### Custom Ignore Files (.ricegrepignore)
RiceGrep supports custom ignore files with .gitignore-style syntax:

```gitignore
# Ignore all log files
*.log

# Ignore temporary directories
temp/
tmp/

# Ignore specific files
secrets.txt
config.local.*

# Negation (don't ignore these patterns)
!important.log
!src/important/
```

Use with: `ricegrep search --ignore-file .ricegrepignore PATTERN`

## AI Features

### Content Display
Show full file contents instead of just matching lines:
```bash
ricegrep search --content 'function' src/
```

### Answer Generation
Generate AI-powered answers from search results:
```bash
ricegrep search --answer 'how does authentication work' .
```

### Deterministic Fallback
Ensure consistent results without AI dependencies:
```bash
ricegrep search --no-rerank 'query' .
```

## Performance

- **Startup**: <5s in release mode
- **Search**: <3s for typical queries on large codebases
- **Indexing**: Parallel processing with memory mapping for large files
- **Memory**: Efficient memory usage with configurable limits
- **AI Processing**: Optional enhancement with graceful fallback

## Compatibility

RiceGrep maintains full backward compatibility with ripgrep:

### Legacy Mode (ripgrep-compatible)
```bash
ricegrep --ignore-case 'TODO' .           # Case-insensitive search
ricegrep --word-regexp 'function' .       # Word-based matching
ricegrep --count 'FIXME' src/             # Count matches
ricegrep --line-number 'error' logs/      # Show line numbers
```

### Modern Subcommands
```bash
ricegrep search --ignore-case 'TODO' .    # Same functionality
ricegrep search --word-regexp 'function' .
ricegrep search --count 'FIXME' src/
ricegrep search --line-number 'error' logs/
```

All standard ripgrep options are supported identically.

## Examples

### Basic Search
```bash
# Find all TODO comments (case-insensitive)
ricegrep search --ignore-case 'todo' .

# Count matches per file
ricegrep search --count 'FIXME' src/

# Show only filenames with matches
ricegrep search --files-with-matches 'deprecated' .

# Search with context
ricegrep search --before-context 2 --after-context 2 'error' logs/
```

### Advanced Features
```bash
# Content display - see full files containing matches
ricegrep search --content 'database' src/

# AI answer generation
ricegrep search --answer 'explain the authentication flow' .

# Deterministic results (no AI variability)
ricegrep search --no-rerank 'function' .

# JSON output for tooling integration
ricegrep search --json 'error' . | jq '.matches[0].line_content'
```

### Legacy Compatibility
```bash
# All traditional ripgrep commands still work
ricegrep --ignore-case 'todo' .
ricegrep --count 'FIXME' src/
ricegrep --before-context 2 --after-context 2 'error' logs/
```

## Architecture

RiceGrep is built with a modular architecture:

- **Search Engine**: High-performance regex matching with optional indexing
- **AI Integration**: Query understanding and result enhancement (optional)
- **Output System**: Flexible formatting (text, JSON) with content display
- **Configuration**: Cascading config with CLI, env vars, and TOML files
- **CLI**: Modern subcommand architecture with backward compatibility

## Error Handling

RiceGrep implements comprehensive error handling with graceful degradation:

- **AI Failures**: Automatic fallback to deterministic ranking
- **Index Issues**: File-by-file search when indexing unavailable
- **Configuration Errors**: Sensible defaults with clear error messages
- **Memory Limits**: Efficient processing with configurable constraints

## Contributing

RiceGrep is part of the RiceCoder project. See the main project documentation for contribution guidelines.

### Development
```bash
# Run tests
cargo test -p ricegrep

# Run integration tests
cargo test --test integration_tests

# Build documentation
cargo doc -p ricegrep
```