/// Completion history tracking for frequency and recency scoring
use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Represents a single completion usage event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionUsage {
    /// The completion label
    pub label: String,
    /// The language context
    pub language: String,
    /// Timestamp of usage
    pub timestamp: DateTime<Utc>,
    /// Number of times used
    pub frequency: u32,
}

impl CompletionUsage {
    pub fn new(label: String, language: String) -> Self {
        Self {
            label,
            language,
            timestamp: Utc::now(),
            frequency: 1,
        }
    }

    /// Update the usage record with a new occurrence
    pub fn record_usage(&mut self) {
        self.frequency += 1;
        self.timestamp = Utc::now();
    }
}

/// Completion history tracker
pub struct CompletionHistory {
    /// In-memory cache of completion usage
    usage_cache: Arc<RwLock<HashMap<String, CompletionUsage>>>,
    /// Path to persist history
    history_path: Option<PathBuf>,
}

impl CompletionHistory {
    /// Create a new completion history tracker
    pub fn new() -> Self {
        Self {
            usage_cache: Arc::new(RwLock::new(HashMap::new())),
            history_path: None,
        }
    }

    /// Create a new completion history tracker with persistence
    pub fn with_path(history_path: PathBuf) -> Self {
        Self {
            usage_cache: Arc::new(RwLock::new(HashMap::new())),
            history_path: Some(history_path),
        }
    }

    /// Record a completion usage
    pub fn record_usage(&self, label: String, language: String) -> CompletionResult<()> {
        let mut cache = self.usage_cache.write().map_err(|_| {
            CompletionError::InternalError("Failed to acquire write lock on history cache".to_string())
        })?;

        let key = format!("{}:{}", language, label);
        cache
            .entry(key)
            .and_modify(|usage| usage.record_usage())
            .or_insert_with(|| CompletionUsage::new(label, language));

        Ok(())
    }

    /// Get the frequency score for a completion (0.0 to 1.0)
    pub fn get_frequency_score(&self, label: &str, language: &str) -> CompletionResult<f32> {
        let cache = self.usage_cache.read().map_err(|_| {
            CompletionError::InternalError("Failed to acquire read lock on history cache".to_string())
        })?;

        let key = format!("{}:{}", language, label);
        if let Some(usage) = cache.get(&key) {
            // Normalize frequency to 0.0-1.0 range
            // Assume max frequency of 100 for normalization
            let score = (usage.frequency as f32 / 100.0).min(1.0);
            Ok(score)
        } else {
            Ok(0.0)
        }
    }

    /// Get the recency score for a completion (0.0 to 1.0)
    /// Recent completions get higher scores
    pub fn get_recency_score(&self, label: &str, language: &str) -> CompletionResult<f32> {
        let cache = self.usage_cache.read().map_err(|_| {
            CompletionError::InternalError("Failed to acquire read lock on history cache".to_string())
        })?;

        let key = format!("{}:{}", language, label);
        if let Some(usage) = cache.get(&key) {
            // Calculate recency score based on time since last use
            let now = Utc::now();
            let duration = now.signed_duration_since(usage.timestamp);
            let hours_ago = duration.num_hours() as f32;

            // Score decays over time: 1.0 if used now, 0.5 if used 24 hours ago, 0.0 if used 7+ days ago
            let score = if hours_ago <= 0.0 {
                1.0
            } else if hours_ago >= 168.0 {
                // 7 days
                0.0
            } else {
                1.0 - (hours_ago / 168.0)
            };

            Ok(score)
        } else {
            Ok(0.0)
        }
    }

    /// Get combined frequency and recency score
    pub fn get_usage_score(
        &self,
        label: &str,
        language: &str,
        frequency_weight: f32,
        recency_weight: f32,
    ) -> CompletionResult<f32> {
        let frequency_score = self.get_frequency_score(label, language)?;
        let recency_score = self.get_recency_score(label, language)?;

        let total_weight = frequency_weight + recency_weight;
        if total_weight == 0.0 {
            return Ok(0.0);
        }

        let combined_score =
            (frequency_score * frequency_weight + recency_score * recency_weight) / total_weight;
        Ok(combined_score)
    }

    /// Load history from file
    pub fn load(&self) -> CompletionResult<()> {
        if let Some(path) = &self.history_path {
            if path.exists() {
                let content = std::fs::read_to_string(path)
                    .map_err(CompletionError::IoError)?;
                let usages: Vec<CompletionUsage> = serde_json::from_str(&content)
                    .map_err(CompletionError::SerializationError)?;

                let mut cache = self.usage_cache.write().map_err(|_| {
                    CompletionError::InternalError(
                        "Failed to acquire write lock on history cache".to_string(),
                    )
                })?;

                for usage in usages {
                    let key = format!("{}:{}", usage.language, usage.label);
                    cache.insert(key, usage);
                }
            }
        }

        Ok(())
    }

    /// Save history to file
    pub fn save(&self) -> CompletionResult<()> {
        if let Some(path) = &self.history_path {
            let cache = self.usage_cache.read().map_err(|_| {
                CompletionError::InternalError(
                    "Failed to acquire read lock on history cache".to_string(),
                )
            })?;

            let usages: Vec<CompletionUsage> = cache.values().cloned().collect();
            let content = serde_json::to_string_pretty(&usages)
                .map_err(CompletionError::SerializationError)?;

            std::fs::write(path, content).map_err(CompletionError::IoError)?;
        }

        Ok(())
    }

    /// Clear all history
    pub fn clear(&self) -> CompletionResult<()> {
        let mut cache = self.usage_cache.write().map_err(|_| {
            CompletionError::InternalError("Failed to acquire write lock on history cache".to_string())
        })?;

        cache.clear();
        Ok(())
    }

    /// Get all usage records
    pub fn get_all_usages(&self) -> CompletionResult<Vec<CompletionUsage>> {
        let cache = self.usage_cache.read().map_err(|_| {
            CompletionError::InternalError("Failed to acquire read lock on history cache".to_string())
        })?;

        Ok(cache.values().cloned().collect())
    }
}

impl Default for CompletionHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_usage() {
        let history = CompletionHistory::new();
        assert!(history.record_usage("test".to_string(), "rust".to_string()).is_ok());
    }

    #[test]
    fn test_frequency_score_new_completion() {
        let history = CompletionHistory::new();
        let score = history.get_frequency_score("test", "rust").unwrap();
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_frequency_score_after_usage() {
        let history = CompletionHistory::new();
        history.record_usage("test".to_string(), "rust".to_string()).unwrap();
        let score = history.get_frequency_score("test", "rust").unwrap();
        assert!(score > 0.0);
    }

    #[test]
    fn test_frequency_score_multiple_usages() {
        let history = CompletionHistory::new();
        for _ in 0..5 {
            history.record_usage("test".to_string(), "rust".to_string()).unwrap();
        }
        let score = history.get_frequency_score("test", "rust").unwrap();
        assert!(score > 0.0);
    }

    #[test]
    fn test_recency_score_new_completion() {
        let history = CompletionHistory::new();
        let score = history.get_recency_score("test", "rust").unwrap();
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_recency_score_after_usage() {
        let history = CompletionHistory::new();
        history.record_usage("test".to_string(), "rust".to_string()).unwrap();
        let score = history.get_recency_score("test", "rust").unwrap();
        assert!(score > 0.9); // Should be very recent
    }

    #[test]
    fn test_combined_usage_score() {
        let history = CompletionHistory::new();
        history.record_usage("test".to_string(), "rust".to_string()).unwrap();
        let score = history.get_usage_score("test", "rust", 0.3, 0.2).unwrap();
        assert!(score > 0.0);
    }

    #[test]
    fn test_clear_history() {
        let history = CompletionHistory::new();
        history.record_usage("test".to_string(), "rust".to_string()).unwrap();
        history.clear().unwrap();
        let score = history.get_frequency_score("test", "rust").unwrap();
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_get_all_usages() {
        let history = CompletionHistory::new();
        history.record_usage("test1".to_string(), "rust".to_string()).unwrap();
        history.record_usage("test2".to_string(), "rust".to_string()).unwrap();
        let usages = history.get_all_usages().unwrap();
        assert_eq!(usages.len(), 2);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let history_path = temp_dir.path().join("history.json");

        let history = CompletionHistory::with_path(history_path.clone());
        history.record_usage("test".to_string(), "rust".to_string()).unwrap();
        assert!(history.save().is_ok());

        let history2 = CompletionHistory::with_path(history_path);
        assert!(history2.load().is_ok());
        let usages = history2.get_all_usages().unwrap();
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].label, "test");
    }
}

