//! Dependency injection support for ricecoder-github
//!
//! Note: GitHub managers require runtime configuration (token, owner, repo)
//! so they are instantiated on-demand rather than via static DI registration.
//! This module is kept for future factory-based initialization patterns.

// TODO: Implement factory-based DI when ServiceFactory supports async config loading
// All GitHub managers require: token, owner, repo - which are runtime values

/*
use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::managers::{
    BranchManager,
    IssueManager,
    PrManager,
    GistManager,
    DiscussionManager,
    ReleaseManager,
    RepositoryAnalyzer,
    ProjectManager,
    WebhookHandler,
};

inventory::submit! {
    ServiceFactory::new("github", create_github_services)
}

fn create_github_services() -> Vec<ServiceEntry> {
    // Requires runtime config - cannot be statically initialized
    vec![]
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_github_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"github"), "Factory should be registered");
    }
}
