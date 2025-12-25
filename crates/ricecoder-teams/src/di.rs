//! Dependency injection support for ricecoder-teams

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{AnalyticsDashboard, TeamConfigManager, SyncService};

inventory::submit! {
    ServiceFactory::new("teams", create_teams_services)
}

fn create_teams_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<AnalyticsDashboard>(Arc::new(AnalyticsDashboard::new())),
        ServiceEntry::new::<TeamConfigManager>(Arc::new(TeamConfigManager::new())),
        ServiceEntry::new::<SyncService>(Arc::new(SyncService::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_teams_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"teams"), "Factory should be registered");
    }
}
