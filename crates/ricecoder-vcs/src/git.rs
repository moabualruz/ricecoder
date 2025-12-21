//! Git repository implementation

use crate::error::{Result, VcsError};
use crate::repository::Repository;
use crate::status::{CommitInfo, RepositoryStatus};
use crate::types::{Branch, FileStatus, ModifiedFile};
use chrono::{TimeZone, Utc};
use git2::{BranchType, Repository as Git2Repository, Status, StatusOptions};
use std::path::{Path, PathBuf};
use tracing::{debug, trace};

/// Git repository implementation
pub struct GitRepository {
    /// The underlying git2 repository
    repo: Git2Repository,
    /// Repository root path
    root_path: PathBuf,
}

impl GitRepository {
    /// Open a Git repository at the specified path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        debug!("Opening Git repository at: {}", path.display());

        let repo = Git2Repository::open(path).map_err(|e| {
            debug!("Failed to open repository: {}", e);
            VcsError::RepositoryNotFound {
                path: path.display().to_string(),
            }
        })?;

        let root_path = repo
            .workdir()
            .ok_or_else(|| VcsError::InvalidState {
                message: "Repository has no working directory".to_string(),
            })?
            .to_path_buf();

        debug!(
            "Successfully opened Git repository at: {}",
            root_path.display()
        );

        Ok(Self { repo, root_path })
    }

    /// Discover a Git repository starting from the given path
    pub fn discover<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        debug!("Discovering Git repository from: {}", path.display());

        let repo = Git2Repository::discover(path).map_err(|e| {
            debug!("Failed to discover repository: {}", e);
            VcsError::RepositoryNotFound {
                path: path.display().to_string(),
            }
        })?;
        let root_path = repo
            .workdir()
            .ok_or_else(|| VcsError::InvalidState {
                message: "Repository has no working directory".to_string(),
            })?
            .to_path_buf();

        debug!("Discovered Git repository at: {}", root_path.display());

        Ok(Self { repo, root_path })
    }

    /// Check if a directory contains a Git repository
    pub fn is_git_repository<P: AsRef<Path>>(path: P) -> bool {
        Git2Repository::discover(path).is_ok()
    }

    /// Get the last commit information
    fn get_last_commit(&self) -> Result<Option<CommitInfo>> {
        let head = match self.repo.head() {
            Ok(head) => head,
            Err(_) => {
                debug!("No HEAD found, repository might be empty");
                return Ok(None);
            }
        };

        let commit = head.peel_to_commit()?;
        let hash = commit.id().to_string();
        let short_hash = &hash[..7];

        let message = commit
            .message()
            .unwrap_or("No commit message")
            .lines()
            .next()
            .unwrap_or("No commit message")
            .to_string();

        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();

        let timestamp = Utc
            .timestamp_opt(commit.time().seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        Ok(Some(CommitInfo::new(
            short_hash,
            message,
            author_name,
            timestamp,
        )))
    }

    /// Get file status from git2 status
    fn get_file_statuses(&self) -> Result<Vec<(PathBuf, Status)>> {
        let mut status_options = StatusOptions::new();
        status_options.include_untracked(true);
        status_options.include_ignored(false);

        let statuses = self.repo.statuses(Some(&mut status_options))?;
        let mut files = Vec::new();

        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                let path_buf = PathBuf::from(path);
                files.push((path_buf, entry.status()));
            }
        }

        trace!("Found {} files with status changes", files.len());
        Ok(files)
    }
}

impl Repository for GitRepository {
    fn get_status(&self) -> Result<RepositoryStatus> {
        debug!("Getting repository status");

        let current_branch = self.get_current_branch()?;
        let file_statuses = self.get_file_statuses()?;

        let mut uncommitted = 0;
        let mut untracked = 0;
        let mut staged = 0;
        let mut has_conflicts = false;

        for (_, status) in &file_statuses {
            if status.contains(Status::CONFLICTED) {
                has_conflicts = true;
            }

            if status.contains(Status::INDEX_MODIFIED)
                || status.contains(Status::INDEX_NEW)
                || status.contains(Status::INDEX_DELETED)
                || status.contains(Status::INDEX_RENAMED)
            {
                staged += 1;
            }

            if status.contains(Status::WT_MODIFIED)
                || status.contains(Status::WT_DELETED)
                || status.contains(Status::WT_RENAMED)
            {
                uncommitted += 1;
            }

            if status.contains(Status::WT_NEW) {
                untracked += 1;
            }
        }

        let mut repo_status =
            RepositoryStatus::new(current_branch, self.root_path.display().to_string())
                .with_counts(uncommitted, untracked, staged, has_conflicts);

        if let Ok(Some(last_commit)) = self.get_last_commit() {
            repo_status = repo_status.with_last_commit(last_commit);
        }

        debug!(
            "Repository status: {} uncommitted, {} untracked, {} staged, conflicts: {}",
            uncommitted, untracked, staged, has_conflicts
        );

        Ok(repo_status)
    }

    fn get_current_branch(&self) -> Result<Branch> {
        debug!("Getting current branch");

        let head = self.repo.head()?;

        if !head.is_branch() {
            // Detached HEAD
            let commit = head.peel_to_commit()?;
            let hash = commit.id().to_string();
            let short_hash = &hash[..7];

            return Ok(Branch::new(format!("HEAD detached at {}", short_hash)).current());
        }

        let branch_name = head
            .shorthand()
            .ok_or_else(|| VcsError::InvalidState {
                message: "Could not get branch name".to_string(),
            })?
            .to_string();

        let mut branch = Branch::new(branch_name).current();

        // Add commit information if available
        if let Ok(Some(commit_info)) = self.get_last_commit() {
            branch =
                branch.with_commit(commit_info.hash, commit_info.message, commit_info.timestamp);
        }

        debug!("Current branch: {}", branch.name);
        Ok(branch)
    }

    fn get_branches(&self) -> Result<Vec<Branch>> {
        debug!("Getting all branches");

        let mut branches = Vec::new();
        let current_branch_name = self.get_current_branch()?.name;

        // Get local branches
        let local_branches = self.repo.branches(Some(BranchType::Local))?;
        for branch_result in local_branches {
            let (branch, _) = branch_result?;
            if let Some(name) = branch.name()? {
                let mut branch_obj = Branch::new(name.to_string());
                if name == current_branch_name {
                    branch_obj = branch_obj.current();
                }

                // Try to get commit info
                if let Ok(commit) = branch.get().peel_to_commit() {
                    let hash = commit.id().to_string();
                    let short_hash = &hash[..7];
                    let message = commit
                        .message()
                        .unwrap_or("No commit message")
                        .lines()
                        .next()
                        .unwrap_or("No commit message")
                        .to_string();
                    let timestamp = Utc
                        .timestamp_opt(commit.time().seconds(), 0)
                        .single()
                        .unwrap_or_else(Utc::now);

                    branch_obj = branch_obj.with_commit(short_hash, message, timestamp);
                }

                branches.push(branch_obj);
            }
        }

        debug!("Found {} branches", branches.len());
        Ok(branches)
    }

    fn get_modified_files(&self) -> Result<Vec<ModifiedFile>> {
        debug!("Getting modified files");

        let file_statuses = self.get_file_statuses()?;
        let mut modified_files = Vec::new();

        for (path, status) in file_statuses {
            let file_status = FileStatus::from_git2_status(status);
            let staged = status.contains(Status::INDEX_MODIFIED)
                || status.contains(Status::INDEX_NEW)
                || status.contains(Status::INDEX_DELETED)
                || status.contains(Status::INDEX_RENAMED);

            let mut modified_file = ModifiedFile::new(path, file_status);
            if staged {
                modified_file = modified_file.staged();
            }

            modified_files.push(modified_file);
        }

        debug!("Found {} modified files", modified_files.len());
        Ok(modified_files)
    }

    fn get_root_path(&self) -> Result<String> {
        Ok(self.root_path.display().to_string())
    }

    fn is_clean(&self) -> Result<bool> {
        let status = self.get_status()?;
        Ok(status.is_clean)
    }

    fn count_uncommitted_changes(&self) -> Result<usize> {
        let status = self.get_status()?;
        Ok(status.uncommitted_changes + status.untracked_files)
    }

    fn get_file_diff(&self, file_path: &Path) -> Result<String> {
        debug!("Getting diff for file: {}", file_path.display());

        // Get the diff between HEAD and working directory
        let head_tree = self.repo.head()?.peel_to_tree()?;
        let diff = self
            .repo
            .diff_tree_to_workdir_with_index(Some(&head_tree), None)?;

        let mut diff_output = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            match line.origin() {
                '+' | '-' | ' ' => {
                    diff_output.push(line.origin());
                    diff_output.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
                }
                _ => {}
            }
            true
        })?;

        Ok(diff_output)
    }

    fn stage_file(&self, file_path: &Path) -> Result<()> {
        debug!("Staging file: {}", file_path.display());

        let mut index = self.repo.index()?;
        index.add_path(file_path)?;
        index.write()?;

        debug!("Successfully staged file: {}", file_path.display());
        Ok(())
    }

    fn unstage_file(&self, file_path: &Path) -> Result<()> {
        debug!("Unstaging file: {}", file_path.display());

        let head = self.repo.head()?.peel_to_commit()?;
        let _head_tree = head.tree()?;

        self.repo
            .reset_default(Some(&head.into_object()), [file_path])?;

        debug!("Successfully unstaged file: {}", file_path.display());
        Ok(())
    }

    fn stage_all(&self) -> Result<()> {
        debug!("Staging all changes");

        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        debug!("Successfully staged all changes");
        Ok(())
    }

    fn reset_all(&self) -> Result<()> {
        debug!("Resetting all changes");

        let head = self.repo.head()?.peel_to_commit()?;
        self.repo
            .reset(&head.into_object(), git2::ResetType::Hard, None)?;

        debug!("Successfully reset all changes");
        Ok(())
    }
}
