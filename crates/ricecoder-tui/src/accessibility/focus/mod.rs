//! Focus management module
//!
//! Provides focus tracking, visual indicators, and keyboard navigation support
//! for accessible user interfaces.

mod indicator;
mod manager;

pub use indicator::FocusIndicatorStyle;
pub use manager::FocusManager;
