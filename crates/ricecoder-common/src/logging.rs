//! Unified logging system for RiceCoder
//!
//! Matches OpenCode logging behavior with:
//! - Unified initialization (level + destination)
//! - Minimum log level configuration
//! - File output for general logs
//! - Log rotation and cleanup
//! - ISO timestamp + key=value format
//! - Error cause-chain formatting
//! - Lightweight performance timing helper
//! - Optional per-service logger caching

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tracing::{debug, error, info, warn};

/// Log levels matching OpenCode
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl LogLevel {
    /// Parse log level from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARN" => Some(LogLevel::Warn),
            "ERROR" => Some(LogLevel::Error),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// Logging configuration options
pub struct LogOptions {
    /// Whether to print to stderr
    pub print: bool,
    /// Development mode (use dev.log instead of timestamped log)
    pub dev: Option<bool>,
    /// Minimum log level
    pub level: Option<LogLevel>,
}

/// Structured field tags for logging
pub type Tags = HashMap<String, serde_json::Value>;

/// Performance timing helper
pub struct Timer {
    message: String,
    start: Instant,
    extra: Tags,
    logger: Arc<LoggerInner>,
}

impl Timer {
    /// Stop the timer and log duration
    pub fn stop(self) {
        // Timer automatically stops and logs on drop
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let mut fields = self.extra.clone();
        fields.insert(
            "status".to_string(),
            serde_json::Value::String("completed".to_string()),
        );
        fields.insert(
            "duration".to_string(),
            serde_json::Value::Number(serde_json::Number::from(duration.as_millis() as u64)),
        );
        self.logger.info(&self.message, Some(fields));
    }
}

/// Logger instance with optional tags
pub struct Logger {
    inner: Arc<LoggerInner>,
}

struct LoggerInner {
    tags: Mutex<Tags>,
    state: Arc<LoggerState>,
}

struct LoggerState {
    min_level: Mutex<LogLevel>,
    file_writer: Mutex<Option<File>>,
    log_path: Mutex<Option<PathBuf>>,
    last_log_time: Mutex<Instant>,
}

impl Logger {
    /// Log a debug message
    pub fn debug(&self, message: &str, extra: Option<Tags>) {
        self.inner.debug(message, extra);
    }

    /// Log an info message
    pub fn info(&self, message: &str, extra: Option<Tags>) {
        self.inner.info(message, extra);
    }

    /// Log a warning message
    pub fn warn(&self, message: &str, extra: Option<Tags>) {
        self.inner.warn(message, extra);
    }

    /// Log an error message
    pub fn error(&self, message: &str, extra: Option<Tags>) {
        self.inner.error(message, extra);
    }

    /// Add a tag to this logger (mutates tags)
    pub fn tag(&self, key: String, value: serde_json::Value) {
        let mut tags = self.inner.tags.lock().unwrap();
        tags.insert(key, value);
    }

    /// Clone this logger with copied tags
    pub fn clone_logger(&self) -> Self {
        let tags = self.inner.tags.lock().unwrap().clone();
        Logger {
            inner: Arc::new(LoggerInner {
                tags: Mutex::new(tags),
                state: Arc::clone(&self.inner.state),
            }),
        }
    }

    /// Start a performance timer
    pub fn time(&self, message: String, extra: Option<Tags>) -> Timer {
        let extra = extra.unwrap_or_default();
        let mut start_fields = extra.clone();
        start_fields.insert(
            "status".to_string(),
            serde_json::Value::String("started".to_string()),
        );
        self.inner.info(&message, Some(start_fields));

        Timer {
            message,
            start: Instant::now(),
            extra,
            logger: Arc::clone(&self.inner),
        }
    }
}

impl LoggerInner {
    fn should_log(&self, level: LogLevel) -> bool {
        let min_level = *self.state.min_level.lock().unwrap();
        level >= min_level
    }

    fn build_message(&self, message: &str, extra: Option<Tags>) -> String {
        let tags = self.tags.lock().unwrap();
        let mut all_fields = tags.clone();
        if let Some(extra_fields) = extra {
            all_fields.extend(extra_fields);
        }

        // Build key=value fields
        let fields: Vec<String> = all_fields
            .iter()
            .filter_map(|(key, value)| {
                let formatted_value = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => return None,
                    other => serde_json::to_string(other).unwrap_or_default(),
                };
                Some(format!("{}={}", key, formatted_value))
            })
            .collect();

        // Calculate time diff
        let mut last_time = self.state.last_log_time.lock().unwrap();
        let now = Instant::now();
        let diff = now.duration_since(*last_time);
        *last_time = now;

        // ISO timestamp (seconds precision)
        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();

        // Combine all parts
        let parts: Vec<String> = vec![
            timestamp,
            format!("+{}ms", diff.as_millis()),
            fields.join(" "),
            message.to_string(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();

        parts.join(" ")
    }

    fn write_log(&self, level: LogLevel, message: &str) {
        let formatted = format!("{} {}\n", level.as_str(), message);

        // Write to file if configured
        if let Some(mut writer) = self.state.file_writer.lock().unwrap().as_mut() {
            let _ = writer.write_all(formatted.as_bytes());
            let _ = writer.flush();
        } else {
            // Print to stderr
            eprint!("{}", formatted);
        }
    }

    fn debug(&self, message: &str, extra: Option<Tags>) {
        if self.should_log(LogLevel::Debug) {
            let msg = self.build_message(message, extra);
            self.write_log(LogLevel::Debug, &msg);
        }
    }

    fn info(&self, message: &str, extra: Option<Tags>) {
        if self.should_log(LogLevel::Info) {
            let msg = self.build_message(message, extra);
            self.write_log(LogLevel::Info, &msg);
        }
    }

    fn warn(&self, message: &str, extra: Option<Tags>) {
        if self.should_log(LogLevel::Warn) {
            let msg = self.build_message(message, extra);
            self.write_log(LogLevel::Warn, &msg);
        }
    }

    fn error(&self, message: &str, extra: Option<Tags>) {
        if self.should_log(LogLevel::Error) {
            let msg = self.build_message(message, extra);
            self.write_log(LogLevel::Error, &msg);
        }
    }
}

/// Global logger state
static LOGGER_CACHE: Mutex<Option<HashMap<String, Logger>>> = Mutex::new(None);
static GLOBAL_STATE: Mutex<Option<Arc<LoggerState>>> = Mutex::new(None);

/// Initialize logging system
pub fn init(options: LogOptions) -> std::io::Result<()> {
    let min_level = options.level.unwrap_or(LogLevel::Info);

    // Determine log path
    let log_path = if options.print {
        None
    } else {
        let log_dir = std::env::current_dir()?.join(".ricecoder").join("logs");
        fs::create_dir_all(&log_dir)?;

        let filename = if options.dev.unwrap_or(false) {
            "dev.log".to_string()
        } else {
            let timestamp = chrono::Utc::now()
                .format("%Y-%m-%dT%H%M%S")
                .to_string();
            format!("{}.log", timestamp)
        };

        let path = log_dir.join(filename);

        // Create/truncate file
        let _ = File::create(&path)?;

        // Cleanup old logs
        cleanup(&log_dir)?;

        Some(path)
    };

    // Open file writer if needed
    let file_writer = if let Some(ref path) = log_path {
        Some(
            File::options()
                .create(true)
                .append(true)
                .open(path)?,
        )
    } else {
        None
    };

    // Create global state
    let state = Arc::new(LoggerState {
        min_level: Mutex::new(min_level),
        file_writer: Mutex::new(file_writer),
        log_path: Mutex::new(log_path),
        last_log_time: Mutex::new(Instant::now()),
    });

    *GLOBAL_STATE.lock().unwrap() = Some(state);
    *LOGGER_CACHE.lock().unwrap() = Some(HashMap::new());

    Ok(())
}

/// Get the current log file path
pub fn file() -> Option<PathBuf> {
    GLOBAL_STATE
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|state| state.log_path.lock().unwrap().clone())
}

/// Cleanup old log files (keeps newest 10)
fn cleanup(log_dir: &Path) -> std::io::Result<()> {
    let mut log_files: Vec<_> = fs::read_dir(log_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "log" {
                let metadata = fs::metadata(&path).ok()?;
                let modified = metadata.modified().ok()?;
                Some((path, modified))
            } else {
                None
            }
        })
        .collect();

    // Sort by modification time (newest first)
    log_files.sort_by(|a, b| b.1.cmp(&a.1));

    // Delete files beyond the 10 newest
    for (path, _) in log_files.iter().skip(10) {
        let _ = fs::remove_file(path);
    }

    Ok(())
}

/// Create a logger with optional tags
pub fn create(tags: Option<Tags>) -> Logger {
    let tags = tags.unwrap_or_default();

    // Check for service tag for caching
    if let Some(serde_json::Value::String(service)) = tags.get("service") {
        let mut cache = LOGGER_CACHE.lock().unwrap();
        if let Some(cache_map) = cache.as_mut() {
            if let Some(logger) = cache_map.get(service) {
                return logger.clone_logger();
            }
        }
    }

    // Create new logger
    let state = GLOBAL_STATE
        .lock()
        .unwrap()
        .as_ref()
        .expect("Logging not initialized")
        .clone();

    let logger = Logger {
        inner: Arc::new(LoggerInner {
            tags: Mutex::new(tags.clone()),
            state,
        }),
    };

    // Cache by service if present
    if let Some(serde_json::Value::String(service)) = tags.get("service") {
        let mut cache = LOGGER_CACHE.lock().unwrap();
        if let Some(cache_map) = cache.as_mut() {
            cache_map.insert(service.clone(), logger.clone_logger());
        }
    }

    logger
}

/// Format an error with cause chain
pub fn format_error(error: &dyn std::error::Error) -> String {
    format_error_recursive(error, 0)
}

fn format_error_recursive(error: &dyn std::error::Error, depth: usize) -> String {
    const MAX_DEPTH: usize = 10;

    if depth >= MAX_DEPTH {
        return error.to_string();
    }

    let base = error.to_string();

    if let Some(source) = error.source() {
        format!("{} Caused by: {}", base, format_error_recursive(source, depth + 1))
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from_str("DEBUG"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("info"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("WARN"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_logger_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        init(LogOptions {
            print: true,
            dev: Some(true),
            level: Some(LogLevel::Debug),
        })
        .unwrap();

        let logger = create(None);
        logger.info("test message", None);
    }

    #[test]
    fn test_logger_with_tags() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        init(LogOptions {
            print: true,
            dev: Some(true),
            level: Some(LogLevel::Debug),
        })
        .unwrap();

        let mut tags = Tags::new();
        tags.insert(
            "service".to_string(),
            serde_json::Value::String("test".to_string()),
        );

        let logger = create(Some(tags));
        logger.info("test with tags", None);
    }

    #[test]
    fn test_error_formatting() {
        use std::io;

        let inner_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let outer_error = io::Error::new(io::ErrorKind::Other, format!("operation failed: {}", inner_error));

        let formatted = format_error(&outer_error);
        assert!(formatted.contains("operation failed"));
    }
}
