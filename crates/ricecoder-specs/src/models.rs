//! Core data models for specifications and steering

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A specification document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    /// Unique identifier for the spec
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Version string
    pub version: String,
    /// Requirements for this spec
    pub requirements: Vec<Requirement>,
    /// Design document (optional)
    pub design: Option<Design>,
    /// Implementation tasks
    pub tasks: Vec<Task>,
    /// Metadata about the spec
    pub metadata: SpecMetadata,
    /// Inheritance information (optional)
    pub inheritance: Option<SpecInheritance>,
}

/// Metadata about a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecMetadata {
    /// Author of the spec
    pub author: Option<String>,
    /// When the spec was created
    pub created_at: DateTime<Utc>,
    /// When the spec was last updated
    pub updated_at: DateTime<Utc>,
    /// Current phase of the spec
    pub phase: SpecPhase,
    /// Current status of the spec
    pub status: SpecStatus,
}

/// Phase of specification development
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpecPhase {
    /// Discovery phase
    Discovery,
    /// Requirements phase
    Requirements,
    /// Design phase
    Design,
    /// Tasks phase
    Tasks,
    /// Execution phase
    Execution,
}

/// Status of a specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpecStatus {
    /// Draft - work in progress
    Draft,
    /// In review - awaiting approval
    InReview,
    /// Approved - ready for implementation
    Approved,
    /// Archived - no longer active
    Archived,
}

/// Inheritance information for hierarchical specs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecInheritance {
    /// ID of parent spec (if any)
    pub parent_id: Option<String>,
    /// Precedence level (0=project, 1=feature, 2=task)
    pub precedence_level: u32,
    /// IDs of specs this was merged from
    pub merged_from: Vec<String>,
}

/// A requirement with acceptance criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    /// Unique identifier
    pub id: String,
    /// User story
    pub user_story: String,
    /// Acceptance criteria
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    /// Priority level
    pub priority: Priority,
}

/// An acceptance criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    /// Unique identifier
    pub id: String,
    /// When condition
    pub when: String,
    /// Then expected outcome
    pub then: String,
}

/// Priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    /// Must have
    Must,
    /// Should have
    Should,
    /// Could have
    Could,
}

/// Design document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Design {
    /// Overview of the design
    pub overview: String,
    /// Architecture description
    pub architecture: String,
    /// Components
    pub components: Vec<Component>,
    /// Data models
    pub data_models: Vec<DataModel>,
    /// Correctness properties
    pub correctness_properties: Vec<Property>,
}

/// A component in the design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// Component name
    pub name: String,
    /// Component description
    pub description: String,
}

/// A data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataModel {
    /// Model name
    pub name: String,
    /// Model description
    pub description: String,
}

/// A correctness property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    /// Property identifier
    pub id: String,
    /// Property description
    pub description: String,
    /// Requirements this property validates
    pub validates: Vec<String>,
}

/// An implementation task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task identifier
    pub id: String,
    /// Task description
    pub description: String,
    /// Subtasks
    pub subtasks: Vec<Task>,
    /// Related requirement IDs
    pub requirements: Vec<String>,
    /// Task status
    pub status: TaskStatus,
    /// Whether this task is optional
    pub optional: bool,
}

/// Status of a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Not started
    NotStarted,
    /// In progress
    InProgress,
    /// Complete
    Complete,
}

/// Steering rules and standards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Steering {
    /// Steering rules
    pub rules: Vec<SteeringRule>,
    /// Standards
    pub standards: Vec<Standard>,
    /// Template references
    pub templates: Vec<TemplateRef>,
}

/// A steering rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringRule {
    /// Rule identifier
    pub id: String,
    /// Rule description
    pub description: String,
    /// Pattern to match
    pub pattern: String,
    /// Action to take
    pub action: String,
}

/// A standard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Standard {
    /// Standard identifier
    pub id: String,
    /// Standard description
    pub description: String,
}

/// A template reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRef {
    /// Template identifier
    pub id: String,
    /// Template path
    pub path: String,
}

/// A change to a spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecChange {
    /// Change identifier
    pub id: String,
    /// Spec that was changed
    pub spec_id: String,
    /// When the change was made
    pub timestamp: DateTime<Utc>,
    /// Who made the change
    pub author: Option<String>,
    /// Why the change was made
    pub rationale: String,
    /// Details of the changes
    pub changes: Vec<ChangeDetail>,
}

/// Details of a single change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDetail {
    /// Field that was changed
    pub field: String,
    /// Old value (if any)
    pub old_value: Option<String>,
    /// New value (if any)
    pub new_value: Option<String>,
}

/// A query for specs
#[derive(Debug, Clone, Default)]
pub struct SpecQuery {
    /// Query by name
    pub name: Option<String>,
    /// Filter by type
    pub spec_type: Option<SpecType>,
    /// Filter by status
    pub status: Option<SpecStatus>,
    /// Filter by priority
    pub priority: Option<Priority>,
    /// Filter by phase
    pub phase: Option<SpecPhase>,
    /// Custom filters
    pub custom_filters: Vec<(String, String)>,
}

/// Type of specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpecType {
    /// Feature spec
    Feature,
    /// Component spec
    Component,
    /// Task spec
    Task,
}

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Message identifier
    pub id: String,
    /// Spec this message belongs to
    pub spec_id: String,
    /// Role of the message sender
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// When the message was created
    pub timestamp: DateTime<Utc>,
}

/// Role of a conversation message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// System message
    System,
}

/// An approval gate for phase transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalGate {
    /// Phase being gated
    pub phase: SpecPhase,
    /// Whether this phase has been approved
    pub approved: bool,
    /// When the phase was approved (if at all)
    pub approved_at: Option<DateTime<Utc>>,
    /// Who approved the phase
    pub approved_by: Option<String>,
    /// Feedback on the phase
    pub feedback: Option<String>,
}

/// A spec writing session with conversation history and approval gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecWritingSession {
    /// Session identifier
    pub id: String,
    /// Spec being written
    pub spec_id: String,
    /// Current phase of the session
    pub phase: SpecPhase,
    /// Conversation history
    pub conversation_history: Vec<ConversationMessage>,
    /// Approval gates for each phase
    pub approval_gates: Vec<ApprovalGate>,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session was last updated
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Metadata Tests
    // ============================================================================

    #[test]
    fn test_spec_metadata_creation() {
        let now = Utc::now();
        let metadata = SpecMetadata {
            author: Some("Test Author".to_string()),
            created_at: now,
            updated_at: now,
            phase: SpecPhase::Requirements,
            status: SpecStatus::Draft,
        };

        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert_eq!(metadata.phase, SpecPhase::Requirements);
        assert_eq!(metadata.status, SpecStatus::Draft);
        assert_eq!(metadata.created_at, now);
        assert_eq!(metadata.updated_at, now);
    }

    #[test]
    fn test_spec_metadata_no_author() {
        let now = Utc::now();
        let metadata = SpecMetadata {
            author: None,
            created_at: now,
            updated_at: now,
            phase: SpecPhase::Discovery,
            status: SpecStatus::Draft,
        };

        assert!(metadata.author.is_none());
    }

    // ============================================================================
    // Phase Serialization Tests
    // ============================================================================

    #[test]
    fn test_spec_phase_serialization() {
        let phases = vec![
            SpecPhase::Discovery,
            SpecPhase::Requirements,
            SpecPhase::Design,
            SpecPhase::Tasks,
            SpecPhase::Execution,
        ];

        for phase in phases {
            let json = serde_json::to_string(&phase).unwrap();
            let deserialized: SpecPhase = serde_json::from_str(&json).unwrap();
            assert_eq!(phase, deserialized);
        }
    }

    #[test]
    fn test_spec_status_serialization() {
        let statuses = vec![
            SpecStatus::Draft,
            SpecStatus::InReview,
            SpecStatus::Approved,
            SpecStatus::Archived,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: SpecStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_priority_serialization() {
        let priorities = vec![Priority::Must, Priority::Should, Priority::Could];

        for priority in priorities {
            let json = serde_json::to_string(&priority).unwrap();
            let deserialized: Priority = serde_json::from_str(&json).unwrap();
            assert_eq!(priority, deserialized);
        }
    }

    #[test]
    fn test_task_status_serialization() {
        let statuses = vec![
            TaskStatus::NotStarted,
            TaskStatus::InProgress,
            TaskStatus::Complete,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    // ============================================================================
    // Requirement Serialization Tests
    // ============================================================================

    #[test]
    fn test_acceptance_criterion_serialization() {
        let criterion = AcceptanceCriterion {
            id: "AC-1.1".to_string(),
            when: "user clicks button".to_string(),
            then: "dialog opens".to_string(),
        };

        let json = serde_json::to_string(&criterion).unwrap();
        let deserialized: AcceptanceCriterion = serde_json::from_str(&json).unwrap();

        assert_eq!(criterion.id, deserialized.id);
        assert_eq!(criterion.when, deserialized.when);
        assert_eq!(criterion.then, deserialized.then);
    }

    #[test]
    fn test_requirement_serialization() {
        let requirement = Requirement {
            id: "REQ-1".to_string(),
            user_story: "As a user, I want to create tasks".to_string(),
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    id: "AC-1.1".to_string(),
                    when: "user enters task".to_string(),
                    then: "task is added".to_string(),
                },
                AcceptanceCriterion {
                    id: "AC-1.2".to_string(),
                    when: "user submits empty task".to_string(),
                    then: "error is shown".to_string(),
                },
            ],
            priority: Priority::Must,
        };

        let json = serde_json::to_string(&requirement).unwrap();
        let deserialized: Requirement = serde_json::from_str(&json).unwrap();

        assert_eq!(requirement.id, deserialized.id);
        assert_eq!(requirement.user_story, deserialized.user_story);
        assert_eq!(requirement.priority, deserialized.priority);
        assert_eq!(
            requirement.acceptance_criteria.len(),
            deserialized.acceptance_criteria.len()
        );
    }

    // ============================================================================
    // Design Serialization Tests
    // ============================================================================

    #[test]
    fn test_component_serialization() {
        let component = Component {
            name: "TaskManager".to_string(),
            description: "Manages task lifecycle".to_string(),
        };

        let json = serde_json::to_string(&component).unwrap();
        let deserialized: Component = serde_json::from_str(&json).unwrap();

        assert_eq!(component.name, deserialized.name);
        assert_eq!(component.description, deserialized.description);
    }

    #[test]
    fn test_data_model_serialization() {
        let model = DataModel {
            name: "Task".to_string(),
            description: "Represents a task in the system".to_string(),
        };

        let json = serde_json::to_string(&model).unwrap();
        let deserialized: DataModel = serde_json::from_str(&json).unwrap();

        assert_eq!(model.name, deserialized.name);
        assert_eq!(model.description, deserialized.description);
    }

    #[test]
    fn test_property_serialization() {
        let property = Property {
            id: "PROP-1".to_string(),
            description: "For any task list, adding a task increases length by 1".to_string(),
            validates: vec!["REQ-1.1".to_string(), "REQ-1.2".to_string()],
        };

        let json = serde_json::to_string(&property).unwrap();
        let deserialized: Property = serde_json::from_str(&json).unwrap();

        assert_eq!(property.id, deserialized.id);
        assert_eq!(property.description, deserialized.description);
        assert_eq!(property.validates, deserialized.validates);
    }

    #[test]
    fn test_design_serialization() {
        let design = Design {
            overview: "Task management system".to_string(),
            architecture: "Layered architecture".to_string(),
            components: vec![Component {
                name: "TaskManager".to_string(),
                description: "Manages tasks".to_string(),
            }],
            data_models: vec![DataModel {
                name: "Task".to_string(),
                description: "Task entity".to_string(),
            }],
            correctness_properties: vec![Property {
                id: "PROP-1".to_string(),
                description: "Task addition property".to_string(),
                validates: vec!["REQ-1".to_string()],
            }],
        };

        let json = serde_json::to_string(&design).unwrap();
        let deserialized: Design = serde_json::from_str(&json).unwrap();

        assert_eq!(design.overview, deserialized.overview);
        assert_eq!(design.architecture, deserialized.architecture);
        assert_eq!(design.components.len(), deserialized.components.len());
        assert_eq!(design.data_models.len(), deserialized.data_models.len());
        assert_eq!(
            design.correctness_properties.len(),
            deserialized.correctness_properties.len()
        );
    }

    // ============================================================================
    // Task Serialization Tests
    // ============================================================================

    #[test]
    fn test_task_serialization() {
        let task = Task {
            id: "1".to_string(),
            description: "Implement task manager".to_string(),
            subtasks: vec![Task {
                id: "1.1".to_string(),
                description: "Create data model".to_string(),
                subtasks: vec![],
                requirements: vec!["REQ-1".to_string()],
                status: TaskStatus::NotStarted,
                optional: false,
            }],
            requirements: vec!["REQ-1".to_string()],
            status: TaskStatus::InProgress,
            optional: false,
        };

        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();

        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.description, deserialized.description);
        assert_eq!(task.status, deserialized.status);
        assert_eq!(task.optional, deserialized.optional);
        assert_eq!(task.subtasks.len(), deserialized.subtasks.len());
    }

    // ============================================================================
    // Inheritance Serialization Tests
    // ============================================================================

    #[test]
    fn test_spec_inheritance_serialization() {
        let inheritance = SpecInheritance {
            parent_id: Some("parent-spec".to_string()),
            precedence_level: 1,
            merged_from: vec!["spec-a".to_string(), "spec-b".to_string()],
        };

        let json = serde_json::to_string(&inheritance).unwrap();
        let deserialized: SpecInheritance = serde_json::from_str(&json).unwrap();

        assert_eq!(inheritance.parent_id, deserialized.parent_id);
        assert_eq!(inheritance.precedence_level, deserialized.precedence_level);
        assert_eq!(inheritance.merged_from, deserialized.merged_from);
    }

    #[test]
    fn test_spec_inheritance_no_parent() {
        let inheritance = SpecInheritance {
            parent_id: None,
            precedence_level: 0,
            merged_from: vec![],
        };

        let json = serde_json::to_string(&inheritance).unwrap();
        let deserialized: SpecInheritance = serde_json::from_str(&json).unwrap();

        assert!(deserialized.parent_id.is_none());
        assert_eq!(deserialized.precedence_level, 0);
        assert!(deserialized.merged_from.is_empty());
    }

    #[test]
    fn test_spec_inheritance_precedence_levels() {
        let levels = vec![0, 1, 2];

        for level in levels {
            let inheritance = SpecInheritance {
                parent_id: Some("parent".to_string()),
                precedence_level: level,
                merged_from: vec![],
            };

            let json = serde_json::to_string(&inheritance).unwrap();
            let deserialized: SpecInheritance = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.precedence_level, level);
        }
    }

    // ============================================================================
    // Change Tracking Serialization Tests
    // ============================================================================

    #[test]
    fn test_change_detail_serialization() {
        let change = ChangeDetail {
            field: "status".to_string(),
            old_value: Some("Draft".to_string()),
            new_value: Some("Approved".to_string()),
        };

        let json = serde_json::to_string(&change).unwrap();
        let deserialized: ChangeDetail = serde_json::from_str(&json).unwrap();

        assert_eq!(change.field, deserialized.field);
        assert_eq!(change.old_value, deserialized.old_value);
        assert_eq!(change.new_value, deserialized.new_value);
    }

    #[test]
    fn test_change_detail_with_none_values() {
        let change = ChangeDetail {
            field: "new_field".to_string(),
            old_value: None,
            new_value: Some("value".to_string()),
        };

        let json = serde_json::to_string(&change).unwrap();
        let deserialized: ChangeDetail = serde_json::from_str(&json).unwrap();

        assert!(deserialized.old_value.is_none());
        assert_eq!(deserialized.new_value, Some("value".to_string()));
    }

    #[test]
    fn test_spec_change_serialization() {
        let now = Utc::now();
        let spec_change = SpecChange {
            id: "change-1".to_string(),
            spec_id: "spec-1".to_string(),
            timestamp: now,
            author: Some("John Doe".to_string()),
            rationale: "Updated requirements".to_string(),
            changes: vec![
                ChangeDetail {
                    field: "status".to_string(),
                    old_value: Some("Draft".to_string()),
                    new_value: Some("Approved".to_string()),
                },
                ChangeDetail {
                    field: "phase".to_string(),
                    old_value: Some("Requirements".to_string()),
                    new_value: Some("Design".to_string()),
                },
            ],
        };

        let json = serde_json::to_string(&spec_change).unwrap();
        let deserialized: SpecChange = serde_json::from_str(&json).unwrap();

        assert_eq!(spec_change.id, deserialized.id);
        assert_eq!(spec_change.spec_id, deserialized.spec_id);
        assert_eq!(spec_change.author, deserialized.author);
        assert_eq!(spec_change.rationale, deserialized.rationale);
        assert_eq!(spec_change.changes.len(), deserialized.changes.len());
    }

    // ============================================================================
    // Steering Serialization Tests
    // ============================================================================

    #[test]
    fn test_steering_rule_serialization() {
        let rule = SteeringRule {
            id: "rule-1".to_string(),
            description: "Use snake_case for variables".to_string(),
            pattern: "^[a-z_]+$".to_string(),
            action: "enforce".to_string(),
        };

        let json = serde_json::to_string(&rule).unwrap();
        let deserialized: SteeringRule = serde_json::from_str(&json).unwrap();

        assert_eq!(rule.id, deserialized.id);
        assert_eq!(rule.description, deserialized.description);
        assert_eq!(rule.pattern, deserialized.pattern);
        assert_eq!(rule.action, deserialized.action);
    }

    #[test]
    fn test_standard_serialization() {
        let standard = Standard {
            id: "std-1".to_string(),
            description: "All public APIs must have tests".to_string(),
        };

        let json = serde_json::to_string(&standard).unwrap();
        let deserialized: Standard = serde_json::from_str(&json).unwrap();

        assert_eq!(standard.id, deserialized.id);
        assert_eq!(standard.description, deserialized.description);
    }

    #[test]
    fn test_template_ref_serialization() {
        let template = TemplateRef {
            id: "tpl-1".to_string(),
            path: "templates/rust-entity.rs".to_string(),
        };

        let json = serde_json::to_string(&template).unwrap();
        let deserialized: TemplateRef = serde_json::from_str(&json).unwrap();

        assert_eq!(template.id, deserialized.id);
        assert_eq!(template.path, deserialized.path);
    }

    #[test]
    fn test_steering_serialization() {
        let steering = Steering {
            rules: vec![SteeringRule {
                id: "rule-1".to_string(),
                description: "Use snake_case".to_string(),
                pattern: "^[a-z_]+$".to_string(),
                action: "enforce".to_string(),
            }],
            standards: vec![Standard {
                id: "std-1".to_string(),
                description: "Test all public APIs".to_string(),
            }],
            templates: vec![TemplateRef {
                id: "tpl-1".to_string(),
                path: "templates/entity.rs".to_string(),
            }],
        };

        let json = serde_json::to_string(&steering).unwrap();
        let deserialized: Steering = serde_json::from_str(&json).unwrap();

        assert_eq!(steering.rules.len(), deserialized.rules.len());
        assert_eq!(steering.standards.len(), deserialized.standards.len());
        assert_eq!(steering.templates.len(), deserialized.templates.len());
    }

    // ============================================================================
    // Full Spec Serialization Tests
    // ============================================================================

    #[test]
    fn test_spec_serialization_complete() {
        let now = Utc::now();
        let spec = Spec {
            id: "feature-1".to_string(),
            name: "Task Management".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![Requirement {
                id: "REQ-1".to_string(),
                user_story: "As a user, I want to create tasks".to_string(),
                acceptance_criteria: vec![AcceptanceCriterion {
                    id: "AC-1.1".to_string(),
                    when: "user enters task".to_string(),
                    then: "task is added".to_string(),
                }],
                priority: Priority::Must,
            }],
            design: Some(Design {
                overview: "Task management system".to_string(),
                architecture: "Layered".to_string(),
                components: vec![],
                data_models: vec![],
                correctness_properties: vec![],
            }),
            tasks: vec![Task {
                id: "1".to_string(),
                description: "Implement task manager".to_string(),
                subtasks: vec![],
                requirements: vec!["REQ-1".to_string()],
                status: TaskStatus::NotStarted,
                optional: false,
            }],
            metadata: SpecMetadata {
                author: Some("Developer".to_string()),
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: Some(SpecInheritance {
                parent_id: None,
                precedence_level: 0,
                merged_from: vec![],
            }),
        };

        let json = serde_json::to_string(&spec).unwrap();
        let deserialized: Spec = serde_json::from_str(&json).unwrap();

        assert_eq!(spec.id, deserialized.id);
        assert_eq!(spec.name, deserialized.name);
        assert_eq!(spec.version, deserialized.version);
        assert_eq!(spec.requirements.len(), deserialized.requirements.len());
        assert!(deserialized.design.is_some());
        assert_eq!(spec.tasks.len(), deserialized.tasks.len());
        assert_eq!(spec.metadata.author, deserialized.metadata.author);
        assert!(deserialized.inheritance.is_some());
    }

    #[test]
    fn test_spec_serialization_minimal() {
        let now = Utc::now();
        let spec = Spec {
            id: "minimal".to_string(),
            name: "Minimal Spec".to_string(),
            version: "0.1.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Discovery,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let json = serde_json::to_string(&spec).unwrap();
        let deserialized: Spec = serde_json::from_str(&json).unwrap();

        assert_eq!(spec.id, deserialized.id);
        assert!(deserialized.design.is_none());
        assert!(deserialized.inheritance.is_none());
        assert!(deserialized.metadata.author.is_none());
    }

    // ============================================================================
    // YAML Serialization Tests
    // ============================================================================

    #[test]
    fn test_spec_yaml_serialization() {
        let now = Utc::now();
        let spec = Spec {
            id: "yaml-test".to_string(),
            name: "YAML Test Spec".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Test Author".to_string()),
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let yaml = serde_yaml::to_string(&spec).unwrap();
        let deserialized: Spec = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(spec.id, deserialized.id);
        assert_eq!(spec.name, deserialized.name);
        assert_eq!(spec.metadata.author, deserialized.metadata.author);
    }

    #[test]
    fn test_requirement_yaml_serialization() {
        let requirement = Requirement {
            id: "REQ-1".to_string(),
            user_story: "As a user, I want to manage tasks".to_string(),
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "AC-1.1".to_string(),
                when: "user clicks add".to_string(),
                then: "task is created".to_string(),
            }],
            priority: Priority::Must,
        };

        let yaml = serde_yaml::to_string(&requirement).unwrap();
        let deserialized: Requirement = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(requirement.id, deserialized.id);
        assert_eq!(requirement.priority, deserialized.priority);
    }

    // ============================================================================
    // Query Tests
    // ============================================================================

    #[test]
    fn test_spec_query_default() {
        let query = SpecQuery::default();

        assert!(query.name.is_none());
        assert!(query.spec_type.is_none());
        assert!(query.status.is_none());
        assert!(query.priority.is_none());
        assert!(query.phase.is_none());
        assert!(query.custom_filters.is_empty());
    }

    #[test]
    fn test_spec_query_with_filters() {
        let query = SpecQuery {
            name: Some("task-management".to_string()),
            spec_type: Some(SpecType::Feature),
            status: Some(SpecStatus::Approved),
            priority: Some(Priority::Must),
            phase: Some(SpecPhase::Design),
            custom_filters: vec![("author".to_string(), "John".to_string())],
        };

        assert_eq!(query.name, Some("task-management".to_string()));
        assert_eq!(query.spec_type, Some(SpecType::Feature));
        assert_eq!(query.status, Some(SpecStatus::Approved));
        assert_eq!(query.priority, Some(Priority::Must));
        assert_eq!(query.phase, Some(SpecPhase::Design));
        assert_eq!(query.custom_filters.len(), 1);
    }

    // ============================================================================
    // Approval Gate Tests
    // ============================================================================

    #[test]
    fn test_approval_gate_creation() {
        let gate = ApprovalGate {
            phase: SpecPhase::Requirements,
            approved: false,
            approved_at: None,
            approved_by: None,
            feedback: None,
        };

        assert_eq!(gate.phase, SpecPhase::Requirements);
        assert!(!gate.approved);
        assert!(gate.approved_at.is_none());
        assert!(gate.approved_by.is_none());
        assert!(gate.feedback.is_none());
    }

    #[test]
    fn test_approval_gate_approved() {
        let now = Utc::now();
        let gate = ApprovalGate {
            phase: SpecPhase::Design,
            approved: true,
            approved_at: Some(now),
            approved_by: Some("reviewer".to_string()),
            feedback: Some("Looks good".to_string()),
        };

        assert_eq!(gate.phase, SpecPhase::Design);
        assert!(gate.approved);
        assert_eq!(gate.approved_at, Some(now));
        assert_eq!(gate.approved_by, Some("reviewer".to_string()));
        assert_eq!(gate.feedback, Some("Looks good".to_string()));
    }

    #[test]
    fn test_approval_gate_serialization() {
        let now = Utc::now();
        let gate = ApprovalGate {
            phase: SpecPhase::Tasks,
            approved: true,
            approved_at: Some(now),
            approved_by: Some("john".to_string()),
            feedback: Some("Ready to implement".to_string()),
        };

        let json = serde_json::to_string(&gate).unwrap();
        let deserialized: ApprovalGate = serde_json::from_str(&json).unwrap();

        assert_eq!(gate.phase, deserialized.phase);
        assert_eq!(gate.approved, deserialized.approved);
        assert_eq!(gate.approved_by, deserialized.approved_by);
        assert_eq!(gate.feedback, deserialized.feedback);
    }

    // ============================================================================
    // Spec Writing Session Tests
    // ============================================================================

    #[test]
    fn test_spec_writing_session_creation() {
        let now = Utc::now();
        let session = SpecWritingSession {
            id: "session-1".to_string(),
            spec_id: "spec-1".to_string(),
            phase: SpecPhase::Requirements,
            conversation_history: vec![],
            approval_gates: vec![],
            created_at: now,
            updated_at: now,
        };

        assert_eq!(session.id, "session-1");
        assert_eq!(session.spec_id, "spec-1");
        assert_eq!(session.phase, SpecPhase::Requirements);
        assert!(session.conversation_history.is_empty());
        assert!(session.approval_gates.is_empty());
    }

    #[test]
    fn test_spec_writing_session_with_messages() {
        let now = Utc::now();
        let messages = vec![
            ConversationMessage {
                id: "msg-1".to_string(),
                spec_id: "spec-1".to_string(),
                role: MessageRole::User,
                content: "Create a task management system".to_string(),
                timestamp: now,
            },
            ConversationMessage {
                id: "msg-2".to_string(),
                spec_id: "spec-1".to_string(),
                role: MessageRole::Assistant,
                content: "I'll help you create a task management system".to_string(),
                timestamp: now,
            },
        ];

        let session = SpecWritingSession {
            id: "session-1".to_string(),
            spec_id: "spec-1".to_string(),
            phase: SpecPhase::Requirements,
            conversation_history: messages.clone(),
            approval_gates: vec![],
            created_at: now,
            updated_at: now,
        };

        assert_eq!(session.conversation_history.len(), 2);
        assert_eq!(session.conversation_history[0].role, MessageRole::User);
        assert_eq!(session.conversation_history[1].role, MessageRole::Assistant);
    }

    #[test]
    fn test_spec_writing_session_with_approval_gates() {
        let now = Utc::now();
        let gates = vec![
            ApprovalGate {
                phase: SpecPhase::Requirements,
                approved: true,
                approved_at: Some(now),
                approved_by: Some("reviewer".to_string()),
                feedback: None,
            },
            ApprovalGate {
                phase: SpecPhase::Design,
                approved: false,
                approved_at: None,
                approved_by: None,
                feedback: None,
            },
        ];

        let session = SpecWritingSession {
            id: "session-1".to_string(),
            spec_id: "spec-1".to_string(),
            phase: SpecPhase::Design,
            conversation_history: vec![],
            approval_gates: gates.clone(),
            created_at: now,
            updated_at: now,
        };

        assert_eq!(session.approval_gates.len(), 2);
        assert!(session.approval_gates[0].approved);
        assert!(!session.approval_gates[1].approved);
    }

    #[test]
    fn test_spec_writing_session_serialization() {
        let now = Utc::now();
        let session = SpecWritingSession {
            id: "session-1".to_string(),
            spec_id: "spec-1".to_string(),
            phase: SpecPhase::Requirements,
            conversation_history: vec![ConversationMessage {
                id: "msg-1".to_string(),
                spec_id: "spec-1".to_string(),
                role: MessageRole::User,
                content: "Create a system".to_string(),
                timestamp: now,
            }],
            approval_gates: vec![ApprovalGate {
                phase: SpecPhase::Requirements,
                approved: false,
                approved_at: None,
                approved_by: None,
                feedback: None,
            }],
            created_at: now,
            updated_at: now,
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: SpecWritingSession = serde_json::from_str(&json).unwrap();

        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.spec_id, deserialized.spec_id);
        assert_eq!(session.phase, deserialized.phase);
        assert_eq!(
            session.conversation_history.len(),
            deserialized.conversation_history.len()
        );
        assert_eq!(
            session.approval_gates.len(),
            deserialized.approval_gates.len()
        );
    }

    #[test]
    fn test_spec_writing_session_yaml_serialization() {
        let now = Utc::now();
        let session = SpecWritingSession {
            id: "session-1".to_string(),
            spec_id: "spec-1".to_string(),
            phase: SpecPhase::Design,
            conversation_history: vec![],
            approval_gates: vec![],
            created_at: now,
            updated_at: now,
        };

        let yaml = serde_yaml::to_string(&session).unwrap();
        let deserialized: SpecWritingSession = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.spec_id, deserialized.spec_id);
        assert_eq!(session.phase, deserialized.phase);
    }

    #[test]
    fn test_spec_writing_session_phase_progression() {
        let now = Utc::now();
        let phases = vec![
            SpecPhase::Discovery,
            SpecPhase::Requirements,
            SpecPhase::Design,
            SpecPhase::Tasks,
            SpecPhase::Execution,
        ];

        for phase in phases {
            let session = SpecWritingSession {
                id: "session-1".to_string(),
                spec_id: "spec-1".to_string(),
                phase,
                conversation_history: vec![],
                approval_gates: vec![],
                created_at: now,
                updated_at: now,
            };

            assert_eq!(session.phase, phase);
        }
    }
}
