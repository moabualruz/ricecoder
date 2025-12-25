# Code Style and Conventions

## Naming
- Types: PascalCase (CommandDefinition, ArgumentType)
- Functions/Methods: snake_case (with_description, execute)
- Variables: snake_case
- Constants: SCREAMING_SNAKE_CASE

## Documentation
- Use /// for public items
- Include examples in doc comments

## Error Handling
- Use thiserror for custom errors
- Return Result<T, CommandError>

## Serialization
- Use serde with derive
- Fields are snake_case in JSON/YAML

## Testing
- Unit tests in same file with #[cfg(test)]
- Use assert_eq! for comparisons

## Patterns
- Builder pattern for complex structs
- Comprehensive error types