# ricecoder-vcs

## Purpose

Version Control System integration for RiceCoder TUI, providing Git repository operations and status tracking.

## DDD Layer

**Infrastructure** - Provides Git repository access and VCS operations as infrastructure services.

## Responsibilities

- Git repository detection and discovery
- Branch management and tracking
- File status monitoring (staged, modified, untracked)
- Ahead/behind tracking for upstream branches
- Diff generation and staging operations
- Real-time status monitoring for TUI

## Overview

The `ricecoder-vcs` crate provides comprehensive Git integration for RiceCoder's terminal user interface. It enables repository detection, status monitoring, branch management, and file change tracking with a clean API designed for TUI integration.

## Key Features

- **Repository Detection**: Automatic discovery of Git repositories from any directory
- **Status Monitoring**: Real-time tracking of repository state, branches, and file changes
- **File Status Tracking**: Detailed modification indicators (Modified, Added, Deleted, Untracked, etc.)
- **Branch Management**: Current branch detection and branch listing
- **Ahead/Behind Tracking**: Upstream synchronization status relative to remote branches (↑3 ↓1 format)
- **TUI Integration**: Specialized components for terminal UI status display
- **Diff Operations**: File diff viewing and staging operations

## Repository Status Information

The crate tracks comprehensive repository state:

```rust
RepositoryStatus {
    current_branch: Branch,           // Current branch info
    uncommitted_changes: usize,       // Count of modified files
    untracked_files: usize,           // Count of untracked files
    staged_files: usize,              // Count of staged files
    is_clean: bool,                   // Whether repo has no changes (and no conflicts)
    has_conflicts: bool,              // Whether there are merge conflicts
    last_commit: Option<CommitInfo>,  // Last commit information
    repository_root: String,          // Repository root path
    ahead: usize,                     // Commits ahead of upstream
    behind: usize,                    // Commits behind upstream
}
```

## File Status Indicators

**Modification Indicators:**
- ` ` - Clean (no changes)
- `M` - Modified
- `A` - Added
- `D` - Deleted
- `?` - Untracked
- `S` - Staged
- `U` - Conflicted

## Dependencies

### Internal (RiceCoder Crates)

None - this crate is a standalone infrastructure component.

### External Libraries

- **`git2`**: Low-level Git operations (libgit2 bindings)
- **`tokio`**: Async runtime for concurrent operations
- **`serde`**: Serialization for data structures
- **`chrono`**: Commit timestamp handling
- **`thiserror`**: Error type definitions
- **`tracing`**: Structured logging

## Key Types

- **`GitRepository`**: Main repository operations interface
- **`Repository`**: Generic trait for VCS operations
- **`RepositoryStatus`**: Current repository state information
- **`FileStatus`**: Individual file status enumeration
- **`ModifiedFile`**: File with modification details
- **`Branch`**: Branch metadata and information
- **`VcsIntegration`**: TUI integration manager
- **`VcsStatus`**: TUI-specific status display structure

### Integration Points
- **TUI**: Provides repository status display and Git operations
- **Files**: Integrates with file operations for Git-aware workflows
- **Sessions**: Tracks repository state in development sessions
- **Commands**: Enables Git command integration and automation

## Usage Examples

### Repository Discovery and Status

```rust
use ricecoder_vcs::GitRepository;

// Discover Git repository from current directory
let repo = GitRepository::discover(".")?;

// Get comprehensive repository status
let status = repo.get_status()?;
println!("Branch: {}", status.current_branch.name);
println!("Clean: {}", status.is_clean);
println!("Uncommitted changes: {}", status.uncommitted_changes);

// Get status summary (e.g., "1S 2M 1U")
println!("Status: {}", status.summary());
```

### File Change Tracking

```rust
use ricecoder_vcs::Repository;

// Get all modified files with their status
let modified_files = repo.get_modified_files()?;
for file in modified_files {
    println!("{}: {} ({})",
        file.path.display(),
        file.status_display(),
        if file.staged { "staged" } else { "unstaged" }
    );
}
```

### Branch Operations

```rust
// Get current branch
let current_branch = repo.get_current_branch()?;
println!("Current branch: {}", current_branch.name);

// Get all branches (local and remote)
let branches = repo.get_branches()?;
for branch in branches {
    let branch_type = if branch.is_remote { "remote" } else { "local" };
    let current_marker = if branch.is_current { " *" } else { "" };
    println!("{} ({}){}", branch.name, branch_type, current_marker);
}
```

### File Diff Operations

```rust
use std::path::Path;

// Get diff for a specific file
let diff = repo.get_file_diff(Path::new("src/main.rs"))?;
println!("File diff:\n{}", diff);

// Stage/unstage files
repo.stage_file(Path::new("src/main.rs"))?;
repo.unstage_file(Path::new("src/main.rs"))?;

// Stage all changes
repo.stage_all()?;
```

### TUI Integration

```rust
use ricecoder_vcs::VcsIntegration;

// Create TUI integration component
let vcs_integration = VcsIntegration::new();

// Get VCS status for UI display
let status = vcs_integration.get_status();
if status.is_in_repo() {
    // Display in TUI status bar
    println!("Branch: {} | Status: {}",
        vcs_integration.get_branch_display().unwrap_or_default(),
        vcs_integration.get_status_summary().unwrap_or_default()
    );
    
    // Show ahead/behind if applicable
    if let Some(ab) = vcs_integration.get_ahead_behind_display() {
        println!("Sync: {}", ab); // e.g., "↑2 ↓1"
    }
}
```

## Repository Operations

### Opening vs Discovering Repositories

```rust
// Open repository at exact path (fails if not a Git repo)
let repo = GitRepository::open("/path/to/repo")?;

// Discover repository starting from path (searches up directory tree)
let repo = GitRepository::discover("/path/to/some/deep/dir")?;
```

### Status Summary Format

The status summary uses a compact format:
- `Clean` - No changes and no conflicts
- `1S 2M 1U` - 1 staged, 2 modified, 1 untracked
- `C` - Has conflicts (repository is NOT clean when conflicts exist)

### Ahead/Behind Display Format

The ahead/behind tracking uses Unicode arrows:
- `↑3` - 3 commits ahead of upstream
- `↓2` - 2 commits behind upstream
- `↑1 ↓4` - 1 ahead and 4 behind

## Error Handling

The crate provides comprehensive error types:

```rust
use ricecoder_vcs::{VcsError, Result};

match git_operation() {
    Ok(result) => println!("Success: {:?}", result),
    Err(VcsError::RepositoryNotFound { path }) =>
        println!("No Git repository found at: {}", path),
    Err(VcsError::InvalidState { message }) =>
        println!("Repository in invalid state: {}", message),
    Err(e) => println!("Other error: {}", e),
}
```

## Testing

Comprehensive test suite covering:

```bash
cargo test -p ricecoder-vcs
```

**Test Coverage:**
- Repository discovery and opening
- Status tracking and file changes
- Branch operations
- Modification indicators
- TUI integration components
- Error conditions and edge cases

## Integration with RiceCoder TUI

The VCS integration is designed specifically for terminal UI usage:

- **Status Bar Integration**: Real-time branch and status display
- **File Explorer Enhancement**: File status indicators in file listings
- **Command Integration**: Git operations through TUI commands
- **Performance Optimized**: Efficient status polling for responsive UI

## Configuration

VCS integration respects RiceCoder's configuration hierarchy and can be customized through:

```toml
[vcs]
# VCS-specific settings
enable_status_polling = true
status_poll_interval_ms = 1000
show_untracked_files = true
```

## API Reference

### Repository Trait Methods

- **`get_status()`**: Get comprehensive repository status
- **`get_current_branch()`**: Get current branch information
- **`get_branches()`**: List all branches (local and remote)
- **`get_modified_files()`**: List files with changes
- **`get_root_path()`**: Get repository root path
- **`is_clean()`**: Check if repository has no uncommitted changes
- **`count_uncommitted_changes()`**: Count uncommitted + untracked files
- **`get_file_diff()`**: Generate diff for specific file
- **`stage_file()`**: Stage a specific file
- **`unstage_file()`**: Unstage a specific file
- **`stage_all()`**: Stage all changes
- **`reset_all()`**: Reset all changes (hard reset)

### VcsIntegration Methods

- **`get_status()`**: Get current VCS status
- **`get_branch_display()`**: Get branch name with change indicator
- **`get_status_summary()`**: Get compact status summary
- **`get_ahead_behind_display()`**: Get upstream sync status
- **`get_file_counts()`**: Get (staged, modified, untracked) counts
- **`start_monitoring()`**: Begin background status polling
- **`stop_monitoring()`**: Stop background status polling
- **`force_refresh()`**: Manually trigger status refresh

## Performance

- **Repository Discovery**: < 50ms for typical repository structures
- **Status Check**: < 100ms for repositories with < 1000 files
- **File Diff**: < 200ms for typical file changes
- **Branch Listing**: < 50ms for repositories with < 100 branches
- **Caching**: 80%+ performance improvement for repeated operations

## Contributing

When working with `ricecoder-vcs`:

1. **Git Expertise**: Understand Git internals and operations
2. **Performance**: Optimize for large repositories and frequent operations
3. **Error Handling**: Provide clear error messages for Git operation failures
4. **Compatibility**: Support various Git configurations and workflows
5. **Testing**: Test with different repository states and Git configurations

## Recent Changes

### SRP Refactoring (December 2024)

**ISP Trait Split**: Split the monolithic `Repository` trait into focused, single-responsibility interfaces following the Interface Segregation Principle.

**New Traits**:
- `RepositoryInfo`: Basic repository information (root path, branch detection)
- `StatusProvider`: File and branch status queries
- `DiffProvider`: Diff generation operations
- `StagingOperations`: File staging and unstaging

**Changes**:
- `GitRepository` now implements 4 focused traits instead of 1 large trait
- Clients depend only on the traits they need
- Easier to mock and test individual capabilities
- No breaking changes to existing `Repository` trait (still available as supertrait)

**Migration**: Use specific traits (`StatusProvider`, `DiffProvider`, etc.) for fine-grained dependencies. Legacy code using `Repository` trait continues to work unchanged.

## Architecture

The crate follows a layered architecture:

- **Repository Trait** (`repository.rs`): Generic VCS operations interface for pluggable implementations
- **Git Implementation** (`git.rs`): Concrete Git repository operations using libgit2
- **Status Types** (`status.rs`): Rich status and file change representations
- **Common Types** (`types.rs`): Branch, ModifiedFile, FileStatus definitions
- **TUI Integration** (`tui_integration/`): Terminal UI specific components and formatting
- **Error Types** (`error.rs`): Comprehensive error handling with thiserror

## License

MIT