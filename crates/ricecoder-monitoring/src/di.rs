//! Dependency injection support for ricecoder-monitoring

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{
    dashboards::DashboardManager,
    reporting::{ReportGenerator, ReportScheduler},
    performance::PerformanceProfiler,
};

inventory::submit! {
    ServiceFactory::new("monitoring", create_monitoring_services)
}

fn create_monitoring_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<DashboardManager>(Arc::new(DashboardManager::new())),
        ServiceEntry::new::<ReportGenerator>(Arc::new(ReportGenerator::new())),
        ServiceEntry::new::<ReportScheduler>(Arc::new(ReportScheduler::new())),
        ServiceEntry::new::<PerformanceProfiler>(Arc::new(PerformanceProfiler::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_monitoring_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"monitoring"), "Factory should be registered");
    }
}
