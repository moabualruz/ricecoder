//! Pattern storage and retrieval

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use super::{PatternScope, RefactoringPattern};
use crate::error::{RefactoringError, Result};

/// Stores and manages refactoring patterns
pub struct PatternStore {
    global_patterns: Arc<RwLock<HashMap<String, RefactoringPattern>>>,
    project_patterns: Arc<RwLock<HashMap<String, RefactoringPattern>>>,
}

impl PatternStore {
    /// Create a new pattern store
    pub fn new() -> Self {
        Self {
            global_patterns: Arc::new(RwLock::new(HashMap::new())),
            project_patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a pattern to the store
    pub async fn add_pattern(&self, pattern: RefactoringPattern) -> Result<()> {
        match pattern.scope {
            PatternScope::Global => {
                let mut patterns = self.global_patterns.write().await;
                patterns.insert(pattern.name.clone(), pattern);
            }
            PatternScope::Project => {
                let mut patterns = self.project_patterns.write().await;
                patterns.insert(pattern.name.clone(), pattern);
            }
        }
        Ok(())
    }

    /// Get a pattern by name (project patterns take precedence)
    pub async fn get_pattern(&self, name: &str) -> Result<Option<RefactoringPattern>> {
        // Check project patterns first
        {
            let patterns = self.project_patterns.read().await;
            if let Some(pattern) = patterns.get(name) {
                return Ok(Some(pattern.clone()));
            }
        }

        // Fall back to global patterns
        let patterns = self.global_patterns.read().await;
        Ok(patterns.get(name).cloned())
    }

    /// List all patterns
    pub async fn list_patterns(&self) -> Result<Vec<RefactoringPattern>> {
        let mut patterns = vec![];

        // Add global patterns
        {
            let global = self.global_patterns.read().await;
            patterns.extend(global.values().cloned());
        }

        // Add project patterns
        {
            let project = self.project_patterns.read().await;
            patterns.extend(project.values().cloned());
        }

        Ok(patterns)
    }

    /// Remove a pattern
    pub async fn remove_pattern(&self, name: &str) -> Result<()> {
        let mut project = self.project_patterns.write().await;
        if project.remove(name).is_some() {
            return Ok(());
        }

        let mut global = self.global_patterns.write().await;
        if global.remove(name).is_some() {
            return Ok(());
        }

        Err(RefactoringError::Other(format!(
            "Pattern not found: {}",
            name
        )))
    }

    /// Clear all patterns
    pub async fn clear(&self) -> Result<()> {
        self.global_patterns.write().await.clear();
        self.project_patterns.write().await.clear();
        Ok(())
    }

    /// Get pattern count
    pub async fn pattern_count(&self) -> Result<usize> {
        let global_count = self.global_patterns.read().await.len();
        let project_count = self.project_patterns.read().await.len();
        Ok(global_count + project_count)
    }
}

impl Default for PatternStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_get_pattern() -> Result<()> {
        let store = PatternStore::new();
        let pattern = RefactoringPattern {
            name: "test".to_string(),
            description: "Test pattern".to_string(),
            template: "template".to_string(),
            parameters: vec![],
            scope: PatternScope::Global,
        };

        store.add_pattern(pattern.clone()).await?;
        let retrieved = store.get_pattern("test").await?;

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test");

        Ok(())
    }

    #[tokio::test]
    async fn test_project_patterns_take_precedence() -> Result<()> {
        let store = PatternStore::new();

        let global_pattern = RefactoringPattern {
            name: "test".to_string(),
            description: "Global pattern".to_string(),
            template: "global".to_string(),
            parameters: vec![],
            scope: PatternScope::Global,
        };

        let project_pattern = RefactoringPattern {
            name: "test".to_string(),
            description: "Project pattern".to_string(),
            template: "project".to_string(),
            parameters: vec![],
            scope: PatternScope::Project,
        };

        store.add_pattern(global_pattern).await?;
        store.add_pattern(project_pattern).await?;

        let retrieved = store.get_pattern("test").await?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().template, "project");

        Ok(())
    }

    #[tokio::test]
    async fn test_list_patterns() -> Result<()> {
        let store = PatternStore::new();

        let pattern1 = RefactoringPattern {
            name: "pattern1".to_string(),
            description: "Pattern 1".to_string(),
            template: "template1".to_string(),
            parameters: vec![],
            scope: PatternScope::Global,
        };

        let pattern2 = RefactoringPattern {
            name: "pattern2".to_string(),
            description: "Pattern 2".to_string(),
            template: "template2".to_string(),
            parameters: vec![],
            scope: PatternScope::Project,
        };

        store.add_pattern(pattern1).await?;
        store.add_pattern(pattern2).await?;

        let patterns = store.list_patterns().await?;
        assert_eq!(patterns.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_remove_pattern() -> Result<()> {
        let store = PatternStore::new();
        let pattern = RefactoringPattern {
            name: "test".to_string(),
            description: "Test pattern".to_string(),
            template: "template".to_string(),
            parameters: vec![],
            scope: PatternScope::Global,
        };

        store.add_pattern(pattern).await?;
        store.remove_pattern("test").await?;

        let retrieved = store.get_pattern("test").await?;
        assert!(retrieved.is_none());

        Ok(())
    }
}
