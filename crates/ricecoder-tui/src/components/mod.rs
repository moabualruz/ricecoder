//! TUI component system with ISP-compliant trait hierarchy

pub mod traits;
pub mod mode_indicator;
pub mod menu;
pub mod list;
pub mod dialog;
pub mod split_view;
pub mod tabs;
pub mod vim;
pub mod messaging;
pub mod input_area;
pub mod tool_output;

use std::collections::HashMap;

// ============================================================================
// Core Component Types
// ============================================================================

/// Unique identifier for components (type alias for backward compatibility)
pub type ComponentId = String;

/// Focus navigation direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Next,
    Previous,
    Up,
    Down,
    Left,
    Right,
    First,
    Last,
    Forward,
    Backward,
}

/// Result of a focus operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusResult {
    Focused,
    NotFocusable,
    AlreadyFocused,
    Delegated(ComponentId),
    Handled,
    Boundary,
}

/// Component lifecycle events
#[derive(Debug, Clone)]
pub enum ComponentEvent {
    Created(ComponentId),
    Mounted(ComponentId),
    Updated(ComponentId),
    Unmounted(ComponentId),
    Destroyed(ComponentId),
    FocusGained(ComponentId),
    FocusLost(ComponentId),
}

/// Event propagation phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPhase {
    Capture,
    Target,
    Bubble,
}

/// Event propagation control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPropagation {
    Continue,
    Stop,
    StopImmediate,
}

/// Result of event handling
#[derive(Debug, Clone)]
pub enum EventResult {
    Handled,
    NotHandled,
    Propagate,
    Error(String),
}

/// Custom event for component communication
#[derive(Debug, Clone)]
pub struct CustomEvent {
    pub name: String,
    pub data: Option<String>,
    pub source: Option<ComponentId>,
    pub target: Option<ComponentId>,
}

impl CustomEvent {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data: None,
            source: None,
            target: None,
        }
    }

    pub fn with_data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }

    pub fn with_source(mut self, source: ComponentId) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_target(mut self, target: ComponentId) -> Self {
        self.target = Some(target);
        self
    }
}

/// Focus event details
#[derive(Debug, Clone)]
pub struct FocusEvent {
    pub direction: FocusDirection,
    pub from: Option<ComponentId>,
    pub to: Option<ComponentId>,
}

/// State change event
#[derive(Debug, Clone)]
pub struct StateChangeEvent {
    pub component_id: ComponentId,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// Input event types
#[derive(Debug, Clone)]
pub enum InputEvent {
    Key(KeyboardEvent),
    Mouse(MouseEvent),
    Focus(FocusEvent),
    Custom(CustomEvent),
}

/// Keyboard event
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub key: crossterm::event::KeyCode,
    pub modifiers: crossterm::event::KeyModifiers,
    pub kind: crossterm::event::KeyEventKind,
}

/// Mouse event
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub x: u16,
    pub y: u16,
    pub button: Option<crossterm::event::MouseButton>,
    pub kind: crossterm::event::MouseEventKind,
    pub modifiers: crossterm::event::KeyModifiers,
}

/// Event context for handling
#[derive(Debug, Clone)]
pub struct EventContext {
    pub phase: EventPhase,
    pub propagation: EventPropagation,
    pub target: Option<ComponentId>,
    pub current_target: Option<ComponentId>,
}

impl Default for EventContext {
    fn default() -> Self {
        Self {
            phase: EventPhase::Target,
            propagation: EventPropagation::Continue,
            target: None,
            current_target: None,
        }
    }
}

/// Trait for components that handle events
pub trait EventComponent {
    fn handle_event(&mut self, event: &InputEvent, ctx: &mut EventContext) -> EventResult;
    fn accepts_event(&self, event: &InputEvent) -> bool;
}

/// Component registry for managing component instances
pub struct ComponentRegistry {
    components: HashMap<ComponentId, Box<dyn std::any::Any + Send + Sync>>,
    focus_order: Vec<ComponentId>,
    current_focus: Option<ComponentId>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            focus_order: Vec::new(),
            current_focus: None,
        }
    }

    pub fn register<T: 'static + Send + Sync>(&mut self, id: ComponentId, component: T) {
        self.components.insert(id.clone(), Box::new(component));
        self.focus_order.push(id);
    }

    pub fn unregister(&mut self, id: &ComponentId) -> bool {
        self.focus_order.retain(|i| i != id);
        if self.current_focus.as_ref() == Some(id) {
            self.current_focus = None;
        }
        self.components.remove(id).is_some()
    }

    pub fn get<T: 'static>(&self, id: &ComponentId) -> Option<&T> {
        self.components.get(id)?.downcast_ref()
    }

    pub fn get_mut<T: 'static>(&mut self, id: &ComponentId) -> Option<&mut T> {
        self.components.get_mut(id)?.downcast_mut()
    }

    pub fn focus_next(&mut self) -> Option<&ComponentId> {
        if self.focus_order.is_empty() {
            return None;
        }
        let current_idx = self.current_focus.as_ref()
            .and_then(|id| self.focus_order.iter().position(|i| i == id))
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.focus_order.len();
        self.current_focus = Some(self.focus_order[next_idx].clone());
        self.current_focus.as_ref()
    }

    pub fn focus_previous(&mut self) -> Option<&ComponentId> {
        if self.focus_order.is_empty() {
            return None;
        }
        let current_idx = self.current_focus.as_ref()
            .and_then(|id| self.focus_order.iter().position(|i| i == id))
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            self.focus_order.len() - 1
        } else {
            current_idx - 1
        };
        self.current_focus = Some(self.focus_order[prev_idx].clone());
        self.current_focus.as_ref()
    }

    pub fn current_focus(&self) -> Option<&ComponentId> {
        self.current_focus.as_ref()
    }

    pub fn set_focus(&mut self, id: ComponentId) -> bool {
        if self.components.contains_key(&id) {
            self.current_focus = Some(id);
            true
        } else {
            false
        }
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Re-exports
// ============================================================================

pub use traits::*;
pub use mode_indicator::ModeIndicator;
pub use menu::{MenuItem, MenuWidget, ModeSelectionMenu};
pub use list::ListWidget;
pub use dialog::{DialogWidget, DialogType, DialogResult};
pub use split_view::{SplitViewWidget, SplitDirection};
pub use tabs::TabWidget;
pub use vim::{VimKeybindings, VimMode};
pub use input_area::InputArea;
pub use tool_output::{ToolOutput, ToolResult};
