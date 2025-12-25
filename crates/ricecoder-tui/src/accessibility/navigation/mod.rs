//! Navigation module for accessibility

pub mod keyboard;
pub mod semantic;

pub use keyboard::{EnhancedKeyboardNavigation, KeyboardNavigationManager};
pub use semantic::{Heading, Landmark, NavigationDirection, NavigationPosition, SemanticNavigator};
