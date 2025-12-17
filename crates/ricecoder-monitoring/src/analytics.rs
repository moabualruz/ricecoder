//! Usage analytics and business intelligence

use crate::types::*;
use chrono::{DateTime, Utc, TimeDelta, Timelike};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::mpsc;
use tokio::time;

pub use crate::types::AnalyticsConfig;

/// Global usage events storage
static USAGE_EVENTS: Lazy<DashMap<EventId, UsageEvent>> = Lazy::new(DashMap::new);

/// Global BI reports storage
static BI_REPORTS: Lazy<DashMap<EventId, BIReport>> = Lazy::new(DashMap::new);

/// Analytics engine
pub struct AnalyticsEngine {
    config: AnalyticsConfig,
    shutdown_tx: Option<mpsc::Sender<()>>,
    analytics_task: Option<tokio::task::JoinHandle<()>>,
    event_buffer: Arc<RwLock<Vec<UsageEvent>>>,
}

impl AnalyticsEngine {
    /// Create a new analytics engine
    pub fn new(config: AnalyticsConfig) -> Self {
        Self {
            config,
            shutdown_tx: None,
            analytics_task: None,
            event_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start the analytics engine
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Ok(());
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let flush_interval = self.config.flush_interval.to_std().unwrap();
        let event_buffer = Arc::clone(&self.event_buffer);

        let task = tokio::spawn(async move {
            let mut interval = time::interval(flush_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let events = {
                            let mut buffer = event_buffer.write();
                            let events = buffer.clone();
                            buffer.clear();
                            events
                        };

                        if !events.is_empty() {
                            if let Err(e) = Self::flush_events(&events).await {
                                tracing::error!("Failed to flush analytics events: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Analytics engine shutting down");
                        break;
                    }
                }
            }
        });

        self.analytics_task = Some(task);
        Ok(())
    }

    /// Stop the analytics engine
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        if let Some(task) = self.analytics_task.take() {
            let _ = task.await;
        }

        Ok(())
    }

    /// Track a usage event
    pub fn track_event(&self, event: UsageEvent) {
        if !self.config.enabled {
            return;
        }

        // Add to buffer
        self.event_buffer.write().push(event.clone());

        // Store globally
        USAGE_EVENTS.insert(event.id, event);
    }

    /// Track a user action
    pub fn track_action(&self, user_id: Option<String>, action: &str, properties: HashMap<String, serde_json::Value>) {
        let event = UsageEvent {
            id: EventId::new_v4(),
            event_type: action.to_string(),
            user_id,
            session_id: None, // Would be set from session context
            properties,
            timestamp: chrono::Utc::now(),
        };

        self.track_event(event);
    }

    /// Get usage events with filtering
    pub fn get_usage_events(
        &self,
        event_type: Option<&str>,
        user_id: Option<&str>,
        since: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Vec<UsageEvent> {
        let mut events: Vec<_> = USAGE_EVENTS
            .iter()
            .filter_map(|entry| {
                let event = entry.value();
                let matches_type = event_type.map_or(true, |t| event.event_type == t);
                let matches_user = user_id.map_or(true, |u| event.user_id.as_ref() == Some(&u.to_string()));
                let matches_time = since.map_or(true, |t| event.timestamp >= t);

                if matches_type && matches_user && matches_time {
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

    /// Get usage statistics
    pub fn get_usage_stats(&self, since: Option<DateTime<Utc>>) -> UsageStats {
        let events = self.get_usage_events(None, None, since, None);

        let total_events = events.len();
        let unique_users = events.iter()
            .filter_map(|e| e.user_id.as_ref())
            .collect::<std::collections::HashSet<_>>()
            .len();

        let events_by_type = events.iter().fold(HashMap::new(), |mut acc, event| {
            *acc.entry(event.event_type.clone()).or_insert(0) += 1;
            acc
        });

        let events_by_user = events.iter().fold(HashMap::new(), |mut acc, event| {
            if let Some(user_id) = &event.user_id {
                *acc.entry(user_id.clone()).or_insert(0) += 1;
            }
            acc
        });

        let hourly_distribution = Self::calculate_hourly_distribution(&events);

        UsageStats {
            total_events,
            unique_users,
            events_by_type,
            events_by_user,
            hourly_distribution,
            period_start: since.unwrap_or_else(|| chrono::Utc::now() - chrono::TimeDelta::days(7)),
            period_end: chrono::Utc::now(),
        }
    }

    /// Generate a business intelligence report
    pub fn generate_bi_report(&self, title: String, description: String, query: String) -> Result<BIReport, Box<dyn std::error::Error + Send + Sync>> {
        // This is a simplified implementation - in practice, you'd have a proper query engine
        let data = self.execute_bi_query(&query)?;

        let report = BIReport {
            id: EventId::new_v4(),
            title,
            description,
            query,
            data,
            generated_at: chrono::Utc::now(),
            parameters: HashMap::new(),
        };

        BI_REPORTS.insert(report.id, report.clone());
        Ok(report)
    }

    /// Get BI reports
    pub fn get_bi_reports(&self, limit: Option<usize>) -> Vec<BIReport> {
        let mut reports: Vec<_> = BI_REPORTS.iter().map(|entry| entry.value().clone()).collect();

        reports.sort_by(|a, b| b.generated_at.cmp(&a.generated_at));

        if let Some(limit) = limit {
            reports.truncate(limit);
        }

        reports
    }

    /// Flush events to external analytics service
    async fn flush_events(events: &[UsageEvent]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would send to analytics services like Mixpanel, Amplitude, etc.
        tracing::info!("Flushed {} analytics events", events.len());
        Ok(())
    }

    /// Calculate hourly distribution of events
    fn calculate_hourly_distribution(events: &[UsageEvent]) -> HashMap<u32, usize> {
        events.iter().fold(HashMap::new(), |mut acc, event| {
            let hour = event.timestamp.hour();
            *acc.entry(hour).or_insert(0) += 1;
            acc
        })
    }

    /// Execute a BI query (simplified implementation)
    fn execute_bi_query(&self, query: &str) -> Result<Vec<HashMap<String, serde_json::Value>>, Box<dyn std::error::Error + Send + Sync>> {
        // This is a very simplified query execution - in practice, you'd have a proper SQL-like query engine
        match query.to_lowercase().as_str() {
            "select event_type, count(*) from events group by event_type" => {
                let stats = self.get_usage_stats(Some(chrono::Utc::now() - chrono::TimeDelta::days(30)));
                Ok(stats.events_by_type.iter().map(|(event_type, count)| {
                    let mut row = HashMap::new();
                    row.insert("event_type".to_string(), serde_json::Value::String(event_type.clone()));
                    row.insert("count".to_string(), serde_json::Value::Number((*count as u64).into()));
                    row
                }).collect())
            }
            "select user_id, count(*) from events where user_id is not null group by user_id order by count desc limit 10" => {
                let stats = self.get_usage_stats(Some(chrono::Utc::now() - chrono::TimeDelta::days(30)));
                Ok(stats.events_by_user.iter()
                    .take(10)
                    .map(|(user_id, count)| {
                        let mut row = HashMap::new();
                        row.insert("user_id".to_string(), serde_json::Value::String(user_id.clone()));
                        row.insert("count".to_string(), serde_json::Value::Number((*count as u64).into()));
                        row
                    }).collect())
            }
            _ => {
                // Return sample data for unknown queries
                Ok(vec![
                    {
                        let mut row = HashMap::new();
                        row.insert("sample_column".to_string(), serde_json::Value::String("sample_value".to_string()));
                        row
                    }
                ])
            }
        }
    }
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_events: usize,
    pub unique_users: usize,
    pub events_by_type: HashMap<String, usize>,
    pub events_by_user: HashMap<String, usize>,
    pub hourly_distribution: HashMap<u32, usize>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// User behavior analyzer
pub struct UserBehaviorAnalyzer {
    session_timeout: TimeDelta,
}

impl UserBehaviorAnalyzer {
    pub fn new(session_timeout: TimeDelta) -> Self {
        Self { session_timeout }
    }

    /// Analyze user behavior patterns
    pub fn analyze_behavior(&self, user_id: &str) -> UserBehaviorProfile {
        let user_events: Vec<_> = USAGE_EVENTS
            .iter()
            .filter_map(|entry| {
                let event = entry.value();
                if event.user_id.as_ref() == Some(&user_id.to_string()) {
                    Some(event.clone())
                } else {
                    None
                }
            })
            .collect();

        if user_events.is_empty() {
            return UserBehaviorProfile {
                user_id: user_id.to_string(),
                total_sessions: 0,
                avg_session_duration: chrono::TimeDelta::zero(),
                most_used_features: Vec::new(),
                activity_patterns: HashMap::new(),
                engagement_score: 0.0,
                last_active: None,
            };
        }

        let sessions = self.identify_sessions(&user_events);
        let total_sessions = sessions.len();

        let avg_session_duration = if !sessions.is_empty() {
            sessions.iter()
                .map(|s| s.duration)
                .sum::<chrono::TimeDelta>() / sessions.len() as i32
        } else {
            chrono::TimeDelta::zero()
        };

        let most_used_features = self.calculate_most_used_features(&user_events);
        let activity_patterns = self.calculate_activity_patterns(&user_events);
        let engagement_score = self.calculate_engagement_score(&user_events, &sessions);
        let last_active = user_events.iter()
            .map(|e| e.timestamp)
            .max();

        UserBehaviorProfile {
            user_id: user_id.to_string(),
            total_sessions,
            avg_session_duration,
            most_used_features,
            activity_patterns,
            engagement_score,
            last_active,
        }
    }

    /// Identify user sessions from events
    fn identify_sessions(&self, events: &[UsageEvent]) -> Vec<UserSession> {
        let mut sessions = Vec::new();
        let mut current_session: Option<UserSession> = None;

        let mut sorted_events = events.to_vec();
        sorted_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        for event in sorted_events {
            match &mut current_session {
                Some(session) => {
                    if event.timestamp.signed_duration_since(session.end_time) > self.session_timeout {
                        // Session timeout, start new session
                        sessions.push(session.clone());
                        current_session = Some(UserSession {
                            start_time: event.timestamp,
                            end_time: event.timestamp,
                            duration: chrono::TimeDelta::zero(),
                            event_count: 1,
                        });
                    } else {
                        // Continue current session
                        session.end_time = event.timestamp;
                        session.event_count += 1;
                        session.duration = session.end_time - session.start_time;
                    }
                }
                None => {
                    // Start new session
                    current_session = Some(UserSession {
                        start_time: event.timestamp,
                        end_time: event.timestamp,
                        duration: chrono::TimeDelta::zero(),
                        event_count: 1,
                    });
                }
            }
        }

        if let Some(session) = current_session {
            sessions.push(session);
        }

        sessions
    }

    /// Calculate most used features
    fn calculate_most_used_features(&self, events: &[UsageEvent]) -> Vec<(String, usize)> {
        let mut feature_counts = HashMap::new();

        for event in events {
            *feature_counts.entry(event.event_type.clone()).or_insert(0) += 1;
        }

        let mut features: Vec<_> = feature_counts.into_iter().collect();
        features.sort_by(|a, b| b.1.cmp(&a.1));
        features.truncate(10); // Top 10 features
        features
    }

    /// Calculate activity patterns by hour
    fn calculate_activity_patterns(&self, events: &[UsageEvent]) -> HashMap<u32, usize> {
        events.iter().fold(HashMap::new(), |mut acc, event| {
            let hour = event.timestamp.hour();
            *acc.entry(hour).or_insert(0) += 1;
            acc
        })
    }

    /// Calculate user engagement score
    fn calculate_engagement_score(&self, events: &[UsageEvent], sessions: &[UserSession]) -> f64 {
        if events.is_empty() {
            return 0.0;
        }

        let total_events = events.len() as f64;
        let total_sessions = sessions.len() as f64;
        let avg_events_per_session = if total_sessions > 0.0 {
            total_events / total_sessions
        } else {
            0.0
        };

        let days_active = events.iter()
            .map(|e| e.timestamp.date())
            .collect::<std::collections::HashSet<_>>()
            .len() as f64;

        let weeks_since_first_event = {
            let first_event = events.iter()
                .map(|e| e.timestamp)
                .min()
                .unwrap_or(chrono::Utc::now());
            let weeks = chrono::Utc::now().signed_duration_since(first_event).num_weeks() as f64;
            weeks.max(1.0)
        };

        let activity_frequency = days_active / weeks_since_first_event;

        // Simple engagement score calculation
        (avg_events_per_session * 0.4 + activity_frequency * 0.6).min(10.0)
    }
}

/// User session
#[derive(Debug, Clone)]
struct UserSession {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    duration: chrono::TimeDelta,
    event_count: usize,
}

/// User behavior profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBehaviorProfile {
    pub user_id: String,
    pub total_sessions: usize,
    pub avg_session_duration: chrono::TimeDelta,
    pub most_used_features: Vec<(String, usize)>,
    pub activity_patterns: HashMap<u32, usize>,
    pub engagement_score: f64,
    pub last_active: Option<DateTime<Utc>>,
}

/// Feature adoption analyzer
pub struct FeatureAdoptionAnalyzer;

impl FeatureAdoptionAnalyzer {
    /// Analyze feature adoption rates
    pub fn analyze_adoption(&self, feature_name: &str, since: Option<DateTime<Utc>>) -> FeatureAdoptionMetrics {
        let events = USAGE_EVENTS
            .iter()
            .filter_map(|entry| {
                let event = entry.value();
                if event.event_type == feature_name {
                    if let Some(since) = since {
                        if event.timestamp >= since {
                            Some(event.clone())
                        } else {
                            None
                        }
                    } else {
                        Some(event.clone())
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let total_users = events.iter()
            .filter_map(|e| e.user_id.as_ref())
            .collect::<std::collections::HashSet<_>>()
            .len();

        let total_usage = events.len();

        let daily_usage = events.iter().fold(HashMap::new(), |mut acc, event| {
            let date = event.timestamp.date();
            *acc.entry(date).or_insert(0) += 1;
            acc
        });

        let avg_daily_usage = if !daily_usage.is_empty() {
            daily_usage.values().sum::<usize>() as f64 / daily_usage.len() as f64
        } else {
            0.0
        };

        FeatureAdoptionMetrics {
            feature_name: feature_name.to_string(),
            total_users,
            total_usage,
            avg_daily_usage,
            adoption_trend: Self::calculate_adoption_trend(&daily_usage),
            period_start: since.unwrap_or_else(|| chrono::Utc::now() - chrono::TimeDelta::days(30)),
            period_end: chrono::Utc::now(),
        }
    }

    /// Calculate adoption trend
    fn calculate_adoption_trend(daily_usage: &HashMap<chrono::Date<chrono::Utc>, usize>) -> f64 {
        if daily_usage.len() < 2 {
            return 0.0;
        }

        let mut dates: Vec<_> = daily_usage.keys().collect();
        dates.sort();

        let first_half: Vec<_> = dates.iter().take(dates.len() / 2).collect();
        let second_half: Vec<_> = dates.iter().skip(dates.len() / 2).collect();

        let first_avg = first_half.iter()
            .map(|date| daily_usage.get(date).unwrap_or(&0))
            .sum::<usize>() as f64 / first_half.len() as f64;

        let second_avg = second_half.iter()
            .map(|date| daily_usage.get(date).unwrap_or(&0))
            .sum::<usize>() as f64 / second_half.len() as f64;

        if first_avg > 0.0 {
            ((second_avg - first_avg) / first_avg) * 100.0
        } else {
            0.0
        }
    }
}

/// Feature adoption metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureAdoptionMetrics {
    pub feature_name: String,
    pub total_users: usize,
    pub total_usage: usize,
    pub avg_daily_usage: f64,
    pub adoption_trend: f64, // Percentage change
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}