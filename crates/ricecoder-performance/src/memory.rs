//! Memory profiling utilities

use std::time::{Duration, Instant};
use sysinfo::{Pid, System};

/// Memory profiler for tracking memory usage
pub struct MemoryProfiler {
    system: System,
    process_id: Option<Pid>,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let process_id = sysinfo::get_current_pid().ok();

        Self { system, process_id }
    }

    /// Get current memory usage in bytes
    pub fn get_current_memory_usage(&mut self) -> u64 {
        self.system.refresh_all();

        if let Some(pid) = self.process_id {
            if let Some(process) = self.system.process(pid) {
                return process.memory(); // Already in bytes
            }
        }

        // Fallback: get system memory usage
        self.system.used_memory()
    }

    /// Profile memory usage during a function execution
    pub fn profile_memory_usage<F, R>(&mut self, f: F) -> (R, MemoryProfile)
    where
        F: FnOnce() -> R,
    {
        let start_memory = self.get_current_memory_usage();
        let start_time = Instant::now();

        let result = f();

        let end_time = Instant::now();
        let end_memory = self.get_current_memory_usage();

        let profile = MemoryProfile {
            start_memory_bytes: start_memory,
            end_memory_bytes: end_memory,
            peak_memory_bytes: end_memory.max(start_memory), // Simplified
            duration: end_time - start_time,
            memory_delta_bytes: end_memory as i64 - start_memory as i64,
        };

        (result, profile)
    }

    /// Monitor memory usage over time
    pub fn monitor_memory_over_time(&mut self, duration: Duration) -> Vec<MemorySnapshot> {
        let mut snapshots = Vec::new();
        let start_time = Instant::now();
        let interval = Duration::from_millis(100); // Sample every 100ms

        while start_time.elapsed() < duration {
            let memory_usage = self.get_current_memory_usage();
            snapshots.push(MemorySnapshot {
                timestamp: Instant::now(),
                memory_bytes: memory_usage,
            });

            std::thread::sleep(interval);
        }

        snapshots
    }
}

/// Memory profile for a function execution
#[derive(Debug, Clone)]
pub struct MemoryProfile {
    /// Memory usage at start in bytes
    pub start_memory_bytes: u64,
    /// Memory usage at end in bytes
    pub end_memory_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Execution duration
    pub duration: Duration,
    /// Memory delta (end - start)
    pub memory_delta_bytes: i64,
}

/// Memory usage snapshot
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// Timestamp of snapshot
    pub timestamp: Instant,
    /// Memory usage in bytes
    pub memory_bytes: u64,
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}
