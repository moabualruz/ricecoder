/// Safe synchronization utilities for robust error handling
///
/// This module provides utilities for safe mutex locking and error recovery,
/// ensuring the application doesn't panic when mutex locks fail.

use std::sync::{Mutex, MutexGuard};
use tracing::{debug, warn};

/// Result type for mutex operations
pub type SyncResult<T> = Result<T, SyncError>;

/// Error type for synchronization operations
#[derive(Debug, Clone)]
pub enum SyncError {
    /// Mutex was poisoned (lock holder panicked)
    MutexPoisoned,
    /// Other synchronization error
    Other(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::MutexPoisoned => write!(f, "Mutex was poisoned"),
            SyncError::Other(msg) => write!(f, "Synchronization error: {}", msg),
        }
    }
}

impl std::error::Error for SyncError {}

/// Safe mutex lock helper that recovers from poisoning
///
/// This function attempts to lock a mutex and recover gracefully if the lock
/// is poisoned (i.e., the previous lock holder panicked).
///
/// # Arguments
///
/// * `mutex` - The mutex to lock
/// * `context` - A description of what operation is being performed (for logging)
///
/// # Returns
///
/// * `Ok(guard)` - Successfully acquired the lock
/// * `Err(SyncError::MutexPoisoned)` - Lock was poisoned and recovered
///
/// # Example
///
/// ```ignore
/// let data = Mutex::new(vec![1, 2, 3]);
/// match safe_lock(&data, "accessing data") {
///     Ok(guard) => {
///         // Use guard
///     }
///     Err(SyncError::MutexPoisoned) => {
///         warn!("Mutex was poisoned, recovered");
///     }
///     Err(e) => {
///         error!("Failed to lock: {}", e);
///     }
/// }
/// ```
pub fn safe_lock<'a, T>(mutex: &'a Mutex<T>, context: &str) -> SyncResult<MutexGuard<'a, T>> {
    match mutex.lock() {
        Ok(guard) => Ok(guard),
        Err(poisoned) => {
            warn!("Mutex poisoned in {}, recovering", context);
            debug!("Recovering from poisoned mutex: {}", context);
            // Recover the lock by extracting the inner value
            Ok(poisoned.into_inner())
        }
    }
}

/// Safe mutex lock helper that returns a default value on failure
///
/// This function attempts to lock a mutex and returns a default value if the lock
/// is poisoned. This is useful for operations that can tolerate missing data.
///
/// # Arguments
///
/// * `mutex` - The mutex to lock
/// * `context` - A description of what operation is being performed (for logging)
/// * `default` - The default value to return if lock fails
///
/// # Returns
///
/// The locked guard or the default value
///
/// # Example
///
/// ```ignore
/// let data = Mutex::new(vec![1, 2, 3]);
/// let guard = safe_lock_or_default(&data, "accessing data", vec![]);
/// // Use guard (either real data or empty vec)
/// ```
pub fn safe_lock_or_default<'a, T: Default>(
    mutex: &'a Mutex<T>,
    context: &str,
) -> MutexGuard<'a, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("Mutex poisoned in {}, using default", context);
            debug!("Recovering from poisoned mutex with default: {}", context);
            poisoned.into_inner()
        }
    }
}

/// Safe mutex lock helper that logs and continues on failure
///
/// This function attempts to lock a mutex and logs a warning if the lock fails.
/// It returns an Option that is None if the lock failed.
///
/// # Arguments
///
/// * `mutex` - The mutex to lock
/// * `context` - A description of what operation is being performed (for logging)
///
/// # Returns
///
/// * `Some(guard)` - Successfully acquired the lock
/// * `None` - Lock was poisoned or failed
///
/// # Example
///
/// ```ignore
/// let data = Mutex::new(vec![1, 2, 3]);
/// if let Some(guard) = safe_lock_optional(&data, "accessing data") {
///     // Use guard
/// } else {
///     warn!("Could not acquire lock, skipping operation");
/// }
/// ```
pub fn safe_lock_optional<'a, T>(mutex: &'a Mutex<T>, context: &str) -> Option<MutexGuard<'a, T>> {
    match mutex.lock() {
        Ok(guard) => Some(guard),
        Err(_poisoned) => {
            warn!("Mutex poisoned in {}, operation skipped", context);
            debug!("Could not recover from poisoned mutex: {}", context);
            None
        }
    }
}

/// Trait for types that can be safely locked
pub trait SafeLockable<T> {
    /// Attempt to lock with error recovery
    fn safe_lock<'a>(&'a self, context: &str) -> SyncResult<MutexGuard<'a, T>>;

    /// Attempt to lock with default fallback
    fn safe_lock_or_default<'a>(&'a self, context: &str) -> MutexGuard<'a, T>
    where
        T: Default;

    /// Attempt to lock with optional result
    fn safe_lock_optional<'a>(&'a self, context: &str) -> Option<MutexGuard<'a, T>>;
}

impl<T> SafeLockable<T> for Mutex<T> {
    fn safe_lock<'a>(&'a self, context: &str) -> SyncResult<MutexGuard<'a, T>> {
        safe_lock(self, context)
    }

    fn safe_lock_or_default<'a>(&'a self, context: &str) -> MutexGuard<'a, T>
    where
        T: Default,
    {
        safe_lock_or_default(self, context)
    }

    fn safe_lock_optional<'a>(&'a self, context: &str) -> Option<MutexGuard<'a, T>> {
        safe_lock_optional(self, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_safe_lock_success() {
        let mutex = Mutex::new(vec![1, 2, 3]);
        let result = safe_lock(&mutex, "test");
        assert!(result.is_ok());
        let guard = result.unwrap();
        assert_eq!(guard.len(), 3);
    }

    #[test]
    fn test_safe_lock_or_default() {
        let mutex: Mutex<Vec<i32>> = Mutex::new(vec![1, 2, 3]);
        let guard = safe_lock_or_default(&mutex, "test");
        assert_eq!(guard.len(), 3);
    }

    #[test]
    fn test_safe_lock_optional_success() {
        let mutex = Mutex::new(vec![1, 2, 3]);
        let result = safe_lock_optional(&mutex, "test");
        assert!(result.is_some());
        let guard = result.unwrap();
        assert_eq!(guard.len(), 3);
    }

    #[test]
    fn test_trait_safe_lock() {
        let mutex = Mutex::new(42);
        let result = mutex.safe_lock("test");
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), 42);
    }

    #[test]
    fn test_trait_safe_lock_or_default() {
        let mutex: Mutex<Vec<i32>> = Mutex::new(vec![1, 2, 3]);
        let guard = mutex.safe_lock_or_default("test");
        assert_eq!(guard.len(), 3);
    }

    #[test]
    fn test_trait_safe_lock_optional() {
        let mutex = Mutex::new(42);
        let result = mutex.safe_lock_optional("test");
        assert!(result.is_some());
        assert_eq!(*result.unwrap(), 42);
    }

    #[test]
    fn test_concurrent_access() {
        let data = Arc::new(Mutex::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let data_clone = Arc::clone(&data);
            let handle = thread::spawn(move || {
                if let Ok(mut guard) = safe_lock(&data_clone, "concurrent test") {
                    *guard += 1;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_value = safe_lock(&data, "final check").unwrap();
        assert_eq!(*final_value, 10);
    }
}
