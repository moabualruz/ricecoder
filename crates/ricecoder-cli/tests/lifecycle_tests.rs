//! Unit tests for CLI lifecycle management

use ricecoder_cli::lifecycle::*;
use std::sync::atomic::{AtomicBool, Ordering};

struct TestComponent {
    name: &'static str,
    initialized: AtomicBool,
    started: AtomicBool,
    stopped: AtomicBool,
}

impl TestComponent {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            initialized: AtomicBool::new(false),
            started: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
        }
    }
}

#[async_trait::async_trait]
impl LifecycleComponent for TestComponent {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.initialized.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.started.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.stopped.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn test_lifecycle_manager() {
    let manager = LifecycleManager::new();

    let component1 = TestComponent::new("component1");
    let component2 = TestComponent::new("component2");

    manager.register_component(component1, vec![]).unwrap();
    manager.register_component(component2, vec!["component1".to_string()]).unwrap();

    // Initialize
    manager.initialize_all().await.unwrap();

    // Check states
    assert_eq!(manager.get_component_state("component1"), Some(LifecycleState::Ready));
    assert_eq!(manager.get_component_state("component2"), Some(LifecycleState::Ready));

    // Start
    manager.start_all().await.unwrap();

    // Stop
    manager.stop_all().await.unwrap();

    // Check final states
    assert_eq!(manager.get_component_state("component1"), Some(LifecycleState::ShutDown));
    assert_eq!(manager.get_component_state("component2"), Some(LifecycleState::ShutDown));
}