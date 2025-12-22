//! Async processing optimizations and Tokio task scheduling
//!
//! This module provides optimizations for async processing including:
//! - Task scheduling and prioritization
//! - Concurrent execution limits
//! - Resource-aware task spawning
//! - Async processing pipelines

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};

use futures::FutureExt;
use tokio::{
    sync::{mpsc, RwLock, Semaphore},
    task::{self, JoinHandle},
};
use tracing::{debug, info, warn};

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

/// Async task with priority and metadata
pub struct PrioritizedTask {
    pub id: String,
    pub priority: TaskPriority,
    pub task: Box<dyn Fn() -> futures::future::BoxFuture<'static, ()> + Send + 'static>,
    pub created_at: Instant,
    pub metadata: HashMap<String, String>,
}

impl PrioritizedTask {
    pub fn new<F, Fut>(
        id: String,
        priority: TaskPriority,
        task: F,
        metadata: HashMap<String, String>,
    ) -> Self
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        Self {
            id,
            priority,
            task: Box::new(move || task().boxed()),
            created_at: Instant::now(),
            metadata,
        }
    }
}

/// Async task scheduler with prioritization
pub struct TaskScheduler {
    task_queue: Arc<RwLock<HashMap<TaskPriority, VecDeque<PrioritizedTask>>>>,
    concurrency_limit: Arc<Semaphore>,
    active_tasks: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    max_queue_size: usize,
    task_counter: Arc<std::sync::atomic::AtomicU64>,
}

impl TaskScheduler {
    /// Create a new task scheduler
    pub fn new(max_concurrent: usize, max_queue_size: usize) -> Self {
        let mut task_queue = HashMap::new();
        for &priority in &[
            TaskPriority::Critical,
            TaskPriority::High,
            TaskPriority::Normal,
            TaskPriority::Low,
        ] {
            task_queue.insert(priority, VecDeque::new());
        }

        Self {
            task_queue: Arc::new(RwLock::new(task_queue)),
            concurrency_limit: Arc::new(Semaphore::new(max_concurrent)),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            max_queue_size,
            task_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Schedule a task with priority
    pub async fn schedule_task(&self, task: PrioritizedTask) -> Result<String, TaskSchedulerError> {
        let task_id = task.id.clone();

        let mut queue = self.task_queue.write().await;

        // Check queue size limits
        let total_queued = queue.values().map(|q| q.len()).sum::<usize>();
        if total_queued >= self.max_queue_size {
            return Err(TaskSchedulerError::QueueFull);
        }

        // Add to appropriate priority queue
        if let Some(priority_queue) = queue.get_mut(&task.priority) {
            let priority = task.priority;
            priority_queue.push_back(task);
            debug!("Scheduled task {} with priority {:?}", task_id, priority);
        } else {
            return Err(TaskSchedulerError::InvalidPriority);
        }

        // Try to start processing if possible
        drop(queue);
        self.try_process_next_task().await;

        Ok(task_id)
    }

    /// Process the next highest priority task
    async fn try_process_next_task(&self) {
        // Acquire concurrency permit (await if necessary)
        let permit = self.concurrency_limit.clone().acquire_owned().await;

        let task = {
            let mut queue = self.task_queue.write().await;

            // Find highest priority non-empty queue
            let mut found_task = None;
            for priority in &[
                TaskPriority::Critical,
                TaskPriority::High,
                TaskPriority::Normal,
                TaskPriority::Low,
            ] {
                if let Some(priority_queue) = queue.get_mut(priority) {
                    if let Some(task) = priority_queue.pop_front() {
                        found_task = Some(task);
                        break;
                    }
                }
            }
            found_task
        };

        if let Some(task) = task {
            // Spawn the task
            let active_tasks = self.active_tasks.clone();
            let task_id = task.id.clone();
            let task_fn = task.task;

            let task_id_clone = task_id.clone();
            let handle = task::spawn(async move {
                let start_time = Instant::now();
                debug!("Starting task {}", task_id_clone);

                // Execute the task
                let future = task_fn();
                future.await;

                let duration = start_time.elapsed();
                debug!("Completed task {} in {:?}", task_id_clone, duration);

                // Remove from active tasks
                let mut active = active_tasks.write().await;
                active.remove(&task_id_clone);

                // Release permit
                drop(permit);
            });

            // Track active task
            let mut active = self.active_tasks.write().await;
            active.insert(task_id, handle);
        } else {
            // No tasks to process, release permit
            drop(permit);
        }
    }

    /// Wait for a specific task to complete
    pub async fn wait_for_task(&self, task_id: &str) -> Result<(), TaskSchedulerError> {
        let handle = {
            let mut active = self.active_tasks.write().await;
            active.remove(task_id)
        };

        if let Some(handle) = handle {
            handle.await.map_err(|_| TaskSchedulerError::TaskPanic)?;
        } else {
            // Check if task is queued
            let queued = self.task_queue.read().await;
            let is_queued = queued.values().any(|q| q.iter().any(|t| t.id == task_id));

            if !is_queued {
                return Err(TaskSchedulerError::TaskNotFound);
            }

            // Task is queued but not active, wait for it to become active
            loop {
                tokio::time::sleep(Duration::from_millis(10)).await;
                let mut active = self.active_tasks.write().await;
                if let Some(handle) = active.remove(task_id) {
                    handle.await.map_err(|_| TaskSchedulerError::TaskPanic)?;
                    break;
                }
            }
        }

        Ok(())
    }

    /// Cancel a queued or running task
    pub async fn cancel_task(&self, task_id: &str) -> Result<bool, TaskSchedulerError> {
        // Try to remove from queue first
        let mut queue = self.task_queue.write().await;
        let mut removed_from_queue = false;

        for priority_queue in queue.values_mut() {
            if let Some(pos) = priority_queue.iter().position(|t| t.id == task_id) {
                priority_queue.remove(pos);
                removed_from_queue = true;
                break;
            }
        }

        if removed_from_queue {
            return Ok(true);
        }

        // Try to abort active task
        let mut active = self.active_tasks.write().await;
        if let Some(handle) = active.remove(task_id) {
            handle.abort();
            return Ok(true);
        }

        Err(TaskSchedulerError::TaskNotFound)
    }

    /// Get scheduler statistics
    pub async fn stats(&self) -> SchedulerStats {
        let queue = self.task_queue.read().await;
        let active = self.active_tasks.read().await;

        let queued_tasks = queue.values().map(|q| q.len()).sum::<usize>();

        SchedulerStats {
            active_tasks: active.len(),
            queued_tasks,
            available_permits: self.concurrency_limit.available_permits(),
            total_processed: self.task_counter.load(std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// Shutdown the scheduler gracefully
    pub async fn shutdown(&self) -> Result<(), TaskSchedulerError> {
        info!("Shutting down task scheduler...");

        // Cancel all queued tasks
        let mut queue = self.task_queue.write().await;
        for priority_queue in queue.values_mut() {
            priority_queue.clear();
        }

        // Wait for active tasks to complete
        let active_handles: Vec<JoinHandle<()>> = {
            let mut active = self.active_tasks.write().await;
            active.drain().map(|(_, h)| h).collect()
        };

        for handle in active_handles {
            let _ = handle.await;
        }

        info!("Task scheduler shutdown complete");
        Ok(())
    }
}

/// Scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub active_tasks: usize,
    pub queued_tasks: usize,
    pub available_permits: usize,
    pub total_processed: u64,
}

/// Task scheduler errors
#[derive(Debug, thiserror::Error)]
pub enum TaskSchedulerError {
    #[error("Task queue is full")]
    QueueFull,
    #[error("Invalid task priority")]
    InvalidPriority,
    #[error("Task not found")]
    TaskNotFound,
    #[error("Task panicked during execution")]
    TaskPanic,
}

/// Async processing pipeline for batch operations
pub struct AsyncPipeline<T, R> {
    processors:
        Vec<Arc<dyn Fn(T) -> futures::future::BoxFuture<'static, R> + Send + Sync + 'static>>,
    concurrency_limit: usize,
}

impl<T, R> AsyncPipeline<T, R>
where
    T: Send + 'static,
    R: Send + 'static,
{
    /// Create a new async pipeline
    pub fn new(concurrency_limit: usize) -> Self {
        Self {
            processors: Vec::new(),
            concurrency_limit,
        }
    }

    /// Add a processing stage
    pub fn add_stage<F, Fut>(mut self, processor: F) -> Self
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = R> + Send + 'static,
    {
        self.processors
            .push(Arc::new(move |input| processor(input).boxed())
                as Arc<
                    dyn Fn(T) -> futures::future::BoxFuture<'static, R> + Send + Sync + 'static,
                >);
        self
    }

    /// Process items through the pipeline
    pub async fn process_batch(&self, items: Vec<T>) -> Result<Vec<R>, AsyncPipelineError> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));
        let mut handles = Vec::new();

        for item in items {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| AsyncPipelineError::ConcurrencyLimitReached)?;

            let processor = self
                .processors
                .first()
                .ok_or(AsyncPipelineError::NoProcessors)?
                .clone();

            let handle = task::spawn(async move {
                let result = processor(item).await;
                drop(permit);
                result
            });

            handles.push(handle);
        }

        // Collect results
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.map_err(|_| AsyncPipelineError::TaskPanic)?);
        }

        Ok(results)
    }
}

/// Async pipeline errors
#[derive(Debug, thiserror::Error)]
pub enum AsyncPipelineError {
    #[error("No processors configured in pipeline")]
    NoProcessors,
    #[error("Concurrency limit reached")]
    ConcurrencyLimitReached,
    #[error("Task panicked during execution")]
    TaskPanic,
}

/// Resource-aware task spawner
pub struct ResourceAwareSpawner {
    cpu_semaphore: Arc<Semaphore>,
    memory_semaphore: Arc<Semaphore>,
    active_tasks: Arc<RwLock<HashMap<String, TaskResourceUsage>>>,
}

#[derive(Debug, Clone)]
struct TaskResourceUsage {
    cpu_cores: usize,
    memory_mb: usize,
    start_time: Instant,
}

impl ResourceAwareSpawner {
    /// Create a new resource-aware spawner
    pub fn new(max_cpu_cores: usize, max_memory_mb: usize) -> Self {
        Self {
            cpu_semaphore: Arc::new(Semaphore::new(max_cpu_cores)),
            memory_semaphore: Arc::new(Semaphore::new(max_memory_mb)),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Spawn a task with resource constraints
    pub async fn spawn_task<F, Fut>(
        &self,
        task_id: String,
        cpu_cores: usize,
        memory_mb: usize,
        task: F,
    ) -> Result<JoinHandle<()>, ResourceAwareSpawnerError>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        // Acquire CPU resources
        let cpu_permits = if cpu_cores > 0 {
            Some(
                self.cpu_semaphore
                    .clone()
                    .acquire_many_owned(cpu_cores as u32)
                    .await
                    .map_err(|_| ResourceAwareSpawnerError::ResourceLimitExceeded)?,
            )
        } else {
            None
        };

        // Acquire memory resources
        let memory_permits = if memory_mb > 0 {
            Some(
                self.memory_semaphore
                    .clone()
                    .acquire_many_owned(memory_mb as u32)
                    .await
                    .map_err(|_| ResourceAwareSpawnerError::ResourceLimitExceeded)?,
            )
        } else {
            None
        };

        // Track resource usage
        {
            let mut active = self.active_tasks.write().await;
            active.insert(
                task_id.clone(),
                TaskResourceUsage {
                    cpu_cores,
                    memory_mb,
                    start_time: Instant::now(),
                },
            );
        }

        // Spawn the task
        let active_tasks = self.active_tasks.clone();
        let handle = task::spawn(async move {
            let start_time = Instant::now();
            debug!("Starting resource-aware task {}", task_id);

            // Execute the task
            task().await;

            let duration = start_time.elapsed();
            debug!(
                "Completed resource-aware task {} in {:?}",
                task_id, duration
            );

            // Release resources
            drop(cpu_permits);
            drop(memory_permits);

            // Remove from tracking
            let mut active = active_tasks.write().await;
            active.remove(&task_id);
        });

        Ok(handle)
    }

    /// Get current resource usage
    pub async fn current_usage(&self) -> ResourceUsage {
        let active = self.active_tasks.read().await;

        let total_cpu = active.values().map(|u| u.cpu_cores).sum::<usize>();
        let total_memory = active.values().map(|u| u.memory_mb).sum::<usize>();
        let active_task_count = active.len();

        ResourceUsage {
            used_cpu_cores: total_cpu,
            available_cpu_cores: self.cpu_semaphore.available_permits(),
            used_memory_mb: total_memory,
            available_memory_mb: self.memory_semaphore.available_permits(),
            active_tasks: active_task_count,
        }
    }
}

/// Resource usage information
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub used_cpu_cores: usize,
    pub available_cpu_cores: usize,
    pub used_memory_mb: usize,
    pub available_memory_mb: usize,
    pub active_tasks: usize,
}

/// Resource-aware spawner errors
#[derive(Debug, thiserror::Error)]
pub enum ResourceAwareSpawnerError {
    #[error("Resource limit exceeded")]
    ResourceLimitExceeded,
}

/// Create a default task scheduler optimized for enterprise workloads
pub fn create_enterprise_scheduler() -> TaskScheduler {
    TaskScheduler::new(50, 1000) // 50 concurrent tasks, 1000 queue size
}

/// Create a resource-aware spawner for enterprise deployments
pub fn create_enterprise_spawner() -> ResourceAwareSpawner {
    ResourceAwareSpawner::new(16, 8192) // 16 CPU cores, 8GB memory
}
