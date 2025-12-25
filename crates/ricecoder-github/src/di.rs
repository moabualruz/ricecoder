//! Dependency injection support for ricecoder-github

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
    vec![
        ServiceEntry::new::<BranchManager>(Arc::new(BranchManager::new())),
        ServiceEntry::new::<IssueManager>(Arc::new(IssueManager::new())),
        ServiceEntry::new::<PrManager>(Arc::new(PrManager::new())),
        ServiceEntry::new::<GistManager>(Arc::new(GistManager::new())),
        ServiceEntry::new::<DiscussionManager>(Arc::new(DiscussionManager::new())),
        ServiceEntry::new::<ReleaseManager>(Arc::new(ReleaseManager::new())),
        ServiceEntry::new::<RepositoryAnalyzer>(Arc::new(RepositoryAnalyzer::new())),
        ServiceEntry::new::<ProjectManager>(Arc::new(ProjectManager::new())),
        ServiceEntry::new::<WebhookHandler>(Arc::new(WebhookHandler::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories();

    #[test]
    fn test_github_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"github"), "Factory should be registered");
    }
}
