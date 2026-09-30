//! Dependency injection support for ricecoder-undo-redo

use crate::{ChangeTracker, CheckpointManager, HistoryManager};
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use std::sync::Arc;

inventory::submit! {
    ServiceFactory::new("undo-redo", create_undo_redo_services)
}

fn create_undo_redo_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<HistoryManager>(Arc::new(HistoryManager::new())),
        ServiceEntry::new::<CheckpointManager>(Arc::new(CheckpointManager::new())),
        ServiceEntry::new::<ChangeTracker>(Arc::new(ChangeTracker::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_undo_redo_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"undo-redo"),
            "Factory should be registered"
        );
    }
}
