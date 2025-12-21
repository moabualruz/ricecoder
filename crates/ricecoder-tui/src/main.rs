//! RiceCoder TUI - Terminal User Interface entry point

use anyhow::Result;
use ricecoder_storage::TuiConfig;
use ricecoder_tui::view::view;
use ricecoder_tui::{
    accessibility::*, app::App, banner::*, clipboard::*, code_editor_widget::*, command_blocks::*,
    command_palette::*, components::*, di::*, diff::*, error::*, event::*, event_dispatcher::*,
    file_picker::*, image_integration::*, image_widget::*, input::*, integration::*, layout::*,
    lifecycle::*, logger_widget::*, markdown::*, model::*, monitoring::*, performance::*,
    plugins::*, popup_widget::*, progressive_enhancement::*, project_bootstrap::*, prompt::*,
    prompt_context::*, providers::*, reactive_ui_updates::*, real_time_updates::*,
    render_pipeline::*, scrollview_widget::*, status_bar::*, style::*, terminal_state::*,
    textarea_widget::*, tree_widget::*, ui_components::*, update::*, widgets::*, CancellationToken,
};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

/// Resource that needs cleanup during shutdown
#[async_trait::async_trait]
trait CleanupResource: Send + Sync + std::fmt::Debug {
    async fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Timeout configuration for different operation types
#[derive(Debug, Clone)]
struct TimeoutConfig {
    /// Timeout for UI operations (rendering, input handling)
    ui_operations: Duration,
    /// Timeout for I/O operations (file access, network)
    io_operations: Duration,
    /// Timeout for plugin operations
    plugin_operations: Duration,
    /// Timeout for background tasks
    background_tasks: Duration,
    /// Timeout for command execution
    command_execution: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            ui_operations: Duration::from_millis(100), // 100ms for responsive UI
            io_operations: Duration::from_secs(30),    // 30s for I/O operations
            plugin_operations: Duration::from_secs(10), // 10s for plugin operations
            background_tasks: Duration::from_secs(60), // 1min for background tasks
            command_execution: Duration::from_secs(300), // 5min for command execution
        }
    }
}

/// Async runtime manager for proper cancellation and concurrency control
#[derive(Debug)]
struct AsyncRuntimeManager {
    /// Main cancellation token for graceful shutdown
    cancellation_token: CancellationToken,
    /// Semaphore for limiting concurrent operations
    concurrency_limiter: Arc<Semaphore>,
    /// Active task handles for cleanup
    active_tasks: Arc<std::sync::Mutex<Vec<JoinHandle<()>>>>,
    /// Resources that need cleanup during shutdown
    cleanup_resources: Arc<std::sync::Mutex<Vec<Box<dyn CleanupResource>>>>,
    /// Background job queue
    job_queue: Arc<std::sync::RwLock<ricecoder_tui::JobQueue>>,
    /// Performance profiler
    // profiler: Arc<std::sync::RwLock<ricecoder_tui::PerformanceProfiler>>, // TODO: not Debug
    /// CPU monitor
    cpu_monitor: Arc<std::sync::RwLock<ricecoder_tui::CpuMonitor>>,
    /// Memory profiler
    memory_profiler: Arc<std::sync::RwLock<ricecoder_tui::MemoryProfiler>>,
    /// Operation timeout configuration
    timeout_config: TimeoutConfig,
    /// Shutdown timeout
    shutdown_timeout: Duration,
}

impl AsyncRuntimeManager {
    /// Create a new async runtime manager
    fn new() -> Self {
        Self {
            cancellation_token: CancellationToken::new(),
            concurrency_limiter: Arc::new(Semaphore::new(10)), // Allow up to 10 concurrent operations
            active_tasks: Arc::new(std::sync::Mutex::new(Vec::new())),
            cleanup_resources: Arc::new(std::sync::Mutex::new(Vec::new())),
            job_queue: Arc::new(std::sync::RwLock::new(ricecoder_tui::JobQueue::new())),
            // profiler: Arc::new(std::sync::RwLock::new(ricecoder_tui::PerformanceProfiler::new())),
            cpu_monitor: Arc::new(std::sync::RwLock::new(ricecoder_tui::CpuMonitor::new())),
            memory_profiler: Arc::new(std::sync::RwLock::new(ricecoder_tui::MemoryProfiler::new())),
            timeout_config: TimeoutConfig::default(),
            shutdown_timeout: Duration::from_secs(10), // 10 second shutdown timeout
        }
    }

    /// Create a new async runtime manager with custom timeout configuration
    fn with_timeout_config(timeout_config: TimeoutConfig) -> Self {
        Self {
            cancellation_token: CancellationToken::new(),
            concurrency_limiter: Arc::new(Semaphore::new(10)),
            active_tasks: Arc::new(std::sync::Mutex::new(Vec::new())),
            cleanup_resources: Arc::new(std::sync::Mutex::new(Vec::new())),
            job_queue: Arc::new(std::sync::RwLock::new(ricecoder_tui::JobQueue::new())),
            // profiler: Arc::new(std::sync::RwLock::new(ricecoder_tui::PerformanceProfiler::new())),
            cpu_monitor: Arc::new(std::sync::RwLock::new(ricecoder_tui::CpuMonitor::new())),
            memory_profiler: Arc::new(std::sync::RwLock::new(ricecoder_tui::MemoryProfiler::new())),
            timeout_config,
            shutdown_timeout: Duration::from_secs(10),
        }
    }

    /// Spawn a cancellable task
    fn spawn_cancellable<F>(&self, future: F) -> JoinHandle<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let token = self.cancellation_token.clone();
        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = token.cancelled() => {
                    tracing::debug!("Task cancelled due to shutdown signal");
                }
                _ = future => {
                    // Task completed normally
                }
            }
        });

        // Track the task handle for cleanup
        // Note: JoinHandle doesn't implement Clone, so we don't track it

        handle
    }

    /// Spawn a task with timeout
    fn spawn_with_timeout<F>(&self, future: F, timeout: Duration) -> JoinHandle<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let token = self.cancellation_token.clone();
        let handle = tokio::spawn(async move {
            let timeout_future = tokio::time::sleep(timeout);
            tokio::pin!(timeout_future);

            tokio::select! {
                _ = token.cancelled() => {
                    tracing::debug!("Task cancelled due to shutdown signal");
                }
                _ = &mut timeout_future => {
                    tracing::warn!("Task timed out after {:?}", timeout);
                }
                _ = future => {
                    // Task completed normally
                }
            }
        });

        // Track the task handle for cleanup
        // Note: JoinHandle doesn't implement Clone, so we don't track it

        handle
    }

    /// Spawn a UI operation with timeout
    fn spawn_ui_operation<F>(&self, future: F) -> JoinHandle<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.spawn_with_timeout(future, self.timeout_config.ui_operations)
    }

    /// Spawn an I/O operation with timeout
    fn spawn_io_operation<F>(&self, future: F) -> JoinHandle<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.spawn_with_timeout(future, self.timeout_config.io_operations)
    }

    /// Spawn a plugin operation with timeout
    fn spawn_plugin_operation<F>(&self, future: F) -> JoinHandle<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.spawn_with_timeout(future, self.timeout_config.plugin_operations)
    }

    /// Spawn a background task with timeout
    fn spawn_background_task<F>(&self, future: F) -> JoinHandle<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.spawn_with_timeout(future, self.timeout_config.background_tasks)
    }

    /// Spawn a command execution with timeout
    fn spawn_command_execution<F>(&self, future: F) -> JoinHandle<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.spawn_with_timeout(future, self.timeout_config.command_execution)
    }

    /// Execute an operation with concurrency limiting
    async fn execute_concurrent<F, Fut, T>(&self, operation: F) -> Option<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Acquire semaphore permit
        let permit = self.concurrency_limiter.acquire().await.ok()?;

        // Execute operation
        let future = operation();
        let token = self.cancellation_token.clone();

        tokio::select! {
            result = future => {
                drop(permit); // Release semaphore
                Some(result)
            }
            _ = token.cancelled() => {
                drop(permit); // Release semaphore
                None
            }
        }
    }

    /// Initiate graceful shutdown
    fn shutdown(&self) {
        tracing::info!("Initiating graceful shutdown...");
        self.cancellation_token.cancel();
    }

    /// Wait for all active tasks to complete
    async fn wait_for_completion(&self) {
        let tasks = {
            if let Ok(mut tasks) = self.active_tasks.lock() {
                std::mem::take(&mut *tasks)
            } else {
                Vec::new()
            }
        };

        for handle in tasks {
            let _ = handle.await;
        }

        tracing::info!("All active tasks completed");
    }

    /// Check if shutdown has been initiated
    fn is_shutdown(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    /// Get the cancellation token
    fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }

    /// Register a resource for cleanup during shutdown
    fn register_cleanup_resource(&self, resource: Box<dyn CleanupResource>) {
        if let Ok(mut resources) = self.cleanup_resources.lock() {
            resources.push(resource);
        }
    }

    /// Perform graceful shutdown with resource cleanup
    async fn graceful_shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Starting graceful shutdown sequence...");

        // Step 1: Signal cancellation to all tasks
        self.cancellation_token.cancel();

        // Step 2: Cancel all pending jobs
        {
            let mut queue = self.job_queue.write().unwrap();
            // Cancel all active jobs
            let active_job_ids: Vec<_> = vec![]; // TODO: fix private field
            for job_id in &active_job_ids {
                queue.cancel_job(&job_id);
            }
            tracing::info!("Cancelled {} active background jobs", active_job_ids.len());
        }

        // Step 3: Wait for active tasks and jobs to complete with timeout
        let shutdown_start = Instant::now();
        let cleanup_future = async {
            // Wait for regular tasks
            self.wait_for_completion().await;

            // Process remaining jobs until queue is empty
            loop {
                self.process_jobs().await;
                let stats = self.job_stats();
                if stats.queued_jobs == 0 && stats.active_jobs == 0 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        };

        tokio::select! {
            _ = cleanup_future => {
                tracing::info!("All tasks and jobs completed gracefully");
            }
            _ = tokio::time::sleep(self.shutdown_timeout) => {
                tracing::warn!("Shutdown timeout reached, some tasks or jobs may not have completed");
            }
        }

        // Step 4: Clean up registered resources
        if let Ok(mut resources) = self.cleanup_resources.lock() {
            for (i, resource) in resources.iter_mut().enumerate() {
                tracing::debug!("Cleaning up resource {}", i);
                if let Err(e) = resource.cleanup().await {
                    tracing::error!("Failed to cleanup resource {}: {}", i, e);
                }
            }
            resources.clear();
        }

        let shutdown_duration = shutdown_start.elapsed();
        tracing::info!("Graceful shutdown completed in {:?}", shutdown_duration);

        Ok(())
    }

    /// Check if shutdown is in progress
    fn is_shutting_down(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    /// Submit a job to the background queue
    pub fn submit_job(
        &self,
        task: ricecoder_tui::JobTask,
        priority: ricecoder_tui::JobPriority,
    ) -> ricecoder_tui::JobId {
        let mut queue = self.job_queue.write().unwrap();
        queue.submit_job(task, priority)
    }

    /// Submit a job with completion callback
    pub fn submit_job_with_callback<F>(
        &self,
        task: ricecoder_tui::JobTask,
        priority: ricecoder_tui::JobPriority,
        callback: F,
    ) -> ricecoder_tui::JobId
    where
        F: Fn(ricecoder_tui::JobResult) + Send + Sync + 'static,
    {
        let mut queue = self.job_queue.write().unwrap();
        queue.submit_job_with_callback(task, priority, callback)
    }

    /// Cancel a background job
    pub fn cancel_job(&self, job_id: &ricecoder_tui::JobId) -> bool {
        let mut queue = self.job_queue.write().unwrap();
        queue.cancel_job(job_id)
    }

    /// Process pending background jobs
    pub async fn process_jobs(&self) {
        let mut queue = self.job_queue.write().unwrap();
        queue.process_jobs().await;
    }

    /// Get job queue statistics
    pub fn job_stats(&self) -> ricecoder_tui::JobQueueStats {
        let queue = self.job_queue.read().unwrap();
        queue.stats()
    }

    /// Subscribe to progress updates for a job
    pub fn subscribe_job_progress(
        &self,
        job_id: &ricecoder_tui::JobId,
    ) -> Option<tokio::sync::broadcast::Receiver<ricecoder_tui::ProgressUpdate>> {
        let queue = self.job_queue.read().unwrap();
        None // TODO: fix return type
    }

    /// Get progress reporter
    pub fn progress_reporter(&self) -> std::sync::RwLockReadGuard<ricecoder_tui::JobQueue> {
        self.job_queue.read().unwrap()
    }

    /// Get progress statistics
    pub fn progress_stats(&self) -> ricecoder_tui::ProgressStats {
        let queue = self.job_queue.read().unwrap();
        ricecoder_tui::ProgressStats {
            total_trackers: 0,
            active_trackers: 0,
            completed_trackers: 0,
            failed_trackers: 0,
        }
    }

    /// Enable performance profiling
    pub fn enable_profiling(&self) {
        // TODO: implement
    }

    /// Disable performance profiling
    pub fn disable_profiling(&self) {
        // TODO: implement
    }

    /// Check if profiling is enabled
    pub fn is_profiling_enabled(&self) -> bool {
        false
        // profiler.is_enabled() // TODO: fix method
    }

    /// Start a profiling span
    pub fn start_profile_span(&self, name: &str) -> Option<ricecoder_tui::ProfileSpanHandle> {
        None // TODO: fix method
    }

    /// Generate flame graph data
    pub fn generate_flame_graph(&self) -> String {
        // TODO: implement
        String::new()
    }

    /// Get profiling statistics
    pub fn profiling_stats(&self) -> ricecoder_tui::ProfileStats {
        ricecoder_tui::ProfileStats {
            enabled: false,
            total_spans: 0,
            active_spans: 0,
            session_duration: std::time::Duration::default(),
            total_duration: std::time::Duration::default(),
            avg_span_duration: std::time::Duration::default(),
        }
    }

    /// Record CPU usage sample
    pub fn record_cpu_sample(&self, user_percent: f64, system_percent: f64, thread_count: usize) {
        let mut monitor = self.cpu_monitor.write().unwrap();
        monitor.record_sample(user_percent, system_percent, thread_count);
    }

    /// Get current CPU statistics
    pub fn cpu_stats(&self) -> Option<ricecoder_tui::CpuStats> {
        let monitor = self.cpu_monitor.read().unwrap();
        monitor.current_stats()
    }

    /// Record memory allocation
    pub fn record_memory_allocation(&self, size: u64, heap_size: u64) {
        let mut profiler = self.memory_profiler.write().unwrap();
        profiler.record_allocation(size, heap_size);
    }

    /// Record memory deallocation
    pub fn record_memory_deallocation(&self, size: u64, heap_size: u64) {
        let mut profiler = self.memory_profiler.write().unwrap();
        profiler.record_deallocation(size, heap_size);
    }

    /// Get current memory statistics
    pub fn memory_stats(&self) -> ricecoder_tui::MemoryStats {
        let profiler = self.memory_profiler.read().unwrap();
        profiler.current_stats().clone()
    }

    /// Detect memory leaks
    pub fn detect_memory_leaks(&self) -> Vec<ricecoder_tui::MemoryLeak> {
        let profiler = self.memory_profiler.read().unwrap();
        profiler.detect_leaks()
    }

    /// Get the timeout configuration
    fn timeout_config(&self) -> &TimeoutConfig {
        &self.timeout_config
    }

    /// Update timeout configuration
    fn set_timeout_config(&mut self, config: TimeoutConfig) {
        self.timeout_config = config;
    }

    /// Execute an operation with type-specific timeout
    async fn execute_with_timeout<F, Fut, T>(
        &self,
        operation_type: OperationType,
        operation: F,
    ) -> Result<T, TimeoutError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let timeout = match operation_type {
            OperationType::Ui => self.timeout_config.ui_operations,
            OperationType::Io => self.timeout_config.io_operations,
            OperationType::Plugin => self.timeout_config.plugin_operations,
            OperationType::Background => self.timeout_config.background_tasks,
            OperationType::Command => self.timeout_config.command_execution,
        };

        let future = operation();
        let token = self.cancellation_token.clone();

        tokio::select! {
            result = future => Ok(result),
            _ = tokio::time::sleep(timeout) => Err(TimeoutError {
                operation_type,
                timeout,
            }),
            _ = token.cancelled() => Err(TimeoutError {
                operation_type,
                timeout,
            }),
        }
    }
}

/// Operation types for timeout configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Ui,
    Io,
    Plugin,
    Background,
    Command,
}

/// Timeout error with context
#[derive(Debug, Clone)]
pub struct TimeoutError {
    pub operation_type: OperationType,
    pub timeout: Duration,
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} operation timed out after {:?}",
            self.operation_type, self.timeout
        )
    }
}

impl std::error::Error for TimeoutError {}

impl Default for AsyncRuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Cleanup resource for reactive state
#[derive(Debug)]
struct ReactiveStateCleanup {
    state: Option<ricecoder_tui::tea::ReactiveState>,
}

#[async_trait::async_trait]
impl CleanupResource for ReactiveStateCleanup {
    async fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(state) = self.state.take() {
            tracing::debug!("Cleaning up reactive state");
            // The ReactiveState doesn't need explicit cleanup, but we could add
            // any necessary cleanup logic here
        }
        Ok(())
    }
}

/// Cleanup resource for event channels
#[derive(Debug)]
struct EventChannelCleanup {
    tx: Option<tokio::sync::mpsc::UnboundedSender<ricecoder_tui::AppMessage>>,
}

#[async_trait::async_trait]
impl CleanupResource for EventChannelCleanup {
    async fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tx) = self.tx.take() {
            tracing::debug!("Cleaning up event channel");
            // Close the channel by dropping the sender
            drop(tx);
        }
        Ok(())
    }
}

/// Cleanup resource for performance tracker
#[derive(Debug)]
struct PerformanceTrackerCleanup {
    tracker: Option<ricecoder_tui::performance::RenderPerformanceTracker>,
}

#[async_trait::async_trait]
impl CleanupResource for PerformanceTrackerCleanup {
    async fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tracker) = self.tracker.take() {
            tracing::debug!("Cleaning up performance tracker");
            let stats = tracker.metrics();
            tracing::info!(
                "Final performance stats: {:.1} FPS, {:.1}ms avg frame time",
                stats.current_fps,
                stats.average_frame_time_ms
            );
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize DI container first
    if let Err(e) = ricecoder_tui::di::initialize_di_container() {
        eprintln!("DI container initialization failed: {}", e);
        std::process::exit(1);
    }

    // Initialize TUI lifecycle manager
    let lifecycle_manager = ricecoder_tui::lifecycle::initialize_tui_lifecycle_manager();

    // Initialize all TUI components
    if let Err(e) = lifecycle_manager.initialize_all().await {
        eprintln!("TUI component initialization failed: {}", e);
        std::process::exit(1);
    }

    // Start all TUI components
    if let Err(e) = lifecycle_manager.start_all().await {
        eprintln!("TUI component startup failed: {}", e);
        std::process::exit(1);
    }

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Capture terminal state before TUI initialization
    // Requirements: 4.1, 10.1 - Detect capabilities and capture terminal state before TUI initialization
    let mut terminal_state = TerminalState::capture()?;

    // Log detected capabilities for debugging and adaptation
    // Requirements: 4.1 - Log detected capabilities via ricecoder-logging
    let caps = terminal_state.capabilities();
    tracing::info!(
        "Terminal capabilities detected - Type: {:?}, Colors: {:?}, Mouse: {}, Sixel: {}, Unicode: {}, SSH: {}, TMUX: {}, Size: {}x{}",
        caps.terminal_type,
        caps.color_support,
        caps.mouse_support,
        caps.sixel_support,
        caps.unicode_support,
        caps.is_ssh,
        caps.is_tmux,
        caps.size.0,
        caps.size.1
    );

    // Adapt behavior based on capabilities
    // Requirements: 4.2, 4.3 - Adapt UI based on detected capabilities and handle SSH limitations
    if caps.should_reduce_graphics() {
        tracing::info!("SSH session detected - reducing graphics complexity");
    }

    if caps.should_wrap_osc52() {
        tracing::info!("TMUX session detected - will wrap OSC 52 sequences for clipboard");
    }

    // Create a flag to signal graceful shutdown
    // Requirements: 10.1 - Install signal handler for Ctrl+C
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_clone = shutdown_flag.clone();

    // Install Ctrl+C handler
    // Requirements: 10.1 - Install signal handler in ricecoder-tui/src/app.rs (or main.rs)
    ctrlc::set_handler(move || {
        tracing::info!("Ctrl+C received, initiating graceful shutdown");
        shutdown_flag_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    // Create initial TEA model
    let config = TuiConfig::default();
    let theme = ricecoder_themes::Theme::default();
    let terminal_caps = terminal_state.capabilities().clone();
    let initial_model = AppModel::init(config, theme, terminal_caps);

    // Create reactive state manager
    let reactive_state = ricecoder_tui::RealTimeStats {
        total_operations: 0,
        queued_operations: 0,
        running_operations: 0,
        paused_operations: 0,
        completed_operations: 0,
        failed_operations: 0,
        cancelled_operations: 0,
        chat_operations: 0,
        file_operations: 0,
        network_operations: 0,
        background_operations: 0,
        system_operations: 0,
    };

    // Create event channels for TEA
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();

    // Create async runtime manager for proper cancellation and concurrency
    let runtime_manager = Arc::new(AsyncRuntimeManager::new());
    let runtime_manager_clone = Arc::clone(&runtime_manager);

    // Install Ctrl+C handler with runtime manager
    let shutdown_flag_clone = shutdown_flag.clone();
    let runtime_manager_ctrlc = Arc::clone(&runtime_manager);
    ctrlc::set_handler(move || {
        tracing::info!("Ctrl+C received, initiating graceful shutdown");
        shutdown_flag_clone.store(true, Ordering::SeqCst);
        runtime_manager_ctrlc.shutdown();
    })
    .expect("Error setting Ctrl+C handler");

    // Start event polling task with UI operation timeout
    let event_tx_clone = event_tx.clone();
    let runtime_manager_events = Arc::clone(&runtime_manager);
    runtime_manager.spawn_ui_operation(async move {
        let mut event_loop = ricecoder_tui::EventLoop::new();
        loop {
            match event_loop.poll().await {
                Ok(Some(evt)) => {
                    let message = event_to_message(evt);
                    if event_tx_clone.send(message).is_err() {
                        break;
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    tracing::error!("Event polling error: {}", e);
                    break;
                }
            }
        }
    });

    // Run TEA event loop with runtime manager
    let result = run_tea_event_loop(
        reactive_state,
        initial_model,
        &mut event_rx,
        runtime_manager.cancellation_token(),
        terminal_state.capabilities(),
    )
    .await;

    // Restore terminal state on exit (normal, Ctrl+C, or error)
    // Requirements: 10.2, 10.3 - Restore terminal on normal exit, Ctrl+C, and error exit
    if let Err(e) = terminal_state.restore() {
        tracing::error!("Failed to restore terminal state: {}", e);
    }

    // Stop TUI components first
    if let Err(e) = lifecycle_manager.stop_all().await {
        tracing::error!("Error stopping TUI components: {}", e);
    }

    // Perform graceful shutdown with resource cleanup
    if let Err(e) = runtime_manager_clone.graceful_shutdown().await {
        tracing::error!("Error during graceful shutdown: {}", e);
    }

    result
}

/// Run TEA-based event loop with graceful shutdown support
///
/// This function implements the Elm Architecture event loop:
/// 1. Poll for events and convert to messages
/// 2. Update model with messages
/// 3. Render view based on current model
/// 4. Handle graceful shutdown with cancellation tokens
async fn run_tea_event_loop(
    mut reactive_state: ricecoder_tui::RealTimeStats,
    initial_model: ricecoder_tui::AppModel,
    event_rx: &mut mpsc::UnboundedReceiver<ricecoder_tui::AppMessage>,
    cancel_token: &CancellationToken,
    capabilities: &ricecoder_tui::TerminalCapabilities,
) -> Result<()> {
    use crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;
    use std::io;

    // Set up terminal
    let mut stdout = io::stdout();

    // Enable mouse capture only if supported
    if capabilities.mouse_support {
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        tracing::debug!("Mouse capture enabled");
    } else {
        execute!(stdout, EnterAlternateScreen)?;
        tracing::debug!("Mouse capture disabled - not supported");
    }
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Initialize performance tracker for 60 FPS target
    let mut perf_tracker = RenderPerformanceTracker::new();

    // TEA Event Loop with tokio::select! for cancellation
    let result = async {
        loop {
            let frame_start = Instant::now();

            // Use tokio::select! to handle events and cancellation concurrently
            tokio::select! {
                // Handle cancellation
                _ = cancel_token.cancelled() => {
                    tracing::info!("Cancellation received, exiting gracefully");
                    break;
                }

                // Handle incoming messages
                message = event_rx.recv() => {
                    match message {
                         Some(msg) => {
                             // Update model with message
                             // TODO: fix method
                             // match reactive_state.update(msg) {
                             //     Ok(diff) => {
                             //         tracing::debug!("Model updated with diff: {:?}", diff.changes);
                             //     }
                             //     Err(e) => {
                             //         tracing::error!("Failed to update model: {}", e);
                             //     }
                             // }
                        }
                        None => {
                            tracing::info!("Event channel closed, exiting");
                            break;
                        }
                    }
                }

                 // Periodic tick for animations/updates
                 _ = tokio::time::sleep(Duration::from_millis(250)) => {
                     // Send tick message
                     let tick_msg = ricecoder_tui::AppMessage::Tick;
                     // TODO: fix method
                     // if let Err(e) = reactive_state.update(tick_msg) {
                     //     tracing::error!("Failed to process tick: {}", e);
                     // }
                 }
            }

            // Render the current model state
            let current_model = &initial_model;
            terminal.draw(|f| {
                view(f, current_model);
            })?;

            // Record frame time for performance tracking
            let frame_time = frame_start.elapsed();
            perf_tracker.record_frame(frame_time);

            // Log performance warnings if not meeting 60 FPS target
            if !perf_tracker.is_meeting_target() && perf_tracker.frame_count % 60 == 0 {
                let metrics = perf_tracker.metrics();
                tracing::warn!(
                    "Performance warning: {:.1} FPS (target: 60 FPS), avg frame time: {:.1}ms",
                    metrics.current_fps,
                    metrics.average_frame_time_ms
                );
            }

            // Throttle to ~60 FPS to prevent excessive CPU usage
            let target_frame_time = Duration::from_millis(16); // ~60 FPS
            if frame_time < target_frame_time {
                tokio::time::sleep(target_frame_time - frame_time).await;
            }
        }

        tracing::info!("TEA event loop exited successfully");
        Ok::<(), anyhow::Error>(())
    }
    .await;

    // Clean up terminal
    if capabilities.mouse_support {
        execute!(
            terminal.backend_mut(),
            DisableMouseCapture,
            LeaveAlternateScreen
        )?;
    } else {
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    }
    terminal.show_cursor()?;

    result
}
