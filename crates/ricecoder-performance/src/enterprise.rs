//! Enterprise monitoring and alerting system

use crate::monitor::PerformanceMetrics;
use crate::regression::RegressionAlert;
use crate::validation::ValidationResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enterprise performance monitor with alerting capabilities
pub struct EnterpriseMonitor {
    alert_config: AlertConfig,
    alert_history: Vec<AlertRecord>,
    performance_history: Vec<PerformanceRecord>,
    max_history_size: usize,
}

impl EnterpriseMonitor {
    /// Create a new enterprise monitor
    pub fn new(alert_config: AlertConfig) -> Self {
        Self {
            alert_config,
            alert_history: Vec::new(),
            performance_history: Vec::new(),
            max_history_size: 10000, // Keep last 10k records
        }
    }

    /// Monitor performance and check for alerts
    pub async fn monitor_performance(
        &mut self,
        metrics: &[PerformanceMetrics],
    ) -> Vec<EnterpriseAlert> {
        let mut alerts = Vec::new();
        let now = Utc::now();

        // Record performance data
        for metric in metrics {
            self.performance_history.push(PerformanceRecord {
                metric: metric.clone(),
                timestamp: now,
            });
        }

        // Maintain history size
        if self.performance_history.len() > self.max_history_size {
            let excess = self.performance_history.len() - self.max_history_size;
            self.performance_history.drain(0..excess);
        }

        // Check for performance alerts
        alerts.extend(self.check_performance_alerts(metrics, now).await);

        // Check for trend alerts
        alerts.extend(self.check_trend_alerts(now).await);

        // Check for anomaly alerts
        alerts.extend(self.check_anomaly_alerts(now).await);

        // Record alerts
        for alert in &alerts {
            self.alert_history.push(AlertRecord {
                alert: alert.clone(),
                timestamp: now,
            });
        }

        // Send alerts to configured destinations
        self.send_alerts(&alerts).await;

        alerts
    }

    /// Monitor validation results
    pub async fn monitor_validation(
        &mut self,
        results: &[ValidationResult],
    ) -> Vec<EnterpriseAlert> {
        let mut alerts = Vec::new();
        let now = Utc::now();

        for result in results {
            if !result.passed {
                let alert = EnterpriseAlert {
                    alert_type: AlertType::ValidationFailure,
                    severity: AlertSeverity::Critical,
                    title: format!("Performance validation failed: {}", result.test_name),
                    description: format!(
                        "Test '{}' failed with P95 time: {:.2}ms",
                        result.test_name,
                        result.metrics.p95_time_ns as f64 / 1_000_000.0
                    ),
                    affected_systems: vec![result.test_name.clone()],
                    recommended_actions: vec![
                        "Review performance metrics".to_string(),
                        "Check for regressions".to_string(),
                        "Consider optimization".to_string(),
                    ],
                    metadata: HashMap::from([
                        ("test_name".to_string(), result.test_name.clone()),
                        (
                            "p95_time_ns".to_string(),
                            result.metrics.p95_time_ns.to_string(),
                        ),
                        (
                            "memory_mb".to_string(),
                            (result.metrics.peak_memory_bytes as f64 / (1024.0 * 1024.0))
                                .to_string(),
                        ),
                    ]),
                };
                alerts.push(alert);
            }
        }

        // Send alerts
        self.send_alerts(&alerts).await;

        alerts
    }

    /// Check for performance threshold alerts
    async fn check_performance_alerts(
        &self,
        metrics: &[PerformanceMetrics],
        now: DateTime<Utc>,
    ) -> Vec<EnterpriseAlert> {
        let mut alerts = Vec::new();

        for metric in metrics {
            // Check startup time alert
            if metric.test_name.contains("startup") && metric.p95_time_ns > 3_000_000_000 {
                alerts.push(EnterpriseAlert {
                    alert_type: AlertType::PerformanceThreshold,
                    severity: AlertSeverity::Critical,
                    title: "Startup Time Exceeded".to_string(),
                    description: format!(
                        "Application startup time exceeded 3s threshold: {:.2}s",
                        metric.p95_time_ns as f64 / 1_000_000_000.0
                    ),
                    affected_systems: vec!["application_startup".to_string()],
                    recommended_actions: vec![
                        "Profile startup code".to_string(),
                        "Optimize initialization".to_string(),
                        "Consider lazy loading".to_string(),
                    ],
                    metadata: HashMap::from([
                        ("threshold_ns".to_string(), "3000000000".to_string()),
                        ("actual_ns".to_string(), metric.p95_time_ns.to_string()),
                    ]),
                });
            }

            // Check response time alert
            if metric.test_name.contains("response") && metric.p95_time_ns > 500_000_000 {
                alerts.push(EnterpriseAlert {
                    alert_type: AlertType::PerformanceThreshold,
                    severity: AlertSeverity::High,
                    title: "Response Time Exceeded".to_string(),
                    description: format!(
                        "Response time exceeded 500ms threshold: {:.2}ms",
                        metric.p95_time_ns as f64 / 1_000_000.0
                    ),
                    affected_systems: vec!["api_responses".to_string()],
                    recommended_actions: vec![
                        "Optimize database queries".to_string(),
                        "Implement caching".to_string(),
                        "Review network calls".to_string(),
                    ],
                    metadata: HashMap::from([
                        ("threshold_ns".to_string(), "500000000".to_string()),
                        ("actual_ns".to_string(), metric.p95_time_ns.to_string()),
                    ]),
                });
            }

            // Check memory usage alert
            if metric.peak_memory_bytes > 300 * 1024 * 1024 {
                alerts.push(EnterpriseAlert {
                    alert_type: AlertType::ResourceThreshold,
                    severity: AlertSeverity::High,
                    title: "Memory Usage Exceeded".to_string(),
                    description: format!(
                        "Memory usage exceeded 300MB threshold: {:.1}MB",
                        metric.peak_memory_bytes as f64 / (1024.0 * 1024.0)
                    ),
                    affected_systems: vec!["memory_management".to_string()],
                    recommended_actions: vec![
                        "Profile memory allocations".to_string(),
                        "Implement memory pooling".to_string(),
                        "Check for memory leaks".to_string(),
                    ],
                    metadata: HashMap::from([
                        ("threshold_bytes".to_string(), "314572800".to_string()),
                        (
                            "actual_bytes".to_string(),
                            metric.peak_memory_bytes.to_string(),
                        ),
                    ]),
                });
            }
        }

        alerts
    }

    /// Check for performance trend alerts
    async fn check_trend_alerts(&self, now: DateTime<Utc>) -> Vec<EnterpriseAlert> {
        let mut alerts = Vec::new();

        // Analyze recent performance trends (last hour)
        let one_hour_ago = now - chrono::Duration::hours(1);
        let recent_metrics: Vec<_> = self
            .performance_history
            .iter()
            .filter(|record| record.timestamp > one_hour_ago)
            .collect();

        if recent_metrics.len() < 10 {
            return alerts; // Not enough data for trend analysis
        }

        // Group by test name and check for degradation trends
        let mut trends: HashMap<String, Vec<&PerformanceRecord>> = HashMap::new();
        for record in &recent_metrics {
            trends
                .entry(record.metric.test_name.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }

        for (test_name, records) in trends {
            if records.len() < 5 {
                continue; // Need at least 5 data points
            }

            // Calculate trend (simple linear regression slope)
            let slope = self.calculate_trend_slope(&records);

            if slope > 0.1 {
                // Significant upward trend (degradation)
                alerts.push(EnterpriseAlert {
                    alert_type: AlertType::PerformanceTrend,
                    severity: AlertSeverity::Medium,
                    title: format!("Performance Degradation Trend: {}", test_name),
                    description: format!(
                        "Performance is degrading over time for '{}'. Trend slope: {:.3}",
                        test_name, slope
                    ),
                    affected_systems: vec![test_name.clone()],
                    recommended_actions: vec![
                        "Investigate recent changes".to_string(),
                        "Run performance profiling".to_string(),
                        "Consider rollback if recent deployment".to_string(),
                    ],
                    metadata: HashMap::from([
                        ("test_name".to_string(), test_name),
                        ("trend_slope".to_string(), slope.to_string()),
                        ("data_points".to_string(), records.len().to_string()),
                    ]),
                });
            }
        }

        alerts
    }

    /// Check for anomaly alerts using statistical analysis
    async fn check_anomaly_alerts(&self, now: DateTime<Utc>) -> Vec<EnterpriseAlert> {
        let mut alerts = Vec::new();

        // Analyze recent performance for anomalies (last 24 hours)
        let one_day_ago = now - chrono::Duration::hours(24);
        let recent_metrics: Vec<_> = self
            .performance_history
            .iter()
            .filter(|record| record.timestamp > one_day_ago)
            .collect();

        if recent_metrics.len() < 50 {
            return alerts; // Not enough data for anomaly detection
        }

        // Group by test name
        let mut groups: HashMap<String, Vec<&PerformanceRecord>> = HashMap::new();
        for record in &recent_metrics {
            groups
                .entry(record.metric.test_name.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }

        for (test_name, records) in groups {
            if let Some(anomaly) = self.detect_performance_anomaly(&test_name, &records) {
                alerts.push(anomaly);
            }
        }

        alerts
    }

    /// Calculate trend slope using simple linear regression
    fn calculate_trend_slope(&self, records: &[&PerformanceRecord]) -> f64 {
        let n = records.len() as f64;
        if n < 2.0 {
            return 0.0;
        }

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for (i, record) in records.iter().enumerate() {
            let x = i as f64;
            let y = record.metric.p95_time_ns as f64 / 1_000_000_000.0; // Convert to seconds

            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        slope
    }

    /// Detect performance anomalies using statistical methods
    fn detect_performance_anomaly(
        &self,
        test_name: &str,
        records: &[&PerformanceRecord],
    ) -> Option<EnterpriseAlert> {
        if records.len() < 10 {
            return None;
        }

        // Calculate mean and standard deviation
        let values: Vec<f64> = records
            .iter()
            .map(|r| r.metric.p95_time_ns as f64 / 1_000_000_000.0)
            .collect();

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        // Check if the most recent value is an outlier (3 sigma)
        if let Some(latest) = values.last() {
            let z_score = (latest - mean) / std_dev;
            if z_score > 3.0 {
                return Some(EnterpriseAlert {
                    alert_type: AlertType::PerformanceAnomaly,
                    severity: AlertSeverity::High,
                    title: format!("Performance Anomaly Detected: {}", test_name),
                    description: format!(
                        "Recent performance measurement is {:.1} standard deviations above mean for '{}'. Value: {:.3}s, Mean: {:.3}s",
                        z_score, test_name, latest, mean
                    ),
                    affected_systems: vec![test_name.to_string()],
                    recommended_actions: vec![
                        "Investigate recent changes".to_string(),
                        "Check system resources".to_string(),
                        "Review error logs".to_string(),
                    ],
                    metadata: HashMap::from([
                        ("test_name".to_string(), test_name.to_string()),
                        ("z_score".to_string(), z_score.to_string()),
                        ("latest_value".to_string(), latest.to_string()),
                        ("mean_value".to_string(), mean.to_string()),
                    ]),
                });
            }
        }

        None
    }

    /// Send alerts to configured destinations
    async fn send_alerts(&self, alerts: &[EnterpriseAlert]) {
        for alert in alerts {
            for destination in &self.alert_config.destinations {
                match destination {
                    AlertDestination::Console => {
                        self.send_to_console(alert);
                    }
                    AlertDestination::Slack { webhook_url } => {
                        self.send_to_slack(alert, webhook_url).await;
                    }
                    AlertDestination::Email {
                        smtp_config,
                        recipients,
                    } => {
                        self.send_to_email(alert, smtp_config, recipients).await;
                    }
                    AlertDestination::Webhook { url, headers } => {
                        self.send_to_webhook(alert, url, headers).await;
                    }
                }
            }
        }
    }

    fn send_to_console(&self, alert: &EnterpriseAlert) {
        let severity_icon = match alert.severity {
            AlertSeverity::Critical => "🚨",
            AlertSeverity::High => "⚠️",
            AlertSeverity::Medium => "ℹ️",
            AlertSeverity::Low => "📝",
        };

        println!("{} [{}] {}", severity_icon, alert.severity, alert.title);
        println!("  {}", alert.description);
        for action in &alert.recommended_actions {
            println!("  • {}", action);
        }
        println!();
    }

    async fn send_to_slack(&self, alert: &EnterpriseAlert, webhook_url: &str) {
        // In a real implementation, this would send HTTP request to Slack webhook
        println!("Would send Slack alert to {}: {}", webhook_url, alert.title);
    }

    async fn send_to_email(
        &self,
        alert: &EnterpriseAlert,
        smtp_config: &SmtpConfig,
        recipients: &[String],
    ) {
        // In a real implementation, this would send email via SMTP
        println!(
            "Would send email alert to {:?}: {}",
            recipients, alert.title
        );
    }

    async fn send_to_webhook(
        &self,
        alert: &EnterpriseAlert,
        url: &str,
        headers: &HashMap<String, String>,
    ) {
        // In a real implementation, this would send HTTP request to webhook
        println!("Would send webhook alert to {}: {}", url, alert.title);
    }

    /// Get alert history
    pub fn alert_history(&self) -> &[AlertRecord] {
        &self.alert_history
    }

    /// Get performance history
    pub fn performance_history(&self) -> &[PerformanceRecord] {
        &self.performance_history
    }

    /// Generate enterprise monitoring report
    pub fn generate_report(&self) -> String {
        let mut report = format!("=== Enterprise Performance Report ===\n");
        report.push_str(&format!("Generated: {}\n\n", Utc::now().to_rfc3339()));

        // Alert summary
        let critical_alerts = self
            .alert_history
            .iter()
            .filter(|r| matches!(r.alert.severity, AlertSeverity::Critical))
            .count();
        let high_alerts = self
            .alert_history
            .iter()
            .filter(|r| matches!(r.alert.severity, AlertSeverity::High))
            .count();

        report.push_str("=== Alert Summary ===\n");
        report.push_str(&format!("Critical Alerts: {}\n", critical_alerts));
        report.push_str(&format!("High Alerts: {}\n", high_alerts));
        report.push_str(&format!("Total Alerts: {}\n\n", self.alert_history.len()));

        // Recent alerts
        report.push_str("=== Recent Alerts (Last 10) ===\n");
        for record in self.alert_history.iter().rev().take(10) {
            report.push_str(&format!(
                "[{}] {} - {}\n",
                record.timestamp.format("%Y-%m-%d %H:%M:%S"),
                record.alert.severity,
                record.alert.title
            ));
        }

        // Performance summary
        if let Some(latest) = self.performance_history.last() {
            report.push_str(&format!("\n=== Latest Performance Metrics ===\n"));
            report.push_str(&format!("Test: {}\n", latest.metric.test_name));
            report.push_str(&format!(
                "P95 Time: {:.2}ms\n",
                latest.metric.p95_time_ns as f64 / 1_000_000.0
            ));
            report.push_str(&format!(
                "Peak Memory: {:.1}MB\n",
                latest.metric.peak_memory_bytes as f64 / (1024.0 * 1024.0)
            ));
            report.push_str(&format!(
                "CPU Usage: {:.1}%\n",
                latest.metric.avg_cpu_percent
            ));
        }

        report
    }
}

/// Enterprise alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Alert destinations
    pub destinations: Vec<AlertDestination>,
    /// Minimum severity level to alert on
    pub minimum_severity: AlertSeverity,
    /// Alert cooldown period in seconds
    pub cooldown_seconds: u64,
}

/// Alert destination types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertDestination {
    /// Print to console
    Console,
    /// Send to Slack webhook
    Slack { webhook_url: String },
    /// Send via email
    Email {
        smtp_config: SmtpConfig,
        recipients: Vec<String>,
    },
    /// Send to HTTP webhook
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
}

/// SMTP configuration for email alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

/// Enterprise alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseAlert {
    /// Type of alert
    pub alert_type: AlertType,
    /// Severity level
    pub severity: AlertSeverity,
    /// Alert title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Affected systems
    pub affected_systems: Vec<String>,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    PerformanceThreshold,
    ResourceThreshold,
    PerformanceTrend,
    PerformanceAnomaly,
    ValidationFailure,
    SystemFailure,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Critical => write!(f, "Critical"),
            AlertSeverity::High => write!(f, "High"),
            AlertSeverity::Medium => write!(f, "Medium"),
            AlertSeverity::Low => write!(f, "Low"),
        }
    }
}

/// Alert record with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRecord {
    pub alert: EnterpriseAlert,
    pub timestamp: DateTime<Utc>,
}

/// Performance record with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecord {
    pub metric: PerformanceMetrics,
    pub timestamp: DateTime<Utc>,
}
