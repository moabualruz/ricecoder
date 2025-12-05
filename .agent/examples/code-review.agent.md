---
name: code-review
description: AI-powered code review agent that analyzes code for quality, security, and best practices
model: gpt-4
temperature: 0.7
max_tokens: 2000
tools:
  - syntax-analyzer
  - security-checker
  - best-practices-validator
---

# Code Review Agent

This agent performs comprehensive code reviews by analyzing code for:

- **Code Quality**: Identifies complexity, duplication, and maintainability issues
- **Security**: Detects potential security vulnerabilities and unsafe patterns
- **Best Practices**: Validates adherence to language-specific best practices
- **Performance**: Suggests performance optimizations where applicable

## Usage

The code review agent is invoked when you request a code review:

```
ricecoder review <file>
```

## Configuration

The agent uses the following configuration:

- **Model**: GPT-4 for high-quality analysis
- **Temperature**: 0.7 for balanced creativity and consistency
- **Max Tokens**: 2000 for detailed reviews
- **Tools**: Syntax analyzer, security checker, best practices validator

## Output

The agent produces a structured review with:

1. Summary of findings
2. Detailed issues with line numbers
3. Severity levels (Critical, Warning, Info)
4. Suggested fixes for each issue
5. Overall quality score

## Examples

### Example 1: Reviewing a Rust file

```
ricecoder review src/main.rs
```

Output:
```
Code Review for src/main.rs
===========================

Summary: 3 issues found (1 Critical, 1 Warning, 1 Info)

Critical Issues:
- Line 42: Potential panic on unwrap() - use ? operator instead

Warnings:
- Line 15: Unused variable 'temp_buffer'

Info:
- Line 28: Consider using iterator instead of manual loop
```

### Example 2: Reviewing a TypeScript file

```
ricecoder review src/api.ts
```

## Customization

To customize the code review agent, edit `~/.ricecoder/agents/code-review.agent.md`:

```yaml
model: gpt-3.5-turbo  # Use faster model
temperature: 0.5      # More consistent reviews
max_tokens: 1500      # Shorter reviews
```

## See Also

- [Agent Configuration Guide](https://github.com/moabualruz/ricecoder/wiki/Configuration.md)
- [CLI Commands Reference](https://github.com/moabualruz/ricecoder/wiki/CLI-Commands.md)
