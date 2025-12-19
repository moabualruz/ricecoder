# RiceGrep

AI-enhanced code search tool with full ripgrep compatibility.

## Features

- **Ripgrep Compatible**: Drop-in replacement for ripgrep with identical CLI options
- **AI-Enhanced Search**: Intelligent query understanding and result reranking
- **Spelling Correction**: Automatic correction of typos in search queries
- **Language Awareness**: Programming language detection and context-aware ranking
- **Indexing**: Fast search acceleration for large codebases
- **Watch Mode**: Continuous monitoring with automatic index updates
- **Safe Replace**: Preview and execute find-replace operations safely

## Installation

```bash
cargo install --path crates/ricegrep
```

## Usage

### Basic Search
```bash
# Search for function definitions
ricegrep 'fn main' src/

# Case-insensitive search
ricegrep --ignore-case 'TODO' .

# Word-based search
ricegrep --word-regexp 'function' .
```

### AI-Enhanced Search
```bash
# Natural language queries
ricegrep --ai-enhanced 'find all functions that handle errors'

# Intelligent reranking
ricegrep --ai-enhanced 'database connection code'
```

### Indexing for Performance
```bash
# Build search index
ricegrep --index-build .

# Check index status
ricegrep --index-status

# Watch mode for continuous updates
ricegrep --index-watch
```

### Safe Replace Operations
```bash
# Preview changes
ricegrep 'old_name' --replace 'new_name' --preview file.rs

# Execute changes
ricegrep 'old_name' --replace 'new_name' --force file.rs
```

## Configuration

RiceGrep supports configuration via:
- Command-line options (highest priority)
- Environment variables with `RICEGREP_` prefix
- TOML configuration file (`.ricegrep.toml`)

## Performance

- **Startup**: <5s in release mode
- **Search**: <3s for typical queries on large codebases
- **Indexing**: Parallel processing with memory mapping for large files
- **Memory**: Efficient memory usage with configurable limits

## Compatibility

RiceGrep is designed as a drop-in replacement for ripgrep. All standard options work identically:

- `--ignore-case`, `--word-regexp`, `--count`, `--line-number`
- `--before-context`, `--after-context`, `--context`
- `--files`, `--files-with-matches`, `--invert-match`
- And many more...

## Examples

```bash
# Find all TODO comments (case-insensitive)
ricegrep --ignore-case 'todo' .

# Count matches per file
ricegrep --count 'FIXME' src/

# Show only filenames with matches
ricegrep --files-with-matches 'deprecated' .

# Search with context
ricegrep --before-context 2 --after-context 2 'error' logs/
```

## Contributing

RiceGrep is part of the RiceCoder project. See the main project documentation for contribution guidelines.