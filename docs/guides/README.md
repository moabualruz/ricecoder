# RiceCoder Integration Guides

This directory contains comprehensive integration guides for RiceCoder, covering various use cases and integration scenarios.

## Available Guides

### Getting Started

- [Quick Start Guide](quick-start.md) - Get up and running in 5 minutes
- [Installation Guide](installation.md) - Detailed installation instructions
- [Configuration Guide](configuration.md) - Configure RiceCoder for your environment

### Development Workflows

- [Spec-Driven Development](spec-driven-development.md) - Systematic development with specifications
- [Code Generation](code-generation.md) - Generate code from specs with AI enhancement
- [Code Review](code-review.md) - AI-assisted code review workflows
- [Refactoring](refactoring.md) - Safe code refactoring with RiceCoder

### AI Provider Integration

- [OpenAI Integration](providers/openai.md) - Configure OpenAI GPT models
- [Anthropic Integration](providers/anthropic.md) - Configure Claude models
- [Ollama Integration](providers/ollama.md) - Local model setup
- [Multi-Provider Setup](providers/multi-provider.md) - Use multiple providers

### Enterprise Integration

- [GitHub Integration](enterprise/github.md) - GitHub API integration
- [Jira Integration](enterprise/jira.md) - Jira issue tracking
- [Slack Integration](enterprise/slack.md) - Slack notifications
- [CI/CD Integration](enterprise/ci-cd.md) - Integrate with CI/CD pipelines

### Advanced Features

- [MCP Integration](mcp-integration.md) - Model Context Protocol setup
- [Session Management](session-management.md) - Multi-session workflows
- [Team Collaboration](team-collaboration.md) - Team features and permissions
- [Performance Optimization](performance.md) - Optimize RiceCoder performance

## Contributing to Guides

### Guide Standards

All guides must follow these standards:

1. **Clear Structure**: Use headings, subheadings, and consistent formatting
2. **Code Examples**: Include practical, runnable code examples
3. **Prerequisites**: List requirements at the beginning
4. **Troubleshooting**: Include common issues and solutions
5. **Cross-references**: Link to related guides and API documentation

### Writing Guidelines

- Use Markdown formatting consistently
- Include table of contents for longer guides
- Test all code examples
- Keep content up-to-date with latest features
- Use inclusive, accessible language

### Review Process

1. Create guide in appropriate subdirectory
2. Test all instructions and examples
3. Run link validation: `node scripts/validate-documentation-links.js`
4. Submit PR with guide
5. Review for clarity, accuracy, and completeness