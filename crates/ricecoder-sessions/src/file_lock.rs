//! File locking for concurrent session access control
//!
//! SSTATE-001: File locking / concurrent access control
//!
//! Implements both in-process (OpenCode-compatible) and OS-level locking.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use fs2::FileExt;
use std::fs::File;

use crate::error::{SessionError, SessionResult};

/// Lock type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LockType {
    /// Read lock (shared)
    Read,
    /// Write lock (exclusive)
    Write,
}

/// File lock guard (RAII-style lock release)
pub struct FileLockGuard {
    /// Path that is locked
    path: PathBuf,
    /// Lock type
    lock_type: LockType,
    /// OS-level file lock (if acquired)
    os_lock: Option<File>,
    /// In-process lock manager (for release)
    manager: Arc<FileLockManager>,
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {
        // Release OS lock first
        if let Some(file) = &self.os_lock {
            let _ = file.unlock();
        }
        
        // Release in-process lock
        self.manager.release_in_process(&self.path, self.lock_type);
    }
}

/// In-process lock state
#[derive(Debug, Clone)]
struct InProcessLock {
    /// Number of active read locks
    read_count: usize,
    /// Whether write lock is held
    write_held: bool,
}

/// File lock manager (handles both in-process and OS-level locking)
#[derive(Clone)]
pub struct FileLockManager {
    /// In-process lock state (path -> lock state)
    in_process_locks: Arc<Mutex<HashMap<PathBuf, InProcessLock>>>,
    /// Writer priority queue (path -> pending write count)
    writer_queue: Arc<Mutex<HashMap<PathBuf, usize>>>,
    /// Enable OS-level locking (in addition to in-process)
    use_os_locks: bool,
}

impl FileLockManager {
    /// Create a new file lock manager
    ///
    /// By default, uses both in-process locks (OpenCode-compatible) and OS-level locks (RiceCoder enhancement)
    pub fn new() -> Self {
        Self {
            in_process_locks: Arc::new(Mutex::new(HashMap::new())),
            writer_queue: Arc::new(Mutex::new(HashMap::new())),
            use_os_locks: true, // Enable OS locks by default
        }
    }
    
    /// Create with only in-process locks (OpenCode-compatible mode)
    pub fn new_in_process_only() -> Self {
        Self {
            in_process_locks: Arc::new(Mutex::new(HashMap::new())),
            writer_queue: Arc::new(Mutex::new(HashMap::new())),
            use_os_locks: false,
        }
    }
    
    /// Acquire a read lock
    ///
    /// Multiple readers can hold read locks simultaneously, but no writer can hold a write lock.
    /// Writers are prioritized: if writers are waiting, new readers will block.
    pub async fn read<P: AsRef<Path>>(&self, path: P) -> SessionResult<FileLockGuard> {
        let path = path.as_ref().to_path_buf();
        
        // Wait for any pending writers (writer priority)
        loop {
            let writer_count = {
                let queue = self.writer_queue.lock().unwrap();
                queue.get(&path).copied().unwrap_or(0)
            };
            
            if writer_count == 0 {
                break;
            }
            
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        // Acquire in-process read lock
        self.acquire_in_process_read(&path).await?;
        
        // Acquire OS-level read lock (if enabled)
        let os_lock = if self.use_os_locks {
            self.acquire_os_read(&path)?
        } else {
            None
        };
        
        Ok(FileLockGuard {
            path,
            lock_type: LockType::Read,
            os_lock,
            manager: Arc::new(self.clone()),
        })
    }
    
    /// Acquire a write lock
    ///
    /// Only one writer can hold a write lock, and no readers can hold read locks.
    /// Writers are prioritized over readers.
    pub async fn write<P: AsRef<Path>>(&self, path: P) -> SessionResult<FileLockGuard> {
        let path = path.as_ref().to_path_buf();
        
        // Signal that a writer is waiting (for priority)
        {
            let mut queue = self.writer_queue.lock().unwrap();
            *queue.entry(path.clone()).or_insert(0) += 1;
        }
        
        // Wait for exclusive access
        self.acquire_in_process_write(&path).await?;
        
        // Acquire OS-level write lock (if enabled)
        let os_lock = if self.use_os_locks {
            self.acquire_os_write(&path)?
        } else {
            None
        };
        
        // Remove from writer queue
        {
            let mut queue = self.writer_queue.lock().unwrap();
            if let Some(count) = queue.get_mut(&path) {
                *count -= 1;
                if *count == 0 {
                    queue.remove(&path);
                }
            }
        }
        
        Ok(FileLockGuard {
            path,
            lock_type: LockType::Write,
            os_lock,
            manager: Arc::new(self.clone()),
        })
    }
    
    /// Try to acquire in-process read lock (non-blocking check)
    fn try_acquire_in_process_read(&self, path: &Path) -> bool {
        let mut locks = self.in_process_locks.lock().unwrap();
        let lock_state = locks.entry(path.to_path_buf()).or_insert(InProcessLock {
            read_count: 0,
            write_held: false,
        });
        
        if lock_state.write_held {
            false
        } else {
            lock_state.read_count += 1;
            true
        }
    }
    
    /// Acquire in-process read lock (blocking)
    async fn acquire_in_process_read(&self, path: &Path) -> SessionResult<()> {
        loop {
            if self.try_acquire_in_process_read(path) {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    
    /// Try to acquire in-process write lock (non-blocking check)
    fn try_acquire_in_process_write(&self, path: &Path) -> bool {
        let mut locks = self.in_process_locks.lock().unwrap();
        let lock_state = locks.entry(path.to_path_buf()).or_insert(InProcessLock {
            read_count: 0,
            write_held: false,
        });
        
        if lock_state.write_held || lock_state.read_count > 0 {
            false
        } else {
            lock_state.write_held = true;
            true
        }
    }
    
    /// Acquire in-process write lock (blocking)
    async fn acquire_in_process_write(&self, path: &Path) -> SessionResult<()> {
        loop {
            if self.try_acquire_in_process_write(path) {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    
    /// Acquire OS-level read lock
    fn acquire_os_read(&self, path: &Path) -> SessionResult<Option<File>> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Create file if it doesn't exist
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        
        // Acquire shared lock
        file.try_lock_shared()
            .map_err(|e| SessionError::LockError(format!("Failed to acquire read lock: {}", e)))?;
        
        Ok(Some(file))
    }
    
    /// Acquire OS-level write lock
    fn acquire_os_write(&self, path: &Path) -> SessionResult<Option<File>> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Create file if it doesn't exist
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        
        // Acquire exclusive lock
        file.try_lock_exclusive()
            .map_err(|e| SessionError::LockError(format!("Failed to acquire write lock: {}", e)))?;
        
        Ok(Some(file))
    }
    
    /// Release in-process lock
    fn release_in_process(&self, path: &Path, lock_type: LockType) {
        let mut locks = self.in_process_locks.lock().unwrap();
        
        if let Some(lock_state) = locks.get_mut(path) {
            match lock_type {
                LockType::Read => {
                    if lock_state.read_count > 0 {
                        lock_state.read_count -= 1;
                    }
                }
                LockType::Write => {
                    lock_state.write_held = false;
                }
            }
            
            // Clean up if no locks held
            if lock_state.read_count == 0 && !lock_state.write_held {
                locks.remove(path);
            }
        }
    }
}

impl Default for FileLockManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_single_reader() {
        let manager = FileLockManager::new();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lock");
        
        let _guard = manager.read(&path).await.unwrap();
        // Lock acquired successfully
    }
    
    #[tokio::test]
    async fn test_multiple_readers() {
        let manager = FileLockManager::new();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lock");
        
        let _guard1 = manager.read(&path).await.unwrap();
        let _guard2 = manager.read(&path).await.unwrap();
        // Both readers can hold locks simultaneously
    }
    
    #[tokio::test]
    async fn test_exclusive_writer() {
        let manager = FileLockManager::new();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lock");
        
        let _guard = manager.write(&path).await.unwrap();
        
        // Try to acquire another write lock (should fail)
        let manager2 = manager.clone();
        let path2 = path.clone();
        let result = tokio::time::timeout(
            Duration::from_millis(100),
            manager2.write(&path2)
        ).await;
        
        assert!(result.is_err()); // Timeout = lock not acquired
    }
    
    #[tokio::test]
    async fn test_writer_priority() {
        let manager = FileLockManager::new();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lock");
        
        // Acquire read lock
        let read_guard = manager.read(&path).await.unwrap();
        
        // Start writer (will queue)
        let manager_clone = manager.clone();
        let path_clone = path.clone();
        let writer_task = tokio::spawn(async move {
            let _write_guard = manager_clone.write(&path_clone).await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
        });
        
        // Wait for writer to queue
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        // Try to acquire new read lock (should wait for writer)
        let manager_clone2 = manager.clone();
        let path_clone2 = path.clone();
        let reader_task = tokio::spawn(async move {
            let result = tokio::time::timeout(
                Duration::from_millis(100),
                manager_clone2.read(&path_clone2)
            ).await;
            result.is_err() // Should timeout due to writer priority
        });
        
        // Release original read lock
        drop(read_guard);
        
        // Wait for tasks
        writer_task.await.unwrap();
        let reader_blocked = reader_task.await.unwrap();
        
        // Reader should have been blocked by writer priority
        assert!(reader_blocked);
    }
    
    #[tokio::test]
    async fn test_lock_release() {
        let manager = FileLockManager::new();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lock");
        
        {
            let _guard = manager.write(&path).await.unwrap();
            // Lock held
        }
        // Lock released
        
        // Should be able to acquire again
        let _guard2 = manager.write(&path).await.unwrap();
    }
}
