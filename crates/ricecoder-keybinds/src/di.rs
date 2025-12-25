//! Dependency injection support for ricecoder-keybinds

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{KeybindRegistry, ParserRegistry, ProfileManager, KeybindEngine};

inventory::submit! {
    ServiceFactory::new("keybinds", create_keybinds_services)
}

fn create_keybinds_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<KeybindRegistry>(Arc::new(KeybindRegistry::new())),
        ServiceEntry::new::<ParserRegistry>(Arc::new(ParserRegistry::new())),
        ServiceEntry::new::<ProfileManager>(Arc::new(ProfileManager::new())),
        ServiceEntry::new::<KeybindEngine>(Arc::new(KeybindEngine::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_keybinds_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"keybinds"), "Factory should be registered");
    }
}
