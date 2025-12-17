# RiceCoder API Reference

This directory contains the API reference documentation generated from the RiceCoder codebase.

## Generating API Documentation

### Local Development

Generate documentation locally:

```bash
cd projects/ricecoder
cargo doc --open --no-deps
```

This will generate HTML documentation in `target/doc/` and open it in your browser.

### CI/CD Generation

API documentation is automatically generated during CI/CD pipelines and published to GitHub Pages.

### Documentation Features

- **Cross-references**: All types, functions, and modules are cross-referenced
- **Examples**: Code examples are included where available
- **Search**: Full-text search functionality
- **Source links**: Direct links to source code on GitHub

## API Structure

### Core Modules

- `ricecoder` - Main CLI interface
- `ricecoder_domain` - Core business logic and entities
- `ricecoder_security` - Security and authentication
- `ricecoder_cache` - Caching infrastructure
- `ricecoder_parsers` - Code parsing and AST handling
- `ricecoder_patterns` - Code pattern detection
- `ricecoder_providers` - AI provider integrations
- `ricecoder_sessions` - Session management
- `ricecoder_mcp` - Model Context Protocol implementation

### Enterprise Features

- `ricecoder_industry` - Industry integrations (GitHub, Jira, etc.)
- `ricecoder_activity_log` - Activity logging and audit trails
- `ricecoder_safety` - Safety constraints and risk scoring

## Documentation Standards

All public APIs must include:

- Comprehensive doc comments with `///`
- Parameter descriptions with `@param`
- Return value descriptions with `@return`
- Error conditions and examples
- Code examples where applicable

Example:

```rust
/// Analyze a project and return analysis results
///
/// # Arguments
///
/// * `project_path` - Path to the project root directory
/// * `config` - Analysis configuration options
///
/// # Returns
///
/// Returns a `Result` containing the analysis results or an error
///
/// # Examples
///
/// ```rust
/// use ricecoder_research::ProjectAnalyzer;
///
/// let analyzer = ProjectAnalyzer::new();
/// let results = analyzer.analyze("/path/to/project", &config)?;
/// ```
pub fn analyze_project(
    project_path: &Path,
    config: &AnalysisConfig
) -> Result<AnalysisResult, AnalysisError> {
    // Implementation
}
```