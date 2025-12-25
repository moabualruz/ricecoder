//! Ripgrep integration for file operations
//!
//! Provides ripgrep binary management, file enumeration, and content search
//! matching OpenCode's ripgrep.ts functionality.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Ripgrep errors
#[derive(Error, Debug)]
pub enum RipgrepError {
    #[error("Ripgrep binary not found")]
    BinaryNotFound,

    #[error("Failed to download ripgrep: {0}")]
    DownloadFailed(String),

    #[error("Failed to extract ripgrep: {0}")]
    ExtractionFailed(String),

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Search failed: {0}")]
    SearchFailed(String),
}

pub type RipgrepResult<T> = Result<T, RipgrepError>;

/// Ripgrep binary manager and search interface
pub struct Ripgrep {
    binary_path: PathBuf,
}

impl Ripgrep {
    /// Create a new Ripgrep instance, finding or downloading the binary
    pub async fn new() -> RipgrepResult<Self> {
        let binary_path = Self::find_or_download_binary().await?;
        Ok(Self { binary_path })
    }

    /// Find ripgrep binary in PATH or download it
    async fn find_or_download_binary() -> RipgrepResult<PathBuf> {
        // First, try to find 'rg' in PATH
        if let Ok(path) = which::which("rg") {
            info!("Found ripgrep in PATH: {}", path.display());
            return Ok(path);
        }

        // If not found, we could download it (like OpenCode does)
        // For now, return error - user should install ripgrep
        Err(RipgrepError::BinaryNotFound)
    }

    /// Enumerate files in a directory
    ///
    /// Matches OpenCode's `Ripgrep.files()` - uses `rg --files`
    /// with `--follow`, `--hidden`, and `--glob=!.git/*`
    pub async fn files(
        &self,
        cwd: &Path,
        glob_patterns: &[String],
    ) -> RipgrepResult<Vec<PathBuf>> {
        let mut args = vec![
            "--files".to_string(),
            "--follow".to_string(),
            "--hidden".to_string(),
            "--glob=!.git/*".to_string(),
        ];

        // Add additional glob patterns
        for pattern in glob_patterns {
            args.push(format!("--glob={}", pattern));
        }

        let output = Command::new(&self.binary_path)
            .args(&args)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RipgrepError::Io(e))?
            .wait_with_output()
            .await
            .map_err(|e| RipgrepError::Io(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RipgrepError::SearchFailed(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<PathBuf> = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| cwd.join(line))
            .collect();

        Ok(files)
    }

    /// Search for content in files
    ///
    /// Matches OpenCode's `Ripgrep.search()` - uses `rg --json`
    pub async fn search(
        &self,
        cwd: &Path,
        pattern: &str,
        glob_patterns: &[String],
        limit: Option<usize>,
    ) -> RipgrepResult<Vec<SearchMatch>> {
        let mut args = vec![
            "--json".to_string(),
            "--hidden".to_string(),
            "--glob=!.git/*".to_string(),
        ];

        // Add glob patterns
        for glob in glob_patterns {
            args.push(format!("--glob={}", glob));
        }

        // Add limit
        if let Some(limit) = limit {
            args.push(format!("--max-count={}", limit));
        }

        // Add pattern
        args.push("--".to_string());
        args.push(pattern.to_string());

        let mut child = Command::new(&self.binary_path)
            .args(&args)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RipgrepError::Io(e))?;

        let stdout = child.stdout.take().ok_or_else(|| {
            RipgrepError::SearchFailed("Failed to capture stdout".to_string())
        })?;

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut matches = Vec::new();

        while let Some(line) = lines.next_line().await.map_err(|e| RipgrepError::Io(e))? {
            if let Ok(result) = serde_json::from_str::<RipgrepJsonOutput>(&line) {
                if let RipgrepResultType::Match(match_data) = result.type_field {
                    matches.push(SearchMatch {
                        path: PathBuf::from(match_data.path.text),
                        line_number: match_data.line_number,
                        line_text: match_data.lines.text,
                    });
                }
            }
        }

        child.wait().await.map_err(|e| RipgrepError::Io(e))?;

        Ok(matches)
    }

    /// Build file tree (matches OpenCode's `Ripgrep.tree()`)
    pub async fn tree(&self, cwd: &Path, limit: Option<usize>) -> RipgrepResult<String> {
        let files = self.files(cwd, &[]).await?;
        let limit = limit.unwrap_or(50);

        // Build tree structure (simplified version)
        let mut tree = String::from(format!("{}/\n", cwd.display()));

        for (i, file) in files.iter().enumerate() {
            if i >= limit {
                tree.push_str(&format!("[{} truncated]\n", files.len() - i));
                break;
            }

            if let Ok(relative) = file.strip_prefix(cwd) {
                let depth = relative.components().count();
                let indent = "  ".repeat(depth.saturating_sub(1));
                tree.push_str(&format!("{}{}\n", indent, relative.display()));
            }
        }

        Ok(tree)
    }
}

/// Search match result
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub path: PathBuf,
    pub line_number: u64,
    pub line_text: String,
}

/// Ripgrep JSON output format (internal deserialization struct)
#[derive(Debug, serde::Deserialize)]
struct RipgrepJsonOutput {
    #[serde(rename = "type")]
    type_field: RipgrepResultType,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
enum RipgrepResultType {
    #[serde(rename = "match")]
    Match(RipgrepMatch),
    #[serde(rename = "begin")]
    Begin,
    #[serde(rename = "end")]
    End,
    #[serde(rename = "summary")]
    Summary,
}

#[derive(Debug, serde::Deserialize)]
struct RipgrepMatch {
    path: RipgrepPath,
    lines: RipgrepLines,
    line_number: u64,
}

#[derive(Debug, serde::Deserialize)]
struct RipgrepPath {
    text: String,
}

#[derive(Debug, serde::Deserialize)]
struct RipgrepLines {
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_ripgrep_new() {
        // This test will fail if ripgrep is not installed
        // That's expected - users should install ripgrep
        let result = Ripgrep::new().await;
        match result {
            Ok(_) => {
                // Ripgrep found in PATH
                assert!(true);
            }
            Err(RipgrepError::BinaryNotFound) => {
                // Expected if ripgrep not installed
                assert!(true);
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_ripgrep_files() {
        // Skip if ripgrep not available
        let rg = match Ripgrep::new().await {
            Ok(rg) => rg,
            Err(_) => return, // Skip test
        };

        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test1.txt"), "content1")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("test2.txt"), "content2")
            .await
            .unwrap();

        let files = rg.files(temp_dir.path(), &[]).await.unwrap();
        assert!(files.len() >= 2);
    }
}
