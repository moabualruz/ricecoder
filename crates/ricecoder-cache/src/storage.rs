//! Cache storage backends

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::RwLock};

use crate::{CacheError, Result};

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T: Clone> {
    /// The cached data
    pub data: T,
    /// When the entry was created
    pub created_at: std::time::SystemTime,
    /// When the entry expires (optional)
    pub expires_at: Option<std::time::SystemTime>,
    /// Size of the entry in bytes
    pub size_bytes: u64,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl<T: Clone> CacheEntry<T> {
    /// Create a new cache entry
    pub fn new(data: T, ttl: Option<std::time::Duration>) -> Self
    where
        T: Serialize,
    {
        let created_at = std::time::SystemTime::now();
        let expires_at = ttl.map(|t| created_at + t);
        let size_bytes = serde_json::to_string(&data)
            .map(|s| s.len() as u64)
            .unwrap_or(0);

        Self {
            data,
            created_at,
            expires_at,
            size_bytes,
            metadata: HashMap::new(),
        }
    }

    /// Check if the entry has expired
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|expires| std::time::SystemTime::now() > expires)
            .unwrap_or(false)
    }

    /// Get remaining TTL
    pub fn ttl_remaining(&self) -> Option<std::time::Duration> {
        self.expires_at?
            .duration_since(std::time::SystemTime::now())
            .ok()
    }
}

/// Cache storage trait
#[async_trait]
pub trait CacheStorage: Send + Sync {
    /// Store a value
    async fn set(&self, key: &str, value: &serde_json::Value) -> Result<()>;

    /// Retrieve a value
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>>;

    /// Remove a value
    async fn remove(&self, key: &str) -> Result<bool>;

    /// Check if key exists
    async fn contains(&self, key: &str) -> Result<bool>;

    /// Clear all entries
    async fn clear(&self) -> Result<()>;

    /// Get number of entries
    async fn len(&self) -> Result<usize>;

    /// Get total size in bytes
    async fn size_bytes(&self) -> Result<u64>;

    /// Get all keys
    async fn keys(&self) -> Result<Vec<String>>;
}

/// In-memory cache storage
pub struct MemoryStorage {
    data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl MemoryStorage {
    /// Create new in-memory storage
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::with_capacity(capacity))),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheStorage for MemoryStorage {
    async fn set(&self, key: &str, value: &serde_json::Value) -> Result<()> {
        let mut data = self.data.write().await;
        data.insert(key.to_string(), value.clone());
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }

    async fn remove(&self, key: &str) -> Result<bool> {
        let mut data = self.data.write().await;
        Ok(data.remove(key).is_some())
    }

    async fn contains(&self, key: &str) -> Result<bool> {
        let data = self.data.read().await;
        Ok(data.contains_key(key))
    }

    async fn clear(&self) -> Result<()> {
        let mut data = self.data.write().await;
        data.clear();
        Ok(())
    }

    async fn len(&self) -> Result<usize> {
        let data = self.data.read().await;
        Ok(data.len())
    }

    async fn size_bytes(&self) -> Result<u64> {
        let data = self.data.read().await;
        let mut total_size = 0u64;
        for value in data.values() {
            if let Ok(size) = serde_json::to_string(value).map(|s| s.len() as u64) {
                total_size += size;
            }
        }
        Ok(total_size)
    }

    async fn keys(&self) -> Result<Vec<String>> {
        let data = self.data.read().await;
        Ok(data.keys().cloned().collect())
    }
}

/// Disk-based cache storage
pub struct DiskStorage {
    base_path: PathBuf,
}

impl DiskStorage {
    /// Create new disk storage with base path
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Get file path for a key
    fn key_path(&self, key: &str) -> PathBuf {
        // Sanitize key for filesystem
        let safe_key = key.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        self.base_path.join(format!("{}.cache", safe_key))
    }

    /// Ensure base directory exists
    async fn ensure_base_dir(&self) -> Result<()> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl CacheStorage for DiskStorage {
    async fn set(&self, key: &str, value: &serde_json::Value) -> Result<()> {
        self.ensure_base_dir().await?;

        let file_path = self.key_path(key);
        let json_data =
            serde_json::to_string_pretty(value).map_err(|e| CacheError::Serialization {
                message: e.to_string(),
            })?;

        fs::write(&file_path, json_data).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let file_path = self.key_path(key);

        if !file_path.exists() {
            return Ok(None);
        }

        let json_data = fs::read_to_string(&file_path).await?;
        let value: serde_json::Value =
            serde_json::from_str(&json_data).map_err(|e| CacheError::Deserialization {
                message: e.to_string(),
            })?;

        Ok(Some(value))
    }

    async fn remove(&self, key: &str) -> Result<bool> {
        let file_path = self.key_path(key);

        if file_path.exists() {
            fs::remove_file(&file_path).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn contains(&self, key: &str) -> Result<bool> {
        let file_path = self.key_path(key);
        Ok(file_path.exists())
    }

    async fn clear(&self) -> Result<()> {
        if self.base_path.exists() {
            // Remove all .cache files in the directory
            let mut entries = fs::read_dir(&self.base_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "cache") {
                    fs::remove_file(&path).await?;
                }
            }
        }
        Ok(())
    }

    async fn len(&self) -> Result<usize> {
        if !self.base_path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        let mut entries = fs::read_dir(&self.base_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().map_or(false, |ext| ext == "cache") {
                count += 1;
            }
        }
        Ok(count)
    }

    async fn size_bytes(&self) -> Result<u64> {
        if !self.base_path.exists() {
            return Ok(0);
        }

        let mut total_size = 0u64;
        let mut entries = fs::read_dir(&self.base_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().map_or(false, |ext| ext == "cache") {
                if let Ok(metadata) = entry.metadata().await {
                    total_size += metadata.len();
                }
            }
        }
        Ok(total_size)
    }

    async fn keys(&self) -> Result<Vec<String>> {
        if !self.base_path.exists() {
            return Ok(Vec::new());
        }

        let mut keys = Vec::new();
        let mut entries = fs::read_dir(&self.base_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Some(file_name) = entry.path().file_stem() {
                if let Some(name) = file_name.to_str() {
                    keys.push(name.to_string());
                }
            }
        }
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_memory_storage_basic_operations() {
        let storage = MemoryStorage::new();

        let entry = CacheEntry::new("test data", Some(std::time::Duration::from_secs(3600)));
        let json_value = serde_json::to_value(&entry).unwrap();

        // Test set and get
        storage.set("test_key", &json_value).await.unwrap();
        let retrieved_value = storage.get("test_key").await.unwrap().unwrap();
        let retrieved: CacheEntry<String> = serde_json::from_value(retrieved_value).unwrap();
        assert_eq!(retrieved.data, "test data");

        // Test contains
        assert!(storage.contains("test_key").await.unwrap());
        assert!(!storage.contains("nonexistent").await.unwrap());

        // Test remove
        assert!(storage.remove("test_key").await.unwrap());
        assert!(!storage.contains("test_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_disk_storage_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = DiskStorage::new(temp_dir.path());

        let entry = CacheEntry::new("test data", Some(std::time::Duration::from_secs(3600)));
        let json_value = serde_json::to_value(&entry).unwrap();

        // Test set and get
        storage.set("test_key", &json_value).await.unwrap();
        let retrieved_value = storage.get("test_key").await.unwrap().unwrap();
        let retrieved: CacheEntry<String> = serde_json::from_value(retrieved_value).unwrap();
        assert_eq!(retrieved.data, "test data");

        // Test contains
        assert!(storage.contains("test_key").await.unwrap());

        // Test remove
        assert!(storage.remove("test_key").await.unwrap());
        assert!(!storage.contains("test_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_cache_entry_expiration() {
        let entry = CacheEntry::new("data", Some(std::time::Duration::from_millis(1)));
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        assert!(entry.is_expired());
    }

    #[tokio::test]
    async fn test_cache_entry_ttl_remaining() {
        let entry = CacheEntry::new("data", Some(std::time::Duration::from_secs(10)));
        let remaining = entry.ttl_remaining().unwrap();
        assert!(remaining.as_secs() <= 10);
        assert!(remaining.as_secs() > 5); // Should have most of the time left
    }
}
