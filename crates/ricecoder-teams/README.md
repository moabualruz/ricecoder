# ricecoder-teams

**Purpose**: Team collaboration system providing shared standards, rule promotion, access control, and organizational workflow management for RiceCoder

## Overview

`ricecoder-teams` enables collaborative development workflows with shared standards, rule promotion, and access control across organizational, team, and project levels. It provides inheritance and override capabilities for configurations, code review rules, templates, and compliance requirements, fostering consistency while allowing flexibility.

## Features

- **Shared Standards**: Organization-wide code review rules and development standards
- **Hierarchical Configuration**: Organization → Team → Project inheritance with overrides
- **Access Control**: Role-based permissions and audit logging
- **Rule Promotion**: Automated rule sharing and adoption tracking
- **Template Management**: Shared project templates and boilerplates
- **Compliance Tracking**: Automated compliance monitoring and reporting
- **Analytics Dashboard**: Team productivity and standards adoption metrics
- **Synchronization**: Cross-team configuration synchronization and updates

## Architecture

### Responsibilities
- Team hierarchy management and configuration inheritance
- Shared standards definition and enforcement
- Access control and permission management
- Rule promotion and adoption analytics
- Configuration synchronization across teams
- Audit logging and compliance tracking
- Template sharing and version management

### Dependencies
- **Storage**: `ricecoder-storage` for configuration persistence
- **Learning**: `ricecoder-learning` for rule analytics and promotion
- **Permissions**: Access control integration
- **Async Runtime**: `tokio` for concurrent operations

### Integration Points
- **Storage**: Persists team configurations and shared resources
- **Learning**: Tracks rule adoption and provides analytics
- **Sessions**: Applies team-specific configurations to sessions
- **TUI**: Provides team management interfaces
- **Commands**: Team administration and configuration commands

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-teams = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_teams::{TeamManager, TeamConfigManager};

// Create team manager
let team_manager = TeamManager::new().await?;

// Create a new team
let team = team_manager.create_team("backend-team", "Backend Development Team").await?;

// Add team members
team_manager.add_member(&team.id, "user@example.com", TeamRole::Developer).await?;
```

### Shared Standards Management

```rust
use ricecoder_teams::{SharedRulesManager, CodeReviewRule};

// Create shared rules manager
let rules_manager = SharedRulesManager::new();

// Define organization-wide rule
let rule = CodeReviewRule {
    name: "no-print-debug".to_string(),
    description: "Disallow print statements in production code".to_string(),
    pattern: r"println!\(".to_string(),
    scope: RuleScope::Organization,
    ..Default::default()
};

// Share rule across teams
rules_manager.share_rule(rule, &organization_id).await?;
```

### Configuration Inheritance

```rust
use ricecoder_teams::TeamConfigManager;

// Create config manager
let config_manager = TeamConfigManager::new();

// Load configuration with inheritance
let effective_config = config_manager.load_effective_config(&project_id).await?;

// Configuration is merged: Organization → Team → Project
println!("Effective theme: {}", effective_config.ui.theme);
```

## Configuration

Team configuration is managed hierarchically via YAML:

```yaml
teams:
  # Organization-level settings
  organization:
    name: "MyCompany"
    standards:
      code_review:
        require_approvals: 2
        max_line_length: 120
      testing:
        coverage_minimum: 0.8
        require_integration_tests: true

  # Team-level overrides
  teams:
    backend:
      name: "Backend Team"
      standards:
        code_review:
          require_approvals: 1  # Override org setting
        languages: ["rust", "go"]

    frontend:
      name: "Frontend Team"
      standards:
        code_review:
          require_approvals: 3  # Stricter than org
        languages: ["typescript", "react"]

  # Project-level specific settings
  projects:
    api-server:
      team: "backend"
      standards:
        testing:
          coverage_minimum: 0.9  # Stricter than team
```

## API Reference

### Key Types

- **`TeamManager`**: Central team lifecycle and membership management
- **`SharedRulesManager`**: Manages shared code review rules and standards
- **`TeamConfigManager`**: Handles hierarchical configuration inheritance
- **`AccessControlManager`**: Role-based access control and permissions
- **`AnalyticsDashboard`**: Team metrics and standards adoption tracking

### Key Functions

- **`create_team()`**: Create new teams with configuration
- **`share_rule()`**: Share code review rules across teams
- **`load_effective_config()`**: Load configuration with inheritance resolution
- **`check_access()`**: Verify user permissions for operations

## Error Handling

```rust
use ricecoder_teams::TeamError;

match team_manager.create_team(name, description).await {
    Ok(team) => println!("Team created: {}", team.name),
    Err(TeamError::DuplicateTeamName) => eprintln!("Team name already exists"),
    Err(TeamError::InsufficientPermissions) => eprintln!("Insufficient permissions"),
    Err(TeamError::ConfigInheritanceError(msg)) => eprintln!("Configuration error: {}", msg),
}
```

## Testing

Run comprehensive team collaboration tests:

```bash
# Run all tests
cargo test -p ricecoder-teams

# Run property tests for configuration inheritance
cargo test -p ricecoder-teams property

# Test access control scenarios
cargo test -p ricecoder-teams access

# Test rule sharing functionality
cargo test -p ricecoder-teams rules
```

Key test areas:
- Configuration inheritance correctness
- Access control enforcement
- Rule sharing and adoption
- Team hierarchy management
- Audit logging accuracy

## Performance

- **Configuration Loading**: < 50ms with inheritance resolution
- **Access Checks**: < 10ms per permission verification
- **Rule Evaluation**: < 5ms per rule check
- **Team Operations**: < 20ms for typical team management tasks
- **Analytics Generation**: < 100ms for dashboard data

## Contributing

When working with `ricecoder-teams`:

1. **Security First**: Implement comprehensive access control and audit logging
2. **Inheritance Clarity**: Ensure configuration inheritance is predictable and documented
3. **Performance**: Optimize for large organizations with many teams
4. **Testing**: Test inheritance scenarios and permission edge cases thoroughly
5. **Documentation**: Keep configuration schemas and inheritance rules clear

## License

MIT