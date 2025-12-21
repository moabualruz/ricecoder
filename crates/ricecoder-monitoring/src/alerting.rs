//! Alerting systems and incident response

use crate::types::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub use crate::types::AlertingConfig;

/// Alert manager (re-export from error_tracking for convenience)
pub use crate::error_tracking::AlertManager;

/// Incident manager (re-export from error_tracking for convenience)
pub use crate::error_tracking::IncidentManager;

/// Alert rule engine
pub struct AlertRuleEngine;

impl AlertRuleEngine {
    /// Evaluate alert rules against metrics
    pub fn evaluate_rules(
        rules: &[AlertRule],
        metrics: &HashMap<String, Vec<DataPoint>>,
    ) -> Vec<Alert> {
        let mut alerts = Vec::new();

        for rule in rules.iter().filter(|r| r.enabled) {
            if let Some(alert) = Self::evaluate_rule(rule, metrics) {
                alerts.push(alert);
            }
        }

        alerts
    }

    /// Evaluate a single alert rule
    fn evaluate_rule(rule: &AlertRule, metrics: &HashMap<String, Vec<DataPoint>>) -> Option<Alert> {
        // Simple threshold-based evaluation
        // In a real implementation, this would parse the query and evaluate it
        if let Some(data_points) = metrics.get(&rule.query) {
            if let Some(latest) = data_points.last() {
                if latest.value >= rule.threshold {
                    return Some(Alert {
                        id: EventId::new_v4(),
                        rule_id: rule.id.clone(),
                        message: format!(
                            "Alert rule '{}' triggered: {} >= {}",
                            rule.name, latest.value, rule.threshold
                        ),
                        severity: rule.severity,
                        status: AlertStatus::Firing,
                        created_at: chrono::Utc::now(),
                        resolved_at: None,
                        labels: HashMap::new(),
                    });
                }
            }
        }

        None
    }
}
