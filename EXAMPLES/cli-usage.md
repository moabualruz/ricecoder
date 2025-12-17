# CLI Usage Examples

This directory contains practical examples of using RiceCoder's command-line interface for common development tasks.

## Basic Commands

### Starting RiceCoder

```bash
# Start interactive chat mode
rice chat

# Start with a specific provider
rice chat --provider openai

# Start with a specific model
rice chat --model gpt-4

# Start with verbose logging
rice chat --verbose
```

### Project Initialization

```bash
# Initialize a new project
rice init

# Initialize with specific language detection
rice init --language rust

# Initialize with custom configuration
rice init --config my-config.toml
```

### Code Generation

```bash
# Generate code from a specification file
rice gen --spec user-auth.spec.md

# Generate with specific provider
rice gen --spec api-endpoint.spec.md --provider anthropic

# Generate and automatically apply changes
rice gen --spec database-model.spec.md --apply

# Generate with custom context
rice gen --spec auth-system.spec.md --context "Use JWT tokens with RS256 signing"
```

### Code Review

```bash
# Review a single file
rice review src/main.rs

# Review multiple files
rice review src/lib.rs src/main.rs

# Review with specific focus areas
rice review --focus security,performance src/auth.rs

# Review an entire directory
rice review src/

# Generate review report
rice review --output review-report.md src/
```

### Project Analysis

```bash
# Analyze current project structure
rice analyze

# Analyze with security focus
rice analyze --focus security

# Analyze dependencies
rice analyze --deps

# Generate comprehensive project report
rice analyze --report project-analysis.md
```

## Advanced CLI Usage

### Configuration Management

```bash
# View current configuration
rice config show

# Edit configuration interactively
rice config edit

# Validate configuration
rice config validate

# Reset to defaults
rice config reset
```

### Session Management

```bash
# List all sessions
rice session list

# Switch to a specific session
rice session switch abc-123-def

# Export session data
rice session export my-session --output session-backup.json

# Import session data
rice session import session-backup.json

# Clean up old sessions
rice session cleanup --older-than 30d
```

### Provider Management

```bash
# List available providers
rice provider list

# Configure a provider
rice provider config openai --api-key sk-your-key-here

# Test provider connection
rice provider test openai

# Set default provider
rice provider default openai

# Monitor provider performance
rice provider monitor
```

### Batch Operations

```bash
# Process multiple specification files
rice gen --spec specs/*.md --batch

# Review multiple files with custom criteria
rice review --criteria "security,maintainability" --files "src/**/*.rs"

# Run analysis on multiple projects
rice analyze --projects project1,project2,project3
```

## Scripting Examples

### Automated Code Review (Shell Script)

```bash
#!/bin/bash
# automated-review.sh

# Get list of changed files
CHANGED_FILES=$(git diff --name-only HEAD~1)

# Run RiceCoder review on changed files
echo "$CHANGED_FILES" | rice review --stdin --output review-report.md

# Check if review found issues
if grep -q "CRITICAL\|HIGH" review-report.md; then
    echo "Critical issues found in code review"
    exit 1
fi
```

### CI/CD Integration (GitHub Actions)

```yaml
# .github/workflows/code-review.yml
name: Code Review
on: [pull_request]

jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install RiceCoder
        run: curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install | bash
      - name: Run Code Review
        run: rice review --pr ${{ github.event.pull_request.number }} --output review.md
      - name: Comment PR
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const review = fs.readFileSync('review.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: review
            });
```

### Project Analysis Report (Shell Script)

```bash
#!/bin/bash
# project-analysis.sh

PROJECT_NAME=$(basename $(pwd))
OUTPUT_DIR="reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create reports directory
mkdir -p "$OUTPUT_DIR"

# Run comprehensive analysis
rice analyze --report "$OUTPUT_DIR/${PROJECT_NAME}_analysis_$TIMESTAMP.md"

# Generate security report
rice analyze --focus security --report "$OUTPUT_DIR/${PROJECT_NAME}_security_$TIMESTAMP.md"

# Generate dependency analysis
rice analyze --deps --report "$OUTPUT_DIR/${PROJECT_NAME}_deps_$TIMESTAMP.md"

echo "Analysis reports generated in $OUTPUT_DIR/"
```

## Error Handling

### Common Error Scenarios

```bash
# Handle provider authentication errors
rice provider test openai || echo "OpenAI API key may be invalid"

# Retry failed operations
rice gen --spec my-spec.md --retry 3

# Debug mode for troubleshooting
RUST_LOG=debug rice chat

# Check system health
rice doctor
```

### Logging and Debugging

```bash
# Enable verbose logging
export RUST_LOG=info
rice chat

# Log to file
rice chat 2>&1 | tee ricecoder.log

# Debug specific components
RUST_LOG=ricecoder::mcp=debug rice chat

# Profile performance
rice profile chat --duration 30s --output profile.json
```

## Best Practices

### Development Workflow

```bash
# 1. Initialize project
rice init

# 2. Configure providers
rice provider config openai --api-key $OPENAI_API_KEY
rice provider config anthropic --api-key $ANTHROPIC_API_KEY

# 3. Start development session
rice chat --provider openai --model gpt-4

# 4. Generate code from specs
rice gen --spec features/user-auth.spec.md --apply

# 5. Review generated code
rice review src/auth/

# 6. Run tests and analysis
rice analyze --focus testing
```

### Team Collaboration

```bash
# Share session with team
rice session share --team my-team --read-only

# Review team contributions
rice review --team my-team --since yesterday

# Generate team reports
rice report team-activity --output team-report.md
```

This collection demonstrates the versatility of RiceCoder's CLI for various development scenarios. Each example can be adapted to your specific workflow needs.