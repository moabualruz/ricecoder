//! Helper Context Creation Utilities
//!
//! This module provides helper functions for creating context providers
//! with common patterns (async initialization, lazy loading, etc.).

use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use std::future::Future;
use std::pin::Pin;

/// Type alias for async init function
pub type AsyncInit<T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send>>;

/// Generic context provider with lazy initialization
#[derive(Clone)]
pub struct LazyProvider<T: Clone + Send + Sync> {
    value: Arc<RwLock<Option<T>>>,
    init: Arc<Mutex<Option<Box<dyn Fn() -> AsyncInit<T> + Send + Sync>>>>,
}

impl<T: Clone + Send + Sync + 'static> LazyProvider<T> {
    /// Create new lazy provider with init function
    pub fn new<F, Fut>(init_fn: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, String>> + Send + 'static,
    {
        let boxed_fn = Box::new(move || -> AsyncInit<T> {
            Box::pin(init_fn())
        });

        Self {
            value: Arc::new(RwLock::new(None)),
            init: Arc::new(Mutex::new(Some(boxed_fn))),
        }
    }

    /// Get value, initializing if needed
    pub async fn get(&self) -> Result<T, String> {
        // Fast path: already initialized
        {
            let read = self.value.read().await;
            if let Some(ref v) = *read {
                return Ok(v.clone());
            }
        }

        // Slow path: initialize
        let init_fn = {
            let mut init = self.init.lock().await;
            init.take().ok_or_else(|| "Already initializing".to_string())?
        };

        let result = (init_fn)().await;

        match result {
            Ok(value) => {
                let mut write = self.value.write().await;
                *write = Some(value.clone());
                Ok(value)
            }
            Err(e) => {
                // Put init function back for retry
                let mut init = self.init.lock().await;
                *init = Some(init_fn);
                Err(e)
            }
        }
    }

    /// Check if initialized
    pub async fn is_initialized(&self) -> bool {
        self.value.read().await.is_some()
    }

    /// Reset to uninitialized state
    pub async fn reset(&self) {
        let mut write = self.value.write().await;
        *write = None;
    }
}

/// Simple context provider (eager initialization)
#[derive(Clone)]
pub struct SimpleProvider<T: Clone + Send + Sync> {
    value: Arc<RwLock<T>>,
}

impl<T: Clone + Send + Sync> SimpleProvider<T> {
    /// Create new simple provider
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
        }
    }

    /// Get current value
    pub async fn get(&self) -> T {
        self.value.read().await.clone()
    }

    /// Update value
    pub async fn set(&self, value: T) {
        let mut write = self.value.write().await;
        *write = value;
    }

    /// Update value with function
    pub async fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut write = self.value.write().await;
        f(&mut *write);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lazy_provider_initialization() {
        let provider = LazyProvider::new(|| async {
            Ok("initialized".to_string())
        });

        assert!(!provider.is_initialized().await);
        let value = provider.get().await.unwrap();
        assert_eq!(value, "initialized");
        assert!(provider.is_initialized().await);
    }

    #[tokio::test]
    async fn test_lazy_provider_caching() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let provider = LazyProvider::new(move || {
            let c = counter_clone.clone();
            async move {
                let mut count = c.lock().await;
                *count += 1;
                Ok(*count)
            }
        });

        let v1 = provider.get().await.unwrap();
        let v2 = provider.get().await.unwrap();
        assert_eq!(v1, v2);
        assert_eq!(v1, 1); // Only initialized once
    }

    #[tokio::test]
    async fn test_lazy_provider_reset() {
        let provider = LazyProvider::new(|| async {
            Ok(42)
        });

        provider.get().await.unwrap();
        assert!(provider.is_initialized().await);

        provider.reset().await;
        assert!(!provider.is_initialized().await);
    }

    #[tokio::test]
    async fn test_simple_provider_creation() {
        let provider = SimpleProvider::new(100);
        assert_eq!(provider.get().await, 100);
    }

    #[tokio::test]
    async fn test_simple_provider_update() {
        let provider = SimpleProvider::new(10);
        provider.set(20).await;
        assert_eq!(provider.get().await, 20);
    }

    #[tokio::test]
    async fn test_simple_provider_update_fn() {
        let provider = SimpleProvider::new(vec![1, 2, 3]);
        provider.update(|v| v.push(4)).await;
        assert_eq!(provider.get().await, vec![1, 2, 3, 4]);
    }
}
