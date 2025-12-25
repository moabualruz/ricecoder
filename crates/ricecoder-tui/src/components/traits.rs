//! Component traits for TUI
//!
//! Provides both the monolithic Component trait for backward compatibility
//! and segregated traits following ISP for new implementations.

use ratatui::{layout::Rect, Frame};
use crate::model::{AppMessage, AppModel};
use super::{FocusDirection, FocusResult};

// ComponentId is a type alias for String - import from parent
pub use super::ComponentId;

// ============================================================================
// Main Component Trait (backward compatible)
// ============================================================================

/// Core Component trait - all TUI components implement this
/// 
/// This is a monolithic trait for backward compatibility.
/// New components can use the segregated traits (Renderable, Interactive, etc.)
/// and get Component through the blanket implementation.
#[deprecated(
    since = "0.1.8",
    note = "Use segregated traits (Renderable, Interactive, Focusable, etc.) instead. \
            Types implementing all segregated traits automatically implement Component via blanket impl."
)]
pub trait Component: Send + Sync {
    // ========== Core (Required) ==========
    
    /// Get the unique identifier for this component
    fn id(&self) -> ComponentId;
    
    /// Render the component to the given frame and area
    fn render(&self, frame: &mut Frame, area: Rect, model: &AppModel);
    
    // ========== Bounds ==========
    
    /// Get the component's bounding rectangle
    fn bounds(&self) -> Rect {
        Rect::default()
    }
    
    /// Set the component's bounding rectangle
    fn set_bounds(&mut self, _bounds: Rect) {}
    
    // ========== Input Handling ==========
    
    /// Handle an incoming message
    /// Returns true if the component handled the message
    fn update(&mut self, _message: &AppMessage, _model: &AppModel) -> bool {
        false
    }
    
    /// Handle focus navigation
    fn handle_focus(&mut self, _direction: FocusDirection) -> FocusResult {
        FocusResult::NotFocusable
    }
    
    // ========== Focus State ==========
    
    /// Get the component's focus state
    fn is_focused(&self) -> bool {
        false
    }
    
    /// Set the component's focus state
    fn set_focused(&mut self, _focused: bool) {}
    
    /// Check if the component can receive focus
    fn can_focus(&self) -> bool {
        false
    }
    
    /// Get the component's tab order index
    fn tab_order(&self) -> Option<usize> {
        None
    }
    
    /// Set the component's tab order index
    fn set_tab_order(&mut self, _order: Option<usize>) {}
    
    // ========== Visibility ==========
    
    /// Get the component's visibility state
    fn is_visible(&self) -> bool {
        true
    }
    
    /// Set the component's visibility state
    fn set_visible(&mut self, _visible: bool) {}
    
    /// Get the component's enabled state
    fn is_enabled(&self) -> bool {
        true
    }
    
    /// Set the component's enabled state
    fn set_enabled(&mut self, _enabled: bool) {}
    
    // ========== Layering ==========
    
    /// Get the component's z-index (for layering)
    fn z_index(&self) -> i32 {
        0
    }
    
    /// Set the component's z-index
    fn set_z_index(&mut self, _z_index: i32) {}
    
    // ========== Composite (Children) ==========
    
    /// Get child components (for composite components)
    fn children(&self) -> Vec<&dyn Component> {
        Vec::new()
    }
    
    /// Get child components mutably (for composite components)
    fn children_mut(&mut self) -> Vec<&mut dyn Component> {
        Vec::new()
    }
    
    /// Find a child component by ID
    fn find_child(&self, _id: &ComponentId) -> Option<&dyn Component> {
        None
    }
    
    /// Find a child component by ID mutably
    fn find_child_mut(&mut self, _id: &ComponentId) -> Option<&mut dyn Component> {
        None
    }
    
    /// Add a child component
    fn add_child(&mut self, _child: Box<dyn Component>) {}
    
    /// Remove a child component
    fn remove_child(&mut self, _id: &ComponentId) -> Option<Box<dyn Component>> {
        None
    }
    
    // ========== Validation ==========
    
    /// Validate the component's current state
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
    
    // ========== Cloning ==========
    
    /// Clone the component into a Box
    fn clone_box(&self) -> Box<dyn Component>;
}

// ============================================================================
// Segregated Traits (ISP - Interface Segregation Principle)
// For new code that wants fine-grained control
// ============================================================================

/// Core rendering capability - every component must render
pub trait Renderable {
    fn id(&self) -> ComponentId;
    fn render(&self, frame: &mut Frame, area: Rect, model: &AppModel);
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, bounds: Rect);
}

/// Interactive capability - components that handle user input
pub trait Interactive {
    fn update(&mut self, message: &AppMessage, model: &AppModel) -> bool;
    fn handle_focus(&mut self, direction: FocusDirection) -> FocusResult;
}

/// Focusable capability - components that can receive focus
pub trait Focusable {
    fn is_focused(&self) -> bool;
    fn set_focused(&mut self, focused: bool);
    fn can_focus(&self) -> bool;
    fn tab_order(&self) -> Option<usize>;
    fn set_tab_order(&mut self, order: Option<usize>);
}

/// Visibility capability - components that can be shown/hidden
pub trait Visible {
    fn is_visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
}

/// Layering capability - components with z-index/depth
pub trait Layered {
    fn z_index(&self) -> i32;
    fn set_z_index(&mut self, z_index: i32);
}

/// Composite capability - components that can contain children
pub trait Composite {
    fn children(&self) -> Vec<&dyn Component>;
    fn children_mut(&mut self) -> Vec<&mut dyn Component>;
    fn find_child(&self, id: &ComponentId) -> Option<&dyn Component>;
    fn find_child_mut(&mut self, id: &ComponentId) -> Option<&mut dyn Component>;
    fn add_child(&mut self, child: Box<dyn Component>);
    fn remove_child(&mut self, id: &ComponentId) -> Option<Box<dyn Component>>;
}

/// Validatable capability - components that can validate their state
pub trait Validatable {
    fn validate(&self) -> Result<(), String>;
}

// ============================================================================
// Blanket Implementation (Backward Compatibility Bridge)
// ============================================================================

/// Blanket implementation for backward compatibility
/// 
/// Any type implementing all segregated traits automatically implements Component.
/// This allows new components to use the fine-grained ISP-compliant traits while
/// maintaining compatibility with code expecting the monolithic Component trait.
impl<T> Component for T
where
    T: Renderable + Interactive + Focusable + Visible + Layered + Composite + Validatable + Clone + Send + Sync + 'static,
{
    // ========== Core (Required) ==========
    
    fn id(&self) -> ComponentId {
        Renderable::id(self)
    }
    
    fn render(&self, frame: &mut Frame, area: Rect, model: &AppModel) {
        Renderable::render(self, frame, area, model)
    }
    
    // ========== Bounds ==========
    
    fn bounds(&self) -> Rect {
        Renderable::bounds(self)
    }
    
    fn set_bounds(&mut self, bounds: Rect) {
        Renderable::set_bounds(self, bounds)
    }
    
    // ========== Input Handling ==========
    
    fn update(&mut self, message: &AppMessage, model: &AppModel) -> bool {
        Interactive::update(self, message, model)
    }
    
    fn handle_focus(&mut self, direction: FocusDirection) -> FocusResult {
        Interactive::handle_focus(self, direction)
    }
    
    // ========== Focus State ==========
    
    fn is_focused(&self) -> bool {
        Focusable::is_focused(self)
    }
    
    fn set_focused(&mut self, focused: bool) {
        Focusable::set_focused(self, focused)
    }
    
    fn can_focus(&self) -> bool {
        Focusable::can_focus(self)
    }
    
    fn tab_order(&self) -> Option<usize> {
        Focusable::tab_order(self)
    }
    
    fn set_tab_order(&mut self, order: Option<usize>) {
        Focusable::set_tab_order(self, order)
    }
    
    // ========== Visibility ==========
    
    fn is_visible(&self) -> bool {
        Visible::is_visible(self)
    }
    
    fn set_visible(&mut self, visible: bool) {
        Visible::set_visible(self, visible)
    }
    
    fn is_enabled(&self) -> bool {
        Visible::is_enabled(self)
    }
    
    fn set_enabled(&mut self, enabled: bool) {
        Visible::set_enabled(self, enabled)
    }
    
    // ========== Layering ==========
    
    fn z_index(&self) -> i32 {
        Layered::z_index(self)
    }
    
    fn set_z_index(&mut self, z_index: i32) {
        Layered::set_z_index(self, z_index)
    }
    
    // ========== Composite (Children) ==========
    
    fn children(&self) -> Vec<&dyn Component> {
        Composite::children(self)
    }
    
    fn children_mut(&mut self) -> Vec<&mut dyn Component> {
        Composite::children_mut(self)
    }
    
    fn find_child(&self, id: &ComponentId) -> Option<&dyn Component> {
        Composite::find_child(self, id)
    }
    
    fn find_child_mut(&mut self, id: &ComponentId) -> Option<&mut dyn Component> {
        Composite::find_child_mut(self, id)
    }
    
    fn add_child(&mut self, child: Box<dyn Component>) {
        Composite::add_child(self, child)
    }
    
    fn remove_child(&mut self, id: &ComponentId) -> Option<Box<dyn Component>> {
        Composite::remove_child(self, id)
    }
    
    // ========== Validation ==========
    
    fn validate(&self) -> Result<(), String> {
        Validatable::validate(self)
    }
    
    // ========== Cloning ==========
    
    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}
