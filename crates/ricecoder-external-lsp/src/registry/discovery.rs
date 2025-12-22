//! LSP server discovery and verification

use std::{path::PathBuf, process::Command};

use tracing::{debug, warn};

use crate::error::{ExternalLspError, Result};

/// Discovers and verifies LSP server executables
pub struct ServerDiscovery;

impl ServerDiscovery {
    /// Check if an executable exists and is runnable
    pub fn verify_executable(executable: &str) -> Result<PathBuf> {
        debug!("Verifying executable: {}", executable);

        // Try to find the executable in PATH
        if let Ok(output) = Self::which_command(executable) {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    debug!("Found executable at: {}", path);
                    return Ok(PathBuf::from(path));
                }
            }
        }

        // If not found in PATH, check if it's an absolute path
        let path = PathBuf::from(executable);
        if path.is_absolute() && path.exists() {
            debug!("Found executable at absolute path: {:?}", path);
            return Ok(path);
        }

        // Try common installation paths
        let common_paths = Self::common_installation_paths(executable);
        for path in common_paths {
            if path.exists() {
                debug!("Found executable at common path: {:?}", path);
                return Ok(path);
            }
        }

        warn!("LSP server executable not found: {}", executable);
        Err(ExternalLspError::ServerNotFound {
            executable: executable.to_string(),
        })
    }

    /// Get common installation paths for an executable
    fn common_installation_paths(executable: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Windows paths
        #[cfg(target_os = "windows")]
        {
            paths.push(PathBuf::from(format!(
                "C:\\Program Files\\{}\\{}.exe",
                executable, executable
            )));
            paths.push(PathBuf::from(format!(
                "C:\\Program Files (x86)\\{}\\{}.exe",
                executable, executable
            )));
            paths.push(PathBuf::from(format!(
                "{}\\{}.exe",
                std::env::var("APPDATA").unwrap_or_default(),
                executable
            )));
        }

        // macOS paths
        #[cfg(target_os = "macos")]
        {
            paths.push(PathBuf::from(format!("/usr/local/bin/{}", executable)));
            paths.push(PathBuf::from(format!("/opt/homebrew/bin/{}", executable)));
            paths.push(PathBuf::from(format!(
                "{}/.cargo/bin/{}",
                std::env::var("HOME").unwrap_or_default(),
                executable
            )));
        }

        // Linux paths
        #[cfg(target_os = "linux")]
        {
            paths.push(PathBuf::from(format!("/usr/local/bin/{}", executable)));
            paths.push(PathBuf::from(format!("/usr/bin/{}", executable)));
            paths.push(PathBuf::from(format!(
                "{}/.cargo/bin/{}",
                std::env::var("HOME").unwrap_or_default(),
                executable
            )));
        }

        // Generic paths
        paths.push(PathBuf::from(format!("/opt/{}/{}", executable, executable)));
        paths.push(PathBuf::from(format!(
            "{}/.local/bin/{}",
            std::env::var("HOME").unwrap_or_default(),
            executable
        )));

        paths
    }

    /// Get installation instructions for an LSP server
    pub fn installation_instructions(language: &str, executable: &str) -> String {
        match language {
            "rust" => "Install rust-analyzer:\n\
                 - Via rustup: rustup component add rust-analyzer\n\
                 - Via cargo: cargo install rust-analyzer\n\
                 - See: https://rust-analyzer.github.io/manual.html#installation"
                .to_string(),
            "typescript" => "Install typescript-language-server:\n\
                 - Via npm: npm install -g typescript-language-server typescript\n\
                 - Via yarn: yarn global add typescript-language-server typescript\n\
                 - See: https://github.com/typescript-language-server/typescript-language-server"
                .to_string(),
            "python" => "Install python-lsp-server:\n\
                 - Via pip: pip install python-lsp-server\n\
                 - Via conda: conda install -c conda-forge python-lsp-server\n\
                 - See: https://github.com/python-lsp/python-lsp-server"
                .to_string(),
            "go" => "Install gopls:\n\
                 - Via go: go install github.com/golang/tools/gopls@latest\n\
                 - See: https://github.com/golang/tools/tree/master/gopls"
                .to_string(),
            _ => {
                format!(
                    "LSP server '{}' not found at: {}\n\
                     Please install it and ensure it's in your PATH.\n\
                     See the documentation for installation instructions.",
                    language, executable
                )
            }
        }
    }

    /// Execute 'which' command to find executable in PATH
    #[cfg(unix)]
    fn which_command(executable: &str) -> std::io::Result<std::process::Output> {
        Command::new("which").arg(executable).output()
    }

    /// Execute 'where' command to find executable in PATH (Windows)
    #[cfg(windows)]
    fn which_command(executable: &str) -> std::io::Result<std::process::Output> {
        Command::new("where").arg(executable).output()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_executable_not_found() {
        let result = ServerDiscovery::verify_executable("nonexistent-lsp-server-xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_installation_instructions_rust() {
        let instructions = ServerDiscovery::installation_instructions("rust", "rust-analyzer");
        assert!(instructions.contains("rust-analyzer"));
        assert!(instructions.contains("rustup"));
    }

    #[test]
    fn test_installation_instructions_typescript() {
        let instructions =
            ServerDiscovery::installation_instructions("typescript", "typescript-language-server");
        assert!(instructions.contains("typescript-language-server"));
        assert!(instructions.contains("npm"));
    }

    #[test]
    fn test_installation_instructions_python() {
        let instructions = ServerDiscovery::installation_instructions("python", "pylsp");
        assert!(instructions.contains("python-lsp-server"));
        assert!(instructions.contains("pip"));
    }

    #[test]
    fn test_common_installation_paths() {
        let paths = ServerDiscovery::common_installation_paths("rust-analyzer");
        assert!(!paths.is_empty());
    }
}
