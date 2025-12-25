//! Dependency injection support for ricecoder-config

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{ConfigManager, TuiConfig};

inventory::submit! {
    ServiceFactory::new("config", create_config_services)
}

fn create_config_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<ConfigManager>(Arc::new(ConfigManager::new())),
        ServiceEntry::new::<TuiConfig>(Arc::new(TuiConfig::default())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_config_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"config"), "Factory should be registered");
    }
}
