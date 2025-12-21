//! Synchronization manager for cross-project updates

use crate::error::{OrchestrationError, Result};
use crate::models::{Operation, Transaction, TransactionState};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SyncConflict {
    pub project: String,
    pub description: String,
    pub resolution_strategy: ConflictResolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    KeepExisting,
    UseNew,
    Manual,
    Merge,
}

#[derive(Debug, Clone)]
pub struct SyncLogEntry {
    pub timestamp: String,
    pub project: String,
    pub operation: String,
    pub status: String,
    pub details: String,
}

pub struct SyncManager {
    transactions: Arc<RwLock<HashMap<String, Transaction>>>,
    sync_log: Arc<RwLock<Vec<SyncLogEntry>>>,
    conflicts: Arc<RwLock<Vec<SyncConflict>>>,
}

impl SyncManager {
    pub fn new() -> Self {
        debug!("Creating SyncManager");
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            sync_log: Arc::new(RwLock::new(Vec::new())),
            conflicts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_transaction(&self, operations: Vec<Operation>) -> Result<String> {
        let txn_id = Uuid::new_v4().to_string();
        info!("Starting transaction: {}", txn_id);

        let transaction = Transaction {
            id: txn_id.clone(),
            operations,
            state: TransactionState::Pending,
        };

        self.transactions
            .write()
            .await
            .insert(txn_id.clone(), transaction);

        self.log_operation(
            "system",
            "transaction_start",
            "success",
            &format!("Transaction {} started", txn_id),
        )
        .await;

        Ok(txn_id)
    }

    pub async fn get_transaction(&self, txn_id: &str) -> Result<Option<Transaction>> {
        Ok(self.transactions.read().await.get(txn_id).cloned())
    }

    pub async fn commit_transaction(&self, txn_id: &str) -> Result<()> {
        info!("Committing transaction: {}", txn_id);

        {
            let mut transactions = self.transactions.write().await;

            let transaction = transactions.get_mut(txn_id).ok_or_else(|| {
                OrchestrationError::TransactionFailed(format!("Transaction not found: {}", txn_id))
            })?;

            if transaction.state != TransactionState::Pending {
                return Err(OrchestrationError::TransactionFailed(format!(
                    "Cannot commit transaction in state: {:?}",
                    transaction.state
                )));
            }

            for operation in &transaction.operations {
                self.validate_operation(operation).await?;
            }

            transaction.state = TransactionState::Committed;
        }

        self.log_operation(
            "system",
            "transaction_commit",
            "success",
            &format!("Transaction {} committed", txn_id),
        )
        .await;

        Ok(())
    }

    pub async fn rollback_transaction(&self, txn_id: &str) -> Result<()> {
        info!("Rolling back transaction: {}", txn_id);

        {
            let mut transactions = self.transactions.write().await;

            let transaction = transactions.get_mut(txn_id).ok_or_else(|| {
                OrchestrationError::RollbackFailed(format!("Transaction not found: {}", txn_id))
            })?;

            if transaction.state == TransactionState::RolledBack {
                return Ok(());
            }

            for operation in transaction.operations.iter().rev() {
                self.revert_operation(operation).await?;
            }

            transaction.state = TransactionState::RolledBack;
        }

        self.log_operation(
            "system",
            "transaction_rollback",
            "success",
            &format!("Transaction {} rolled back", txn_id),
        )
        .await;

        Ok(())
    }

    pub async fn sync_configuration(
        &self,
        source_project: &str,
        target_projects: &[String],
        config_data: serde_json::Value,
    ) -> Result<()> {
        info!(
            "Synchronizing configuration from {} to {} projects",
            source_project,
            target_projects.len()
        );

        for target in target_projects {
            self.log_operation(
                target,
                "config_sync",
                "in_progress",
                &format!("Syncing config from {}", source_project),
            )
            .await;

            if let Err(e) = self.check_config_conflicts(target, &config_data).await {
                self.conflicts.write().await.push(SyncConflict {
                    project: target.clone(),
                    description: e.to_string(),
                    resolution_strategy: ConflictResolution::Manual,
                });

                self.log_operation(
                    target,
                    "config_sync",
                    "conflict",
                    &format!("Conflict detected: {}", e),
                )
                .await;

                continue;
            }

            self.log_operation(
                target,
                "config_sync",
                "success",
                "Configuration synchronized",
            )
            .await;
        }

        Ok(())
    }

    pub async fn sync_version_updates(
        &self,
        source_project: &str,
        new_version: &str,
        dependent_projects: &[String],
    ) -> Result<()> {
        info!(
            "Synchronizing version {} from {} to {} projects",
            new_version,
            source_project,
            dependent_projects.len()
        );

        for target in dependent_projects {
            self.log_operation(
                target,
                "version_sync",
                "in_progress",
                &format!(
                    "Updating to version {} from {}",
                    new_version, source_project
                ),
            )
            .await;

            if let Err(e) = self
                .validate_version_compatibility(target, new_version)
                .await
            {
                self.log_operation(
                    target,
                    "version_sync",
                    "failed",
                    &format!("Version compatibility check failed: {}", e),
                )
                .await;

                return Err(e);
            }

            self.log_operation(
                target,
                "version_sync",
                "success",
                &format!("Version updated to {}", new_version),
            )
            .await;
        }

        Ok(())
    }

    pub async fn detect_conflicts(
        &self,
        project: &str,
        incoming_data: &serde_json::Value,
    ) -> Result<Vec<SyncConflict>> {
        debug!("Detecting conflicts for project: {}", project);

        let mut detected_conflicts = Vec::new();

        if let Err(e) = self.check_config_conflicts(project, incoming_data).await {
            detected_conflicts.push(SyncConflict {
                project: project.to_string(),
                description: e.to_string(),
                resolution_strategy: ConflictResolution::Manual,
            });
        }

        Ok(detected_conflicts)
    }

    pub async fn resolve_conflict(
        &self,
        conflict: &SyncConflict,
        strategy: ConflictResolution,
    ) -> Result<()> {
        info!(
            "Resolving conflict in {} using strategy: {:?}",
            conflict.project, strategy
        );

        self.log_operation(
            &conflict.project,
            "conflict_resolution",
            "success",
            &format!("Conflict resolved using strategy: {:?}", strategy),
        )
        .await;

        let mut conflicts = self.conflicts.write().await;
        conflicts
            .retain(|c| c.project != conflict.project || c.description != conflict.description);

        Ok(())
    }

    pub async fn get_conflicts(&self) -> Vec<SyncConflict> {
        self.conflicts.read().await.clone()
    }

    pub async fn get_sync_log(&self) -> Vec<SyncLogEntry> {
        self.sync_log.read().await.clone()
    }

    pub async fn clear_sync_log(&self) {
        self.sync_log.write().await.clear();
    }

    async fn log_operation(&self, project: &str, operation: &str, status: &str, details: &str) {
        let entry = SyncLogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            project: project.to_string(),
            operation: operation.to_string(),
            status: status.to_string(),
            details: details.to_string(),
        };

        self.sync_log.write().await.push(entry);
    }

    async fn validate_operation(&self, _operation: &Operation) -> Result<()> {
        debug!("Validating operation");
        Ok(())
    }

    async fn revert_operation(&self, _operation: &Operation) -> Result<()> {
        debug!("Reverting operation");
        Ok(())
    }

    async fn check_config_conflicts(
        &self,
        _project: &str,
        _config_data: &serde_json::Value,
    ) -> Result<()> {
        Ok(())
    }

    async fn validate_version_compatibility(&self, _project: &str, _version: &str) -> Result<()> {
        Ok(())
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_manager_creation() {
        let manager = SyncManager::new();
        assert_eq!(manager.get_conflicts().await.len(), 0);
    }

    #[tokio::test]
    async fn test_start_transaction() {
        let manager = SyncManager::new();
        let txn_id = manager.start_transaction(vec![]).await.unwrap();
        assert!(!txn_id.is_empty());
    }

    #[tokio::test]
    async fn test_commit_transaction() {
        let manager = SyncManager::new();
        let txn_id = manager.start_transaction(vec![]).await.unwrap();
        manager.commit_transaction(&txn_id).await.unwrap();
        let txn = manager.get_transaction(&txn_id).await.unwrap();
        assert_eq!(txn.unwrap().state, TransactionState::Committed);
    }

    #[tokio::test]
    async fn test_rollback_transaction() {
        let manager = SyncManager::new();
        let txn_id = manager.start_transaction(vec![]).await.unwrap();
        manager.rollback_transaction(&txn_id).await.unwrap();
        let txn = manager.get_transaction(&txn_id).await.unwrap();
        assert_eq!(txn.unwrap().state, TransactionState::RolledBack);
    }
}
