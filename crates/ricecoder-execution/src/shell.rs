//! Shell module - Cross-platform shell detection and process management
//!
//! Provides OpenCode-compatible shell selection, process-tree kill, and environment normalization.
//! Implements faithful translation of OpenCode's shell.ts functionality.

use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::Duration;

use tokio::time::sleep;
use tracing::{debug, warn};

/// SIGKILL escalation timeout (matches OpenCode: 200ms)
const SIGKILL_TIMEOUT_MS: u64 = 200;

/// Shells known to be incompatible with command execution
const SHELL_BLACKLIST: &[&str] = &["fish", "nu"];

/// Shell detection and selection utilities
pub struct ShellDetector;

impl ShellDetector {
    /// Get the preferred shell (exactly like OpenCode Shell.preferred)
    ///
    /// Returns `SHELL` env var if set, otherwise falls back to OS-specific default.
    ///
    /// # Examples
    /// ```
    /// use ricecoder_execution::shell::ShellDetector;
    ///
    /// let shell = ShellDetector::preferred();
    /// println!("Preferred shell: {}", shell);
    /// ```
    pub fn preferred() -> String {
        if let Some(shell) = env::var("SHELL").ok() {
            debug!(shell = %shell, "Using SHELL environment variable");
            return shell;
        }

        let fallback = Self::fallback();
        debug!(fallback = %fallback, "Using fallback shell");
        fallback
    }

    /// Get an acceptable shell for command execution (exactly like OpenCode Shell.acceptable)
    ///
    /// Returns `SHELL` env var unless it's blacklisted (fish, nu), otherwise falls back.
    ///
    /// # Examples
    /// ```
    /// use ricecoder_execution::shell::ShellDetector;
    ///
    /// let shell = ShellDetector::acceptable();
    /// // Will never return fish or nu
    /// assert!(!shell.contains("fish") && !shell.contains("nu"));
    /// ```
    pub fn acceptable() -> String {
        if let Some(shell) = env::var("SHELL").ok() {
            let basename = Path::new(&shell)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if !SHELL_BLACKLIST.contains(&basename) {
                debug!(shell = %shell, "Using acceptable SHELL");
                return shell;
            }

            warn!(
                shell = %shell,
                reason = "blacklisted (fish/nu incompatible)",
                "Skipping SHELL environment variable"
            );
        }

        let fallback = Self::fallback();
        debug!(fallback = %fallback, "Using acceptable fallback shell");
        fallback
    }

    /// Get OS-specific fallback shell (matches OpenCode Shell.fallback)
    ///
    /// - Windows: Tries Git Bash via `RICECODER_GIT_BASH_PATH` or git-adjacent bash.exe,
    ///   else `COMSPEC`, else `cmd.exe`
    /// - macOS: `/bin/zsh`
    /// - Linux/other: `bash` if found, else `/bin/sh`
    fn fallback() -> String {
        #[cfg(windows)]
        {
            // Check RICECODER_GIT_BASH_PATH override
            if let Ok(git_bash_path) = env::var("RICECODER_GIT_BASH_PATH") {
                debug!(git_bash_path = %git_bash_path, "Using RICECODER_GIT_BASH_PATH");
                return git_bash_path;
            }

            // Try to find Git Bash via git.exe location
            if let Some(bash) = Self::find_git_bash() {
                debug!(bash = %bash.display(), "Found Git Bash via git.exe");
                return bash.to_string_lossy().to_string();
            }

            // Fall back to COMSPEC or cmd.exe
            if let Ok(comspec) = env::var("COMSPEC") {
                debug!(comspec = %comspec, "Using COMSPEC");
                return comspec;
            }

            debug!("Using default cmd.exe");
            "cmd.exe".to_string()
        }

        #[cfg(target_os = "macos")]
        {
            debug!("Using macOS default /bin/zsh");
            "/bin/zsh".to_string()
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            // Try to find bash
            if let Some(bash) = which::which("bash").ok() {
                debug!(bash = %bash.display(), "Found bash via PATH");
                return bash.to_string_lossy().to_string();
            }

            debug!("Using default /bin/sh");
            "/bin/sh".to_string()
        }
    }

    /// Find Git Bash on Windows by locating git.exe
    ///
    /// Git Bash structure:
    /// - git.exe: `C:\Program Files\Git\cmd\git.exe`
    /// - bash.exe: `C:\Program Files\Git\bin\bash.exe`
    #[cfg(windows)]
    fn find_git_bash() -> Option<PathBuf> {
        let git_exe = which::which("git").ok()?;

        // Navigate from git.exe -> .. -> .. -> bin -> bash.exe
        let bash = git_exe.parent()?.parent()?.join("bin").join("bash.exe");

        if bash.exists() && bash.is_file() {
            Some(bash)
        } else {
            None
        }
    }

    /// Check if a shell is blacklisted
    ///
    /// # Arguments
    /// * `shell` - Shell path or name to check
    ///
    /// # Returns
    /// `true` if the shell is blacklisted (fish, nu)
    ///
    /// # Examples
    /// ```
    /// use ricecoder_execution::shell::ShellDetector;
    ///
    /// assert!(ShellDetector::is_blacklisted("/usr/bin/fish"));
    /// assert!(ShellDetector::is_blacklisted("/usr/bin/nu"));
    /// assert!(!ShellDetector::is_blacklisted("/bin/bash"));
    /// ```
    pub fn is_blacklisted(shell: &str) -> bool {
        let basename = Path::new(shell)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        SHELL_BLACKLIST.contains(&basename)
    }
}

/// Process tree management utilities
pub struct ProcessTree;

impl ProcessTree {
    /// Kill a process and all its descendants (matches OpenCode Shell.killTree)
    ///
    /// - Windows: Uses `taskkill /pid <pid> /f /t`
    /// - Unix: Kills process group via negative PID with SIGTERM→SIGKILL escalation
    ///
    /// # Arguments
    /// * `proc` - Process to kill (uses PID)
    ///
    /// # Returns
    /// `Ok(())` if successful, `Err` if kill failed
    ///
    /// # Examples
    /// ```no_run
    /// use std::process::Command;
    /// use ricecoder_execution::shell::ProcessTree;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let mut child = Command::new("sleep").arg("1000").spawn()?;
    /// ProcessTree::kill_tree(&mut child).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn kill_tree(proc: &mut Child) -> io::Result<()> {
        let pid = proc.id();

        debug!(pid = %pid, "Killing process tree");

        #[cfg(windows)]
        {
            use tokio::process::Command;

            let mut killer = Command::new("taskkill")
                .args(["/pid", &pid.to_string(), "/f", "/t"])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?;

            let _ = killer.wait().await;
            debug!(pid = %pid, "Windows process tree killed with taskkill");
            return Ok(());
        }

        #[cfg(unix)]
        {
            use nix::sys::signal::{killpg, Signal};
            use nix::unistd::Pid;

            let pgid = Pid::from_raw(pid as i32);

            // Try to kill process group with SIGTERM
            match killpg(pgid, Signal::SIGTERM) {
                Ok(_) => debug!(pid = %pid, "Sent SIGTERM to process group"),
                Err(e) => {
                    warn!(pid = %pid, error = %e, "Failed to send SIGTERM to process group, trying process only");
                    // Fallback to killing just the process
                    let _ = proc.kill().await;
                }
            }

            // Wait for SIGKILL escalation timeout
            sleep(Duration::from_millis(SIGKILL_TIMEOUT_MS)).await;

            // Try to kill process group with SIGKILL
            match killpg(pgid, Signal::SIGKILL) {
                Ok(_) => debug!(pid = %pid, "Sent SIGKILL to process group"),
                Err(e) => {
                    warn!(pid = %pid, error = %e, "Failed to send SIGKILL to process group, trying process only");
                    // Fallback to killing just the process
                    let _ = proc.kill().await;
                }
            }

            return Ok(());
        }

        #[allow(unreachable_code)]
        Ok(())
    }
}

/// Environment normalization for non-interactive command execution
pub struct Environment;

impl Environment {
    /// Normalize environment for non-interactive tool execution (matches OpenCode behavior)
    ///
    /// Sets `TERM=dumb` to avoid interactive formatting/control sequences in command output.
    ///
    /// # Examples
    /// ```
    /// use std::collections::HashMap;
    /// use ricecoder_execution::shell::Environment;
    ///
    /// let mut env = HashMap::new();
    /// Environment::normalize_for_tools(&mut env);
    /// assert_eq!(env.get("TERM"), Some(&"dumb".to_string()));
    /// ```
    pub fn normalize_for_tools(env_vars: &mut std::collections::HashMap<String, String>) {
        env_vars.insert("TERM".to_string(), "dumb".to_string());
        debug!("Set TERM=dumb for non-interactive execution");
    }

    /// Create normalized environment for tool execution
    ///
    /// # Returns
    /// HashMap with `TERM=dumb` set
    ///
    /// # Examples
    /// ```
    /// use ricecoder_execution::shell::Environment;
    ///
    /// let env = Environment::create_tool_env();
    /// assert_eq!(env.get("TERM"), Some(&"dumb".to_string()));
    /// ```
    pub fn create_tool_env() -> std::collections::HashMap<String, String> {
        let mut env = std::collections::HashMap::new();
        Self::normalize_for_tools(&mut env);
        env
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_blacklist() {
        assert!(ShellDetector::is_blacklisted("/usr/bin/fish"));
        assert!(ShellDetector::is_blacklisted("/usr/local/bin/nu"));
        assert!(ShellDetector::is_blacklisted("fish"));
        assert!(ShellDetector::is_blacklisted("nu"));

        assert!(!ShellDetector::is_blacklisted("/bin/bash"));
        assert!(!ShellDetector::is_blacklisted("/bin/zsh"));
        assert!(!ShellDetector::is_blacklisted("bash"));
    }

    #[test]
    fn test_preferred_shell() {
        // Should return SHELL env var if set, otherwise fallback
        let shell = ShellDetector::preferred();
        assert!(!shell.is_empty());
    }

    #[test]
    fn test_acceptable_shell() {
        // Should never return blacklisted shells
        let shell = ShellDetector::acceptable();
        assert!(!shell.contains("fish"));
        assert!(!shell.contains("nu"));
    }

    #[test]
    fn test_env_normalization() {
        let mut env = std::collections::HashMap::new();
        Environment::normalize_for_tools(&mut env);
        assert_eq!(env.get("TERM"), Some(&"dumb".to_string()));
    }

    #[test]
    fn test_create_tool_env() {
        let env = Environment::create_tool_env();
        assert_eq!(env.get("TERM"), Some(&"dumb".to_string()));
    }
}
