//! Dependency injection support for ricecoder-security

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{EncryptionService, ValidationService};

inventory::submit! {
    ServiceFactory::new("security", create_security_services)
}

fn create_security_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<EncryptionService>(Arc::new(EncryptionService::new())),
        ServiceEntry::new::<ValidationService>(Arc::new(ValidationService::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_security_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"security"), "Factory should be registered");
    }
}
