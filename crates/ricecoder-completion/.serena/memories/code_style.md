# Code Style and Conventions

## Naming Conventions
- **Modules**: snake_case (e.g., `context_analyzer`, `completion_engine`)
- **Types/Structs/Enums**: CamelCase (e.g., `CompletionItem`, `GenericCompletionEngine`)
- **Functions/Methods**: snake_case (e.g., `generate_completions`, `rank_completions`)
- **Variables**: snake_case (e.g., `context_analyzer`, `completion_items`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Traits**: CamelCase ending with trait name (e.g., `CompletionEngine`, `ContextAnalyzer`)

## Documentation
- **Module docs**: Extensive /// comments at the top of modules explaining purpose, architecture, and usage
- **Function docs**: /// comments with # Arguments, # Returns, # Errors sections
- **Examples**: Code examples in docstrings using ```ignore blocks
- **Inline comments**: // for implementation notes, minimal usage

## Type Hints and Typing
- **Static typing**: All functions and variables have explicit types
- **Generics**: Used extensively for language-agnostic design
- **Traits**: Define interfaces for pluggable components (engines, providers, rankers)
- **Error handling**: Custom error types with detailed error variants

## Code Organization
- **Modules**: Logical separation by functionality (engine, context, providers, etc.)
- **Traits first**: Define traits before implementations
- **Async/await**: Used throughout for non-blocking operations
- **Arc/RefCell**: Smart pointers for shared mutable state
- **Builder pattern**: For complex object construction

## Patterns and Design
- **Pipeline architecture**: Context analysis → Generation → Ranking
- **Provider pattern**: Pluggable language-specific providers
- **Strategy pattern**: Different ranking and generation strategies
- **Observer pattern**: For completion history tracking
- **Graceful degradation**: Fallback mechanisms when LSP unavailable

## Testing
- **Unit tests**: For individual components
- **Integration tests**: For LSP and provider interactions
- **Property tests**: For correctness verification
- **Async testing**: Using tokio::test