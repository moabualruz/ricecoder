//! Dependency injection support for ricecoder-local-models

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::LocalModelManager;

inventory::submit! {
    ServiceFactory::new("local-models", create_local_models_services)
}

fn create_local_models_services() -> Vec<ServiceEntry> {
    vec![
        // LocalModelManager requires endpoint configuration, so we create it with default endpoint
        ServiceEntry::new::<LocalModelManager>(Arc::new(
            LocalModelManager::with_default_endpoint()
                .expect("Failed to create LocalModelManager with default endpoint")
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_local_models_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"local-models"), "Factory should be registered");
    }
}
