//! Dependency injection support for ricecoder-activity-log

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{
    ActivityLogger, AuditLogger, PerformanceMonitor, MetricsCollector, SessionTracker,
};

inventory::submit! {
    ServiceFactory::new("activity-log", create_activity_log_services)
}

fn create_activity_log_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<ActivityLogger>(Arc::new(ActivityLogger::new())),
        ServiceEntry::new::<AuditLogger>(Arc::new(AuditLogger::default())),
        ServiceEntry::new::<PerformanceMonitor>(Arc::new(PerformanceMonitor::new())),
        ServiceEntry::new::<MetricsCollector>(Arc::new(MetricsCollector::new())),
        ServiceEntry::new::<SessionTracker>(Arc::new(SessionTracker::default())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_activity_log_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"activity-log"), "Factory should be registered");
    }
}
