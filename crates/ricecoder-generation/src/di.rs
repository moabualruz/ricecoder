//! Dependency injection support for ricecoder-generation

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{
    TemplateEngine,
    BoilerplateManager,
    CodeValidator,
    ConflictDetector,
};

inventory::submit! {
    ServiceFactory::new("generation", create_generation_services)
}

fn create_generation_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<TemplateEngine>(Arc::new(TemplateEngine::new())),
        ServiceEntry::new::<BoilerplateManager>(Arc::new(BoilerplateManager::new())),
        ServiceEntry::new::<CodeValidator>(Arc::new(CodeValidator::new())),
        ServiceEntry::new::<ConflictDetector>(Arc::new(ConflictDetector::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_generation_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"generation"), "Factory should be registered");
    }
}
