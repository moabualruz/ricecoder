use std::{collections::HashMap, path::PathBuf, sync::Arc};

use ricecoder_storage::manager::PathResolver;
use tokio::{fs, sync::RwLock};

/// Rule storage and persistence
use crate::error::{LearningError, Result};
use crate::models::{Rule, RuleScope};

/// Stores and retrieves rules from persistent storage
pub struct RuleStorage {
    /// In-memory cache of rules for the current scope
    cache: Arc<RwLock<HashMap<String, Rule>>>,
    /// Current scope for rule storage
    scope: RuleScope,
}

impl RuleStorage {
    /// Create a new rule storage for the specified scope
    pub fn new(scope: RuleScope) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            scope,
        }
    }

    /// Get the storage path for the current scope
    fn get_storage_path(&self) -> Result<PathBuf> {
        match self.scope {
            RuleScope::Global => {
                let global_path = PathResolver::resolve_global_path()?;
                Ok(global_path.join("rules"))
            }
            RuleScope::Project => Ok(PathBuf::from(".ricecoder/rules")),
            RuleScope::Session => Err(LearningError::PathResolutionFailed(
                "Session scope has no persistent path".to_string(),
            )),
        }
    }

    /// Ensure the storage directory exists
    async fn ensure_storage_dir(&self) -> Result<()> {
        if self.scope == RuleScope::Session {
            return Ok(());
        }

        let path = self.get_storage_path()?;
        fs::create_dir_all(&path).await.map_err(|e| {
            LearningError::RuleStorageFailed(format!("Failed to create storage directory: {}", e))
        })?;

        Ok(())
    }

    /// Get the file path for a rule
    fn get_rule_file_path(&self, rule_id: &str) -> Result<PathBuf> {
        let storage_path = self.get_storage_path()?;
        Ok(storage_path.join(format!("{}.json", rule_id)))
    }

    /// Store a rule to persistent storage
    pub async fn store_rule(&self, rule: Rule) -> Result<String> {
        if rule.scope != self.scope {
            return Err(LearningError::RuleStorageFailed(format!(
                "Rule scope {:?} does not match storage scope {:?}",
                rule.scope, self.scope
            )));
        }

        // For session scope, only store in memory
        if self.scope == RuleScope::Session {
            let mut cache = self.cache.write().await;
            let rule_id = rule.id.clone();
            cache.insert(rule_id.clone(), rule);
            return Ok(rule_id);
        }

        // For persistent scopes, write to disk
        self.ensure_storage_dir().await?;

        let rule_id = rule.id.clone();
        let file_path = self.get_rule_file_path(&rule_id)?;

        let json =
            serde_json::to_string_pretty(&rule).map_err(LearningError::SerializationError)?;

        fs::write(&file_path, json).await.map_err(|e| {
            LearningError::RuleStorageFailed(format!("Failed to write rule file: {}", e))
        })?;

        // Also update cache
        let mut cache = self.cache.write().await;
        cache.insert(rule_id.clone(), rule);

        Ok(rule_id)
    }

    /// Retrieve a rule by ID
    pub async fn get_rule(&self, rule_id: &str) -> Result<Rule> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(rule) = cache.get(rule_id) {
                return Ok(rule.clone());
            }
        }

        // For session scope, only check cache
        if self.scope == RuleScope::Session {
            return Err(LearningError::RuleNotFound(rule_id.to_string()));
        }

        // For persistent scopes, try to load from disk
        let file_path = self.get_rule_file_path(rule_id)?;

        if !file_path.exists() {
            return Err(LearningError::RuleNotFound(rule_id.to_string()));
        }

        let json = fs::read_to_string(&file_path).await.map_err(|e| {
            LearningError::RuleStorageFailed(format!("Failed to read rule file: {}", e))
        })?;

        let rule: Rule = serde_json::from_str(&json).map_err(LearningError::SerializationError)?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(rule_id.to_string(), rule.clone());
        }

        Ok(rule)
    }

    /// List all rules in storage
    pub async fn list_rules(&self) -> Result<Vec<Rule>> {
        // For session scope, return from cache
        if self.scope == RuleScope::Session {
            let cache = self.cache.read().await;
            return Ok(cache.values().cloned().collect());
        }

        // For persistent scopes, load from disk
        let storage_path = self.get_storage_path()?;

        if !storage_path.exists() {
            return Ok(Vec::new());
        }

        let mut rules = Vec::new();
        let mut dir_entries = fs::read_dir(&storage_path).await.map_err(|e| {
            LearningError::RuleStorageFailed(format!("Failed to read storage directory: {}", e))
        })?;

        while let Some(entry) = dir_entries.next_entry().await.map_err(|e| {
            LearningError::RuleStorageFailed(format!("Failed to read directory entry: {}", e))
        })? {
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                match fs::read_to_string(&path).await {
                    Ok(json) => match serde_json::from_str::<Rule>(&json) {
                        Ok(rule) => {
                            rules.push(rule);
                        }
                        Err(e) => {
                            eprintln!("Failed to deserialize rule from {:?}: {}", path, e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read rule file {:?}: {}", path, e);
                    }
                }
            }
        }

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.clear();
            for rule in &rules {
                cache.insert(rule.id.clone(), rule.clone());
            }
        }

        Ok(rules)
    }

    /// Filter rules by scope
    pub async fn filter_by_scope(&self, scope: RuleScope) -> Result<Vec<Rule>> {
        let all_rules = self.list_rules().await?;
        Ok(all_rules.into_iter().filter(|r| r.scope == scope).collect())
    }

    /// Delete a rule
    pub async fn delete_rule(&self, rule_id: &str) -> Result<()> {
        // Remove from cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(rule_id);
        }

        // For session scope, only remove from cache
        if self.scope == RuleScope::Session {
            return Ok(());
        }

        // For persistent scopes, delete from disk
        let file_path = self.get_rule_file_path(rule_id)?;

        if file_path.exists() {
            fs::remove_file(&file_path).await.map_err(|e| {
                LearningError::RuleStorageFailed(format!("Failed to delete rule file: {}", e))
            })?;
        }

        Ok(())
    }

    /// Update a rule
    pub async fn update_rule(&self, rule: Rule) -> Result<String> {
        if rule.scope != self.scope {
            return Err(LearningError::RuleStorageFailed(format!(
                "Rule scope {:?} does not match storage scope {:?}",
                rule.scope, self.scope
            )));
        }

        // Delete the old rule and store the new one
        self.delete_rule(&rule.id).await?;
        self.store_rule(rule).await
    }

    /// Get the number of rules in storage
    pub async fn rule_count(&self) -> Result<usize> {
        let rules = self.list_rules().await?;
        Ok(rules.len())
    }

    /// Clear all rules from storage
    pub async fn clear_all(&self) -> Result<()> {
        // Clear cache
        {
            let mut cache = self.cache.write().await;
            cache.clear();
        }

        // For session scope, only clear cache
        if self.scope == RuleScope::Session {
            return Ok(());
        }

        // For persistent scopes, delete all files
        let storage_path = self.get_storage_path()?;

        if storage_path.exists() {
            fs::remove_dir_all(&storage_path).await.map_err(|e| {
                LearningError::RuleStorageFailed(format!("Failed to clear storage: {}", e))
            })?;
        }

        Ok(())
    }

    /// Load all rules from storage into cache
    pub async fn load_all(&self) -> Result<()> {
        let rules = self.list_rules().await?;
        let mut cache = self.cache.write().await;
        cache.clear();
        for rule in rules {
            cache.insert(rule.id.clone(), rule);
        }
        Ok(())
    }

    /// Get the current scope
    pub fn get_scope(&self) -> RuleScope {
        self.scope
    }

    /// Get rules by pattern
    pub async fn get_rules_by_pattern(&self, pattern: &str) -> Result<Vec<Rule>> {
        let rules = self.list_rules().await?;
        Ok(rules
            .into_iter()
            .filter(|r| r.pattern.contains(pattern))
            .collect())
    }

    /// Get rules by source
    pub async fn get_rules_by_source(
        &self,
        source: crate::models::RuleSource,
    ) -> Result<Vec<Rule>> {
        let rules = self.list_rules().await?;
        Ok(rules.into_iter().filter(|r| r.source == source).collect())
    }

    /// Get rules with confidence above threshold
    pub async fn get_rules_by_confidence(&self, min_confidence: f32) -> Result<Vec<Rule>> {
        if !(0.0..=1.0).contains(&min_confidence) {
            return Err(LearningError::RuleStorageFailed(
                "Confidence must be between 0.0 and 1.0".to_string(),
            ));
        }

        let rules = self.list_rules().await?;
        Ok(rules
            .into_iter()
            .filter(|r| r.confidence >= min_confidence)
            .collect())
    }

    /// Get rules sorted by usage count (descending)
    pub async fn get_rules_by_usage(&self) -> Result<Vec<Rule>> {
        let mut rules = self.list_rules().await?;
        rules.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        Ok(rules)
    }

    /// Get rules sorted by confidence (descending)
    pub async fn get_rules_by_confidence_sorted(&self) -> Result<Vec<Rule>> {
        let mut rules = self.list_rules().await?;
        rules.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(rules)
    }

    /// Get rules with metadata matching a key-value pair
    pub async fn get_rules_by_metadata(
        &self,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<Vec<Rule>> {
        let rules = self.list_rules().await?;
        Ok(rules
            .into_iter()
            .filter(|r| r.metadata.get(key).is_some_and(|v| v == value))
            .collect())
    }

    /// Get rules created after a specific timestamp
    pub async fn get_rules_after(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Rule>> {
        let rules = self.list_rules().await?;
        Ok(rules
            .into_iter()
            .filter(|r| r.created_at > timestamp)
            .collect())
    }

    /// Get rules updated after a specific timestamp
    pub async fn get_rules_updated_after(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Rule>> {
        let rules = self.list_rules().await?;
        Ok(rules
            .into_iter()
            .filter(|r| r.updated_at > timestamp)
            .collect())
    }

    /// Get rules with usage count above threshold
    pub async fn get_rules_by_usage_count(&self, min_usage: u64) -> Result<Vec<Rule>> {
        let rules = self.list_rules().await?;
        Ok(rules
            .into_iter()
            .filter(|r| r.usage_count >= min_usage)
            .collect())
    }

    /// Get rules with success rate above threshold
    pub async fn get_rules_by_success_rate(&self, min_success_rate: f32) -> Result<Vec<Rule>> {
        if !(0.0..=1.0).contains(&min_success_rate) {
            return Err(LearningError::RuleStorageFailed(
                "Success rate must be between 0.0 and 1.0".to_string(),
            ));
        }

        let rules = self.list_rules().await?;
        Ok(rules
            .into_iter()
            .filter(|r| r.success_rate >= min_success_rate)
            .collect())
    }

    /// Get rules with specific version
    pub async fn get_rules_by_version(&self, version: u32) -> Result<Vec<Rule>> {
        let rules = self.list_rules().await?;
        Ok(rules.into_iter().filter(|r| r.version == version).collect())
    }

    /// Get metadata for all rules (without full rule data)
    pub async fn get_rules_metadata(&self) -> Result<Vec<(String, String, f32, u64)>> {
        let rules = self.list_rules().await?;
        Ok(rules
            .into_iter()
            .map(|r| (r.id, r.pattern, r.confidence, r.usage_count))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RuleSource;

    #[tokio::test]
    async fn test_session_rule_storage() {
        let storage = RuleStorage::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Session,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        let result = storage.store_rule(rule).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), rule_id);

        let retrieved = storage.get_rule(&rule_id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().id, rule_id);
    }

    #[tokio::test]
    async fn test_session_list_rules() {
        let storage = RuleStorage::new(RuleScope::Session);

        let rule1 = Rule::new(
            RuleScope::Session,
            "pattern1".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );

        let rule2 = Rule::new(
            RuleScope::Session,
            "pattern2".to_string(),
            "action2".to_string(),
            RuleSource::Manual,
        );

        storage.store_rule(rule1).await.unwrap();
        storage.store_rule(rule2).await.unwrap();

        let rules = storage.list_rules().await.unwrap();
        assert_eq!(rules.len(), 2);
    }

    #[tokio::test]
    async fn test_session_delete_rule() {
        let storage = RuleStorage::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Session,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        storage.store_rule(rule).await.unwrap();

        assert!(storage.get_rule(&rule_id).await.is_ok());

        storage.delete_rule(&rule_id).await.unwrap();

        assert!(storage.get_rule(&rule_id).await.is_err());
    }

    #[tokio::test]
    async fn test_session_update_rule() {
        let storage = RuleStorage::new(RuleScope::Session);

        let mut rule = Rule::new(
            RuleScope::Session,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        storage.store_rule(rule.clone()).await.unwrap();

        rule.pattern = "new_pattern".to_string();
        storage.update_rule(rule).await.unwrap();

        let retrieved = storage.get_rule(&rule_id).await.unwrap();
        assert_eq!(retrieved.pattern, "new_pattern");
    }

    #[tokio::test]
    async fn test_session_rule_count() {
        let storage = RuleStorage::new(RuleScope::Session);

        assert_eq!(storage.rule_count().await.unwrap(), 0);

        let rule1 = Rule::new(
            RuleScope::Session,
            "pattern1".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );

        let rule2 = Rule::new(
            RuleScope::Session,
            "pattern2".to_string(),
            "action2".to_string(),
            RuleSource::Manual,
        );

        storage.store_rule(rule1).await.unwrap();
        storage.store_rule(rule2).await.unwrap();

        assert_eq!(storage.rule_count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_session_clear_all() {
        let storage = RuleStorage::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Session,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        storage.store_rule(rule).await.unwrap();
        assert_eq!(storage.rule_count().await.unwrap(), 1);

        storage.clear_all().await.unwrap();
        assert_eq!(storage.rule_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_filter_by_scope() {
        let storage = RuleStorage::new(RuleScope::Session);

        let rule1 = Rule::new(
            RuleScope::Session,
            "pattern1".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );

        let rule2 = Rule::new(
            RuleScope::Session,
            "pattern2".to_string(),
            "action2".to_string(),
            RuleSource::Manual,
        );

        storage.store_rule(rule1).await.unwrap();
        storage.store_rule(rule2).await.unwrap();

        let filtered = storage.filter_by_scope(RuleScope::Session).await.unwrap();
        assert_eq!(filtered.len(), 2);
    }

    #[tokio::test]
    async fn test_wrong_scope_error() {
        let storage = RuleStorage::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let result = storage.store_rule(rule).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_nonexistent_rule() {
        let storage = RuleStorage::new(RuleScope::Session);
        let result = storage.get_rule("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_rules_by_pattern() {
        let storage = RuleStorage::new(RuleScope::Session);

        let rule1 = Rule::new(
            RuleScope::Session,
            "pattern_a".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );

        let rule2 = Rule::new(
            RuleScope::Session,
            "pattern_b".to_string(),
            "action2".to_string(),
            RuleSource::Manual,
        );

        let rule3 = Rule::new(
            RuleScope::Session,
            "pattern_a_extended".to_string(),
            "action3".to_string(),
            RuleSource::Learned,
        );

        storage.store_rule(rule1).await.unwrap();
        storage.store_rule(rule2).await.unwrap();
        storage.store_rule(rule3).await.unwrap();

        let pattern_a_rules = storage.get_rules_by_pattern("pattern_a").await.unwrap();
        assert_eq!(pattern_a_rules.len(), 2);
    }

    #[tokio::test]
    async fn test_get_rules_by_source() {
        let storage = RuleStorage::new(RuleScope::Session);

        let rule1 = Rule::new(
            RuleScope::Session,
            "pattern1".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );

        let rule2 = Rule::new(
            RuleScope::Session,
            "pattern2".to_string(),
            "action2".to_string(),
            RuleSource::Manual,
        );

        let rule3 = Rule::new(
            RuleScope::Session,
            "pattern3".to_string(),
            "action3".to_string(),
            RuleSource::Learned,
        );

        storage.store_rule(rule1).await.unwrap();
        storage.store_rule(rule2).await.unwrap();
        storage.store_rule(rule3).await.unwrap();

        let learned_rules = storage
            .get_rules_by_source(RuleSource::Learned)
            .await
            .unwrap();
        assert_eq!(learned_rules.len(), 2);

        let manual_rules = storage
            .get_rules_by_source(RuleSource::Manual)
            .await
            .unwrap();
        assert_eq!(manual_rules.len(), 1);
    }

    #[tokio::test]
    async fn test_get_rules_by_confidence() {
        let storage = RuleStorage::new(RuleScope::Session);

        let mut rule1 = Rule::new(
            RuleScope::Session,
            "pattern1".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );
        rule1.confidence = 0.9;

        let mut rule2 = Rule::new(
            RuleScope::Session,
            "pattern2".to_string(),
            "action2".to_string(),
            RuleSource::Manual,
        );
        rule2.confidence = 0.5;

        let mut rule3 = Rule::new(
            RuleScope::Session,
            "pattern3".to_string(),
            "action3".to_string(),
            RuleSource::Learned,
        );
        rule3.confidence = 0.7;

        storage.store_rule(rule1).await.unwrap();
        storage.store_rule(rule2).await.unwrap();
        storage.store_rule(rule3).await.unwrap();

        let high_confidence = storage.get_rules_by_confidence(0.7).await.unwrap();
        assert_eq!(high_confidence.len(), 2);

        let very_high_confidence = storage.get_rules_by_confidence(0.8).await.unwrap();
        assert_eq!(very_high_confidence.len(), 1);
    }

    #[tokio::test]
    async fn test_get_rules_by_usage() {
        let storage = RuleStorage::new(RuleScope::Session);

        let mut rule1 = Rule::new(
            RuleScope::Session,
            "pattern1".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );
        rule1.usage_count = 10;

        let mut rule2 = Rule::new(
            RuleScope::Session,
            "pattern2".to_string(),
            "action2".to_string(),
            RuleSource::Manual,
        );
        rule2.usage_count = 5;

        let mut rule3 = Rule::new(
            RuleScope::Session,
            "pattern3".to_string(),
            "action3".to_string(),
            RuleSource::Learned,
        );
        rule3.usage_count = 20;

        storage.store_rule(rule1).await.unwrap();
        storage.store_rule(rule2).await.unwrap();
        storage.store_rule(rule3).await.unwrap();

        let sorted = storage.get_rules_by_usage().await.unwrap();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].usage_count, 20);
        assert_eq!(sorted[1].usage_count, 10);
        assert_eq!(sorted[2].usage_count, 5);
    }

    #[tokio::test]
    async fn test_get_rules_by_usage_count() {
        let storage = RuleStorage::new(RuleScope::Session);

        let mut rule1 = Rule::new(
            RuleScope::Session,
            "pattern1".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );
        rule1.usage_count = 10;

        let mut rule2 = Rule::new(
            RuleScope::Session,
            "pattern2".to_string(),
            "action2".to_string(),
            RuleSource::Manual,
        );
        rule2.usage_count = 5;

        storage.store_rule(rule1).await.unwrap();
        storage.store_rule(rule2).await.unwrap();

        let high_usage = storage.get_rules_by_usage_count(8).await.unwrap();
        assert_eq!(high_usage.len(), 1);
    }

    #[tokio::test]
    async fn test_get_rules_by_success_rate() {
        let storage = RuleStorage::new(RuleScope::Session);

        let mut rule1 = Rule::new(
            RuleScope::Session,
            "pattern1".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );
        rule1.success_rate = 0.95;

        let mut rule2 = Rule::new(
            RuleScope::Session,
            "pattern2".to_string(),
            "action2".to_string(),
            RuleSource::Manual,
        );
        rule2.success_rate = 0.5;

        storage.store_rule(rule1).await.unwrap();
        storage.store_rule(rule2).await.unwrap();

        let high_success = storage.get_rules_by_success_rate(0.8).await.unwrap();
        assert_eq!(high_success.len(), 1);
    }

    #[tokio::test]
    async fn test_get_rules_metadata() {
        let storage = RuleStorage::new(RuleScope::Session);

        let mut rule1 = Rule::new(
            RuleScope::Session,
            "pattern1".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );
        rule1.confidence = 0.9;
        rule1.usage_count = 10;

        let mut rule2 = Rule::new(
            RuleScope::Session,
            "pattern2".to_string(),
            "action2".to_string(),
            RuleSource::Manual,
        );
        rule2.confidence = 0.5;
        rule2.usage_count = 5;

        storage.store_rule(rule1).await.unwrap();
        storage.store_rule(rule2).await.unwrap();

        let metadata = storage.get_rules_metadata().await.unwrap();
        assert_eq!(metadata.len(), 2);
    }

    #[tokio::test]
    async fn test_invalid_confidence_threshold() {
        let storage = RuleStorage::new(RuleScope::Session);
        let result = storage.get_rules_by_confidence(1.5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_success_rate_threshold() {
        let storage = RuleStorage::new(RuleScope::Session);
        let result = storage.get_rules_by_success_rate(-0.1).await;
        assert!(result.is_err());
    }
}
