//! Unit tests for TUI DI integration

use ricecoder_tui::di;

#[test]
fn test_di_initialization() {
    di::reset_di_container();
    assert!(!di::is_di_initialized());

    di::initialize_di_container().unwrap();
    assert!(di::is_di_initialized());

    let container = di::get_di_container();
    assert!(container.is_some());
}