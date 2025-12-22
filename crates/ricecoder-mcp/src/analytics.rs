use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::Result;

/// Types of operations that can be tracked in MCP analytics
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    ToolExecution,
    ServerRegistration,
    ServerDiscovery,
    ConnectionPoolAccess,
    HealthCheck,
    ErrorRecovery,
}

/// Individual usage data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPUsageData {
    pub timestamp: DateTime<Utc>,
    pub server_id: String,
    pub tool_name: Option<String>,
    pub operation_type: OperationType,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub success: bool,
    pub execution_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Aggregator for MCP usage analytics
#[derive(Debug)]
pub struct MCPAnalyticsAggregator {
    pub usage_data: Arc<RwLock<Vec<MCPUsageData>>>,
}

impl MCPAnalyticsAggregator {
    pub fn new() -> Self {
        Self {
            usage_data: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn record_usage(&self, data: MCPUsageData) {
        let mut usage_data = self.usage_data.write().await;
        usage_data.push(data);
    }

    pub async fn record_tool_execution(
        &self,
        server_id: &str,
        tool_name: &str,
        result: &crate::tool_execution::ToolExecutionResult,
        user_id: Option<String>,
        session_id: Option<String>,
    ) {
        use crate::tool_execution::ToolExecutionResult;
        let data = MCPUsageData {
            timestamp: chrono::Utc::now(),
            server_id: server_id.to_string(),
            tool_name: Some(tool_name.to_string()),
            operation_type: OperationType::ToolExecution,
            user_id,
            session_id,
            success: result.success,
            execution_time_ms: Some(result.execution_time_ms),
            error_message: result.error.as_ref().map(|e| e.error.clone()),
            metadata: result.metadata.clone(),
        };
        self.record_usage(data).await;
    }

    pub async fn record_health_check(
        &self,
        server_id: &str,
        success: bool,
        duration_ms: u64,
        user_id: Option<String>,
        session_id: Option<String>,
    ) {
        let data = MCPUsageData {
            timestamp: chrono::Utc::now(),
            server_id: server_id.to_string(),
            tool_name: None,
            operation_type: OperationType::HealthCheck,
            user_id,
            session_id,
            success,
            execution_time_ms: Some(duration_ms),
            error_message: if success {
                None
            } else {
                Some("Health check failed".to_string())
            },
            metadata: std::collections::HashMap::new(),
        };
        self.record_usage(data).await;
    }
}

impl Default for MCPAnalyticsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Usage report for a specific time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPUsageReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub average_execution_time_ms: Option<f64>,
    pub operations_by_type: HashMap<OperationType, u64>,
    pub top_tools: Vec<(String, u64)>,
    pub top_servers: Vec<(String, u64)>,
}

/// Key metrics for enterprise dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetrics {
    pub total_tool_executions: u64,
    pub active_servers: u64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
    pub concurrent_connections: u64,
}

/// Enterprise dashboard report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseDashboardReport {
    pub generated_at: DateTime<Utc>,
    pub key_metrics: KeyMetrics,
    pub usage_report: MCPUsageReport,
    pub alerts: Vec<String>,
}

/// Real-time dashboard snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeDashboardSnapshot {
    pub timestamp: DateTime<Utc>,
    pub active_connections: u64,
    pub pending_operations: u64,
    pub recent_errors: Vec<String>,
}

/// Enterprise dashboard for MCP analytics
pub struct MCPEnterpriseDashboard {
    analytics: Arc<MCPAnalyticsAggregator>,
}

impl MCPEnterpriseDashboard {
    pub fn new(analytics: Arc<MCPAnalyticsAggregator>) -> Self {
        Self { analytics }
    }

    pub async fn generate_dashboard_report(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<EnterpriseDashboardReport> {
        let usage_data = self.analytics.usage_data.read().await;
        let filtered_data: Vec<&MCPUsageData> = usage_data
            .iter()
            .filter(|data| data.timestamp >= start_time && data.timestamp <= end_time)
            .collect();

        let total_operations = filtered_data.len() as u64;
        let successful_operations = filtered_data.iter().filter(|d| d.success).count() as u64;
        let failed_operations = total_operations - successful_operations;

        let average_execution_time_ms = if !filtered_data.is_empty() {
            let total_time: u64 = filtered_data
                .iter()
                .filter_map(|d| d.execution_time_ms)
                .sum();
            let count = filtered_data
                .iter()
                .filter(|d| d.execution_time_ms.is_some())
                .count();
            if count > 0 {
                Some(total_time as f64 / count as f64)
            } else {
                None
            }
        } else {
            None
        };

        let mut operations_by_type = HashMap::new();
        for data in &filtered_data {
            *operations_by_type
                .entry(data.operation_type.clone())
                .or_insert(0) += 1;
        }

        let mut tool_counts = HashMap::new();
        let mut server_counts = HashMap::new();
        for data in &filtered_data {
            if let Some(tool) = &data.tool_name {
                *tool_counts.entry(tool.clone()).or_insert(0) += 1;
            }
            *server_counts.entry(data.server_id.clone()).or_insert(0) += 1;
        }

        let mut top_tools: Vec<(String, u64)> = tool_counts.into_iter().collect();
        top_tools.sort_by(|a, b| b.1.cmp(&a.1));
        top_tools.truncate(10);

        let active_servers_count = server_counts.len() as u64;
        let mut top_servers: Vec<(String, u64)> = server_counts.into_iter().collect();
        top_servers.sort_by(|a, b| b.1.cmp(&a.1));
        top_servers.truncate(10);

        let usage_report = MCPUsageReport {
            period_start: start_time,
            period_end: end_time,
            total_operations,
            successful_operations,
            failed_operations,
            average_execution_time_ms,
            operations_by_type,
            top_tools,
            top_servers,
        };

        let key_metrics = KeyMetrics {
            total_tool_executions: filtered_data
                .iter()
                .filter(|d| matches!(d.operation_type, OperationType::ToolExecution))
                .count() as u64,
            active_servers: active_servers_count,
            average_response_time_ms: average_execution_time_ms.unwrap_or(0.0),
            error_rate: if total_operations > 0 {
                failed_operations as f64 / total_operations as f64
            } else {
                0.0
            },
            concurrent_connections: 0, // Placeholder
        };

        Ok(EnterpriseDashboardReport {
            generated_at: Utc::now(),
            key_metrics,
            usage_report,
            alerts: vec![], // Placeholder
        })
    }
}
