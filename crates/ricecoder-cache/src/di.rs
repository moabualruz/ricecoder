//! Dependency injection support for ricecoder-cache

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{CacheBuilder, storage::MemoryStorage};

inventory::submit! {
    ServiceFactory::new("cache", create_cache_services)
}

fn create_cache_services() -> Vec<ServiceEntry> {
    let storage = Arc::new(MemoryStorage::new());
    let cache = CacheBuilder::new()
        .primary_storage(storage)
        .build()
        .expect("Failed to build cache");
    
    vec![
        ServiceEntry::new::<crate::Cache>(Arc::new(cache)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_cache_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"cache"), "Factory should be registered");
    }
}
