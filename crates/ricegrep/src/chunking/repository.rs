use std::path::{Path, PathBuf};

use glob::Pattern;
use ignore::WalkBuilder;

use crate::chunking::{ChunkingError, ChunkingResult};

#[derive(Debug, Clone)]
pub struct RepositorySource {
    pub root: PathBuf,
    pub repository_id: Option<u32>,
}

impl RepositorySource {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self {
            root: root.into(),
            repository_id: None,
        }
    }

    pub fn with_repository_id<P: Into<PathBuf>>(root: P, repository_id: u32) -> Self {
        Self {
            root: root.into(),
            repository_id: Some(repository_id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RepositoryScannerConfig {
    pub include_patterns: Vec<Pattern>,
    pub exclude_patterns: Vec<Pattern>,
    pub follow_symlinks: bool,
    /// If true, ignore .gitignore, .ignore, and other ignore files.
    /// Default is false (respects ignore files).
    pub no_ignore: bool,
}

impl Default for RepositoryScannerConfig {
    fn default() -> Self {
        Self {
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            follow_symlinks: false,
            no_ignore: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Clone)]
pub struct RepositoryScanner {
    config: RepositoryScannerConfig,
}

impl Default for RepositoryScanner {
    fn default() -> Self {
        Self {
            config: RepositoryScannerConfig::default(),
        }
    }
}

impl RepositoryScanner {
    /// Create a new scanner with the given configuration.
    pub fn new(config: RepositoryScannerConfig) -> Self {
        Self { config }
    }

    /// Create a scanner that ignores .gitignore, .ignore, and other ignore files.
    pub fn with_no_ignore(no_ignore: bool) -> Self {
        Self {
            config: RepositoryScannerConfig {
                no_ignore,
                ..Default::default()
            },
        }
    }

    pub fn scan(&self, source: &RepositorySource) -> ChunkingResult<Vec<FileEntry>> {
        let walker = WalkBuilder::new(&source.root)
            .follow_links(self.config.follow_symlinks)
            .git_ignore(!self.config.no_ignore)
            .git_global(!self.config.no_ignore)
            .git_exclude(!self.config.no_ignore)
            .ignore(!self.config.no_ignore)
            .build();
        let mut files = Vec::new();

        for entry in walker {
            let entry = entry?;

            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            if self.should_exclude(entry.path()) {
                continue;
            }

            if !self.matches_includes(entry.path()) {
                continue;
            }

            let metadata = entry.metadata()?;
            files.push(FileEntry {
                path: entry.path().to_path_buf(),
                size: metadata.len(),
            });
        }

        Ok(files)
    }

    fn should_exclude(&self, path: &Path) -> bool {
        self.config
            .exclude_patterns
            .iter()
            .any(|pattern| path.to_str().map(|p| pattern.matches(p)).unwrap_or(false))
    }

    fn matches_includes(&self, path: &Path) -> bool {
        if self.config.include_patterns.is_empty() {
            return true;
        }
        self.config
            .include_patterns
            .iter()
            .any(|pattern| path.to_str().map(|p| pattern.matches(p)).unwrap_or(false))
    }
}
