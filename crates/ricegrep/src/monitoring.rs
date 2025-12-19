//! System monitoring and health checks (inspired by Refact's watchdog system)
//!
//! This module provides system-level monitoring capabilities including:
//! - Process health monitoring
//! - Resource usage tracking
//! - Background process management
//! - System health checks

use crate::error::RiceGrepError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::process::{Command, Stdio};
use tokio::process::{Child, Command as TokioCommand};
use std::time::Duration;

/// System monitor for tracking RiceGrep's health and performance
pub struct SystemMonitor {
    /// Process information
    process_info: Arc<Mutex<ProcessInfo>>,
    /// Health check results
    health_checks: Arc<Mutex<HashMap<String, HealthCheckResult>>>,
}

/// Process information (inspired by Refact's process tracking)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Start time
    pub start_time: i64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Number of threads
    pub thread_count: usize,
}

/// Health check result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthCheckResult {
    /// Check name
    pub name: String,
    /// Whether the check passed
    pub passed: bool,
    /// Check timestamp
    pub timestamp: i64,
    /// Additional details
    pub details: HashMap<String, serde_json::Value>,
    /// Error message if failed
    pub error_message: Option<String>,
}

impl SystemMonitor {
    /// Create a new system monitor
    pub fn new() -> Self {
        Self {
            process_info: Arc::new(Mutex::new(ProcessInfo {
                pid: std::process::id(),
                start_time: chrono::Utc::now().timestamp(),
                cpu_usage: 0.0,
                memory_usage: 0,
                thread_count: 1,
            })),
            health_checks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Update process information (inspired by Refact's monitoring)
    pub async fn update_process_info(&self) -> Result<(), RiceGrepError> {
        let mut info = self.process_info.lock().await;

        // In a real implementation, this would use system APIs to get actual metrics
        // For now, we'll use placeholder values
        info.cpu_usage = 5.0; // Placeholder CPU usage
        info.memory_usage = 50 * 1024 * 1024; // 50MB placeholder
        info.thread_count = 4; // Placeholder thread count

        Ok(())
    }

    /// Perform health check
    pub async fn perform_health_check(&self, check_name: &str) -> Result<HealthCheckResult, RiceGrepError> {
        let result = match check_name {
            "memory" => self.check_memory_usage().await,
            "cpu" => self.check_cpu_usage().await,
            "disk" => self.check_disk_space().await,
            "network" => self.check_network_connectivity().await,
            _ => HealthCheckResult {
                name: check_name.to_string(),
                passed: false,
                timestamp: chrono::Utc::now().timestamp(),
                details: HashMap::new(),
                error_message: Some(format!("Unknown health check: {}", check_name)),
            },
        };

        // Store the result
        let mut checks = self.health_checks.lock().await;
        checks.insert(check_name.to_string(), result.clone());

        Ok(result)
    }

    /// Get all health check results
    pub async fn get_health_checks(&self) -> HashMap<String, HealthCheckResult> {
        self.health_checks.lock().await.clone()
    }

    /// Check memory usage
    async fn check_memory_usage(&self) -> HealthCheckResult {
        let info = self.process_info.lock().await;
        let memory_mb = info.memory_usage / (1024 * 1024);
        let passed = memory_mb < 500; // Less than 500MB

        let mut details = HashMap::new();
        details.insert("memory_mb".to_string(), memory_mb.into());

        HealthCheckResult {
            name: "memory".to_string(),
            passed,
            timestamp: chrono::Utc::now().timestamp(),
            details,
            error_message: if passed { None } else { Some(format!("High memory usage: {}MB", memory_mb)) },
        }
    }

    /// Check CPU usage
    async fn check_cpu_usage(&self) -> HealthCheckResult {
        let info = self.process_info.lock().await;
        let passed = info.cpu_usage < 80.0; // Less than 80%

        let mut details = HashMap::new();
        details.insert("cpu_percent".to_string(), info.cpu_usage.into());

        HealthCheckResult {
            name: "cpu".to_string(),
            passed,
            timestamp: chrono::Utc::now().timestamp(),
            details,
            error_message: if passed { None } else { Some(format!("High CPU usage: {:.1}%", info.cpu_usage)) },
        }
    }

    /// Check disk space
    async fn check_disk_space(&self) -> HealthCheckResult {
        // Simplified disk space check
        let passed = true; // Assume sufficient disk space

        let mut details = HashMap::new();
        details.insert("disk_free_gb".to_string(), 100.into()); // Placeholder

        HealthCheckResult {
            name: "disk".to_string(),
            passed,
            timestamp: chrono::Utc::now().timestamp(),
            details,
            error_message: None,
        }
    }

    /// Check network connectivity
    async fn check_network_connectivity(&self) -> HealthCheckResult {
        // Simplified network check
        let passed = true; // Assume network is available

        let mut details = HashMap::new();
        details.insert("connectivity".to_string(), "ok".into());

        HealthCheckResult {
            name: "network".to_string(),
            passed,
            timestamp: chrono::Utc::now().timestamp(),
            details,
            error_message: None,
        }
    }

    /// Get system status summary (inspired by Refact's status reporting)
    pub async fn get_system_status(&self) -> SystemStatus {
        let info = self.process_info.lock().await;
        let checks = self.health_checks.lock().await;

        let all_healthy = checks.values().all(|check| check.passed);

        SystemStatus {
            process_info: info.clone(),
            healthy: all_healthy,
            total_checks: checks.len(),
            failed_checks: checks.values().filter(|check| !check.passed).count(),
            uptime_seconds: chrono::Utc::now().timestamp() - info.start_time,
        }
    }
}

/// System status summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemStatus {
    /// Process information
    pub process_info: ProcessInfo,
    /// Overall system health
    pub healthy: bool,
    /// Total number of health checks
    pub total_checks: usize,
    /// Number of failed health checks
    pub failed_checks: usize,
    /// System uptime in seconds
    pub uptime_seconds: i64,
}

/// Process manager for lifecycle control (inspired by Refact's watchdog)
pub struct ProcessManager {
    /// Active processes
    processes: Arc<Mutex<HashMap<String, ManagedProcess>>>,
    /// System monitor
    monitor: Arc<SystemMonitor>,
}

/// Managed process information
#[derive(Debug)]
pub struct ManagedProcess {
    /// Process ID
    pub id: String,
    /// Child process handle
    pub child: Child,
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Process configuration
    pub config: ProcessConfig,
    /// Restart count
    pub restart_count: u32,
    /// Health status
    pub healthy: bool,
}

/// Process configuration
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    /// Command to run
    pub command: String,
    /// Arguments
    pub args: Vec<String>,
    /// Working directory
    pub working_dir: Option<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Auto-restart on failure
    pub auto_restart: bool,
    /// Maximum restart attempts
    pub max_restarts: u32,
    /// Health check interval in seconds
    pub health_check_interval: u64,
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: Option<f64>,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            monitor: Arc::new(SystemMonitor::new()),
        }
    }

    /// Start a managed process
    pub async fn start_process(&self, id: String, config: ProcessConfig) -> Result<(), RiceGrepError> {
        // Check if process already exists
        let mut processes = self.processes.lock().await;
        if processes.contains_key(&id) {
            return Err(RiceGrepError::Process { message: format!("Process {} already exists", id) });
        }

        // Build the command
        let mut command = TokioCommand::new(&config.command);
        command
            .args(&config.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(ref dir) = config.working_dir {
            command.current_dir(dir);
        }

        for (key, value) in &config.env {
            command.env(key, value);
        }

        // Spawn the process
        let child = command.spawn().map_err(|e| RiceGrepError::Process {
            message: format!("Failed to spawn process {}: {}", id, e),
        })?;

        let managed_process = ManagedProcess {
            id: id.clone(),
            child,
            start_time: chrono::Utc::now(),
            config,
            restart_count: 0,
            healthy: true,
        };

        processes.insert(id, managed_process);

        Ok(())
    }

    /// Stop a managed process
    pub async fn stop_process(&self, id: &str) -> Result<(), RiceGrepError> {
        let mut processes = self.processes.lock().await;

        if let Some(mut process) = processes.remove(id) {
            // Send SIGTERM first
            if let Err(e) = process.child.kill().await {
                eprintln!("Warning: Failed to kill process {}: {}", id, e);
            }

            // Wait for process to exit
            if let Err(e) = process.child.wait().await {
                eprintln!("Warning: Failed to wait for process {}: {}", id, e);
            }

            Ok(())
        } else {
            Err(RiceGrepError::Process { message: format!("Process {} not found", id) })
        }
    }

    /// Restart a managed process
    pub async fn restart_process(&self, id: &str) -> Result<(), RiceGrepError> {
        let mut processes = self.processes.lock().await;

        if let Some(mut process) = processes.get_mut(id) {
            if process.restart_count >= process.config.max_restarts {
                return Err(RiceGrepError::Process {
                    message: format!("Process {} has exceeded maximum restart attempts ({})",
                           id, process.config.max_restarts),
                });
            }

            // Stop the current process
            if let Err(e) = process.child.kill().await {
                eprintln!("Warning: Failed to kill process {} during restart: {}", id, e);
            }

            // Start a new process
            let mut command = TokioCommand::new(&process.config.command);
            command
                .args(&process.config.args)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            if let Some(ref dir) = process.config.working_dir {
                command.current_dir(dir);
            }

            for (key, value) in &process.config.env {
                command.env(key, value);
            }

            let child = command.spawn().map_err(|e| RiceGrepError::Process {
                message: format!("Failed to restart process {}: {}", id, e),
            })?;

            process.child = child;
            process.start_time = chrono::Utc::now();
            process.restart_count += 1;
            process.healthy = true;

            Ok(())
        } else {
            Err(RiceGrepError::Process { message: format!("Process {} not found", id) })
        }
    }

    /// Get status of a managed process
    pub async fn get_process_status(&self, id: &str) -> Result<Option<ProcessStatus>, RiceGrepError> {
        let mut processes = self.processes.lock().await;

        if let Some(process) = processes.get_mut(id) {
            let status = process.child.try_wait().map_err(|e| RiceGrepError::Process {
                message: format!("Failed to check process {} status: {}", id, e),
            })?;

            Ok(Some(ProcessStatus {
                id: id.to_string(),
                running: status.is_none(),
                exit_code: status.and_then(|s| s.code()),
                start_time: process.start_time,
                restart_count: process.restart_count,
                healthy: process.healthy,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all managed processes
    pub async fn get_all_processes(&self) -> HashMap<String, ProcessStatus> {
        let processes = self.processes.lock().await;
        let mut result = HashMap::new();

        for (id, process) in processes.iter() {
            if let Ok(Some(status)) = self.get_process_status(id).await {
                result.insert(id.clone(), status);
            }
        }

        result
    }

    /// Perform health checks on all processes
    pub async fn perform_health_checks(&self) -> Result<(), RiceGrepError> {
        let process_ids: Vec<String> = {
            let processes = self.processes.lock().await;
            processes.keys().cloned().collect()
        };

        for id in process_ids {
            if let Err(e) = self.check_process_health(&id).await {
                eprintln!("Health check failed for process {}: {}", id, e);
            }
        }

        Ok(())
    }

    /// Check health of a specific process
    async fn check_process_health(&self, id: &str) -> Result<(), RiceGrepError> {
        let mut processes = self.processes.lock().await;

        if let Some(process) = processes.get_mut(id) {
            // Check if process is still running
            let status = process.child.try_wait().map_err(|e| RiceGrepError::Process {
                message: format!("Failed to check process {} health: {}", id, e),
            })?;

            if let Some(exit_code) = status.and_then(|s| s.code()) {
                // Process has exited
                process.healthy = false;

                if process.config.auto_restart && process.restart_count < process.config.max_restarts {
                    eprintln!("Process {} exited with code {}, restarting...", id, exit_code);
                    drop(processes); // Release lock before restart
                    return self.restart_process(id).await;
                } else {
                    eprintln!("Process {} exited with code {} and will not be restarted", id, exit_code);
                }
            } else {
                // Process is still running, perform additional health checks
                process.healthy = self.perform_detailed_health_check(process).await;
            }
        }

        Ok(())
    }

    /// Perform detailed health checks on a process
    async fn perform_detailed_health_check(&self, process: &ManagedProcess) -> bool {
        // Check memory usage if configured
        if let Some(max_memory) = process.config.max_memory_mb {
            // In a real implementation, this would check actual memory usage
            // For now, assume healthy
            if max_memory < 100 { // Placeholder check
                return false;
            }
        }

        // Check CPU usage if configured
        if let Some(max_cpu) = process.config.max_cpu_percent {
            // In a real implementation, this would check actual CPU usage
            // For now, assume healthy
            if max_cpu < 10.0 { // Placeholder check
                return false;
            }
        }

        true // All checks passed
    }
}

/// Process status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessStatus {
    /// Process ID
    pub id: String,
    /// Whether the process is currently running
    pub running: bool,
    /// Exit code if process has terminated
    pub exit_code: Option<i32>,
    /// Process start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Number of times the process has been restarted
    pub restart_count: u32,
    /// Whether the process is healthy
    pub healthy: bool,
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}