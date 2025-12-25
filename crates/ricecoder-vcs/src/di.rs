//! Dependency injection support for ricecoder-vcs

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::VcsIntegration;

inventory::submit! {
    ServiceFactory::new("vcs", create_vcs_services)
}

fn create_vcs_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<VcsIntegration>(Arc::new(VcsIntegration::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_vcs_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"vcs"), "Factory should be registered");
    }
}
