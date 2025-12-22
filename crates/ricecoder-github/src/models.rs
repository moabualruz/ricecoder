//! GitHub Data Models

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Pull Request status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrStatus {
    /// Draft PR
    Draft,
    /// Open PR
    Open,
    /// Merged PR
    Merged,
    /// Closed PR
    Closed,
}

/// Issue status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueStatus {
    /// Open issue
    Open,
    /// In progress
    InProgress,
    /// Closed issue
    Closed,
}

/// File change in a PR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// File path
    pub path: String,
    /// Change type (added, modified, deleted)
    pub change_type: String,
    /// Number of additions
    pub additions: u32,
    /// Number of deletions
    pub deletions: u32,
}

/// Pull Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// PR ID
    pub id: u64,
    /// PR number
    pub number: u32,
    /// PR title
    pub title: String,
    /// PR body/description
    pub body: String,
    /// Branch name
    pub branch: String,
    /// Base branch
    pub base: String,
    /// PR status
    pub status: PrStatus,
    /// Files changed
    pub files: Vec<FileChange>,
    /// Created at timestamp
    pub created_at: DateTime<Utc>,
    /// Updated at timestamp
    pub updated_at: DateTime<Utc>,
}

/// Issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue ID
    pub id: u64,
    /// Issue number
    pub number: u32,
    /// Issue title
    pub title: String,
    /// Issue body/description
    pub body: String,
    /// Labels
    pub labels: Vec<String>,
    /// Assignees
    pub assignees: Vec<String>,
    /// Issue status
    pub status: IssueStatus,
    /// Created at timestamp
    pub created_at: DateTime<Utc>,
    /// Updated at timestamp
    pub updated_at: DateTime<Utc>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Dependency name
    pub name: String,
    /// Current version
    pub version: String,
    /// Latest available version
    pub latest_version: Option<String>,
    /// Is outdated
    pub is_outdated: bool,
}

/// Project structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    /// Main language
    pub language: Option<String>,
    /// Project type (library, binary, etc.)
    pub project_type: String,
    /// Key directories
    pub directories: Vec<String>,
    /// Key files
    pub files: Vec<String>,
}

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository name
    pub name: String,
    /// Repository owner
    pub owner: String,
    /// Repository description
    pub description: String,
    /// Repository URL
    pub url: String,
    /// Primary language
    pub language: Option<String>,
    /// Dependencies
    pub dependencies: Vec<Dependency>,
    /// Project structure
    pub structure: ProjectStructure,
}

/// Project card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCard {
    /// Card ID
    pub id: u64,
    /// Content ID (issue or PR)
    pub content_id: u64,
    /// Content type (Issue or PullRequest)
    pub content_type: String,
    /// Column ID
    pub column_id: u64,
    /// Optional note
    pub note: Option<String>,
}

/// Release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// Release ID
    pub id: u64,
    /// Tag name
    pub tag_name: String,
    /// Release name
    pub name: String,
    /// Release notes
    pub body: String,
    /// Is draft
    pub draft: bool,
    /// Is prerelease
    pub prerelease: bool,
    /// Created at timestamp
    pub created_at: DateTime<Utc>,
}

/// Gist file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistFile {
    /// File name
    pub filename: String,
    /// File content
    pub content: String,
    /// File language
    pub language: Option<String>,
}

/// GitHub Gist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gist {
    /// Gist ID
    pub id: String,
    /// Gist URL
    pub url: String,
    /// Files in gist
    pub files: HashMap<String, GistFile>,
    /// Gist description
    pub description: String,
    /// Is public
    pub public: bool,
}

/// Discussion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discussion {
    /// Discussion ID
    pub id: u64,
    /// Discussion number
    pub number: u32,
    /// Discussion title
    pub title: String,
    /// Discussion body
    pub body: String,
    /// Category
    pub category: String,
    /// Created at timestamp
    pub created_at: DateTime<Utc>,
    /// Updated at timestamp
    pub updated_at: DateTime<Utc>,
}

/// Progress update for an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueProgressUpdate {
    /// Issue number
    pub issue_number: u32,
    /// Update message
    pub message: String,
    /// Current status
    pub status: IssueStatus,
    /// Percentage complete (0-100)
    pub progress_percentage: u32,
}
