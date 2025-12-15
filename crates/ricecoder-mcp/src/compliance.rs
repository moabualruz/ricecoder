//! MCP Compliance Reporting and Enterprise Monitoring
//!
//! This module provides compliance reporting capabilities for SOC 2, GDPR, HIPAA,
//! and enterprise monitoring features for MCP operations.

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::error::{Error, Result};
use crate::audit::MCPAuditLogger;

/// Compliance report types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceReportType {
    Soc2Type2,
    Gdpr,
    Hipaa,
    Custom(String),
}

/// Compliance violation severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Compliance violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub report_type: ComplianceReportType,
    pub severity: ViolationSeverity,
    pub description: String,
    pub resource: String,
    pub user_id: Option<String>,
    pub details: serde_json::Value,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// MCP compliance monitor
pub struct MCPComplianceMonitor {
    violations: Arc<RwLock<VecDeque<ComplianceViolation>>>,
    audit_logger: Arc<MCPAuditLogger>,
    max_violations: usize,
    retention_days: i64,
}

impl MCPComplianceMonitor {
    /// Create a new compliance monitor
    pub fn new(audit_logger: Arc<MCPAuditLogger>) -> Self {
        Self {
            violations: Arc::new(RwLock::new(VecDeque::new())),
            audit_logger,
            max_violations: 10000,
            retention_days: 365,
        }
    }

    /// Configure the monitor
    pub fn with_config(mut self, max_violations: usize, retention_days: i64) -> Self {
        self.max_violations = max_violations;
        self.retention_days = retention_days;
        self
    }

    /// Record a compliance violation
    pub async fn record_violation(
        &self,
        report_type: ComplianceReportType,
        severity: ViolationSeverity,
        description: String,
        resource: String,
        user_id: Option<String>,
        details: serde_json::Value,
    ) -> Result<String> {
        let violation = ComplianceViolation {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            report_type,
            severity,
            description: description.clone(),
            resource: resource.clone(),
            user_id: user_id.clone(),
            details,
            resolved: false,
            resolved_at: None,
        };

        let mut violations = self.violations.write().await;

        // Enforce max violations limit
        if violations.len() >= self.max_violations {
            violations.pop_front();
        }

        violations.push_back(violation.clone());

        // Audit the violation
        if let Some(ref audit_logger) = self.audit_logger {
            let _ = audit_logger.log_protocol_validation(
                "compliance_violation",
                false,
                Some(&description),
                user_id.as_deref(),
                None,
            ).await;
        }

        warn!(
            "Compliance violation recorded: {} - {} for resource {} (severity: {:?})",
            violation.id, description, resource, severity
        );

        Ok(violation.id)
    }

    /// Mark a violation as resolved
    pub async fn resolve_violation(&self, violation_id: &str) -> Result<bool> {
        let mut violations = self.violations.write().await;

        for violation in violations.iter_mut() {
            if violation.id == violation_id {
                violation.resolved = true;
                violation.resolved_at = Some(Utc::now());

                info!("Compliance violation {} marked as resolved", violation_id);
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get violations by type and time range
    pub async fn get_violations(
        &self,
        report_type: Option<ComplianceReportType>,
        since: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<ComplianceViolation>> {
        let violations = self.violations.read().await;

        let mut filtered = violations
            .iter()
            .filter(|v| {
                if let Some(ref rt) = report_type {
                    if !std::mem::discriminant(&v.report_type) == std::mem::discriminant(rt) {
                        return false;
                    }
                }
                if let Some(since_time) = since {
                    if v.timestamp < since_time {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect::<Vec<_>>();

        // Sort by timestamp descending
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = limit {
            filtered.truncate(limit);
        }

        Ok(filtered)
    }

    /// Generate compliance report
    pub async fn generate_report(
        &self,
        report_type: ComplianceReportType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<ComplianceReport> {
        let violations = self.get_violations(Some(report_type.clone()), Some(start_date), None).await?;

        let total_violations = violations.len();
        let unresolved_violations = violations.iter().filter(|v| !v.resolved).count();

        let severity_counts = violations.iter().fold(HashMap::new(), |mut acc, v| {
            *acc.entry(v.severity.clone()).or_insert(0) += 1;
            acc
        });

        let report = ComplianceReport {
            report_type,
            generated_at: Utc::now(),
            period_start: start_date,
            period_end: end_date,
            total_violations,
            unresolved_violations,
            severity_breakdown: severity_counts,
            violations: violations.into_iter().take(100).collect(), // Include top 100 violations
            compliance_status: self.assess_compliance_status(&violations),
        };

        Ok(report)
    }

    /// Assess overall compliance status
    fn assess_compliance_status(&self, violations: &[ComplianceViolation]) -> ComplianceStatus {
        let critical_count = violations.iter().filter(|v| v.severity == ViolationSeverity::Critical && !v.resolved).count();
        let high_count = violations.iter().filter(|v| v.severity == ViolationSeverity::High && !v.resolved).count();

        if critical_count > 0 {
            ComplianceStatus::NonCompliant
        } else if high_count > 5 {
            ComplianceStatus::AtRisk
        } else {
            ComplianceStatus::Compliant
        }
    }

    /// Clean up old violations
    pub async fn cleanup_old_violations(&self) -> Result<usize> {
        let cutoff = Utc::now() - Duration::days(self.retention_days);
        let mut violations = self.violations.write().await;

        let initial_count = violations.len();
        violations.retain(|v| v.timestamp > cutoff);

        let removed_count = initial_count - violations.len();
        if removed_count > 0 {
            info!("Cleaned up {} old compliance violations", removed_count);
        }

        Ok(removed_count)
    }
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub report_type: ComplianceReportType,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_violations: usize,
    pub unresolved_violations: usize,
    pub severity_breakdown: HashMap<ViolationSeverity, usize>,
    pub violations: Vec<ComplianceViolation>,
    pub compliance_status: ComplianceStatus,
}

/// Compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    AtRisk,
    NonCompliant,
}

/// Enterprise monitoring metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPMonitoringMetrics {
    pub timestamp: DateTime<Utc>,
    pub server_count: usize,
    pub active_connections: usize,
    pub tool_executions_total: u64,
    pub tool_executions_success: u64,
    pub tool_executions_failed: u64,
    pub average_response_time_ms: f64,
    pub auth_attempts_total: u64,
    pub auth_attempts_success: u64,
    pub auth_attempts_failed: u64,
    pub violations_recorded: usize,
    pub active_sessions: usize,
}

/// MCP enterprise monitor
pub struct MCPEnterpriseMonitor {
    metrics_history: Arc<RwLock<VecDeque<MCPMonitoringMetrics>>>,
    max_history: usize,
}

impl MCPEnterpriseMonitor {
    /// Create a new enterprise monitor
    pub fn new() -> Self {
        Self {
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            max_history: 1000,
        }
    }

    /// Record monitoring metrics
    pub async fn record_metrics(&self, metrics: MCPMonitoringMetrics) {
        let mut history = self.metrics_history.write().await;

        if history.len() >= self.max_history {
            history.pop_front();
        }

        history.push_back(metrics.clone());

        info!(
            "MCP metrics recorded: {} servers, {} connections, {} tool executions ({}% success rate)",
            metrics.server_count,
            metrics.active_connections,
            metrics.tool_executions_total,
            if metrics.tool_executions_total > 0 {
                (metrics.tool_executions_success as f64 / metrics.tool_executions_total as f64 * 100.0) as u32
            } else {
                0
            }
        );
    }

    /// Get current metrics
    pub async fn get_current_metrics(&self) -> Option<MCPMonitoringMetrics> {
        let history = self.metrics_history.read().await;
        history.back().cloned()
    }

    /// Get metrics history
    pub async fn get_metrics_history(&self, limit: Option<usize>) -> Vec<MCPMonitoringMetrics> {
        let history = self.metrics_history.read().await;
        let limit = limit.unwrap_or(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Generate monitoring report
    pub async fn generate_monitoring_report(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<MonitoringReport> {
        let history = self.metrics_history.read().await;

        let relevant_metrics: Vec<_> = history
            .iter()
            .filter(|m| m.timestamp >= start_time && m.timestamp <= end_time)
            .collect();

        if relevant_metrics.is_empty() {
            return Err(Error::ValidationError("No metrics available for the specified time range".to_string()));
        }

        let avg_servers = relevant_metrics.iter().map(|m| m.server_count).sum::<usize>() as f64 / relevant_metrics.len() as f64;
        let avg_connections = relevant_metrics.iter().map(|m| m.active_connections).sum::<usize>() as f64 / relevant_metrics.len() as f64;
        let total_tool_executions = relevant_metrics.iter().map(|m| m.tool_executions_total).sum::<u64>();
        let total_auth_attempts = relevant_metrics.iter().map(|m| m.auth_attempts_total).sum::<u64>();

        let report = MonitoringReport {
            generated_at: Utc::now(),
            period_start: start_time,
            period_end: end_time,
            average_servers: avg_servers,
            average_connections: avg_connections,
            total_tool_executions,
            total_auth_attempts,
            metrics_samples: relevant_metrics.len(),
            health_status: self.assess_health_status(&relevant_metrics),
        };

        Ok(report)
    }

    /// Assess system health status
    fn assess_health_status(&self, metrics: &[&MCPMonitoringMetrics]) -> HealthStatus {
        if metrics.is_empty() {
            return HealthStatus::Unknown;
        }

        let latest = metrics.last().unwrap();

        // Calculate failure rates
        let tool_failure_rate = if latest.tool_executions_total > 0 {
            latest.tool_executions_failed as f64 / latest.tool_executions_total as f64
        } else {
            0.0
        };

        let auth_failure_rate = if latest.auth_attempts_total > 0 {
            latest.auth_attempts_failed as f64 / latest.auth_attempts_total as f64
        } else {
            0.0
        };

        if tool_failure_rate > 0.5 || auth_failure_rate > 0.3 || latest.violations_recorded > 10 {
            HealthStatus::Critical
        } else if tool_failure_rate > 0.2 || auth_failure_rate > 0.1 || latest.violations_recorded > 5 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Monitoring report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringReport {
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub average_servers: f64,
    pub average_connections: f64,
    pub total_tool_executions: u64,
    pub total_auth_attempts: u64,
    pub metrics_samples: usize,
    pub health_status: HealthStatus,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compliance_monitor_creation() {
        let storage = std::sync::Arc::new(ricecoder_security::audit::MemoryAuditStorage::new());
        let audit_logger = std::sync::Arc::new(ricecoder_security::audit::AuditLogger::new(storage));
        let mcp_audit_logger = std::sync::Arc::new(MCPAuditLogger::new(audit_logger));

        let monitor = MCPComplianceMonitor::new(mcp_audit_logger);
        assert_eq!(monitor.max_violations, 10000);
    }

    #[tokio::test]
    async fn test_record_violation() {
        let storage = std::sync::Arc::new(ricecoder_security::audit::MemoryAuditStorage::new());
        let audit_logger = std::sync::Arc::new(ricecoder_security::audit::AuditLogger::new(storage));
        let mcp_audit_logger = std::sync::Arc::new(MCPAuditLogger::new(audit_logger));

        let monitor = MCPComplianceMonitor::new(mcp_audit_logger);

        let result = monitor.record_violation(
            ComplianceReportType::Soc2Type2,
            ViolationSeverity::High,
            "Test violation".to_string(),
            "test-resource".to_string(),
            Some("test-user".to_string()),
            serde_json::json!({"test": "data"}),
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_enterprise_monitor_creation() {
        let monitor = MCPEnterpriseMonitor::new();
        assert_eq!(monitor.max_history, 1000);
    }
}</content>
<parameter name="filePath">projects/ricecoder/crates/ricecoder-mcp/src/compliance.rs