//! Dependency injection support for ricecoder-parsers

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::parser::Parser;

inventory::submit! {
    ServiceFactory::new("parsers", create_parsers_services)
}

fn create_parsers_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<Parser>(Arc::new(Parser::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_parsers_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"parsers"), "Factory should be registered");
    }
}
