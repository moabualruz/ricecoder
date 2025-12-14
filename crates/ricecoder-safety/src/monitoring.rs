//! Safety monitoring and alerting

use crate::error::{SafetyError, SafetyResult};
use crate::risk::{RiskLevel, RiskScore};
use crate::validation::ValidationResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Safety monitoring metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyMetrics {
    /// Total validations performed
    pub total_validations: u64,
    /// Validations passed
    pub validations_passed: u64,
    /// Validations failed
    pub validations_failed: u64,
    /// Approvals requested
    pub approvals_requested: u64,
    /// Approvals granted
    pub approvals_granted: u64,
    /// Approvals denied
    pub approvals_denied: u64,
    /// Risk assessments performed
    pub risk_assessments: u64,
    /// Average risk score
    pub average_risk_score: f64,
    /// High-risk operations detected
    pub high_risk_operations: u64,
    /// Critical violations detected
    pub critical_violations: u64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertLevel {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Critical alert requiring immediate attention
    Critical,
}

/// Safety alert for monitoring violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyAlert {
    /// Unique alert ID
    pub id: String,
    /// Alert level
    pub level: AlertLevel,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Affected user (if applicable)
    pub user_id: Option<String>,
    /// Affected session (if applicable)
    pub session_id: Option<String>,
    /// Timestamp of the alert
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional alert data
    pub data: HashMap<String, serde_json::Value>,
}

/// Safety monitor for tracking system safety
pub struct SafetyMonitor {
    metrics: Arc<RwLock<SafetyMetrics>>,
    alerts: RwLock<Vec<SafetyAlert>>,
    alert_handlers: RwLock<Vec<Box<dyn AlertHandler + Send + Sync>>>,
    max_alerts: usize,
}

impl SafetyMonitor {
    /// Create a new safety monitor
    pub fn new(max_alerts: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(SafetyMetrics {
                total_validations: 0,
                validations_passed: 0,
                validations_failed: 0,
                approvals_requested: 0,
                approvals_granted: 0,
                approvals_denied: 0,
                risk_assessments: 0,
                average_risk_score: 0.0,
                high_risk_operations: 0,
                critical_violations: 0,
                last_updated: chrono::Utc::now(),
            })),
            alerts: RwLock::new(Vec::new()),
            alert_handlers: RwLock::new(Vec::new()),
            max_alerts,
        }
    }

    /// Record a validation result
    pub async fn record_validation(&self, result: &ValidationResult) -> SafetyResult<()> {
        let mut metrics = self.metrics.write().await;

        metrics.total_validations += 1;

        match result {
            ValidationResult::Passed { .. } => {
                metrics.validations_passed += 1;
            }
            ValidationResult::Failed { violations, .. } => {
                metrics.validations_failed += 1;

                // Check for critical violations
                let critical_count = violations.iter()
                    .filter(|v| matches!(v.severity, crate::constraints::ConstraintSeverity::Critical))
                    .count();

                metrics.critical_violations += critical_count as u64;

                // Generate alerts for critical violations
                if critical_count > 0 {
                    self.generate_alert(
                        AlertLevel::Critical,
                        "Critical Safety Violation".to_string(),
                        format!("{} critical safety violations detected", critical_count),
                        None,
                        None,
                        HashMap::from([
                            ("violation_count".to_string(), serde_json::json!(critical_count)),
                        ]),
                    ).await?;
                }
            }
            ValidationResult::ApprovalRequired { requests, .. } => {
                metrics.approvals_requested += requests.len() as u64;
            }
        }

        metrics.last_updated = chrono::Utc::now();
        Ok(())
    }

    /// Record a risk assessment
    pub async fn record_risk_assessment(&self, score: &RiskScore) -> SafetyResult<()> {
        let mut metrics = self.metrics.write().await;

        metrics.risk_assessments += 1;

        // Update average risk score
        let total_assessments = metrics.risk_assessments as f64;
        let current_avg = metrics.average_risk_score;
        metrics.average_risk_score = (current_avg * (total_assessments - 1.0) + score.score as f64) / total_assessments;

        // Track high-risk operations
        if score.level >= RiskLevel::High {
            metrics.high_risk_operations += 1;

            // Generate alert for high-risk operations
            self.generate_alert(
                AlertLevel::Warning,
                "High-Risk Operation Detected".to_string(),
                format!("Operation with risk score {} detected", score.score),
                None, // Would need user_id from context
                None, // Would need session_id from context
                HashMap::from([
                    ("risk_score".to_string(), serde_json::json!(score.score)),
                    ("risk_level".to_string(), serde_json::json!(score.level)),
                ]),
            ).await?;
        }

        metrics.last_updated = chrono::Utc::now();
        Ok(())
    }

    /// Record approval decision
    pub async fn record_approval(&self, granted: bool) -> SafetyResult<()> {
        let mut metrics = self.metrics.write().await;

        if granted {
            metrics.approvals_granted += 1;
        } else {
            metrics.approvals_denied += 1;
        }

        metrics.last_updated = chrono::Utc::now();
        Ok(())
    }

    /// Generate a safety alert
    pub async fn generate_alert(
        &self,
        level: AlertLevel,
        title: String,
        description: String,
        user_id: Option<String>,
        session_id: Option<String>,
        data: HashMap<String, serde_json::Value>,
    ) -> SafetyResult<()> {
        let alert = SafetyAlert {
            id: uuid::Uuid::new_v4().to_string(),
            level,
            title,
            description,
            user_id,
            session_id,
            timestamp: chrono::Utc::now(),
            data,
        };

        // Add to alerts list
        let mut alerts = self.alerts.write().await;
        alerts.push(alert.clone());

        // Maintain max alerts limit
        if alerts.len() > self.max_alerts {
            alerts.remove(0);
        }

        // Notify alert handlers
        let handlers = self.alert_handlers.read().await;
        for handler in handlers.iter() {
            if let Err(e) = handler.handle_alert(&alert).await {
                tracing::error!("Alert handler failed: {}", e);
            }
        }

        Ok(())
    }

    /// Add an alert handler
    pub async fn add_alert_handler(&self, handler: Box<dyn AlertHandler + Send + Sync>) -> SafetyResult<()> {
        self.alert_handlers.write().await.push(handler);
        Ok(())
    }

    /// Get current safety metrics
    pub async fn get_metrics(&self) -> SafetyMetrics {
        self.metrics.read().await.clone()
    }

    /// Get recent alerts
    pub async fn get_alerts(&self, limit: Option<usize>) -> Vec<SafetyAlert> {
        let alerts = self.alerts.read().await;
        let mut result: Vec<_> = alerts.iter().cloned().collect();

        // Sort by timestamp (newest first)
        result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = limit {
            result.truncate(limit);
        }

        result
    }

    /// Get alerts by level
    pub async fn get_alerts_by_level(&self, level: AlertLevel) -> Vec<SafetyAlert> {
        let alerts = self.alerts.read().await;
        alerts
            .iter()
            .filter(|alert| alert.level == level)
            .cloned()
            .collect()
    }

    /// Clear old alerts
    pub async fn clear_old_alerts(&self, max_age_hours: i64) -> SafetyResult<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours);
        let mut alerts = self.alerts.write().await;

        let initial_count = alerts.len();
        alerts.retain(|alert| alert.timestamp > cutoff);

        Ok(initial_count - alerts.len())
    }

    /// Check if safety thresholds are exceeded
    pub async fn check_thresholds(&self, thresholds: &SafetyThresholds) -> Vec<SafetyAlert> {
        let metrics = self.metrics.read().await;
        let mut alerts = Vec::new();

        if metrics.validations_failed as f64 / metrics.total_validations as f64 > thresholds.max_failure_rate {
            alerts.push(SafetyAlert {
                id: uuid::Uuid::new_v4().to_string(),
                level: AlertLevel::Critical,
                title: "High Validation Failure Rate".to_string(),
                description: format!(
                    "Validation failure rate {:.2}% exceeds threshold {:.2}%",
                    (metrics.validations_failed as f64 / metrics.total_validations as f64) * 100.0,
                    thresholds.max_failure_rate * 100.0
                ),
                user_id: None,
                session_id: None,
                timestamp: chrono::Utc::now(),
                data: HashMap::from([
                    ("failure_rate".to_string(), serde_json::json!((metrics.validations_failed as f64 / metrics.total_validations as f64))),
                    ("threshold".to_string(), serde_json::json!(thresholds.max_failure_rate)),
                ]),
            });
        }

        if metrics.average_risk_score > thresholds.max_average_risk_score {
            alerts.push(SafetyAlert {
                id: uuid::Uuid::new_v4().to_string(),
                level: AlertLevel::Warning,
                title: "High Average Risk Score".to_string(),
                description: format!(
                    "Average risk score {:.2} exceeds threshold {:.2}",
                    metrics.average_risk_score, thresholds.max_average_risk_score
                ),
                user_id: None,
                session_id: None,
                timestamp: chrono::Utc::now(),
                data: HashMap::from([
                    ("average_risk".to_string(), serde_json::json!(metrics.average_risk_score)),
                    ("threshold".to_string(), serde_json::json!(thresholds.max_average_risk_score)),
                ]),
            });
        }

        alerts
    }
}

/// Alert handler trait for processing safety alerts
#[async_trait::async_trait]
pub trait AlertHandler: Send + Sync {
    /// Handle a safety alert
    async fn handle_alert(&self, alert: &SafetyAlert) -> SafetyResult<()>;
}

/// Safety monitoring thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyThresholds {
    /// Maximum validation failure rate (0.0 to 1.0)
    pub max_failure_rate: f64,
    /// Maximum average risk score
    pub max_average_risk_score: f64,
    /// Maximum critical violations per hour
    pub max_critical_violations_per_hour: u32,
    /// Maximum high-risk operations per hour
    pub max_high_risk_operations_per_hour: u32,
}

impl Default for SafetyThresholds {
    fn default() -> Self {
        Self {
            max_failure_rate: 0.05, // 5%
            max_average_risk_score: 40.0,
            max_critical_violations_per_hour: 10,
            max_high_risk_operations_per_hour: 50,
        }
    }
}

impl Default for SafetyMonitor {
    fn default() -> Self {
        Self::new(1000)
    }
}