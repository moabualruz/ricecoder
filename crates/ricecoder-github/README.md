# ricecoder-github

**Purpose**: Comprehensive GitHub API integration providing repository operations, PR management, issue tracking, and automation for RiceCoder

## DDD Layer

**Infrastructure** - External service integration for GitHub API operations

## Overview

`ricecoder-github` provides extensive GitHub API integration enabling seamless repository operations, pull request management, issue tracking, project coordination, and automation workflows. It serves as the primary interface for GitHub operations within RiceCoder, supporting everything from basic repository analysis to complex automation workflows.

## Features

- **PR Management**: Automatic PR creation, updates, reviews, and merging
- **Issue Tracking**: Issue assignment, progress tracking, and lifecycle management
- **Repository Analysis**: Codebase analysis, dependency management, and insights
- **Project Management**: Project boards, cards, columns, and workflow automation
- **Documentation Generation**: Automated README and documentation updates
- **Gist Management**: Gist creation, organization, and sharing
- **Discussion Integration**: Repository discussions and community engagement
- **Release Management**: Automated releases, changelogs, and versioning
- **Code Review Automation**: Automated code review suggestions and standards enforcement
- **Webhook Integration**: Real-time GitHub event processing and automation
- **CI/CD Integration**: Build status monitoring and automated responses

## Architecture

### Responsibilities
- GitHub API communication and authentication
- Repository data synchronization and caching
- PR/issue lifecycle management and automation
- Project board coordination and status tracking
- Documentation generation and maintenance
- Webhook event processing and response automation
- Rate limiting and error handling for API operations

### Dependencies
- **HTTP Client**: `reqwest` for API communication
- **Async Runtime**: `tokio` for concurrent operations
- **Serialization**: `serde` for JSON handling
- **Authentication**: GitHub token management
- **Storage**: `ricecoder-storage` for caching and persistence

### Integration Points
- **Commands**: CLI commands for GitHub operations
- **TUI**: GitHub status display and interaction
- **Sessions**: GitHub operation tracking in conversations
- **Workflows**: GitHub operations in automated workflows
- **Storage**: GitHub data caching and persistence

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-github = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_github::{GitHubManager, GitHubConfig};

// Create GitHub manager
let config = GitHubConfig::new("your-token".to_string());
let manager = GitHubManager::new(config).await?;

// Get repository information
let repo = manager.get_repository("owner", "repo").await?;
println!("Repository: {}", repo.name);
```

### PR Management

```rust
use ricecoder_github::managers::{PrManager, PrOptions};

// Create PR manager
let pr_manager = PrManager::new(manager);

// Create a pull request
let options = PrOptions {
    title: "Feature implementation".to_string(),
    body: "Implements new feature...".to_string(),
    head: "feature-branch".to_string(),
    base: "main".to_string(),
    ..Default::default()
};

let pr = pr_manager.create_pr("owner", "repo", options).await?;
```

### Issue Tracking

```rust
use ricecoder_github::managers::IssueManager;

// Create issue manager
let issue_manager = IssueManager::new(manager);

// Create and assign issue
let issue = issue_manager.create_issue("owner", "repo", "Bug title", "Bug description").await?;
issue_manager.assign_issue(&issue.number, "assignee").await?;
```

### Repository Analysis

```rust
use ricecoder_github::managers::RepositoryAnalyzer;

// Create analyzer
let analyzer = RepositoryAnalyzer::new(manager);

// Analyze repository
let analysis = analyzer.analyze_repository("owner", "repo").await?;
println!("Languages: {:?}", analysis.languages);
println!("Contributors: {}", analysis.contributors.len());
```

## Configuration

GitHub integration is configured via YAML:

```yaml
github:
  # Authentication
  token: "${GITHUB_TOKEN}"
  app_id: "${GITHUB_APP_ID}"
  private_key: "${GITHUB_PRIVATE_KEY}"

  # Repository settings
  default_owner: "myorg"
  default_repo: "myproject"

  # Automation settings
  auto_assign_prs: true
  auto_label_issues: true
  auto_merge_enabled: false

  # Webhook settings
  webhooks:
    enabled: true
    secret: "${GITHUB_WEBHOOK_SECRET}"
    events: ["pull_request", "issues", "release"]

  # Rate limiting
  rate_limit:
    requests_per_hour: 5000
    burst_limit: 100
```

## API Reference

### Key Types

- **`GitHubManager`**: Main GitHub API coordinator
- **`PrManager`**: Pull request operations and management
- **`IssueManager`**: Issue tracking and lifecycle management
- **`RepositoryAnalyzer`**: Repository analysis and insights
- **`ProjectManager`**: Project board coordination

### Key Functions

- **`create_pr()`**: Create pull requests with options
- **`get_repository()`**: Retrieve repository information
- **`analyze_repository()`**: Perform comprehensive repository analysis
- **`create_issue()`**: Create and manage issues
- **`process_webhook()`**: Handle incoming webhook events

## Error Handling

```rust
use ricecoder_github::GitHubError;

match manager.get_repository(owner, repo).await {
    Ok(repository) => println!("Repository: {}", repository.name),
    Err(GitHubError::AuthenticationFailed) => eprintln!("Invalid GitHub token"),
    Err(GitHubError::RateLimitExceeded) => eprintln!("API rate limit exceeded"),
    Err(GitHubError::RepositoryNotFound) => eprintln!("Repository not found"),
    Err(GitHubError::NetworkError(msg)) => eprintln!("Network error: {}", msg),
}
```

## Testing

Run comprehensive GitHub integration tests:

```bash
# Run all tests (600+ tests)
cargo test -p ricecoder-github

# Run property tests for API behavior
cargo test -p ricecoder-github property

# Test webhook processing
cargo test -p ricecoder-github webhook

# Test PR operations (requires token)
cargo test -p ricecoder-github pr -- --ignored
```

**Test Organization**:
- **Unit Tests**: Located in `tests/` directory (31 test files)
- **Property Tests**: Located in `tests/*_property_tests.rs` files
- **Integration Tests**: Located in `tests/integration_*.rs` files
- **Inline Module Tests**: `GitHubManager` has inline tests for core config validation (documented exception for cohesion)

Key test areas:
- API authentication and error handling
- Webhook event processing
- Repository analysis accuracy
- Rate limiting behavior
- Concurrent operation safety

## Performance

- **API Calls**: Efficient batching and caching (< 100ms cached responses)
- **Repository Analysis**: < 2s for typical repositories
- **Webhook Processing**: < 50ms per event
- **Concurrent Operations**: Safe for multiple simultaneous API calls
- **Rate Limit Management**: Automatic backoff and retry logic

## Contributing

When working with `ricecoder-github`:

1. **API Compliance**: Ensure all operations follow GitHub API best practices
2. **Rate Limiting**: Implement proper rate limit handling and backoff
3. **Error Handling**: Provide clear error messages for common failure scenarios
4. **Testing**: Use integration tests for API-dependent functionality
5. **Documentation**: Keep API usage examples current with GitHub API changes

## License

MIT