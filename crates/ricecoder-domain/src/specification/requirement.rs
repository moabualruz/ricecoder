//! Requirement entity within a Specification
//!
//! Contains title, description, and acceptance criteria
//! Must have at least one acceptance criterion

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_objects::RequirementId;

/// Requirement entity within a Specification
///
/// Contains title, description, and acceptance criteria
/// Must have at least one acceptance criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    /// Requirement identity
    pub(crate) id: RequirementId,

    /// Requirement title (e.g., "REQ-001: User Authentication")
    pub(crate) title: String,

    /// Detailed description
    pub(crate) description: String,

    /// Acceptance criteria (at least one required)
    pub(crate) acceptance_criteria: Vec<String>,

    /// Whether the requirement has been approved
    pub(crate) approved: bool,

    /// Creation timestamp
    pub(crate) created_at: DateTime<Utc>,

    /// Last update timestamp
    pub(crate) updated_at: DateTime<Utc>,
}

impl Requirement {
    /// Create a new requirement (internal use by Specification aggregate)
    pub(crate) fn new(title: String, description: String, acceptance_criteria: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: RequirementId::new(),
            title,
            description,
            acceptance_criteria,
            approved: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitute a requirement from persistence
    pub fn reconstitute(
        id: RequirementId,
        title: String,
        description: String,
        acceptance_criteria: Vec<String>,
        approved: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            title,
            description,
            acceptance_criteria,
            approved,
            created_at,
            updated_at,
        }
    }

    /// Get requirement ID
    pub fn id(&self) -> RequirementId {
        self.id
    }

    /// Get requirement title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get requirement description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get acceptance criteria
    pub fn acceptance_criteria(&self) -> &[String] {
        &self.acceptance_criteria
    }

    /// Check if requirement is approved
    pub fn is_approved(&self) -> bool {
        self.approved
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get last update timestamp
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
