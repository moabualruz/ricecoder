//! System events for ricecoder
//!
//! This module defines system events that can trigger hooks, such as file saves,
//! test completions, and code generation completions.

use serde::{Deserialize, Serialize};

/// System event types
///
/// These events are emitted by ricecoder components and can trigger registered hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SystemEvent {
    /// File was saved
    #[serde(rename = "file_saved")]
    FileSaved(FileSavedEvent),

    /// Test execution completed
    #[serde(rename = "test_passed")]
    TestPassed(TestPassedEvent),

    /// Test execution failed
    #[serde(rename = "test_failed")]
    TestFailed(TestFailedEvent),

    /// Code generation completed
    #[serde(rename = "generation_complete")]
    GenerationComplete(GenerationCompleteEvent),

    /// Code refactoring completed
    #[serde(rename = "refactoring_complete")]
    RefactoringComplete(RefactoringCompleteEvent),

    /// Code review completed
    #[serde(rename = "review_complete")]
    ReviewComplete(ReviewCompleteEvent),

    /// Build completed successfully
    #[serde(rename = "build_success")]
    BuildSuccess(BuildSuccessEvent),

    /// Build failed
    #[serde(rename = "build_failed")]
    BuildFailedEvent(BuildFailedEvent),

    /// Deployment completed
    #[serde(rename = "deployment_complete")]
    DeploymentComplete(DeploymentCompleteEvent),

    /// Custom event
    #[serde(rename = "custom")]
    Custom(CustomEvent),
}

/// File saved event
///
/// Emitted when a file is saved in the editor or through the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSavedEvent {
    /// Path to the saved file
    pub file_path: String,

    /// File size in bytes
    pub size: u64,

    /// File hash (SHA256)
    pub hash: String,

    /// Timestamp of the save
    pub timestamp: String,

    /// Language of the file (if detected)
    pub language: Option<String>,
}

/// Test passed event
///
/// Emitted when a test execution completes successfully.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPassedEvent {
    /// Test name or path
    pub test_name: String,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Number of assertions passed
    pub assertions_passed: u32,

    /// Timestamp of the test completion
    pub timestamp: String,
}

/// Test failed event
///
/// Emitted when a test execution fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailedEvent {
    /// Test name or path
    pub test_name: String,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Number of assertions failed
    pub assertions_failed: u32,

    /// Error message
    pub error_message: String,

    /// Timestamp of the test completion
    pub timestamp: String,
}

/// Code generation completed event
///
/// Emitted when code generation from a specification completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationCompleteEvent {
    /// Specification file path
    pub spec_path: String,

    /// Output directory
    pub output_dir: String,

    /// Number of files generated
    pub files_generated: u32,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Timestamp of the completion
    pub timestamp: String,
}

/// Code refactoring completed event
///
/// Emitted when code refactoring completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringCompleteEvent {
    /// File path that was refactored
    pub file_path: String,

    /// Number of changes made
    pub changes_made: u32,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Timestamp of the completion
    pub timestamp: String,
}

/// Code review completed event
///
/// Emitted when a code review completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCompleteEvent {
    /// File path that was reviewed
    pub file_path: String,

    /// Number of issues found
    pub issues_found: u32,

    /// Severity level (info, warning, error)
    pub severity: String,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Timestamp of the completion
    pub timestamp: String,
}

/// Build success event
///
/// Emitted when a build completes successfully.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSuccessEvent {
    /// Build target
    pub target: String,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Output artifacts
    pub artifacts: Vec<String>,

    /// Timestamp of the completion
    pub timestamp: String,
}

/// Build failed event
///
/// Emitted when a build fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildFailedEvent {
    /// Build target
    pub target: String,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Error message
    pub error_message: String,

    /// Timestamp of the completion
    pub timestamp: String,
}

/// Deployment completed event
///
/// Emitted when a deployment completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentCompleteEvent {
    /// Deployment target
    pub target: String,

    /// Deployment environment
    pub environment: String,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Timestamp of the completion
    pub timestamp: String,
}

/// Custom event
///
/// Allows for custom events to be emitted and handled by hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEvent {
    /// Event name
    pub name: String,

    /// Event data
    pub data: serde_json::Value,

    /// Timestamp of the event
    pub timestamp: String,
}

impl SystemEvent {
    /// Get the event type as a string
    pub fn event_type(&self) -> &'static str {
        match self {
            SystemEvent::FileSaved(_) => "file_saved",
            SystemEvent::TestPassed(_) => "test_passed",
            SystemEvent::TestFailed(_) => "test_failed",
            SystemEvent::GenerationComplete(_) => "generation_complete",
            SystemEvent::RefactoringComplete(_) => "refactoring_complete",
            SystemEvent::ReviewComplete(_) => "review_complete",
            SystemEvent::BuildSuccess(_) => "build_success",
            SystemEvent::BuildFailedEvent(_) => "build_failed",
            SystemEvent::DeploymentComplete(_) => "deployment_complete",
            SystemEvent::Custom(_) => "custom",
        }
    }

    /// Convert to event context
    pub fn to_event_context(&self) -> crate::types::EventContext {
        crate::types::EventContext {
            data: serde_json::to_value(self).unwrap_or(serde_json::json!({})),
            metadata: serde_json::json!({
                "event_type": self.event_type(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_saved_event() {
        let event = FileSavedEvent {
            file_path: "/path/to/file.rs".to_string(),
            size: 1024,
            hash: "abc123".to_string(),
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            language: Some("rust".to_string()),
        };

        assert_eq!(event.file_path, "/path/to/file.rs");
        assert_eq!(event.size, 1024);
    }

    #[test]
    fn test_test_passed_event() {
        let event = TestPassedEvent {
            test_name: "test_example".to_string(),
            duration_ms: 100,
            assertions_passed: 5,
            timestamp: "2024-01-01T12:00:00Z".to_string(),
        };

        assert_eq!(event.test_name, "test_example");
        assert_eq!(event.duration_ms, 100);
    }

    #[test]
    fn test_system_event_type() {
        let event = SystemEvent::FileSaved(FileSavedEvent {
            file_path: "/path/to/file.rs".to_string(),
            size: 1024,
            hash: "abc123".to_string(),
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            language: Some("rust".to_string()),
        });

        assert_eq!(event.event_type(), "file_saved");
    }

    #[test]
    fn test_custom_event() {
        let event = SystemEvent::Custom(CustomEvent {
            name: "my_event".to_string(),
            data: serde_json::json!({"key": "value"}),
            timestamp: "2024-01-01T12:00:00Z".to_string(),
        });

        assert_eq!(event.event_type(), "custom");
    }

    #[test]
    fn test_event_serialization() {
        let event = SystemEvent::FileSaved(FileSavedEvent {
            file_path: "/path/to/file.rs".to_string(),
            size: 1024,
            hash: "abc123".to_string(),
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            language: Some("rust".to_string()),
        });

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("file_saved"));
        assert!(json.contains("/path/to/file.rs"));
    }
}
