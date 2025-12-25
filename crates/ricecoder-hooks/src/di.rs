//! Dependency injection support for ricecoder-hooks

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::registry::InMemoryHookRegistry;

inventory::submit! {
    ServiceFactory::new("hooks", create_hooks_services)
}

fn create_hooks_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<InMemoryHookRegistry>(Arc::new(InMemoryHookRegistry::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_hooks_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"hooks"), "Factory should be registered");
    }
}
