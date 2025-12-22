//! Security monitoring and threat detection capabilities

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{audit::AuditLogger, Result, SecurityError};

/// Security event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityEventType {
    FailedLogin,
    SuspiciousActivity,
    BruteForceAttempt,
    SqlInjectionAttempt,
    XssAttempt,
    PathTraversalAttempt,
    UnauthorizedAccess,
    DataExfiltration,
    AnomalousBehavior,
    DdosAttempt,
    MalwareDetected,
}

/// Threat level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Security event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: String,
    pub event_type: SecurityEventType,
    pub timestamp: DateTime<Utc>,
    pub source_ip: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub resource: String,
    pub details: serde_json::Value,
    pub threat_level: ThreatLevel,
    pub mitigated: bool,
}

/// Security alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: ThreatLevel,
    pub events: Vec<SecurityEvent>,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub actions_taken: Vec<String>,
}

/// Threat detection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub event_pattern: EventPattern,
    pub threshold: u32,
    pub window_seconds: i64,
    pub severity: ThreatLevel,
    pub enabled: bool,
}

/// Event pattern for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPattern {
    EventType(SecurityEventType),
    Regex {
        field: String,
        pattern: String,
    },
    Composite {
        patterns: Vec<Box<EventPattern>>,
        operator: PatternOperator,
    },
}

/// Pattern operator for composite patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternOperator {
    And,
    Or,
}

/// Security monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub max_events_per_window: usize,
    pub alert_cooldown_seconds: i64,
    pub enable_anomaly_detection: bool,
    pub anomaly_sensitivity: f64,
    pub geo_blocking_enabled: bool,
    pub rate_limiting_enabled: bool,
    pub rate_limit_requests_per_minute: u32,
}

/// Security monitor for real-time threat detection
pub struct SecurityMonitor {
    config: MonitorConfig,
    audit_logger: Arc<AuditLogger>,
    events: Arc<RwLock<VecDeque<SecurityEvent>>>,
    alerts: Arc<RwLock<Vec<SecurityAlert>>>,
    threat_detector: Arc<RwLock<ThreatDetector>>,
    active_rules: HashMap<String, ThreatRule>,
}

/// Threat detector for pattern-based detection
pub struct ThreatDetector {
    rules: HashMap<String, ThreatRule>,
    event_counts: HashMap<String, Vec<(DateTime<Utc>, u32)>>,
}

impl SecurityMonitor {
    /// Create a new security monitor
    pub fn new(config: MonitorConfig, audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            config,
            audit_logger,
            events: Arc::new(RwLock::new(VecDeque::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            threat_detector: Arc::new(RwLock::new(ThreatDetector::new())),
            active_rules: HashMap::new(),
        }
    }

    /// Add a threat detection rule
    pub async fn add_rule(&mut self, rule: ThreatRule) {
        let mut detector = self.threat_detector.write().await;
        detector.add_rule(rule.clone());
        if rule.enabled {
            self.active_rules.insert(rule.id.clone(), rule);
        }
    }

    /// Record a security event
    pub async fn record_event(&self, event: SecurityEvent) -> Result<()> {
        // Add to event queue
        let mut events = self.events.write().await;
        events.push_back(event.clone());

        // Maintain max events limit
        while events.len() > self.config.max_events_per_window {
            events.pop_front();
        }
        drop(events);

        // Check for threats
        let mut detector = self.threat_detector.write().await;
        if let Some(alert) = detector.detect_threat(&event).await? {
            let mut alerts = self.alerts.write().await;
            alerts.push(alert);
        }

        // Audit the security event
        self.audit_logger
            .log_event(crate::audit::AuditEvent {
                event_type: crate::audit::AuditEventType::SecurityViolation,
                user_id: event.user_id.clone(),
                session_id: event.session_id.clone(),
                action: "security_event".to_string(),
                resource: event.resource.clone(),
                metadata: serde_json::json!({
                    "event_type": format!("{:?}", event.event_type),
                    "threat_level": format!("{:?}", event.threat_level),
                    "source_ip": event.source_ip
                }),
            })
            .await?;

        Ok(())
    }

    /// Get recent security events
    pub async fn get_recent_events(&self, limit: usize) -> Result<Vec<SecurityEvent>> {
        let events = self.events.read().await;
        let recent: Vec<SecurityEvent> = events.iter().rev().take(limit).cloned().collect();
        Ok(recent)
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Result<Vec<SecurityAlert>> {
        let alerts = self.alerts.read().await;
        let active: Vec<SecurityAlert> = alerts
            .iter()
            .filter(|alert| alert.resolved_at.is_none())
            .cloned()
            .collect();
        Ok(active)
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: &str, actions_taken: Vec<String>) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved_at = Some(Utc::now());
            alert.actions_taken = actions_taken.clone();
        }

        // Audit alert resolution
        self.audit_logger
            .log_event(crate::audit::AuditEvent {
                event_type: crate::audit::AuditEventType::SecurityViolation,
                user_id: None,
                session_id: None,
                action: "alert_resolved".to_string(),
                resource: format!("alert:{}", alert_id),
                metadata: serde_json::json!({
                    "actions_taken": actions_taken
                }),
            })
            .await?;

        Ok(())
    }

    /// Analyze events for anomalies
    pub async fn analyze_anomalies(&self) -> Result<Vec<SecurityAlert>> {
        if !self.config.enable_anomaly_detection {
            return Ok(Vec::new());
        }

        let events = self.events.read().await;
        let mut alerts = Vec::new();

        // Simple anomaly detection based on event frequency
        let now = Utc::now();
        let window_start = now - Duration::minutes(5);

        let recent_events: Vec<&SecurityEvent> = events
            .iter()
            .filter(|e| e.timestamp > window_start)
            .collect();

        let event_counts: HashMap<SecurityEventType, usize> =
            recent_events.iter().fold(HashMap::new(), |mut acc, event| {
                *acc.entry(event.event_type.clone()).or_insert(0) += 1;
                acc
            });

        // Flag anomalies based on thresholds
        for (event_type, count) in event_counts {
            let threshold = match event_type {
                SecurityEventType::FailedLogin => 10,
                SecurityEventType::BruteForceAttempt => 5,
                SecurityEventType::SqlInjectionAttempt => 3,
                SecurityEventType::XssAttempt => 3,
                _ => 20, // Default threshold
            };

            if count > threshold {
                let event_type_str = format!("{:?}", event_type);
                let alert = SecurityAlert {
                    id: format!(
                        "anomaly_{}_{}",
                        event_type_str.to_lowercase(),
                        now.timestamp()
                    ),
                    title: format!("Anomalous {} Activity", event_type_str),
                    description: format!(
                        "Detected {} occurrences of {} in 5 minutes (threshold: {})",
                        count, event_type_str, threshold
                    ),
                    severity: ThreatLevel::High,
                    events: recent_events
                        .iter()
                        .filter(|e| {
                            std::mem::discriminant(&e.event_type)
                                == std::mem::discriminant(&event_type)
                        })
                        .take(10)
                        .map(|e| (*e).clone())
                        .collect(),
                    triggered_at: now,
                    resolved_at: None,
                    actions_taken: vec![],
                };
                alerts.push(alert);
            }
        }

        Ok(alerts)
    }

    /// Get security metrics
    pub async fn get_security_metrics(&self) -> Result<serde_json::Value> {
        let events = self.events.read().await;
        let alerts = self.alerts.read().await;

        let now = Utc::now();
        let last_24h = now - Duration::hours(24);
        let last_hour = now - Duration::hours(1);

        let recent_events: Vec<&SecurityEvent> =
            events.iter().filter(|e| e.timestamp > last_24h).collect();

        let hourly_events: Vec<&SecurityEvent> =
            events.iter().filter(|e| e.timestamp > last_hour).collect();

        let threat_distribution: HashMap<String, usize> =
            recent_events.iter().fold(HashMap::new(), |mut acc, event| {
                let level = format!("{:?}", event.threat_level);
                *acc.entry(level).or_insert(0) += 1;
                acc
            });

        let event_type_distribution: HashMap<String, usize> =
            recent_events.iter().fold(HashMap::new(), |mut acc, event| {
                let event_type = format!("{:?}", event.event_type);
                *acc.entry(event_type).or_insert(0) += 1;
                acc
            });

        let metrics = serde_json::json!({
            "total_events_24h": recent_events.len(),
            "events_per_hour": hourly_events.len(),
            "active_alerts": alerts.iter().filter(|a| a.resolved_at.is_none()).count(),
            "threat_distribution": threat_distribution,
            "event_type_distribution": event_type_distribution,
            "rules_active": self.active_rules.len()
        });

        Ok(metrics)
    }
}

impl ThreatDetector {
    /// Create a new threat detector
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            event_counts: HashMap::new(),
        }
    }

    /// Add a detection rule
    pub fn add_rule(&mut self, rule: ThreatRule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    /// Detect threats based on rules
    pub async fn detect_threat(&mut self, event: &SecurityEvent) -> Result<Option<SecurityAlert>> {
        let mut triggered_rules = Vec::new();

        for rule in self.rules.values() {
            if !rule.enabled {
                continue;
            }

            if self.matches_pattern(event, &rule.event_pattern) {
                let unknown_ip = "unknown".to_string();
                let ip = event.source_ip.as_ref().unwrap_or(&unknown_ip);
                let count_key = format!("{}_{}", rule.id, ip);
                let now = Utc::now();

                // Clean old counts
                if let Some(counts) = self.event_counts.get_mut(&count_key) {
                    counts.retain(|(timestamp, _)| {
                        now.signed_duration_since(*timestamp).num_seconds() < rule.window_seconds
                    });
                }

                // Add current event
                self.event_counts
                    .entry(count_key.clone())
                    .or_insert_with(Vec::new)
                    .push((now, 1));

                // Check threshold
                let total_count: u32 = self.event_counts[&count_key]
                    .iter()
                    .map(|(_, count)| count)
                    .sum();

                if total_count >= rule.threshold {
                    triggered_rules.push(rule.clone());
                }
            }
        }

        if triggered_rules.is_empty() {
            return Ok(None);
        }

        // Create alert from highest severity rule
        let highest_severity_rule = triggered_rules
            .into_iter()
            .max_by_key(|r| r.severity.clone())
            .unwrap();

        let alert = SecurityAlert {
            id: format!(
                "threat_{}_{}",
                highest_severity_rule.id,
                Utc::now().timestamp()
            ),
            title: format!("Threat Detected: {}", highest_severity_rule.name),
            description: highest_severity_rule.description.clone(),
            severity: highest_severity_rule.severity,
            events: vec![event.clone()],
            triggered_at: Utc::now(),
            resolved_at: None,
            actions_taken: vec![],
        };

        Ok(Some(alert))
    }

    /// Check if event matches pattern
    fn matches_pattern(&self, event: &SecurityEvent, pattern: &EventPattern) -> bool {
        match pattern {
            EventPattern::EventType(event_type) => {
                std::mem::discriminant(event_type) == std::mem::discriminant(&event.event_type)
            }
            EventPattern::Regex { field, pattern } => {
                let empty = "".to_string();
                let value = match field.as_str() {
                    "source_ip" => event.source_ip.as_ref().unwrap_or(&empty),
                    "user_id" => event.user_id.as_ref().unwrap_or(&empty),
                    "resource" => &event.resource,
                    _ => return false,
                };

                Regex::new(pattern).map_or(false, |re| re.is_match(value))
            }
            EventPattern::Composite { patterns, operator } => match operator {
                PatternOperator::And => patterns.iter().all(|p| self.matches_pattern(event, p)),
                PatternOperator::Or => patterns.iter().any(|p| self.matches_pattern(event, p)),
            },
        }
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            max_events_per_window: 10000,
            alert_cooldown_seconds: 300,
            enable_anomaly_detection: true,
            anomaly_sensitivity: 0.8,
            geo_blocking_enabled: false,
            rate_limiting_enabled: true,
            rate_limit_requests_per_minute: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_monitor_creation() {
        let config = MonitorConfig::default();
        let storage = Arc::new(crate::audit::MemoryAuditStorage::new());
        let audit_logger = Arc::new(crate::audit::AuditLogger::new(storage));
        let monitor = SecurityMonitor::new(config, audit_logger);

        let metrics = monitor.get_security_metrics().await.unwrap();
        assert!(metrics.get("total_events_24h").is_some());
    }

    #[tokio::test]
    async fn test_threat_detection() {
        let mut detector = ThreatDetector::new();

        let rule = ThreatRule {
            id: "test_rule".to_string(),
            name: "Test Failed Login Rule".to_string(),
            description: "Detect multiple failed logins".to_string(),
            event_pattern: EventPattern::EventType(SecurityEventType::FailedLogin),
            threshold: 3,
            window_seconds: 60,
            severity: ThreatLevel::Medium,
            enabled: true,
        };

        detector.add_rule(rule);

        let event = SecurityEvent {
            id: "test_event".to_string(),
            event_type: SecurityEventType::FailedLogin,
            timestamp: Utc::now(),
            source_ip: Some("192.168.1.1".to_string()),
            user_id: Some("user123".to_string()),
            session_id: None,
            resource: "login".to_string(),
            details: serde_json::json!({"attempt": 1}),
            threat_level: ThreatLevel::Low,
            mitigated: false,
        };

        // First two events shouldn't trigger
        assert!(detector.detect_threat(&event).await.unwrap().is_none());
        assert!(detector.detect_threat(&event).await.unwrap().is_none());

        // Third event should trigger alert
        let alert = detector.detect_threat(&event).await.unwrap();
        assert!(alert.is_some());
        assert_eq!(alert.as_ref().unwrap().severity, ThreatLevel::Medium);
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let config = MonitorConfig {
            enable_anomaly_detection: true,
            ..Default::default()
        };
        let storage = Arc::new(crate::audit::MemoryAuditStorage::new());
        let audit_logger = Arc::new(crate::audit::AuditLogger::new(storage));
        let monitor = SecurityMonitor::new(config, audit_logger);

        // Add many failed login events
        for i in 0..15 {
            let event = SecurityEvent {
                id: format!("event_{}", i),
                event_type: SecurityEventType::FailedLogin,
                timestamp: Utc::now(),
                source_ip: Some("192.168.1.1".to_string()),
                user_id: Some("user123".to_string()),
                session_id: None,
                resource: "login".to_string(),
                details: serde_json::json!({"attempt": i}),
                threat_level: ThreatLevel::Low,
                mitigated: false,
            };

            monitor.record_event(event).await.unwrap();
        }

        let anomalies = monitor.analyze_anomalies().await.unwrap();
        assert!(!anomalies.is_empty());
        assert!(anomalies[0].title.contains("FailedLogin"));
    }
}
