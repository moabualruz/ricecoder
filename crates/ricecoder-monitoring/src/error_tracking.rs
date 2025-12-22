//! Error tracking and alerting system

use std::{collections::HashMap, sync::Arc, time::Duration as StdDuration};

use chrono::{DateTime, TimeDelta, Utc};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, time};

use crate::types::*;
pub use crate::types::{AlertingConfig, ErrorTrackingConfig};

/// Global error event storage
static ERROR_EVENTS: Lazy<DashMap<EventId, ErrorEvent>> = Lazy::new(DashMap::new);

/// Global alert storage
static ALERTS: Lazy<DashMap<EventId, Alert>> = Lazy::new(DashMap::new);

/// Error tracker
pub struct ErrorTracker {
    config: ErrorTrackingConfig,
    shutdown_tx: Option<mpsc::Sender<()>>,
    tracking_task: Option<tokio::task::JoinHandle<()>>,
}

impl ErrorTracker {
    /// Create a new error tracker
    pub fn new(config: ErrorTrackingConfig) -> Self {
        Self {
            config,
            shutdown_tx: None,
            tracking_task: None,
        }
    }

    /// Start the error tracker
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync + '_>> {
        if !self.config.enabled {
            return Ok(());
        }

        // Initialize Sentry if DSN is provided
        if let Some(dsn) = &self.config.dsn {
            let _guard = sentry::init((
                dsn.clone(),
                sentry::ClientOptions {
                    release: self.config.release.clone().map(std::borrow::Cow::Owned),
                    environment: Some(self.config.environment.clone().into()),
                    sample_rate: self.config.sample_rate,
                    ..Default::default()
                },
            ));
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let task = tokio::spawn(async move {
            let mut interval = time::interval(StdDuration::from_secs(60)); // Cleanup interval

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        Self::cleanup_old_events();
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Error tracker shutting down");
                        break;
                    }
                }
            }
        });

        self.tracking_task = Some(task);
        Ok(())
    }

    /// Stop the error tracker
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        if let Some(task) = self.tracking_task.take() {
            let _ = task.await;
        }

        Ok(())
    }

    /// Track an error event
    pub fn track_error(&self, event: ErrorEvent) {
        // Store locally
        ERROR_EVENTS.insert(event.id, event.clone());

        // Send to Sentry if configured
        if self.config.enabled && self.config.dsn.is_some() {
            Self::send_to_sentry(&event);
        }

        // Log the error
        tracing::error!(
            error_id = %event.id,
            error_type = %event.error_type,
            message = %event.message,
            user_id = ?event.user_id,
            session_id = ?event.session_id,
            "Error tracked: {}",
            event.message
        );
    }

    /// Get error events with optional filtering
    pub fn get_error_events(
        &self,
        severity: Option<Severity>,
        since: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Vec<ErrorEvent> {
        let mut events: Vec<_> = ERROR_EVENTS
            .iter()
            .filter_map(|entry| {
                let event = entry.value();
                let matches_severity = severity.map_or(true, |s| event.severity == s);
                let matches_time = since.map_or(true, |t| event.timestamp >= t);

                if matches_severity && matches_time {
                    Some(event.clone())
                } else {
                    None
                }
            })
            .collect();

        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            events.truncate(limit);
        }

        events
    }

    /// Get error statistics
    pub fn get_error_stats(&self, since: Option<DateTime<Utc>>) -> ErrorStats {
        let events = self.get_error_events(None, since, None);

        let total_errors = events.len();
        let by_severity = events.iter().fold(HashMap::new(), |mut acc, event| {
            *acc.entry(event.severity).or_insert(0) += 1;
            acc
        });

        let by_type = events.iter().fold(HashMap::new(), |mut acc, event| {
            *acc.entry(event.error_type.clone()).or_insert(0) += 1;
            acc
        });

        ErrorStats {
            total_errors,
            errors_by_severity: by_severity,
            errors_by_type: by_type,
            period_start: since
                .unwrap_or_else(|| chrono::Utc::now() - chrono::TimeDelta::hours(24)),
            period_end: chrono::Utc::now(),
        }
    }

    /// Send error to Sentry
    fn send_to_sentry(event: &ErrorEvent) {
        sentry::capture_event(sentry::protocol::Event {
            message: Some(event.message.clone()),
            level: match event.severity {
                Severity::Low => sentry::Level::Info,
                Severity::Medium => sentry::Level::Warning,
                Severity::High => sentry::Level::Error,
                Severity::Critical => sentry::Level::Fatal,
            },
            tags: {
                let mut tags = std::collections::BTreeMap::new();
                tags.insert("error_type".to_string(), event.error_type.clone());
                if let Some(user_id) = &event.user_id {
                    tags.insert("user_id".to_string(), user_id.clone());
                }
                if let Some(session_id) = &event.session_id {
                    tags.insert("session_id".to_string(), session_id.clone());
                }
                tags
            },
            extra: event
                .context
                .clone()
                .into_iter()
                .map(|(k, v)| (k, sentry::protocol::Value::from(v)))
                .collect(),
            ..Default::default()
        });
    }

    /// Clean up old error events
    fn cleanup_old_events() {
        let cutoff = chrono::Utc::now() - chrono::TimeDelta::days(30); // Keep 30 days of errors

        ERROR_EVENTS.retain(|_, event| event.timestamp > cutoff);
    }
}

/// Error statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    pub total_errors: usize,
    pub errors_by_severity: HashMap<Severity, usize>,
    pub errors_by_type: HashMap<String, usize>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// Alert manager
pub struct AlertManager {
    config: AlertingConfig,
    shutdown_tx: Option<mpsc::Sender<()>>,
    alert_task: Option<tokio::task::JoinHandle<()>>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new(config: AlertingConfig) -> Self {
        Self {
            config,
            shutdown_tx: None,
            alert_task: None,
        }
    }

    /// Start the alert manager
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Ok(());
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let rules = self.config.rules.clone();
        let channels = self.config.channels.clone();

        let task = tokio::spawn(async move {
            let mut interval = time::interval(StdDuration::from_secs(30)); // Check interval

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::check_alert_rules(&rules, &channels).await {
                            tracing::error!("Failed to check alert rules: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Alert manager shutting down");
                        break;
                    }
                }
            }
        });

        self.alert_task = Some(task);
        Ok(())
    }

    /// Stop the alert manager
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        if let Some(task) = self.alert_task.take() {
            let _ = task.await;
        }

        Ok(())
    }

    /// Create a new alert
    pub fn create_alert(
        &self,
        rule_id: String,
        message: String,
        severity: Severity,
        labels: HashMap<String, String>,
    ) {
        let alert = Alert {
            id: EventId::new_v4(),
            rule_id: rule_id.clone(),
            message,
            severity,
            status: AlertStatus::Firing,
            created_at: chrono::Utc::now(),
            resolved_at: None,
            labels,
        };

        ALERTS.insert(alert.id, alert.clone());

        tracing::warn!(
            alert_id = %alert.id,
            rule_id = %rule_id,
            severity = ?severity,
            "Alert created: {}",
            alert.message
        );
    }

    /// Resolve an alert
    pub fn resolve_alert(&self, alert_id: EventId) {
        if let Some(mut alert) = ALERTS.get_mut(&alert_id) {
            alert.status = AlertStatus::Resolved;
            alert.resolved_at = Some(chrono::Utc::now());

            tracing::info!(
                alert_id = %alert_id,
                "Alert resolved: {}",
                alert.message
            );
        }
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        ALERTS
            .iter()
            .filter_map(|entry| {
                let alert = entry.value();
                if alert.status == AlertStatus::Firing {
                    Some(alert.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all alerts with optional filtering
    pub fn get_alerts(
        &self,
        status: Option<AlertStatus>,
        severity: Option<Severity>,
        since: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Vec<Alert> {
        let mut alerts: Vec<_> = ALERTS
            .iter()
            .filter_map(|entry| {
                let alert = entry.value();
                let matches_status = status.map_or(true, |s| alert.status == s);
                let matches_severity = severity.map_or(true, |s| alert.severity == s);
                let matches_time = since.map_or(true, |t| alert.created_at >= t);

                if matches_status && matches_severity && matches_time {
                    Some(alert.clone())
                } else {
                    None
                }
            })
            .collect();

        alerts.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = limit {
            alerts.truncate(limit);
        }

        alerts
    }

    /// Check alert rules and trigger alerts if needed
    async fn check_alert_rules(
        rules: &[AlertRule],
        channels: &[AlertChannel],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for rule in rules.iter().filter(|r| r.enabled) {
            if Self::evaluate_rule(rule).await {
                // Rule condition met, create alert if not already active
                let active_alerts: Vec<_> = ALERTS
                    .iter()
                    .filter(|entry| {
                        let alert = entry.value();
                        alert.rule_id == rule.id && alert.status == AlertStatus::Firing
                    })
                    .collect();

                if active_alerts.is_empty() {
                    // Create new alert
                    let alert = Alert {
                        id: EventId::new_v4(),
                        rule_id: rule.id.clone(),
                        message: format!(
                            "Alert rule '{}' triggered: {}",
                            rule.name, rule.description
                        ),
                        severity: rule.severity,
                        status: AlertStatus::Firing,
                        created_at: chrono::Utc::now(),
                        resolved_at: None,
                        labels: HashMap::new(),
                    };

                    ALERTS.insert(alert.id, alert.clone());

                    // Send notifications
                    Self::send_alert_notifications(&alert, channels).await?;
                }
            }
        }

        Ok(())
    }

    /// Evaluate an alert rule condition
    async fn evaluate_rule(rule: &AlertRule) -> bool {
        // This is a simplified evaluation - in practice, you'd parse the query
        // and evaluate it against metrics data
        // For now, just check if there are recent errors above threshold

        let recent_errors = ERROR_EVENTS
            .iter()
            .filter(|entry| {
                let event = entry.value();
                let one_hour_ago = chrono::Utc::now() - chrono::TimeDelta::hours(1);
                event.timestamp > one_hour_ago
            })
            .count();

        recent_errors as f64 >= rule.threshold
    }

    /// Send alert notifications
    async fn send_alert_notifications(
        alert: &Alert,
        channels: &[AlertChannel],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for channel in channels {
            match channel.channel_type {
                ChannelType::Email => {
                    Self::send_email_alert(alert, channel).await?;
                }
                ChannelType::Slack => {
                    Self::send_slack_alert(alert, channel).await?;
                }
                ChannelType::Webhook => {
                    Self::send_webhook_alert(alert, channel).await?;
                }
                ChannelType::PagerDuty => {
                    Self::send_pagerduty_alert(alert, channel).await?;
                }
                ChannelType::OpsGenie => {
                    Self::send_opsgenie_alert(alert, channel).await?;
                }
            }
        }

        Ok(())
    }

    /// Send email alert
    async fn send_email_alert(
        _alert: &Alert,
        _channel: &AlertChannel,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Email sending implementation would go here
        // For now, just log
        tracing::info!("Email alert sent (not implemented)");
        Ok(())
    }

    /// Send Slack alert
    async fn send_slack_alert(
        alert: &Alert,
        channel: &AlertChannel,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(webhook_url) = channel.config.get("webhook_url") {
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "text": format!("🚨 Alert: {}\nSeverity: {:?}\n{}", alert.message, alert.severity, alert.message),
                "username": "RiceCoder Monitor",
                "icon_emoji": ":warning:"
            });

            let _ = client.post(webhook_url).json(&payload).send().await?;
        }

        Ok(())
    }

    /// Send webhook alert
    async fn send_webhook_alert(
        alert: &Alert,
        channel: &AlertChannel,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(url) = channel.config.get("url") {
            let client = reqwest::Client::new();
            let _ = client.post(url).json(alert).send().await?;
        }

        Ok(())
    }

    /// Send PagerDuty alert
    async fn send_pagerduty_alert(
        _alert: &Alert,
        _channel: &AlertChannel,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // PagerDuty integration would go here
        tracing::info!("PagerDuty alert sent (not implemented)");
        Ok(())
    }

    /// Send OpsGenie alert
    async fn send_opsgenie_alert(
        _alert: &Alert,
        _channel: &AlertChannel,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // OpsGenie integration would go here
        tracing::info!("OpsGenie alert sent (not implemented)");
        Ok(())
    }
}

/// Incident response manager
pub struct IncidentManager {
    incidents: Arc<RwLock<HashMap<EventId, Incident>>>,
}

impl IncidentManager {
    pub fn new() -> Self {
        Self {
            incidents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new incident from an alert
    pub fn create_incident(&self, alert: &Alert) -> Incident {
        let incident = Incident {
            id: EventId::new_v4(),
            title: format!("Incident: {}", alert.message),
            description: alert.message.clone(),
            severity: alert.severity,
            status: IncidentStatus::Investigating,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            resolved_at: None,
            alerts: vec![alert.id],
            assignee: None,
            tags: alert.labels.clone(),
            timeline: vec![IncidentEvent {
                timestamp: chrono::Utc::now(),
                event_type: IncidentEventType::Created,
                description: "Incident created from alert".to_string(),
                user: None,
            }],
        };

        self.incidents.write().insert(incident.id, incident.clone());
        incident
    }

    /// Update incident status
    pub fn update_incident_status(
        &self,
        incident_id: EventId,
        status: IncidentStatus,
        user: Option<String>,
    ) {
        if let Some(mut incident) = self.incidents.write().get_mut(&incident_id) {
            incident.status = status;
            incident.updated_at = chrono::Utc::now();

            if status == IncidentStatus::Resolved {
                incident.resolved_at = Some(chrono::Utc::now());
            }

            incident.timeline.push(IncidentEvent {
                timestamp: chrono::Utc::now(),
                event_type: match status {
                    IncidentStatus::Investigating => IncidentEventType::StatusChanged,
                    IncidentStatus::Identified => IncidentEventType::StatusChanged,
                    IncidentStatus::Monitoring => IncidentEventType::StatusChanged,
                    IncidentStatus::Resolved => IncidentEventType::Resolved,
                    IncidentStatus::Closed => IncidentEventType::Closed,
                },
                description: format!("Status changed to {:?}", status),
                user,
            });
        }
    }

    /// Add event to incident timeline
    pub fn add_incident_event(
        &self,
        incident_id: EventId,
        event_type: IncidentEventType,
        description: String,
        user: Option<String>,
    ) {
        if let Some(mut incident) = self.incidents.write().get_mut(&incident_id) {
            incident.timeline.push(IncidentEvent {
                timestamp: chrono::Utc::now(),
                event_type,
                description,
                user,
            });
            incident.updated_at = chrono::Utc::now();
        }
    }

    /// Get incident by ID
    pub fn get_incident(&self, incident_id: EventId) -> Option<Incident> {
        self.incidents.read().get(&incident_id).cloned()
    }

    /// Get all incidents
    pub fn get_incidents(&self, status: Option<IncidentStatus>) -> Vec<Incident> {
        let incidents = self.incidents.read();
        let mut result: Vec<_> = incidents.values().cloned().collect();

        if let Some(status) = status {
            result.retain(|i| i.status == status);
        }

        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        result
    }
}

/// Incident
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: EventId,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub status: IncidentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub alerts: Vec<EventId>,
    pub assignee: Option<String>,
    pub tags: HashMap<String, String>,
    pub timeline: Vec<IncidentEvent>,
}

/// Incident status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentStatus {
    Investigating,
    Identified,
    Monitoring,
    Resolved,
    Closed,
}

/// Incident event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: IncidentEventType,
    pub description: String,
    pub user: Option<String>,
}

/// Incident event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentEventType {
    Created,
    StatusChanged,
    Comment,
    Assignment,
    Resolved,
    Closed,
}
