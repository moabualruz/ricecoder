//! Unit tests for TUI lifecycle management

use ricecoder_tui::lifecycle::*;
use std::sync::atomic::{AtomicBool, Ordering};

struct TestTuiComponent {
    name: &'static str,
    initialized: AtomicBool,
    started: AtomicBool,
    stopped: AtomicBool,
}

impl TestTuiComponent {
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
impl TuiLifecycleComponent for TestTuiComponent {
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
async fn test_tui_lifecycle_manager() {
    let manager = TuiLifecycleManager::new();

    let component1 = TestTuiComponent::new("tui_component1");
    let component2 = TestTuiComponent::new("tui_component2");

    manager.register_component(component1, vec![]).unwrap();
    manager.register_component(component2, vec!["tui_component1".to_string()]).unwrap();

    // Initialize
    manager.initialize_all().await.unwrap();

    // Check states
    assert_eq!(manager.get_component_state("tui_component1"), Some(TuiLifecycleState::Ready));
    assert_eq!(manager.get_component_state("tui_component2"), Some(TuiLifecycleState::Ready));

    // Start
    manager.start_all().await.unwrap();

    // Stop
    manager.stop_all().await.unwrap();

    // Check final states
    assert_eq!(manager.get_component_state("tui_component1"), Some(TuiLifecycleState::ShutDown));
    assert_eq!(manager.get_component_state("tui_component2"), Some(TuiLifecycleState::ShutDown));
}