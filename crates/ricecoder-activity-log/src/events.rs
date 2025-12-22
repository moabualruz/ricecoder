//! Activity events and logging structures

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ActivityLogResult;

/// Log severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    /// Debug information for development
    Debug,
    /// General information
    Info,
    /// Warning about potential issues
    Warn,
    /// Error that doesn't stop operation
    Error,
    /// Critical error requiring immediate attention
    Critical,
}

/// Event categories for organizing logs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventCategory {
    /// User interface interactions
    UserInterface,
    /// File system operations
    FileSystem,
    /// Network operations
    Network,
    /// AI provider interactions
    AIProvider,
    /// Workflow executions
    Workflow,
    /// Session management
    Session,
    /// Security events
    Security,
    /// Performance metrics
    Performance,
    /// System operations
    System,
    /// Custom category
    Custom(String),
}

/// Activity event representing a logged action or event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    /// Unique event ID
    pub id: Uuid,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Log level/severity
    pub level: LogLevel,
    /// Event category
    pub category: EventCategory,
    /// Specific action performed
    pub action: String,
    /// Actor who performed the action (user ID, system, etc.)
    pub actor: String,
    /// Resource affected by the action
    pub resource: String,
    /// Additional details about the event
    pub details: serde_json::Value,
    /// Session ID if applicable
    pub session_id: Option<String>,
    /// Request ID for tracing related events
    pub request_id: Option<String>,
    /// Duration of the operation in milliseconds
    pub duration_ms: Option<u64>,
    /// IP address or source identifier
    pub source: Option<String>,
    /// User agent or client information
    pub user_agent: Option<String>,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for ActivityEvent {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: LogLevel::Info,
            category: EventCategory::System,
            action: String::new(),
            actor: String::new(),
            resource: String::new(),
            details: serde_json::json!({}),
            session_id: None,
            request_id: None,
            duration_ms: None,
            source: None,
            user_agent: None,
            metadata: HashMap::new(),
        }
    }
}

impl ActivityEvent {
    /// Create a new activity event
    pub fn new(
        level: LogLevel,
        category: EventCategory,
        action: String,
        actor: String,
        resource: String,
    ) -> Self {
        Self {
            level,
            category,
            action,
            actor,
            resource,
            ..Default::default()
        }
    }

    /// Add details to the event
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }

    /// Add session ID to the event
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Add request ID for tracing
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Add duration information
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Add source information
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Validate the event data
    pub fn validate(&self) -> ActivityLogResult<()> {
        if self.action.is_empty() {
            return Err(crate::error::ActivityLogError::ValidationError {
                message: "Action cannot be empty".to_string(),
            });
        }

        if self.actor.is_empty() {
            return Err(crate::error::ActivityLogError::ValidationError {
                message: "Actor cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// Check if this event should be logged based on level
    pub fn should_log(&self, min_level: LogLevel) -> bool {
        self.level >= min_level
    }

    /// Get a human-readable description of the event
    pub fn description(&self) -> String {
        format!(
            "[{}] {} performed '{}' on '{}' ({})",
            self.level_description(),
            self.actor,
            self.action,
            self.resource,
            self.category_description()
        )
    }

    /// Get level as string
    pub fn level_description(&self) -> &'static str {
        match self.level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        }
    }

    /// Get category as string
    pub fn category_description(&self) -> String {
        match &self.category {
            EventCategory::UserInterface => "UI".to_string(),
            EventCategory::FileSystem => "FS".to_string(),
            EventCategory::Network => "NET".to_string(),
            EventCategory::AIProvider => "AI".to_string(),
            EventCategory::Workflow => "WORKFLOW".to_string(),
            EventCategory::Session => "SESSION".to_string(),
            EventCategory::Security => "SECURITY".to_string(),
            EventCategory::Performance => "PERF".to_string(),
            EventCategory::System => "SYSTEM".to_string(),
            EventCategory::Custom(cat) => format!("CUSTOM:{}", cat),
        }
    }
}

/// Event filter for querying logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// Minimum log level
    pub min_level: Option<LogLevel>,
    /// Event categories to include
    pub categories: Option<Vec<EventCategory>>,
    /// Actor filter
    pub actor: Option<String>,
    /// Resource filter (supports wildcards)
    pub resource: Option<String>,
    /// Session ID filter
    pub session_id: Option<String>,
    /// Time range start
    pub start_time: Option<DateTime<Utc>>,
    /// Time range end
    pub end_time: Option<DateTime<Utc>>,
    /// Maximum number of results
    pub limit: Option<usize>,
}

impl Default for EventFilter {
    fn default() -> Self {
        Self {
            min_level: Some(LogLevel::Info),
            categories: None,
            actor: None,
            resource: None,
            session_id: None,
            start_time: None,
            end_time: None,
            limit: Some(100),
        }
    }
}

impl EventFilter {
    /// Create a new filter with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by minimum log level
    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = Some(level);
        self
    }

    /// Filter by categories
    pub fn with_categories(mut self, categories: Vec<EventCategory>) -> Self {
        self.categories = Some(categories);
        self
    }

    /// Filter by actor
    pub fn with_actor(mut self, actor: String) -> Self {
        self.actor = Some(actor);
        self
    }

    /// Filter by session
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Check if an event matches this filter
    pub fn matches(&self, event: &ActivityEvent) -> bool {
        // Check log level
        if let Some(min_level) = self.min_level {
            if event.level < min_level {
                return false;
            }
        }

        // Check categories
        if let Some(ref categories) = self.categories {
            if !categories.contains(&event.category) {
                return false;
            }
        }

        // Check actor
        if let Some(ref actor_filter) = self.actor {
            if !event.actor.contains(actor_filter) {
                return false;
            }
        }

        // Check session ID
        if let Some(ref session_filter) = self.session_id {
            if event.session_id.as_ref() != Some(session_filter) {
                return false;
            }
        }

        // Check resource (simple wildcard support)
        if let Some(ref resource_filter) = self.resource {
            if !Self::matches_pattern(&event.resource, resource_filter) {
                return false;
            }
        }

        // Check time range
        if let Some(start) = self.start_time {
            if event.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if event.timestamp > end {
                return false;
            }
        }

        true
    }

    /// Simple pattern matching with * wildcards
    fn matches_pattern(text: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // Simple wildcard matching
            let regex_pattern = pattern.replace('*', ".*");
            if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
                regex.is_match(text)
            } else {
                text.contains(pattern.trim_matches('*'))
            }
        } else {
            text.contains(pattern)
        }
    }
}
