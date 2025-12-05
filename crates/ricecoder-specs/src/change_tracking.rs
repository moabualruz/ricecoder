//! Change tracking for spec modifications

use crate::models::{ChangeDetail, Spec, SpecChange};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Tracks spec evolution and modifications
pub struct ChangeTracker {
    /// In-memory storage of change history by spec_id
    history: Arc<Mutex<HashMap<String, Vec<SpecChange>>>>,
    /// Counter for generating unique change IDs
    change_counter: Arc<Mutex<u64>>,
}

impl ChangeTracker {
    /// Create a new ChangeTracker
    pub fn new() -> Self {
        Self {
            history: Arc::new(Mutex::new(HashMap::new())),
            change_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Record a spec change with field-level tracking
    pub fn record_change(
        &self,
        spec_id: &str,
        old: &Spec,
        new: &Spec,
        author: Option<String>,
        rationale: String,
    ) -> SpecChange {
        // Generate unique change ID
        let mut counter = self.change_counter.lock().unwrap();
        *counter += 1;
        let change_id = format!("change-{}", counter);
        drop(counter);

        // Detect field-level changes
        let changes = Self::detect_changes(old, new);

        let spec_change = SpecChange {
            id: change_id,
            spec_id: spec_id.to_string(),
            timestamp: Utc::now(),
            author,
            rationale,
            changes,
        };

        // Store in history
        let mut history = self.history.lock().unwrap();
        history
            .entry(spec_id.to_string())
            .or_default()
            .push(spec_change.clone());

        spec_change
    }

    /// Get change history for a spec
    pub fn get_history(&self, spec_id: &str) -> Vec<SpecChange> {
        let history = self.history.lock().unwrap();
        history.get(spec_id).cloned().unwrap_or_default()
    }

    /// Get all changes across all specs
    pub fn get_all_changes(&self) -> Vec<SpecChange> {
        let history = self.history.lock().unwrap();
        history
            .values()
            .flat_map(|changes| changes.clone())
            .collect()
    }

    /// Clear history for a spec (useful for testing)
    pub fn clear_history(&self, spec_id: &str) {
        let mut history = self.history.lock().unwrap();
        history.remove(spec_id);
    }

    /// Detect field-level changes between two specs
    fn detect_changes(old: &Spec, new: &Spec) -> Vec<ChangeDetail> {
        let mut changes = Vec::new();

        // Check name
        if old.name != new.name {
            changes.push(ChangeDetail {
                field: "name".to_string(),
                old_value: Some(old.name.clone()),
                new_value: Some(new.name.clone()),
            });
        }

        // Check version
        if old.version != new.version {
            changes.push(ChangeDetail {
                field: "version".to_string(),
                old_value: Some(old.version.clone()),
                new_value: Some(new.version.clone()),
            });
        }

        // Check metadata phase
        if old.metadata.phase != new.metadata.phase {
            changes.push(ChangeDetail {
                field: "metadata.phase".to_string(),
                old_value: Some(format!("{:?}", old.metadata.phase)),
                new_value: Some(format!("{:?}", new.metadata.phase)),
            });
        }

        // Check metadata status
        if old.metadata.status != new.metadata.status {
            changes.push(ChangeDetail {
                field: "metadata.status".to_string(),
                old_value: Some(format!("{:?}", old.metadata.status)),
                new_value: Some(format!("{:?}", new.metadata.status)),
            });
        }

        // Check metadata author
        if old.metadata.author != new.metadata.author {
            changes.push(ChangeDetail {
                field: "metadata.author".to_string(),
                old_value: old.metadata.author.clone(),
                new_value: new.metadata.author.clone(),
            });
        }

        // Check requirements count
        if old.requirements.len() != new.requirements.len() {
            changes.push(ChangeDetail {
                field: "requirements.count".to_string(),
                old_value: Some(old.requirements.len().to_string()),
                new_value: Some(new.requirements.len().to_string()),
            });
        }

        // Check design presence
        let old_has_design = old.design.is_some();
        let new_has_design = new.design.is_some();
        if old_has_design != new_has_design {
            changes.push(ChangeDetail {
                field: "design".to_string(),
                old_value: Some(old_has_design.to_string()),
                new_value: Some(new_has_design.to_string()),
            });
        }

        // Check tasks count
        if old.tasks.len() != new.tasks.len() {
            changes.push(ChangeDetail {
                field: "tasks.count".to_string(),
                old_value: Some(old.tasks.len().to_string()),
                new_value: Some(new.tasks.len().to_string()),
            });
        }

        changes
    }
}

impl Default for ChangeTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ChangeTracker {
    fn clone(&self) -> Self {
        Self {
            history: Arc::clone(&self.history),
            change_counter: Arc::clone(&self.change_counter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SpecMetadata, SpecPhase, SpecStatus};

    #[test]
    fn test_change_tracker_creation() {
        let tracker = ChangeTracker::new();
        let history = tracker.get_history("test-spec");
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_record_change_with_name_change() {
        let tracker = ChangeTracker::new();
        let now = Utc::now();

        let old_spec = Spec {
            id: "spec-1".to_string(),
            name: "Old Name".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: Some("Author".to_string()),
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let mut new_spec = old_spec.clone();
        new_spec.name = "New Name".to_string();

        let change = tracker.record_change(
            "spec-1",
            &old_spec,
            &new_spec,
            Some("John Doe".to_string()),
            "Updated spec name".to_string(),
        );

        assert_eq!(change.spec_id, "spec-1");
        assert_eq!(change.author, Some("John Doe".to_string()));
        assert_eq!(change.rationale, "Updated spec name");
        assert!(!change.changes.is_empty());

        let name_change = change.changes.iter().find(|c| c.field == "name").unwrap();
        assert_eq!(name_change.old_value, Some("Old Name".to_string()));
        assert_eq!(name_change.new_value, Some("New Name".to_string()));
    }

    #[test]
    fn test_record_change_with_version_change() {
        let tracker = ChangeTracker::new();
        let now = Utc::now();

        let old_spec = Spec {
            id: "spec-1".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let mut new_spec = old_spec.clone();
        new_spec.version = "1.1.0".to_string();

        let change = tracker.record_change(
            "spec-1",
            &old_spec,
            &new_spec,
            None,
            "Version bump".to_string(),
        );

        let version_change = change
            .changes
            .iter()
            .find(|c| c.field == "version")
            .unwrap();
        assert_eq!(version_change.old_value, Some("1.0.0".to_string()));
        assert_eq!(version_change.new_value, Some("1.1.0".to_string()));
    }

    #[test]
    fn test_get_history_preserves_order() {
        let tracker = ChangeTracker::new();
        let now = Utc::now();

        let spec = Spec {
            id: "spec-1".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        // Record multiple changes
        let mut current = spec.clone();
        for i in 0..3 {
            let mut next = current.clone();
            next.version = format!("1.{}.0", i + 1);
            tracker.record_change("spec-1", &current, &next, None, format!("Change {}", i + 1));
            current = next;
        }

        let history = tracker.get_history("spec-1");
        assert_eq!(history.len(), 3);

        // Verify order is preserved
        for (i, change) in history.iter().enumerate() {
            assert_eq!(change.rationale, format!("Change {}", i + 1));
        }
    }

    #[test]
    fn test_change_tracker_with_multiple_specs() {
        let tracker = ChangeTracker::new();
        let now = Utc::now();

        let spec1 = Spec {
            id: "spec-1".to_string(),
            name: "Spec 1".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let spec2 = Spec {
            id: "spec-2".to_string(),
            name: "Spec 2".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Design,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let mut spec1_v2 = spec1.clone();
        spec1_v2.version = "1.1.0".to_string();

        let mut spec2_v2 = spec2.clone();
        spec2_v2.version = "2.0.0".to_string();

        tracker.record_change("spec-1", &spec1, &spec1_v2, None, "Update 1".to_string());
        tracker.record_change("spec-2", &spec2, &spec2_v2, None, "Update 2".to_string());

        assert_eq!(tracker.get_history("spec-1").len(), 1);
        assert_eq!(tracker.get_history("spec-2").len(), 1);
        assert_eq!(tracker.get_all_changes().len(), 2);
    }

    #[test]
    fn test_change_detail_with_no_changes() {
        let tracker = ChangeTracker::new();
        let now = Utc::now();

        let spec = Spec {
            id: "spec-1".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let change = tracker.record_change(
            "spec-1",
            &spec,
            &spec,
            None,
            "No actual changes".to_string(),
        );

        assert_eq!(change.changes.len(), 0);
    }

    #[test]
    fn test_change_timestamps_are_recent() {
        let tracker = ChangeTracker::new();
        let now = Utc::now();

        let spec = Spec {
            id: "spec-1".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let mut new_spec = spec.clone();
        new_spec.version = "1.1.0".to_string();

        let change = tracker.record_change("spec-1", &spec, &new_spec, None, "Update".to_string());

        // Timestamp should be very recent (within last second)
        let time_diff = Utc::now().signed_duration_since(change.timestamp);
        assert!(time_diff.num_seconds() < 1);
    }

    #[test]
    fn test_clear_history() {
        let tracker = ChangeTracker::new();
        let now = Utc::now();

        let spec = Spec {
            id: "spec-1".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let mut new_spec = spec.clone();
        new_spec.version = "1.1.0".to_string();

        tracker.record_change("spec-1", &spec, &new_spec, None, "Update".to_string());
        assert_eq!(tracker.get_history("spec-1").len(), 1);

        tracker.clear_history("spec-1");
        assert_eq!(tracker.get_history("spec-1").len(), 0);
    }

    #[test]
    fn test_change_tracker_clone() {
        let tracker = ChangeTracker::new();
        let now = Utc::now();

        let spec = Spec {
            id: "spec-1".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: now,
                updated_at: now,
                phase: SpecPhase::Requirements,
                status: SpecStatus::Draft,
            },
            inheritance: None,
        };

        let mut new_spec = spec.clone();
        new_spec.version = "1.1.0".to_string();

        tracker.record_change("spec-1", &spec, &new_spec, None, "Update".to_string());

        let cloned_tracker = tracker.clone();
        assert_eq!(cloned_tracker.get_history("spec-1").len(), 1);
    }
}
