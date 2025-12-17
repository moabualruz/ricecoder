# RiceCoder Contribution Templates

This directory contains templates to help contributors create high-quality contributions to RiceCoder.

## Available Templates

### Code Templates

- **[new-crate.md](new-crate.md)** - Template for creating a new RiceCoder crate
- **[domain-entity.rs](domain-entity.rs)** - Template for domain entities
- **[application-use-case.rs](application-use-case.rs)** - Template for application use cases
- **[infrastructure-adapter.rs](infrastructure-adapter.rs)** - Template for infrastructure adapters
- **[mcp-tool.rs](mcp-tool.rs)** - Template for MCP tool implementations
- **[provider-integration.rs](provider-integration.rs)** - Template for AI provider integrations

### Documentation Templates

- **[feature-specification.md](feature-specification.md)** - Template for feature specifications
- **[api-documentation.md](api-documentation.md)** - Template for API documentation
- **[troubleshooting-guide.md](troubleshooting-guide.md)** - Template for troubleshooting guides

### Testing Templates

- **[unit-test.rs](unit-test.rs)** - Template for unit tests
- **[integration-test.rs](integration-test.rs)** - Template for integration tests
- **[property-test.rs](property-test.rs)** - Template for property-based tests

### GitHub Templates

- **[pull-request-template.md](pull-request-template.md)** - Template for pull request descriptions
- **[issue-template-bug.md](issue-template-bug.md)** - Template for bug reports
- **[issue-template-feature.md](issue-template-feature.md)** - Template for feature requests

## How to Use Templates

1. **Copy the template** to your working directory
2. **Replace placeholders** (marked with `{{ }}`) with actual values
3. **Customize** the template for your specific use case
4. **Remove** any sections that don't apply
5. **Follow** the guidelines in each template

## Template Guidelines

### Placeholders

Use these placeholder patterns:
- `{{ClassName}}` - PascalCase class/struct names
- `{{function_name}}` - snake_case function names
- `{{variable_name}}` - snake_case variable names
- `{{description}}` - Brief descriptions
- `{{detailed_description}}` - Detailed explanations

### Code Comments

Include these comment types:
- `// TODO: {{task}}` - Tasks to complete
- `// FIXME: {{issue}}` - Known issues to fix
- `// NOTE: {{explanation}}` - Important notes
- `// SAFETY: {{justification}}` - Unsafe code justification

### Documentation

All public APIs must have:
- Module-level documentation
- Function/struct documentation
- Parameter documentation
- Return value documentation
- Example usage
- Error conditions

### Testing

Templates include:
- Unit test structure
- Integration test patterns
- Property-based test examples
- Mock setup guidance

## Contributing New Templates

To add a new template:

1. Create the template file in the appropriate subdirectory
2. Add documentation in the template header
3. Include usage examples
4. Update this README.md
5. Test the template with a real contribution

## Template Maintenance

Templates should be:
- **Updated** when patterns change
- **Tested** with real contributions
- **Reviewed** during code reviews
- **Versioned** with the project

---

*See individual template files for detailed usage instructions.*