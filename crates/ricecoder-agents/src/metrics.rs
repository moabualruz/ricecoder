//! Performance metrics collection and tracking for agents

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use tracing::{debug, info};

/// Metrics for a single agent execution
#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
    /// Agent ID
    pub agent_id: String,
    /// Execution start time
    pub start_time: Instant,
    /// Execution end time (if completed)
    pub end_time: Option<Instant>,
    /// Number of findings
    pub findings_count: usize,
    /// Severity distribution of findings
    pub severity_distribution: HashMap<String, usize>,
    /// Whether execution succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl ExecutionMetrics {
    /// Create new execution metrics
    pub fn new(agent_id: String) -> Self {
        debug!(agent_id = %agent_id, "Creating execution metrics");
        Self {
            agent_id,
            start_time: Instant::now(),
            end_time: None,
            findings_count: 0,
            severity_distribution: HashMap::new(),
            success: false,
            error: None,
        }
    }

    /// Mark execution as completed
    pub fn complete(&mut self, success: bool, error: Option<String>) {
        self.end_time = Some(Instant::now());
        self.success = success;
        self.error = error;

        let duration_ms = self.duration_ms();
        debug!(
            agent_id = %self.agent_id,
            duration_ms = duration_ms,
            success = success,
            "Execution metrics completed"
        );
    }

    /// Get execution duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        let end = self.end_time.unwrap_or_else(Instant::now);
        end.duration_since(self.start_time).as_millis() as u64
    }

    /// Record findings
    pub fn record_findings(&mut self, count: usize, severity_dist: HashMap<String, usize>) {
        self.findings_count = count;
        self.severity_distribution = severity_dist;
        debug!(
            agent_id = %self.agent_id,
            findings_count = count,
            "Findings recorded"
        );
    }
}

/// Metrics collector for tracking agent performance
pub struct MetricsCollector {
    executions: Arc<Mutex<Vec<ExecutionMetrics>>>,
    agent_stats: Arc<Mutex<HashMap<String, AgentStats>>>,
}

/// Statistics for an agent
#[derive(Debug, Clone)]
pub struct AgentStats {
    /// Agent ID
    pub agent_id: String,
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Total findings
    pub total_findings: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Minimum execution time in milliseconds
    pub min_execution_time_ms: u64,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        info!("Creating metrics collector");
        Self {
            executions: Arc::new(Mutex::new(Vec::new())),
            agent_stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Record an execution
    pub fn record_execution(&self, metrics: ExecutionMetrics) {
        let agent_id = metrics.agent_id.clone();
        let duration_ms = metrics.duration_ms();
        let findings_count = metrics.findings_count;
        let success = metrics.success;

        // Store execution
        {
            let mut executions = self.executions.lock().unwrap();
            executions.push(metrics);
        }

        // Update agent stats
        {
            let mut stats = self.agent_stats.lock().unwrap();
            let agent_stat = stats.entry(agent_id.clone()).or_insert_with(|| AgentStats {
                agent_id: agent_id.clone(),
                total_executions: 0,
                successful_executions: 0,
                failed_executions: 0,
                total_findings: 0,
                avg_execution_time_ms: 0.0,
                min_execution_time_ms: u64::MAX,
                max_execution_time_ms: 0,
            });

            agent_stat.total_executions += 1;
            if success {
                agent_stat.successful_executions += 1;
            } else {
                agent_stat.failed_executions += 1;
            }
            agent_stat.total_findings += findings_count as u64;

            // Update average execution time
            let total_time = agent_stat.avg_execution_time_ms
                * (agent_stat.total_executions - 1) as f64
                + duration_ms as f64;
            agent_stat.avg_execution_time_ms = total_time / agent_stat.total_executions as f64;

            // Update min/max
            agent_stat.min_execution_time_ms = agent_stat.min_execution_time_ms.min(duration_ms);
            agent_stat.max_execution_time_ms = agent_stat.max_execution_time_ms.max(duration_ms);

            debug!(
                agent_id = %agent_id,
                total_executions = agent_stat.total_executions,
                avg_execution_time_ms = agent_stat.avg_execution_time_ms,
                "Agent statistics updated"
            );
        }
    }

    /// Get statistics for an agent
    pub fn get_agent_stats(&self, agent_id: &str) -> Option<AgentStats> {
        let stats = self.agent_stats.lock().unwrap();
        stats.get(agent_id).cloned()
    }

    /// Get all agent statistics
    pub fn all_agent_stats(&self) -> Vec<AgentStats> {
        let stats = self.agent_stats.lock().unwrap();
        stats.values().cloned().collect()
    }

    /// Get all executions
    pub fn all_executions(&self) -> Vec<ExecutionMetrics> {
        let executions = self.executions.lock().unwrap();
        executions.clone()
    }

    /// Get executions for a specific agent
    pub fn get_agent_executions(&self, agent_id: &str) -> Vec<ExecutionMetrics> {
        let executions = self.executions.lock().unwrap();
        executions
            .iter()
            .filter(|e| e.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Clear all metrics
    pub fn clear(&self) {
        let mut executions = self.executions.lock().unwrap();
        let mut stats = self.agent_stats.lock().unwrap();
        executions.clear();
        stats.clear();
        info!("Metrics collector cleared");
    }

    /// Get total execution count
    pub fn total_execution_count(&self) -> usize {
        let executions = self.executions.lock().unwrap();
        executions.len()
    }

    /// Get total findings count
    pub fn total_findings_count(&self) -> u64 {
        let stats = self.agent_stats.lock().unwrap();
        stats.values().map(|s| s.total_findings).sum()
    }

    /// Get average execution time across all agents
    pub fn average_execution_time_ms(&self) -> f64 {
        let stats = self.agent_stats.lock().unwrap();
        if stats.is_empty() {
            return 0.0;
        }

        let total_time: f64 = stats.values().map(|s| s.avg_execution_time_ms).sum();
        total_time / stats.len() as f64
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_metrics_new() {
        let metrics = ExecutionMetrics::new("test-agent".to_string());
        assert_eq!(metrics.agent_id, "test-agent");
        assert!(metrics.end_time.is_none());
        assert_eq!(metrics.findings_count, 0);
        assert!(!metrics.success);
        assert!(metrics.error.is_none());
    }

    #[test]
    fn test_execution_metrics_complete_success() {
        let mut metrics = ExecutionMetrics::new("test-agent".to_string());
        metrics.complete(true, None);

        assert!(metrics.end_time.is_some());
        assert!(metrics.success);
        assert!(metrics.error.is_none());
        // Duration should be non-negative (always true for u64, but validates the function works)
        let _ = metrics.duration_ms();
    }

    #[test]
    fn test_execution_metrics_complete_failure() {
        let mut metrics = ExecutionMetrics::new("test-agent".to_string());
        metrics.complete(false, Some("Test error".to_string()));

        assert!(metrics.end_time.is_some());
        assert!(!metrics.success);
        assert_eq!(metrics.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_execution_metrics_record_findings() {
        let mut metrics = ExecutionMetrics::new("test-agent".to_string());
        let mut severity_dist = HashMap::new();
        severity_dist.insert("Critical".to_string(), 2);
        severity_dist.insert("Warning".to_string(), 5);

        metrics.record_findings(7, severity_dist.clone());

        assert_eq!(metrics.findings_count, 7);
        assert_eq!(metrics.severity_distribution, severity_dist);
    }

    #[test]
    fn test_metrics_collector_new() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.total_execution_count(), 0);
        assert_eq!(collector.total_findings_count(), 0);
    }

    #[test]
    fn test_metrics_collector_record_execution() {
        let collector = MetricsCollector::new();
        let mut metrics = ExecutionMetrics::new("test-agent".to_string());
        metrics.record_findings(5, HashMap::new());
        metrics.complete(true, None);

        collector.record_execution(metrics);

        assert_eq!(collector.total_execution_count(), 1);
        assert_eq!(collector.total_findings_count(), 5);
    }

    #[test]
    fn test_metrics_collector_get_agent_stats() {
        let collector = MetricsCollector::new();
        let mut metrics = ExecutionMetrics::new("test-agent".to_string());
        metrics.record_findings(3, HashMap::new());
        metrics.complete(true, None);

        collector.record_execution(metrics);

        let stats = collector.get_agent_stats("test-agent");
        assert!(stats.is_some());

        let stat = stats.unwrap();
        assert_eq!(stat.agent_id, "test-agent");
        assert_eq!(stat.total_executions, 1);
        assert_eq!(stat.successful_executions, 1);
        assert_eq!(stat.failed_executions, 0);
        assert_eq!(stat.total_findings, 3);
    }

    #[test]
    fn test_metrics_collector_multiple_executions() {
        let collector = MetricsCollector::new();

        for i in 0..3 {
            let mut metrics = ExecutionMetrics::new("test-agent".to_string());
            metrics.record_findings(i + 1, HashMap::new());
            metrics.complete(true, None);
            collector.record_execution(metrics);
        }

        assert_eq!(collector.total_execution_count(), 3);
        assert_eq!(collector.total_findings_count(), 6); // 1 + 2 + 3

        let stats = collector.get_agent_stats("test-agent").unwrap();
        assert_eq!(stats.total_executions, 3);
        assert_eq!(stats.successful_executions, 3);
        assert_eq!(stats.total_findings, 6);
    }

    #[test]
    fn test_metrics_collector_mixed_success_failure() {
        let collector = MetricsCollector::new();

        // Success
        let mut metrics1 = ExecutionMetrics::new("test-agent".to_string());
        metrics1.record_findings(5, HashMap::new());
        metrics1.complete(true, None);
        collector.record_execution(metrics1);

        // Failure
        let mut metrics2 = ExecutionMetrics::new("test-agent".to_string());
        metrics2.record_findings(0, HashMap::new());
        metrics2.complete(false, Some("Error".to_string()));
        collector.record_execution(metrics2);

        let stats = collector.get_agent_stats("test-agent").unwrap();
        assert_eq!(stats.total_executions, 2);
        assert_eq!(stats.successful_executions, 1);
        assert_eq!(stats.failed_executions, 1);
        assert_eq!(stats.total_findings, 5);
    }

    #[test]
    fn test_metrics_collector_get_agent_executions() {
        let collector = MetricsCollector::new();

        let mut metrics1 = ExecutionMetrics::new("agent-1".to_string());
        metrics1.complete(true, None);
        collector.record_execution(metrics1);

        let mut metrics2 = ExecutionMetrics::new("agent-2".to_string());
        metrics2.complete(true, None);
        collector.record_execution(metrics2);

        let mut metrics3 = ExecutionMetrics::new("agent-1".to_string());
        metrics3.complete(true, None);
        collector.record_execution(metrics3);

        let agent1_executions = collector.get_agent_executions("agent-1");
        assert_eq!(agent1_executions.len(), 2);

        let agent2_executions = collector.get_agent_executions("agent-2");
        assert_eq!(agent2_executions.len(), 1);
    }

    #[test]
    fn test_metrics_collector_clear() {
        let collector = MetricsCollector::new();

        let mut metrics = ExecutionMetrics::new("test-agent".to_string());
        metrics.complete(true, None);
        collector.record_execution(metrics);

        assert_eq!(collector.total_execution_count(), 1);

        collector.clear();

        assert_eq!(collector.total_execution_count(), 0);
        assert_eq!(collector.total_findings_count(), 0);
    }

    #[test]
    fn test_metrics_collector_average_execution_time() {
        let collector = MetricsCollector::new();

        for _ in 0..3 {
            let mut metrics = ExecutionMetrics::new("test-agent".to_string());
            metrics.complete(true, None);
            collector.record_execution(metrics);
        }

        let avg = collector.average_execution_time_ms();
        assert!(avg >= 0.0);
    }

    #[test]
    fn test_metrics_collector_all_agent_stats() {
        let collector = MetricsCollector::new();

        let mut metrics1 = ExecutionMetrics::new("agent-1".to_string());
        metrics1.complete(true, None);
        collector.record_execution(metrics1);

        let mut metrics2 = ExecutionMetrics::new("agent-2".to_string());
        metrics2.complete(true, None);
        collector.record_execution(metrics2);

        let all_stats = collector.all_agent_stats();
        assert_eq!(all_stats.len(), 2);
    }

    #[test]
    fn test_metrics_collector_all_executions() {
        let collector = MetricsCollector::new();

        let mut metrics1 = ExecutionMetrics::new("agent-1".to_string());
        metrics1.complete(true, None);
        collector.record_execution(metrics1);

        let mut metrics2 = ExecutionMetrics::new("agent-2".to_string());
        metrics2.complete(true, None);
        collector.record_execution(metrics2);

        let all_executions = collector.all_executions();
        assert_eq!(all_executions.len(), 2);
    }

    #[test]
    fn test_agent_stats_min_max_execution_time() {
        let collector = MetricsCollector::new();

        // First execution
        let mut metrics1 = ExecutionMetrics::new("test-agent".to_string());
        metrics1.complete(true, None);
        let duration1 = metrics1.duration_ms();
        collector.record_execution(metrics1);

        // Second execution
        let mut metrics2 = ExecutionMetrics::new("test-agent".to_string());
        metrics2.complete(true, None);
        let duration2 = metrics2.duration_ms();
        collector.record_execution(metrics2);

        let stats = collector.get_agent_stats("test-agent").unwrap();
        assert!(stats.min_execution_time_ms <= stats.max_execution_time_ms);
        assert!(stats.min_execution_time_ms <= duration1.max(duration2));
        assert!(stats.max_execution_time_ms >= duration1.min(duration2));
    }
}
