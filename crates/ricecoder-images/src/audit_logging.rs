//! Audit logging for image analysis requests.
//!
//! This module handles audit logging of image analysis requests, responses,
//! and cache operations. It integrates with ricecoder-providers' AuditLogger
//! for consistent security event tracking.

use ricecoder_providers::audit_log::{AuditEventType, AuditLogEntry, AuditLogger};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Image analysis audit logger.
///
/// This logger tracks:
/// - Image analysis requests (provider, model, image count, size)
/// - Analysis responses (success/failure, tokens used)
/// - Cache operations (hits, misses, evictions)
/// - Security events (path traversal attempts, invalid formats)
pub struct ImageAuditLogger {
    audit_logger: Arc<AuditLogger>,
}

impl ImageAuditLogger {
    /// Create a new image audit logger.
    ///
    /// # Arguments
    ///
    /// * `log_path` - Path to the audit log file
    pub fn new(log_path: PathBuf) -> Self {
        Self {
            audit_logger: Arc::new(AuditLogger::new(log_path)),
        }
    }

    /// Log an image analysis request.
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider name (openai, anthropic, etc.)
    /// * `model` - Model identifier
    /// * `image_count` - Number of images being analyzed
    /// * `total_size` - Total size of all images in bytes
    /// * `image_hashes` - SHA256 hashes of images (for deduplication tracking)
    pub fn log_analysis_request(
        &self,
        provider: &str,
        model: &str,
        image_count: usize,
        total_size: u64,
        image_hashes: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let details = format!(
            "Image analysis request: {} images, {} bytes total, hashes: {}",
            image_count,
            total_size,
            image_hashes.join(",")
        );

        let entry = AuditLogEntry::new(
            AuditEventType::FileAccessed,
            "ricecoder-images",
            "system",
            &format!("{}/{}", provider, model),
            "request_sent",
            &details,
        );

        self.audit_logger.log(&entry)?;

        info!(
            provider = provider,
            model = model,
            image_count = image_count,
            total_size = total_size,
            "Image analysis request logged"
        );

        Ok(())
    }

    /// Log a successful image analysis response.
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider name
    /// * `model` - Model identifier
    /// * `image_count` - Number of images analyzed
    /// * `tokens_used` - Tokens used for the analysis
    /// * `image_hashes` - SHA256 hashes of analyzed images
    pub fn log_analysis_success(
        &self,
        provider: &str,
        model: &str,
        image_count: usize,
        tokens_used: u32,
        image_hashes: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let details = format!(
            "Image analysis successful: {} images, {} tokens used, hashes: {}",
            image_count,
            tokens_used,
            image_hashes.join(",")
        );

        let entry = AuditLogEntry::new(
            AuditEventType::FileAccessed,
            "ricecoder-images",
            "system",
            &format!("{}/{}", provider, model),
            "analysis_success",
            &details,
        );

        self.audit_logger.log(&entry)?;

        info!(
            provider = provider,
            model = model,
            image_count = image_count,
            tokens_used = tokens_used,
            "Image analysis success logged"
        );

        Ok(())
    }

    /// Log a failed image analysis response.
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider name
    /// * `model` - Model identifier
    /// * `image_count` - Number of images that failed
    /// * `error` - Error message
    /// * `image_hashes` - SHA256 hashes of images that failed
    pub fn log_analysis_failure(
        &self,
        provider: &str,
        model: &str,
        image_count: usize,
        error: &str,
        image_hashes: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let details = format!(
            "Image analysis failed: {} images, error: {}, hashes: {}",
            image_count,
            error,
            image_hashes.join(",")
        );

        let entry = AuditLogEntry::new(
            AuditEventType::SecurityError,
            "ricecoder-images",
            "system",
            &format!("{}/{}", provider, model),
            "analysis_failure",
            &details,
        );

        self.audit_logger.log(&entry)?;

        warn!(
            provider = provider,
            model = model,
            image_count = image_count,
            error = error,
            "Image analysis failure logged"
        );

        Ok(())
    }

    /// Log a cache hit.
    ///
    /// # Arguments
    ///
    /// * `image_hash` - SHA256 hash of the image
    /// * `provider` - Provider that originally analyzed the image
    /// * `age_seconds` - Age of the cached result in seconds
    pub fn log_cache_hit(
        &self,
        image_hash: &str,
        provider: &str,
        age_seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let details = format!(
            "Cache hit for image {}: provider={}, age={}s",
            image_hash, provider, age_seconds
        );

        let entry = AuditLogEntry::new(
            AuditEventType::FileAccessed,
            "ricecoder-images",
            "system",
            &format!("cache/{}", image_hash),
            "cache_hit",
            &details,
        );

        self.audit_logger.log(&entry)?;

        debug!(
            image_hash = image_hash,
            provider = provider,
            age_seconds = age_seconds,
            "Cache hit logged"
        );

        Ok(())
    }

    /// Log a cache miss.
    ///
    /// # Arguments
    ///
    /// * `image_hash` - SHA256 hash of the image
    pub fn log_cache_miss(&self, image_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let details = format!("Cache miss for image {}", image_hash);

        let entry = AuditLogEntry::new(
            AuditEventType::FileAccessed,
            "ricecoder-images",
            "system",
            &format!("cache/{}", image_hash),
            "cache_miss",
            &details,
        );

        self.audit_logger.log(&entry)?;

        debug!(image_hash = image_hash, "Cache miss logged");

        Ok(())
    }

    /// Log a cache eviction.
    ///
    /// # Arguments
    ///
    /// * `image_hash` - SHA256 hash of the evicted image
    /// * `reason` - Reason for eviction (e.g., "LRU", "TTL_expired")
    pub fn log_cache_eviction(
        &self,
        image_hash: &str,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let details = format!("Cache eviction for image {}: reason={}", image_hash, reason);

        let entry = AuditLogEntry::new(
            AuditEventType::FileModified,
            "ricecoder-images",
            "system",
            &format!("cache/{}", image_hash),
            "cache_eviction",
            &details,
        );

        self.audit_logger.log(&entry)?;

        info!(
            image_hash = image_hash,
            reason = reason,
            "Cache eviction logged"
        );

        Ok(())
    }

    /// Log an invalid image format attempt.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file
    /// * `format` - Format that was attempted
    pub fn log_invalid_format(
        &self,
        file_path: &str,
        format: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let details = format!(
            "Invalid image format attempt: file={}, format={}",
            file_path, format
        );

        let entry = AuditLogEntry::new(
            AuditEventType::SecurityError,
            "ricecoder-images",
            "system",
            file_path,
            "invalid_format",
            &details,
        );

        self.audit_logger.log(&entry)?;

        warn!(
            file_path = file_path,
            format = format,
            "Invalid format attempt logged"
        );

        Ok(())
    }

    /// Log a file size violation.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file
    /// * `size_bytes` - Size of the file in bytes
    /// * `max_size_bytes` - Maximum allowed size in bytes
    pub fn log_file_size_violation(
        &self,
        file_path: &str,
        size_bytes: u64,
        max_size_bytes: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let details = format!(
            "File size violation: file={}, size={} bytes, max={} bytes",
            file_path, size_bytes, max_size_bytes
        );

        let entry = AuditLogEntry::new(
            AuditEventType::SecurityError,
            "ricecoder-images",
            "system",
            file_path,
            "size_violation",
            &details,
        );

        self.audit_logger.log(&entry)?;

        warn!(
            file_path = file_path,
            size_bytes = size_bytes,
            max_size_bytes = max_size_bytes,
            "File size violation logged"
        );

        Ok(())
    }

    /// Log a path traversal attempt.
    ///
    /// # Arguments
    ///
    /// * `attempted_path` - The path that was attempted
    pub fn log_path_traversal_attempt(
        &self,
        attempted_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let details = format!("Path traversal attempt detected: {}", attempted_path);

        let entry = AuditLogEntry::new(
            AuditEventType::SecurityError,
            "ricecoder-images",
            "system",
            attempted_path,
            "path_traversal_attempt",
            &details,
        );

        self.audit_logger.log(&entry)?;

        warn!(
            attempted_path = attempted_path,
            "Path traversal attempt logged"
        );

        Ok(())
    }

    /// Log an image analysis timeout.
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider name
    /// * `model` - Model identifier
    /// * `timeout_seconds` - Timeout duration in seconds
    /// * `image_hashes` - SHA256 hashes of images that timed out
    pub fn log_analysis_timeout(
        &self,
        provider: &str,
        model: &str,
        timeout_seconds: u64,
        image_hashes: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let details = format!(
            "Image analysis timeout: provider={}, model={}, timeout={}s, hashes: {}",
            provider,
            model,
            timeout_seconds,
            image_hashes.join(",")
        );

        let entry = AuditLogEntry::new(
            AuditEventType::SecurityError,
            "ricecoder-images",
            "system",
            &format!("{}/{}", provider, model),
            "analysis_timeout",
            &details,
        );

        self.audit_logger.log(&entry)?;

        warn!(
            provider = provider,
            model = model,
            timeout_seconds = timeout_seconds,
            "Analysis timeout logged"
        );

        Ok(())
    }

    /// Get the underlying audit logger.
    pub fn audit_logger(&self) -> &AuditLogger {
        &self.audit_logger
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_logger() -> (ImageAuditLogger, PathBuf, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = ImageAuditLogger::new(log_path.clone());
        (logger, log_path, temp_dir)
    }

    #[test]
    fn test_log_analysis_request() {
        let (logger, log_path, _temp_dir) = create_test_logger();

        let result = logger.log_analysis_request(
            "openai",
            "gpt-4-vision",
            1,
            1024,
            vec!["hash1".to_string()],
        );

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Image analysis request"));
        assert!(content.contains("openai"));
    }

    #[test]
    fn test_log_analysis_success() {
        let (logger, log_path, _temp_dir) = create_test_logger();

        let result = logger.log_analysis_success(
            "openai",
            "gpt-4-vision",
            1,
            100,
            vec!["hash1".to_string()],
        );

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Image analysis successful"));
        assert!(content.contains("100 tokens"));
    }

    #[test]
    fn test_log_analysis_failure() {
        let (logger, log_path, _temp_dir) = create_test_logger();

        let result = logger.log_analysis_failure(
            "openai",
            "gpt-4-vision",
            1,
            "Provider error",
            vec!["hash1".to_string()],
        );

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Image analysis failed"));
        assert!(content.contains("Provider error"));
    }

    #[test]
    fn test_log_cache_hit() {
        let (logger, log_path, _temp_dir) = create_test_logger();

        let result = logger.log_cache_hit("hash1", "openai", 3600);

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Cache hit"));
        assert!(content.contains("hash1"));
    }

    #[test]
    fn test_log_cache_miss() {
        let (logger, log_path, _temp_dir) = create_test_logger();

        let result = logger.log_cache_miss("hash1");

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Cache miss"));
        assert!(content.contains("hash1"));
    }

    #[test]
    fn test_log_cache_eviction() {
        let (logger, log_path, _temp_dir) = create_test_logger();

        let result = logger.log_cache_eviction("hash1", "LRU");

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Cache eviction"));
        assert!(content.contains("LRU"));
    }

    #[test]
    fn test_log_invalid_format() {
        let (logger, log_path, _temp_dir) = create_test_logger();

        let result = logger.log_invalid_format("/path/to/file.bmp", "bmp");

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Invalid image format"));
        assert!(content.contains("bmp"));
    }

    #[test]
    fn test_log_file_size_violation() {
        let (logger, log_path, _temp_dir) = create_test_logger();

        let result =
            logger.log_file_size_violation("/path/to/file.png", 20 * 1024 * 1024, 10 * 1024 * 1024);

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("File size violation"));
    }

    #[test]
    fn test_log_path_traversal_attempt() {
        let (logger, log_path, _temp_dir) = create_test_logger();

        let result = logger.log_path_traversal_attempt("../../etc/passwd");

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Path traversal attempt"));
        assert!(content.contains("../../etc/passwd"));
    }

    #[test]
    fn test_log_analysis_timeout() {
        let (logger, log_path, _temp_dir) = create_test_logger();

        let result =
            logger.log_analysis_timeout("openai", "gpt-4-vision", 10, vec!["hash1".to_string()]);

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Image analysis timeout"));
        assert!(content.contains("10s"));
    }

    #[test]
    fn test_multiple_audit_entries() {
        let (logger, log_path, _temp_dir) = create_test_logger();

        logger
            .log_analysis_request("openai", "gpt-4-vision", 1, 1024, vec!["hash1".to_string()])
            .unwrap();
        logger.log_cache_hit("hash1", "openai", 3600).unwrap();
        logger
            .log_analysis_success("openai", "gpt-4-vision", 1, 100, vec!["hash1".to_string()])
            .unwrap();

        let content = std::fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
    }
}
