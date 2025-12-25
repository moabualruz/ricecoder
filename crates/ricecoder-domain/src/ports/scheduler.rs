//! Task scheduler port interfaces
//!
//! This module defines the contracts for task scheduling operations.
//! Implementations in infrastructure crates provide concrete schedulers
//! (tokio-based, thread pool, distributed, etc.).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::errors::*;

// ============================================================================
// Scheduler Value Objects
// ============================================================================

/// Unique task identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl TaskId {
    /// Create a new task ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a random task ID
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Task execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is pending execution
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with error
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task is paused
    Paused,
}

/// Task priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Lowest priority
    Low = 0,
    /// Normal priority (default)
    Normal = 1,
    /// High priority
    High = 2,
    /// Critical priority - execute immediately
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// Task execution schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskSchedule {
    /// Execute immediately
    Immediate,
    /// Execute after a delay
    Delayed(Duration),
    /// Execute at a specific time
    At(chrono::DateTime<chrono::Utc>),
    /// Execute repeatedly at interval
    Recurring {
        /// Interval between executions
        interval: Duration,
        /// Maximum number of executions (None = unlimited)
        max_runs: Option<u32>,
    },
    /// Cron-like schedule expression
    Cron(String),
}

/// Task metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// Task identifier
    pub id: TaskId,
    /// Task name/description
    pub name: String,
    /// Current status
    pub status: TaskStatus,
    /// Task priority
    pub priority: TaskPriority,
    /// Execution schedule
    pub schedule: TaskSchedule,
    /// When task was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When task started running (if started)
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// When task completed (if completed)
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Maximum retries allowed
    pub max_retries: u32,
    /// Error message if failed
    pub error_message: Option<String>,
}

impl TaskInfo {
    /// Create a new task info
    pub fn new(name: impl Into<String>, schedule: TaskSchedule) -> Self {
        Self {
            id: TaskId::generate(),
            name: name.into(),
            status: TaskStatus::Pending,
            priority: TaskPriority::default(),
            schedule,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
        }
    }

    /// Check if task can be retried
    pub fn can_retry(&self) -> bool {
        self.status == TaskStatus::Failed && self.retry_count < self.max_retries
    }

    /// Duration since creation
    pub fn age(&self) -> Duration {
        let now = chrono::Utc::now();
        (now - self.created_at)
            .to_std()
            .unwrap_or(Duration::from_secs(0))
    }

    /// Execution duration (if started)
    pub fn execution_duration(&self) -> Option<Duration> {
        let started = self.started_at?;
        let ended = self.completed_at.unwrap_or_else(chrono::Utc::now);
        (ended - started).to_std().ok()
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// Total tasks scheduled
    pub total_scheduled: u64,
    /// Tasks currently pending
    pub pending_count: usize,
    /// Tasks currently running
    pub running_count: usize,
    /// Tasks completed successfully
    pub completed_count: u64,
    /// Tasks that failed
    pub failed_count: u64,
    /// Tasks cancelled
    pub cancelled_count: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
}

// ============================================================================
// Task Scheduler Ports (ISP-Compliant)
// ============================================================================

/// Task submission interface (ISP: 5 methods max)
#[async_trait]
pub trait TaskSubmitter: Send + Sync {
    /// Schedule a task for execution
    async fn schedule(&self, task: TaskInfo) -> DomainResult<TaskId>;

    /// Schedule a task to run immediately
    async fn schedule_immediate(&self, name: &str) -> DomainResult<TaskId> {
        let task = TaskInfo::new(name, TaskSchedule::Immediate);
        self.schedule(task).await
    }

    /// Schedule a delayed task
    async fn schedule_delayed(&self, name: &str, delay: Duration) -> DomainResult<TaskId> {
        let task = TaskInfo::new(name, TaskSchedule::Delayed(delay));
        self.schedule(task).await
    }

    /// Schedule a recurring task
    async fn schedule_recurring(
        &self,
        name: &str,
        interval: Duration,
        max_runs: Option<u32>,
    ) -> DomainResult<TaskId> {
        let task = TaskInfo::new(name, TaskSchedule::Recurring { interval, max_runs });
        self.schedule(task).await
    }
}

/// Task control interface (ISP: 5 methods max)
#[async_trait]
pub trait TaskController: Send + Sync {
    /// Cancel a scheduled or running task
    async fn cancel(&self, task_id: &TaskId) -> DomainResult<bool>;

    /// Pause a task (if supported)
    async fn pause(&self, task_id: &TaskId) -> DomainResult<bool>;

    /// Resume a paused task
    async fn resume(&self, task_id: &TaskId) -> DomainResult<bool>;

    /// Retry a failed task
    async fn retry(&self, task_id: &TaskId) -> DomainResult<bool>;
}

/// Task query interface (ISP: 5 methods max)
#[async_trait]
pub trait TaskQuery: Send + Sync {
    /// Get task info by ID
    async fn get(&self, task_id: &TaskId) -> DomainResult<Option<TaskInfo>>;

    /// Get all tasks with a specific status
    async fn by_status(&self, status: TaskStatus) -> DomainResult<Vec<TaskInfo>>;

    /// Get scheduler statistics
    fn statistics(&self) -> SchedulerStats;

    /// Check if a task exists
    async fn exists(&self, task_id: &TaskId) -> DomainResult<bool> {
        Ok(self.get(task_id).await?.is_some())
    }
}

/// Combined task scheduler trait
pub trait TaskScheduler: TaskSubmitter + TaskController + TaskQuery {}

/// Blanket implementation for types implementing all sub-traits
impl<T> TaskScheduler for T where T: TaskSubmitter + TaskController + TaskQuery {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_generation() {
        let id1 = TaskId::generate();
        let id2 = TaskId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_task_info_new() {
        let task = TaskInfo::new("test-task", TaskSchedule::Immediate);
        assert_eq!(task.name, "test-task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, TaskPriority::Normal);
        assert_eq!(task.retry_count, 0);
        assert_eq!(task.max_retries, 3);
    }

    #[test]
    fn test_task_can_retry() {
        let mut task = TaskInfo::new("test-task", TaskSchedule::Immediate);
        assert!(!task.can_retry()); // Not failed yet

        task.status = TaskStatus::Failed;
        assert!(task.can_retry()); // Failed and retries available

        task.retry_count = 3;
        assert!(!task.can_retry()); // Max retries reached
    }

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Low < TaskPriority::Normal);
        assert!(TaskPriority::Normal < TaskPriority::High);
        assert!(TaskPriority::High < TaskPriority::Critical);
    }
}
