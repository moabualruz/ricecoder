//! Dependency injection support for ricecoder-permissions

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::GlobMatcher;

inventory::submit! {
    ServiceFactory::new("permissions", create_permissions_services)
}

fn create_permissions_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<GlobMatcher>(Arc::new(GlobMatcher::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_permissions_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"permissions"), "Factory should be registered");
    }
}
