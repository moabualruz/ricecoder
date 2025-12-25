# Development Guidelines

## Core Principles
1. **LSP First**: Prefer external LSP integration over internal providers for semantic completions
2. **Fallback Robustness**: Ensure graceful degradation when LSP servers are unavailable
3. **Language Agnostic**: Keep core engine language-independent with pluggable providers
4. **Performance**: Maintain sub-100ms response times for completions
5. **Testing**: Test both LSP and fallback code paths thoroughly

## Architecture Guidelines
- **Pipeline Design**: Follow the established pipeline: Context Analysis → Generation → Ranking
- **External Priority**: External LSP completions take priority over internal ones
- **Graceful Degradation**: Always provide some completions, even if not semantic
- **Async First**: Use async/await for all I/O operations
- **Trait-Based**: Define interfaces via traits for extensibility

## Code Quality
- **Documentation**: Extensive docstrings for all public APIs
- **Error Handling**: Detailed error types with specific error variants
- **Type Safety**: Leverage Rust's type system for correctness
- **Memory Safety**: Use appropriate smart pointers (Arc, RefCell) for shared state
- **Performance**: Efficient algorithms with caching where appropriate

## Testing Strategy
- **Unit Tests**: Test individual components in isolation
- **Integration Tests**: Test LSP communication and provider interactions
- **Property Tests**: Verify correctness with generated test cases
- **Coverage**: Aim for high test coverage, especially for critical paths

## Contribution Workflow
1. Understand the LSP-first architecture
2. Implement with fallback mechanisms
3. Add comprehensive tests
4. Update documentation
5. Run full quality checks before submitting

## Language Support
When adding new language support:
1. Create language-specific provider
2. Register in ProviderRegistry
3. Add tree-sitter parser if needed
4. Configure LSP server integration
5. Add comprehensive tests