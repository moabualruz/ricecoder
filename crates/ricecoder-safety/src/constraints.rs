//! Security constraints and policy enforcement

use crate::error::{SafetyError, SafetyResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security constraint types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConstraintType {
    /// Maximum file size in bytes
    MaxFileSize(u64),
    /// Allowed file extensions
    AllowedExtensions(Vec<String>),
    /// Forbidden file extensions
    ForbiddenExtensions(Vec<String>),
    /// Maximum execution time in seconds
    MaxExecutionTime(u64),
    /// Required approval for certain operations
    RequiresApproval(String),
    /// Maximum API calls per minute
    MaxApiCallsPerMinute(u32),
    /// Allowed domains for network access
    AllowedDomains(Vec<String>),
    /// Maximum memory usage in bytes
    MaxMemoryUsage(u64),
    /// Required security context
    SecurityContextRequired(String),
    /// Custom constraint with validation logic
    Custom(String),
}

/// Security constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConstraint {
    /// Unique constraint identifier
    pub id: String,
    /// Constraint name
    pub name: String,
    /// Constraint description
    pub description: String,
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Severity level
    pub severity: ConstraintSeverity,
    /// Whether the constraint is enabled
    pub enabled: bool,
    /// Additional configuration
    pub config: HashMap<String, serde_json::Value>,
}

impl SecurityConstraint {
    /// Create a new security constraint
    pub fn new(id: String, name: String, constraint_type: ConstraintType) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            constraint_type,
            severity: ConstraintSeverity::Medium,
            enabled: true,
            config: HashMap::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// Set the severity
    pub fn with_severity(mut self, severity: ConstraintSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Enable or disable the constraint
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Add configuration
    pub fn with_config(mut self, key: String, value: serde_json::Value) -> Self {
        self.config.insert(key, value);
        self
    }

    /// Create a maximum file size constraint
    pub fn max_file_size(max_size: u64) -> Self {
        Self::new(
            "max_file_size".to_string(),
            "Maximum File Size".to_string(),
            ConstraintType::MaxFileSize(max_size),
        )
        .with_description(format!("Files must not exceed {} bytes", max_size))
    }

    /// Create an allowed extensions constraint
    pub fn allowed_extensions(extensions: Vec<String>) -> Self {
        Self::new(
            "allowed_extensions".to_string(),
            "Allowed File Extensions".to_string(),
            ConstraintType::AllowedExtensions(extensions.clone()),
        )
        .with_description(format!(
            "Only these extensions are allowed: {:?}",
            extensions
        ))
    }

    /// Create a forbidden extensions constraint
    pub fn forbidden_extensions(extensions: Vec<String>) -> Self {
        Self::new(
            "forbidden_extensions".to_string(),
            "Forbidden File Extensions".to_string(),
            ConstraintType::ForbiddenExtensions(extensions.clone()),
        )
        .with_description(format!("These extensions are forbidden: {:?}", extensions))
    }

    /// Create a maximum execution time constraint
    pub fn max_execution_time(seconds: u64) -> Self {
        Self::new(
            "max_execution_time".to_string(),
            "Maximum Execution Time".to_string(),
            ConstraintType::MaxExecutionTime(seconds),
        )
        .with_description(format!(
            "Operations must complete within {} seconds",
            seconds
        ))
    }

    /// Create a requires approval constraint
    pub fn requires_approval(reason: String) -> Self {
        Self::new(
            "requires_approval".to_string(),
            "Requires Approval".to_string(),
            ConstraintType::RequiresApproval(reason.clone()),
        )
        .with_description(format!("Manual approval required: {}", reason))
        .with_severity(ConstraintSeverity::High)
    }

    /// Validate an operation against this constraint
    pub async fn validate(&self, context: &ValidationContext) -> SafetyResult<ConstraintResult> {
        if !self.enabled {
            return Ok(ConstraintResult::Passed);
        }

        match &self.constraint_type {
            ConstraintType::MaxFileSize(max_size) => self.validate_file_size(context, *max_size),
            ConstraintType::AllowedExtensions(extensions) => {
                self.validate_file_extension(context, extensions, true)
            }
            ConstraintType::ForbiddenExtensions(extensions) => {
                self.validate_file_extension(context, extensions, false)
            }
            ConstraintType::MaxExecutionTime(max_seconds) => {
                self.validate_execution_time(context, *max_seconds)
            }
            ConstraintType::RequiresApproval(reason) => {
                Ok(ConstraintResult::ApprovalRequired(reason.clone()))
            }
            ConstraintType::MaxApiCallsPerMinute(limit) => {
                self.validate_api_rate_limit(context, *limit).await
            }
            ConstraintType::AllowedDomains(domains) => {
                self.validate_network_access(context, domains, true)
            }
            ConstraintType::MaxMemoryUsage(max_bytes) => {
                self.validate_memory_usage(context, *max_bytes)
            }
            ConstraintType::SecurityContextRequired(required) => {
                self.validate_security_context(context, required)
            }
            ConstraintType::Custom(logic) => self.validate_custom_constraint(context, logic),
        }
    }

    fn validate_file_size(
        &self,
        context: &ValidationContext,
        max_size: u64,
    ) -> SafetyResult<ConstraintResult> {
        if let Some(file_size) = context.file_size {
            if file_size > max_size {
                return Ok(ConstraintResult::Failed(format!(
                    "File size {} exceeds maximum allowed size {}",
                    file_size, max_size
                )));
            }
        }
        Ok(ConstraintResult::Passed)
    }

    fn validate_file_extension(
        &self,
        context: &ValidationContext,
        extensions: &[String],
        is_allowed: bool,
    ) -> SafetyResult<ConstraintResult> {
        if let Some(file_path) = &context.file_path {
            if let Some(extension) = std::path::Path::new(file_path).extension() {
                let ext_str = extension.to_string_lossy().to_lowercase();
                let contains = extensions.iter().any(|e| e.to_lowercase() == ext_str);

                if is_allowed && !contains {
                    return Ok(ConstraintResult::Failed(format!(
                        "File extension '{}' is not in allowed list: {:?}",
                        ext_str, extensions
                    )));
                } else if !is_allowed && contains {
                    return Ok(ConstraintResult::Failed(format!(
                        "File extension '{}' is forbidden: {:?}",
                        ext_str, extensions
                    )));
                }
            }
        }
        Ok(ConstraintResult::Passed)
    }

    fn validate_execution_time(
        &self,
        context: &ValidationContext,
        max_seconds: u64,
    ) -> SafetyResult<ConstraintResult> {
        if let Some(estimated_time) = context.estimated_execution_time_seconds {
            if estimated_time > max_seconds {
                return Ok(ConstraintResult::Failed(format!(
                    "Estimated execution time {}s exceeds maximum allowed time {}s",
                    estimated_time, max_seconds
                )));
            }
        }
        Ok(ConstraintResult::Passed)
    }

    async fn validate_api_rate_limit(
        &self,
        context: &ValidationContext,
        limit: u32,
    ) -> SafetyResult<ConstraintResult> {
        // This would typically check against a rate limiter
        // For now, return passed
        let _ = (context, limit);
        Ok(ConstraintResult::Passed)
    }

    fn validate_network_access(
        &self,
        context: &ValidationContext,
        domains: &[String],
        is_allowed: bool,
    ) -> SafetyResult<ConstraintResult> {
        if let Some(target_domain) = &context.network_target {
            let contains = domains.iter().any(|d| target_domain.contains(d));

            if is_allowed && !contains {
                return Ok(ConstraintResult::Failed(format!(
                    "Network access to '{}' not in allowed domains: {:?}",
                    target_domain, domains
                )));
            }
        }
        Ok(ConstraintResult::Passed)
    }

    fn validate_memory_usage(
        &self,
        context: &ValidationContext,
        max_bytes: u64,
    ) -> SafetyResult<ConstraintResult> {
        if let Some(memory_usage) = context.estimated_memory_bytes {
            if memory_usage > max_bytes {
                return Ok(ConstraintResult::Failed(format!(
                    "Estimated memory usage {} bytes exceeds maximum allowed {} bytes",
                    memory_usage, max_bytes
                )));
            }
        }
        Ok(ConstraintResult::Passed)
    }

    fn validate_security_context(
        &self,
        context: &ValidationContext,
        required: &str,
    ) -> SafetyResult<ConstraintResult> {
        if let Some(security_context) = &context.security_context {
            if !security_context.iter().any(|ctx| ctx == required) {
                return Ok(ConstraintResult::Failed(format!(
                    "Required security context '{}' not present in: {:?}",
                    required, security_context
                )));
            }
        } else {
            return Ok(ConstraintResult::Failed(format!(
                "Security context required but not provided"
            )));
        }
        Ok(ConstraintResult::Passed)
    }

    fn validate_custom_constraint(
        &self,
        context: &ValidationContext,
        logic: &str,
    ) -> SafetyResult<ConstraintResult> {
        // Placeholder for custom validation logic
        // In a real implementation, this would evaluate custom scripts or rules
        let _ = (context, logic);
        Ok(ConstraintResult::Passed)
    }
}

/// Constraint severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConstraintSeverity {
    /// Low severity - informational
    Low,
    /// Medium severity - requires attention
    Medium,
    /// High severity - blocks operation
    High,
    /// Critical severity - emergency response required
    Critical,
}

/// Result of constraint validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintResult {
    /// Constraint passed
    Passed,
    /// Constraint failed with reason
    Failed(String),
    /// Manual approval required
    ApprovalRequired(String),
}

/// Context for constraint validation
#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
    /// File path being operated on
    pub file_path: Option<String>,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Estimated execution time in seconds
    pub estimated_execution_time_seconds: Option<u64>,
    /// Estimated memory usage in bytes
    pub estimated_memory_bytes: Option<u64>,
    /// Network target domain
    pub network_target: Option<String>,
    /// Security context
    pub security_context: Option<Vec<String>>,
    /// User performing the operation
    pub user_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Additional context data
    pub additional_data: HashMap<String, serde_json::Value>,
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set file path
    pub fn with_file_path(mut self, path: String) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Set file size
    pub fn with_file_size(mut self, size: u64) -> Self {
        self.file_size = Some(size);
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}
