//! Process tree management utilities.
//!
//! Provides best-effort cross-platform process tree termination.

use std::io;
use std::time::Duration;

use tokio::process::Command;

/// Kill a process and all its descendants.
///
/// On Windows this uses `taskkill /t /f /pid {pid}`.
/// On Unix this targets the process group `-pid` (requires the child to be in its own process
/// group; see `CommandExt::process_group(0)` / `setpgid(0, 0)` during spawn).
pub async fn kill_process_tree(pid: u32) -> io::Result<()> {
    if pid == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "pid must be non-zero",
        ));
    }

    #[cfg(windows)]
    {
        let output = Command::new("taskkill")
            .args(["/t", "/f", "/pid", &pid.to_string()])
            .output()
            .await?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "taskkill failed for pid {} (exit {:?})",
                    pid,
                    output.status.code()
                ),
            ));
        }

        return Ok(());
    }

    #[cfg(unix)]
    {
        use tokio::time::sleep;

        // SIGTERM process group.
        unsafe {
            let rc = libc::kill(-(pid as i32), libc::SIGTERM);
            if rc != 0 {
                let err = io::Error::last_os_error();
                // Ignore "no such process".
                if err.raw_os_error() != Some(libc::ESRCH) {
                    return Err(err);
                }
            }
        }

        sleep(Duration::from_millis(200)).await;

        // SIGKILL process group.
        unsafe {
            let rc = libc::kill(-(pid as i32), libc::SIGKILL);
            if rc != 0 {
                let err = io::Error::last_os_error();
                if err.raw_os_error() != Some(libc::ESRCH) {
                    return Err(err);
                }
            }
        }

        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}
