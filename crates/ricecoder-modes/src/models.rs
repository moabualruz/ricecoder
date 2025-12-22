use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

/// Represents a capability that a mode can have
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Code generation capability
    CodeGeneration,
    /// Code modification capability
    CodeModification,
    /// File operations capability
    FileOperations,
    /// Command execution capability
    CommandExecution,
    /// Test execution capability
    TestExecution,
    /// Quality validation capability
    QualityValidation,
    /// Question answering capability
    QuestionAnswering,
    /// Freeform chat capability
    FreeformChat,
    /// Spec conversion capability
    SpecConversion,
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::CodeGeneration => write!(f, "CodeGeneration"),
            Capability::CodeModification => write!(f, "CodeModification"),
            Capability::FileOperations => write!(f, "FileOperations"),
            Capability::CommandExecution => write!(f, "CommandExecution"),
            Capability::TestExecution => write!(f, "TestExecution"),
            Capability::QualityValidation => write!(f, "QualityValidation"),
            Capability::QuestionAnswering => write!(f, "QuestionAnswering"),
            Capability::FreeformChat => write!(f, "FreeformChat"),
            Capability::SpecConversion => write!(f, "SpecConversion"),
        }
    }
}

/// Represents an operation that can be validated against mode constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operation {
    /// Generate code operation
    GenerateCode,
    /// Modify file operation
    ModifyFile,
    /// Execute command operation
    ExecuteCommand,
    /// Run tests operation
    RunTests,
    /// Validate quality operation
    ValidateQuality,
    /// Answer question operation
    AnswerQuestion,
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::GenerateCode => write!(f, "GenerateCode"),
            Operation::ModifyFile => write!(f, "ModifyFile"),
            Operation::ExecuteCommand => write!(f, "ExecuteCommand"),
            Operation::RunTests => write!(f, "RunTests"),
            Operation::ValidateQuality => write!(f, "ValidateQuality"),
            Operation::AnswerQuestion => write!(f, "AnswerQuestion"),
        }
    }
}

/// Constraints that apply to a mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConstraints {
    /// Whether file operations are allowed
    pub allow_file_operations: bool,
    /// Whether command execution is allowed
    pub allow_command_execution: bool,
    /// Whether code generation is allowed
    pub allow_code_generation: bool,
    /// Whether specs are required
    pub require_specs: bool,
    /// Complexity threshold for auto-enabling Think More
    pub auto_think_more_threshold: Option<ComplexityLevel>,
}

/// Complexity level for auto-enabling Think More
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    /// Simple complexity level
    Simple,
    /// Moderate complexity level
    Moderate,
    /// Complex complexity level
    Complex,
}

/// Configuration for Think More extended thinking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkMoreConfig {
    /// Whether Think More is enabled
    pub enabled: bool,
    /// Depth of thinking
    pub depth: ThinkingDepth,
    /// Timeout for thinking
    pub timeout: Duration,
    /// Whether to auto-enable based on complexity
    pub auto_enable: bool,
}

impl Default for ThinkMoreConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            depth: ThinkingDepth::Medium,
            timeout: Duration::from_secs(30),
            auto_enable: false,
        }
    }
}

/// Depth of thinking for extended reasoning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThinkingDepth {
    /// Light thinking depth
    Light,
    /// Medium thinking depth
    Medium,
    /// Deep thinking depth
    Deep,
}

/// Configuration for a mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    /// Temperature for response generation
    pub temperature: f32,
    /// Maximum tokens for response
    pub max_tokens: usize,
    /// System prompt for the mode
    pub system_prompt: String,
    /// Capabilities available in this mode
    pub capabilities: Vec<Capability>,
    /// Constraints for this mode
    pub constraints: ModeConstraints,
}

/// A message in the conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: MessageRole,
    /// Content of the message
    pub content: String,
    /// Timestamp when the message was created
    pub timestamp: SystemTime,
}

/// Role of a message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// System message
    System,
}

/// Context for mode execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeContext {
    /// Session ID
    pub session_id: String,
    /// Project path
    pub project_path: Option<PathBuf>,
    /// Whether Think More is enabled
    pub think_more_enabled: bool,
    /// Think More configuration
    pub think_more_config: ThinkMoreConfig,
    /// Conversation history
    pub conversation_history: Vec<Message>,
    /// Custom context data
    pub custom: HashMap<String, serde_json::Value>,
}

impl ModeContext {
    /// Create a new mode context
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            project_path: None,
            think_more_enabled: false,
            think_more_config: ThinkMoreConfig::default(),
            conversation_history: Vec::new(),
            custom: HashMap::new(),
        }
    }

    /// Add a message to the conversation history
    pub fn add_message(&mut self, role: MessageRole, content: String) {
        self.conversation_history.push(Message {
            role,
            content,
            timestamp: SystemTime::now(),
        });
    }
}

/// An action that a mode can perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModeAction {
    /// Generate code from a specification
    GenerateCode {
        /// The specification
        spec: String,
    },
    /// Modify a file
    ModifyFile {
        /// Path to the file
        path: PathBuf,
        /// Diff to apply
        diff: String,
    },
    /// Run a command
    RunCommand {
        /// The command to run
        command: String,
    },
    /// Run tests
    RunTests {
        /// Paths to test
        paths: Vec<PathBuf>,
    },
    /// Validate quality
    ValidateQuality {
        /// Paths to validate
        paths: Vec<PathBuf>,
    },
    /// Ask a question
    AskQuestion {
        /// The question
        question: String,
    },
    /// Suggest a mode
    SuggestMode {
        /// The mode to suggest
        mode: String,
        /// Reason for suggestion
        reason: String,
    },
    /// Display thinking content
    DisplayThinking {
        /// The thinking content
        content: String,
    },
}

/// Summary of changes made
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChangeSummary {
    /// Number of files created
    pub files_created: usize,
    /// Number of files modified
    pub files_modified: usize,
    /// Number of tests run
    pub tests_run: usize,
    /// Number of tests passed
    pub tests_passed: usize,
    /// Quality issues found
    pub quality_issues: Vec<String>,
}

/// Metadata about a response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// The mode that generated the response
    pub mode: String,
    /// Whether Think More was used
    pub think_more_used: bool,
    /// Thinking content if Think More was used
    pub thinking_content: Option<String>,
    /// Number of tokens used
    pub tokens_used: usize,
    /// Duration of processing
    pub duration: Duration,
    /// Summary of changes made
    pub changes_summary: Option<ChangeSummary>,
}

/// Response from a mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeResponse {
    /// Response content
    pub content: String,
    /// Actions to perform
    pub actions: Vec<ModeAction>,
    /// Suggestions for the user
    pub suggestions: Vec<String>,
    /// Response metadata
    pub metadata: ResponseMetadata,
    /// Summary of changes
    pub summary: Option<String>,
}

impl ModeResponse {
    /// Create a new mode response
    pub fn new(content: String, mode: String) -> Self {
        Self {
            content,
            actions: Vec::new(),
            suggestions: Vec::new(),
            metadata: ResponseMetadata {
                mode,
                think_more_used: false,
                thinking_content: None,
                tokens_used: 0,
                duration: Duration::from_secs(0),
                changes_summary: None,
            },
            summary: None,
        }
    }

    /// Add an action to the response
    pub fn add_action(&mut self, action: ModeAction) {
        self.actions.push(action);
    }

    /// Add a suggestion to the response
    pub fn add_suggestion(&mut self, suggestion: String) {
        self.suggestions.push(suggestion);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_context_creation() {
        let context = ModeContext::new("test-session".to_string());
        assert_eq!(context.session_id, "test-session");
        assert!(context.project_path.is_none());
        assert!(!context.think_more_enabled);
        assert!(context.conversation_history.is_empty());
    }

    #[test]
    fn test_mode_context_add_message() {
        let mut context = ModeContext::new("test-session".to_string());
        context.add_message(MessageRole::User, "Hello".to_string());
        assert_eq!(context.conversation_history.len(), 1);
        assert_eq!(context.conversation_history[0].role, MessageRole::User);
        assert_eq!(context.conversation_history[0].content, "Hello");
    }

    #[test]
    fn test_mode_response_creation() {
        let response = ModeResponse::new("Test response".to_string(), "test-mode".to_string());
        assert_eq!(response.content, "Test response");
        assert_eq!(response.metadata.mode, "test-mode");
        assert!(response.actions.is_empty());
        assert!(response.suggestions.is_empty());
    }

    #[test]
    fn test_mode_response_add_action() {
        let mut response = ModeResponse::new("Test".to_string(), "test-mode".to_string());
        response.add_action(ModeAction::AskQuestion {
            question: "What is this?".to_string(),
        });
        assert_eq!(response.actions.len(), 1);
    }

    #[test]
    fn test_mode_response_add_suggestion() {
        let mut response = ModeResponse::new("Test".to_string(), "test-mode".to_string());
        response.add_suggestion("Try this".to_string());
        assert_eq!(response.suggestions.len(), 1);
        assert_eq!(response.suggestions[0], "Try this");
    }

    #[test]
    fn test_think_more_config_default() {
        let config = ThinkMoreConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.depth, ThinkingDepth::Medium);
        assert!(!config.auto_enable);
    }

    #[test]
    fn test_mode_constraints_creation() {
        let constraints = ModeConstraints {
            allow_file_operations: true,
            allow_command_execution: false,
            allow_code_generation: true,
            require_specs: false,
            auto_think_more_threshold: Some(ComplexityLevel::Complex),
        };
        assert!(constraints.allow_file_operations);
        assert!(!constraints.allow_command_execution);
        assert!(constraints.allow_code_generation);
    }

    #[test]
    fn test_change_summary_default() {
        let summary = ChangeSummary::default();
        assert_eq!(summary.files_created, 0);
        assert_eq!(summary.files_modified, 0);
        assert_eq!(summary.tests_run, 0);
        assert_eq!(summary.tests_passed, 0);
        assert!(summary.quality_issues.is_empty());
    }

    #[test]
    fn test_message_creation() {
        let msg = Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
            timestamp: SystemTime::now(),
        };
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_capability_display() {
        assert_eq!(Capability::CodeGeneration.to_string(), "CodeGeneration");
        assert_eq!(
            Capability::QuestionAnswering.to_string(),
            "QuestionAnswering"
        );
    }

    #[test]
    fn test_operation_display() {
        assert_eq!(Operation::GenerateCode.to_string(), "GenerateCode");
        assert_eq!(Operation::AnswerQuestion.to_string(), "AnswerQuestion");
    }

    #[test]
    fn test_serialization_mode_context() {
        let context = ModeContext::new("test-session".to_string());
        let json = serde_json::to_string(&context).unwrap();
        let deserialized: ModeContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, context.session_id);
    }

    #[test]
    fn test_serialization_mode_response() {
        let response = ModeResponse::new("Test".to_string(), "test-mode".to_string());
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ModeResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.content, response.content);
        assert_eq!(deserialized.metadata.mode, response.metadata.mode);
    }
}
