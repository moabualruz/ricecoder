//! Interactive UI components and Component Architecture for TEA

use crate::model::{AppMessage, AppMode, AppModel};
use ratatui::layout::Rect;
use ratatui::Frame;
use std::any::Any;

/// Unique identifier for components
pub type ComponentId = String;

/// Component messaging system for inter-component communication
pub mod messaging {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Message filter for component subscriptions
    pub enum MessageFilter {
        /// Accept all messages
        All,
        /// Accept messages of specific types
        MessageTypes(Vec<String>),
        /// Accept messages from specific sources
        Sources(Vec<ComponentId>),
        /// Custom filter function
        Custom(Box<dyn Fn(&AppMessage, &ComponentId) -> bool + Send + Sync>),
    }

    impl MessageFilter {
        /// Check if a message passes this filter
        pub fn matches(&self, message: &AppMessage, source: &ComponentId) -> bool {
            match self {
                MessageFilter::All => true,
                MessageFilter::MessageTypes(types) => {
                    let message_type = self::get_message_type(message);
                    types.contains(&message_type)
                }
                MessageFilter::Sources(sources) => sources.contains(source),
                MessageFilter::Custom(filter) => filter(message, source),
            }
        }
    }

    /// Get a string representation of the message type for filtering
    fn get_message_type(message: &AppMessage) -> String {
        match message {
            AppMessage::KeyPress(_) => "KeyPress".to_string(),
            AppMessage::MouseEvent(_) => "MouseEvent".to_string(),
            AppMessage::Resize { .. } => "Resize".to_string(),
            AppMessage::Scroll { .. } => "Scroll".to_string(),
            AppMessage::ModeChanged(_) => "ModeChanged".to_string(),
            AppMessage::ThemeChanged(_) => "ThemeChanged".to_string(),
            AppMessage::FocusChanged(_) => "FocusChanged".to_string(),
            AppMessage::CommandPaletteToggled => "CommandPaletteToggled".to_string(),
            AppMessage::SessionCreated(_) => "SessionCreated".to_string(),
            AppMessage::SessionActivated(_) => "SessionActivated".to_string(),
            AppMessage::SessionClosed(_) => "SessionClosed".to_string(),
            AppMessage::TokensUpdated(_) => "TokensUpdated".to_string(),
            AppMessage::FileChanged(_) => "FileChanged".to_string(),
            AppMessage::FilePickerOpened => "FilePickerOpened".to_string(),
            AppMessage::FilePickerClosed => "FilePickerClosed".to_string(),
            AppMessage::CommandExecuted(_) => "CommandExecuted".to_string(),
            AppMessage::CommandCompleted(_) => "CommandCompleted".to_string(),
            AppMessage::OperationStarted(_) => "OperationStarted".to_string(),
            AppMessage::OperationCompleted(_) => "OperationCompleted".to_string(),
            AppMessage::OperationFailed(_, _) => "OperationFailed".to_string(),
            AppMessage::SendMessage(_) => "SendMessage".to_string(),
            AppMessage::FileSelected(_) => "FileSelected".to_string(),
            AppMessage::ComponentMessage { .. } => "ComponentMessage".to_string(),
            AppMessage::Tick => "Tick".to_string(),
            AppMessage::ExitRequested => "ExitRequested".to_string(),
            AppMessage::McpServerAdded(_) => "McpServerAdded".to_string(),
            AppMessage::McpServerRemoved(_) => "McpServerRemoved".to_string(),
            AppMessage::McpToolExecuted { .. } => "McpToolExecuted".to_string(),
            AppMessage::McpToolExecutionFailed { .. } => "McpToolExecutionFailed".to_string(),
            AppMessage::ProviderSwitched(_) => "ProviderSwitched".to_string(),
            AppMessage::ProviderStatusUpdated { .. } => "ProviderStatusUpdated".to_string(),
            AppMessage::ProviderMetricsUpdated { .. } => "ProviderMetricsUpdated".to_string(),
            AppMessage::ProviderSelected(_) => "ProviderSelected".to_string(),
            AppMessage::ProviderViewModeChanged(_) => "ProviderViewModeChanged".to_string(),
            AppMessage::ProviderFilterChanged(_) => "ProviderFilterChanged".to_string(),
        }
    }

    /// Component subscription for receiving messages from other components
    pub struct ComponentSubscription {
        pub subscriber_id: ComponentId,
        pub filter: MessageFilter,
        pub priority: SubscriptionPriority,
    }

    /// Priority levels for component subscriptions
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum SubscriptionPriority {
        Low = 0,
        Normal = 1,
        High = 2,
    }

    /// Component message bus for inter-component communication
    #[derive(Clone)]
    pub struct ComponentMessageBus {
        subscriptions: Arc<RwLock<HashMap<ComponentId, Vec<ComponentSubscription>>>>,
        message_queue: Arc<RwLock<Vec<QueuedMessage>>>,
    }

    /// Queued message for delivery
    #[derive(Clone)]
    pub struct QueuedMessage {
        pub message: AppMessage,
        pub source: ComponentId,
        pub timestamp: std::time::Instant,
        pub priority: SubscriptionPriority,
    }

    impl ComponentMessageBus {
        /// Create a new component message bus
        pub fn new() -> Self {
            Self {
                subscriptions: Arc::new(RwLock::new(HashMap::new())),
                message_queue: Arc::new(RwLock::new(Vec::new())),
            }
        }

        /// Subscribe a component to messages from other components
        pub async fn subscribe(&self, subscription: ComponentSubscription) {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions
                .entry(subscription.subscriber_id.clone())
                .or_insert_with(Vec::new)
                .push(subscription);
        }

        /// Unsubscribe a component from all messages
        pub async fn unsubscribe(&self, component_id: &ComponentId) {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.remove(component_id);
        }

        /// Unsubscribe from specific message types
        pub async fn unsubscribe_from(&self, component_id: &ComponentId, filter: &MessageFilter) {
            let mut subscriptions = self.subscriptions.write().await;
            if let Some(subs) = subscriptions.get_mut(component_id) {
                subs.retain(|sub| {
                    std::mem::discriminant(&sub.filter) != std::mem::discriminant(filter)
                });
            }
        }

        /// Send a message from a component to subscribed components
        pub async fn send_message(&self, message: AppMessage, source: ComponentId) {
            let subscriptions = self.subscriptions.read().await;
            let mut queued_messages = Vec::new();

            // Find all subscribers that match the message
            for (subscriber_id, subs) in subscriptions.iter() {
                for subscription in subs {
                    if subscription.filter.matches(&message, &source) {
                        queued_messages.push(QueuedMessage {
                            message: message.clone(),
                            source: source.clone(),
                            timestamp: std::time::Instant::now(),
                            priority: subscription.priority,
                        });
                        break; // Only queue once per subscriber
                    }
                }
            }

            // Sort by priority (higher priority first)
            queued_messages.sort_by(|a, b| b.priority.cmp(&a.priority));

            // Add to message queue
            let mut message_queue = self.message_queue.write().await;
            message_queue.extend(queued_messages);
        }

        /// Get pending messages for a component
        pub async fn get_messages(&self, component_id: &ComponentId) -> Vec<AppMessage> {
            let mut message_queue = self.message_queue.write().await;
            let mut result = Vec::new();

            // Filter messages for this component
            let mut remaining = Vec::new();
            for queued in message_queue.drain(..) {
                // In a real implementation, you'd check if the message is for this component
                // For now, return all messages (this is a simplified implementation)
                result.push(queued.message);
            }

            // Put back any remaining messages
            message_queue.extend(remaining);

            result
        }

        /// Clear all pending messages
        pub async fn clear_messages(&self) {
            let mut message_queue = self.message_queue.write().await;
            message_queue.clear();
        }

        /// Get subscription statistics
        pub async fn get_stats(&self) -> MessageBusStats {
            let subscriptions = self.subscriptions.read().await;
            let message_queue = self.message_queue.read().await;

            MessageBusStats {
                total_subscriptions: subscriptions.values().map(|subs| subs.len()).sum(),
                active_subscribers: subscriptions.len(),
                queued_messages: message_queue.len(),
            }
        }
    }

    /// Statistics for the message bus
    #[derive(Debug, Clone)]
    pub struct MessageBusStats {
        pub total_subscriptions: usize,
        pub active_subscribers: usize,
        pub queued_messages: usize,
    }

    /// Component message for direct component-to-component communication
    #[derive(Clone, Debug)]
    pub struct ComponentMessage {
        pub target: ComponentId,
        pub payload: ComponentMessagePayload,
    }

    /// Payload types for component messages
    #[derive(Debug)]
    pub enum ComponentMessagePayload {
        /// String data
        String(String),
        /// JSON data
        Json(serde_json::Value),
        /// Binary data
        Binary(Vec<u8>),
        /// Custom data
        Custom(Box<dyn Any + Send + Sync>),
    }

    impl Clone for ComponentMessagePayload {
        fn clone(&self) -> Self {
            match self {
                Self::String(s) => Self::String(s.clone()),
                Self::Json(j) => Self::Json(j.clone()),
                Self::Binary(b) => Self::Binary(b.clone()),
                Self::Custom(_) => Self::String("Custom payload (not cloneable)".to_string()), // Can't clone Box<dyn Any>
            }
        }
    }
}

/// Component trait defining the interface for UI components
pub trait Component {
    /// Get the unique identifier for this component
    fn id(&self) -> ComponentId;

    /// Render the component to the given frame and area
    fn render(&self, frame: &mut Frame, area: Rect, model: &AppModel);

    /// Handle an incoming message and return updated component state
    /// Returns true if the component handled the message, false otherwise
    fn update(&mut self, message: &AppMessage, model: &AppModel) -> bool;

    /// Get the component's focus state
    fn is_focused(&self) -> bool;

    /// Set the component's focus state
    fn set_focused(&mut self, focused: bool);

    /// Get the component's visibility state
    fn is_visible(&self) -> bool;

    /// Set the component's visibility state
    fn set_visible(&mut self, visible: bool);

    /// Get the component's enabled state
    fn is_enabled(&self) -> bool;

    /// Set the component's enabled state
    fn set_enabled(&mut self, enabled: bool);

    /// Get the component's bounding rectangle
    fn bounds(&self) -> Rect;

    /// Set the component's bounding rectangle
    fn set_bounds(&mut self, bounds: Rect);

    /// Handle focus events (tab, shift+tab, etc.)
    fn handle_focus(&mut self, direction: FocusDirection) -> FocusResult;

    /// Get child components (for composite components)
    fn children(&self) -> Vec<&dyn Component>;

    /// Get child components mutably (for composite components)
    fn children_mut(&mut self) -> Vec<&mut dyn Component>;

    /// Find a child component by ID
    fn find_child(&self, id: &ComponentId) -> Option<&dyn Component>;

    /// Find a child component by ID mutably
    fn find_child_mut(&mut self, id: &ComponentId) -> Option<&mut dyn Component>;

    /// Add a child component
    fn add_child(&mut self, child: Box<dyn Component>);

    /// Remove a child component
    fn remove_child(&mut self, id: &ComponentId) -> Option<Box<dyn Component>>;

    /// Get the component's z-index (for layering)
    fn z_index(&self) -> i32;

    /// Set the component's z-index
    fn set_z_index(&mut self, z_index: i32);

    /// Check if the component can receive focus
    fn can_focus(&self) -> bool;

    /// Get the component's tab order index
    fn tab_order(&self) -> Option<usize>;

    /// Set the component's tab order index
    fn set_tab_order(&mut self, order: Option<usize>);

    /// Get component messaging subscriptions
    fn subscriptions(&self) -> Vec<messaging::ComponentSubscription> {
        Vec::new() // Default implementation returns no subscriptions
    }

    /// Send a message to other components
    fn send_message(&self, _bus: &messaging::ComponentMessageBus, _message: AppMessage) {
        // Default implementation does nothing
    }

    /// Receive messages from other components
    fn receive_messages(&mut self, _messages: &[AppMessage]) -> bool {
        false // Default implementation handles no messages
    }

    /// Validate the component's current state
    fn validate(&self) -> Result<(), String> {
        Ok(()) // Default implementation assumes valid state
    }

    /// Clone the component into a Box
    fn clone_box(&self) -> Box<dyn Component>;
}

/// Focus direction for keyboard navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    /// Move focus forward (Tab)
    Forward,
    /// Move focus backward (Shift+Tab)
    Backward,
    /// Move focus to first element
    First,
    /// Move focus to last element
    Last,
    /// Move focus to next element in specific direction
    Next,
    /// Move focus to previous element in specific direction
    Previous,
}

/// Result of focus handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusResult {
    /// Focus was handled successfully
    Handled,
    /// Focus moved to another component
    Moved(ComponentId),
    /// Focus could not be moved (at boundary)
    Boundary,
    /// Focus handling failed
    Failed(String),
}

/// Component lifecycle events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentEvent {
    /// Component was created
    Created(ComponentId),
    /// Component was destroyed
    Destroyed(ComponentId),
    /// Component gained focus
    Focused(ComponentId),
    /// Component lost focus
    Unfocused(ComponentId),
    /// Component became visible
    Shown(ComponentId),
    /// Component became hidden
    Hidden(ComponentId),
    /// Component was enabled
    Enabled(ComponentId),
    /// Component was disabled
    Disabled(ComponentId),
    /// Component bounds changed
    BoundsChanged(ComponentId, Rect),
    /// Component properties changed
    PropertiesChanged(ComponentId),
    /// Keyboard input event
    Keyboard(crate::event::KeyEvent),
    /// Mouse input event
    Mouse(crate::event::MouseEvent),
}

/// Component registry for managing component instances with messaging support
pub struct ComponentRegistry {
    components: std::collections::HashMap<ComponentId, Box<dyn Component + 'static>>,
    focus_order: Vec<ComponentId>,
    current_focus: Option<ComponentId>,
    message_bus: messaging::ComponentMessageBus,
}

impl ComponentRegistry {
    /// Create a new component registry with messaging support
    pub fn new() -> Self {
        Self {
            components: std::collections::HashMap::new(),
            focus_order: Vec::new(),
            current_focus: None,
            message_bus: messaging::ComponentMessageBus::new(),
        }
    }

    /// Create a component registry with a custom message bus
    pub fn with_message_bus(message_bus: messaging::ComponentMessageBus) -> Self {
        Self {
            components: std::collections::HashMap::new(),
            focus_order: Vec::new(),
            current_focus: None,
            message_bus,
        }
    }

    /// Get the message bus for inter-component communication
    pub fn message_bus(&self) -> &messaging::ComponentMessageBus {
        &self.message_bus
    }

    /// Initialize component subscriptions
    pub async fn initialize_messaging(&self) {
        for component in self.components.values() {
            let subscriptions = component.subscriptions();
            for subscription in subscriptions {
                self.message_bus.subscribe(subscription).await;
            }
        }
    }

    /// Process pending messages for all components
    pub async fn process_messages(&mut self, model: &AppModel) {
        for (id, component) in self.components.iter_mut() {
            let messages = self.message_bus.get_messages(id).await;
            if !messages.is_empty() {
                component.receive_messages(&messages);
                // Re-process each message through the component's update method
                for message in messages {
                    component.update(&message, model);
                }
            }
        }
    }

    /// Send a message from a component to the message bus
    pub async fn send_component_message(&self, component_id: &ComponentId, message: AppMessage) {
        self.message_bus
            .send_message(message, component_id.clone())
            .await;
    }

    /// Register a component
    pub fn register(&mut self, component: Box<dyn Component>) {
        let id = component.id().clone();
        self.components.insert(id.clone(), component);
        self.focus_order.push(id);
    }

    /// Unregister a component
    pub fn unregister(&mut self, id: &str) -> Option<Box<dyn Component>> {
        if let Some(pos) = self.focus_order.iter().position(|x| x == id) {
            self.focus_order.remove(pos);
        }
        if self.current_focus.as_ref() == Some(&id.to_string()) {
            self.current_focus = None;
        }
        self.components.remove(id)
    }

    /// Get a component by ID
    pub fn get(&self, id: &str) -> Option<&dyn Component> {
        self.components.get(id).map(|c| c.as_ref())
    }

    /// Get a mutable component by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut (dyn Component + 'static)> {
        self.components.get_mut(id).map(move |c| c.as_mut())
    }

    /// Get all components
    pub fn all(&self) -> Vec<&(dyn Component + 'static)> {
        self.components.values().map(|c| c.as_ref()).collect()
    }

    /// Get all mutable components
    pub fn all_mut(&mut self) -> Vec<&mut (dyn Component + 'static)> {
        self.components.values_mut().map(|c| c.as_mut()).collect()
    }

    /// Set focus to a specific component
    pub fn set_focus(&mut self, id: Option<&str>) {
        // Unfocus current component
        if let Some(current_id) = &self.current_focus {
            if let Some(component) = self.components.get_mut(current_id) {
                component.set_focused(false);
            }
        }

        // Focus new component
        self.current_focus = id.map(|s| s.to_string());
        if let Some(new_id) = &self.current_focus {
            if let Some(component) = self.components.get_mut(new_id) {
                component.set_focused(true);
            }
        }
    }

    /// Move focus in the specified direction
    pub fn move_focus(&mut self, direction: FocusDirection) -> FocusResult {
        // Get current focus index without holding a reference to self
        let current_focus_id = self.current_focus.clone();
        let current_index = match current_focus_id {
            Some(ref id) => self.focus_order.iter().position(|x| x == id),
            None => None,
        };

        let new_index = match direction {
            FocusDirection::Forward => match current_index {
                Some(idx) => Some((idx + 1) % self.focus_order.len()),
                None => Some(0),
            },
            FocusDirection::Backward => match current_index {
                Some(idx) => Some(if idx == 0 {
                    self.focus_order.len() - 1
                } else {
                    idx - 1
                }),
                None => Some(self.focus_order.len().saturating_sub(1)),
            },
            FocusDirection::First => Some(0),
            FocusDirection::Last => Some(self.focus_order.len().saturating_sub(1)),
            _ => current_index,
        };

        if let Some(idx) = new_index {
            if idx < self.focus_order.len() {
                let new_id = self.focus_order[idx].clone();
                self.set_focus(Some(&new_id));
                return FocusResult::Moved(new_id);
            }
        }

        FocusResult::Boundary
    }

    /// Get the currently focused component
    pub fn focused(&self) -> Option<&dyn Component> {
        self.current_focus
            .as_ref()
            .and_then(|id| self.components.get(id))
            .map(|c| c.as_ref())
    }

    /// Get the currently focused component ID
    pub fn focused_id(&self) -> Option<&str> {
        self.current_focus.as_deref()
    }

    /// Update all components with a message
    pub fn update_all(&mut self, message: &AppMessage, model: &AppModel) -> Vec<ComponentId> {
        let mut updated = Vec::new();
        for (id, component) in &mut self.components {
            if component.update(message, model) {
                updated.push(id.clone());
            }
        }
        updated
    }

    /// Render all visible components
    pub fn render_all(&self, frame: &mut Frame, model: &AppModel) {
        for component in self.components.values() {
            if component.is_visible() && component.is_enabled() {
                let bounds = component.bounds();
                component.render(frame, bounds, model);
            }
        }
    }

    /// Validate all components
    pub fn validate_all(&self) -> Result<(), Vec<(ComponentId, String)>> {
        let mut errors = Vec::new();
        for (id, component) in &self.components {
            if let Err(e) = component.validate() {
                errors.push((id.clone(), e));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for cloning components
pub trait ComponentClone {
    fn clone_box(&self) -> Box<dyn Component>;
}

impl<T> ComponentClone for T
where
    T: 'static + Component + Clone,
{
    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

/// Macro to implement Component for a type
#[macro_export]
macro_rules! impl_component {
    ($type:ty) => {
        impl Component for $type {
            fn validate(&self) -> Result<(), String> {
                Ok(())
            }

            fn clone_box(&self) -> Box<dyn Component> {
                Box::new(self.clone())
            }
        }
    };
}

/// Typed event system for component communication
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Mouse events
    Mouse(MouseEvent),
    /// Keyboard events
    Keyboard(KeyboardEvent),
    /// Focus events
    Focus(FocusEvent),
    /// Component-specific events
    Custom(CustomEvent),
    /// State change events
    StateChange(StateChangeEvent),
}

/// Mouse event data
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub x: u16,
    pub y: u16,
    pub button: Option<crate::event::MouseButton>,
    pub modifiers: crate::event::KeyModifiers,
}

/// Keyboard event data
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub key: crate::event::KeyCode,
    pub modifiers: crate::event::KeyModifiers,
}

/// Focus event data
#[derive(Debug, Clone)]
pub enum FocusEvent {
    Gained,
    Lost,
    Moved { from: ComponentId, to: ComponentId },
}

/// Custom event data
#[derive(Debug, Clone)]
pub struct CustomEvent {
    pub event_type: String,
    pub data: serde_json::Value,
}

/// State change event data
#[derive(Debug, Clone)]
pub struct StateChangeEvent {
    pub component_id: ComponentId,
    pub property: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
}

/// Event propagation control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPropagation {
    /// Continue propagating the event
    Continue,
    /// Stop event propagation
    Stop,
    /// Consume the event (don't propagate further)
    Consume,
}

/// Event handler result
#[derive(Debug, Clone)]
pub struct EventResult {
    pub propagation: EventPropagation,
    pub handled: bool,
    pub data: Option<serde_json::Value>,
}

/// Enhanced component trait with typed event system
pub trait EventComponent: Component {
    /// Handle a typed event with bubbling support
    fn handle_event(&mut self, event: &InputEvent, context: &EventContext) -> EventResult;

    /// Check if component can handle a specific event type
    fn can_handle_event(&self, event_type: &InputEvent) -> bool {
        // Default implementation accepts all events
        let _ = event_type;
        true
    }

    /// Get event bubbling priority (higher numbers bubble first)
    fn event_priority(&self) -> i32 {
        0
    }
}

/// Event context for event handling
#[derive(Debug, Clone)]
pub struct EventContext {
    pub timestamp: std::time::Instant,
    pub target: ComponentId,
    pub current_target: ComponentId,
    pub event_phase: EventPhase,
    pub propagation_path: Vec<ComponentId>,
}

/// Event phases for bubbling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPhase {
    /// Event is being captured (trickling down)
    Capture,
    /// Event is at target
    Target,
    /// Event is bubbling up
    Bubble,
}

/// Event dispatcher for managing event flow
pub struct EventDispatcher {
    components: std::collections::HashMap<ComponentId, Box<dyn EventComponent>>,
    event_queue: std::collections::VecDeque<(InputEvent, EventContext)>,
    max_queue_size: usize,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            components: std::collections::HashMap::new(),
            event_queue: std::collections::VecDeque::new(),
            max_queue_size: 1000,
        }
    }

    /// Register an event component
    pub fn register_component(&mut self, component: Box<dyn EventComponent>) {
        let id = component.id();
        self.components.insert(id, component);
    }

    /// Unregister an event component
    pub fn unregister_component(&mut self, id: &ComponentId) -> Option<Box<dyn EventComponent>> {
        self.components.remove(id)
    }

    /// Dispatch an event with bubbling
    pub async fn dispatch_event(
        &mut self,
        event: InputEvent,
        target_id: ComponentId,
    ) -> Vec<EventResult> {
        let mut results = Vec::new();

        // Build propagation path (simplified - in practice would build full component tree)
        let propagation_path = vec![target_id.clone()];

        // Create event context
        let context = EventContext {
            timestamp: std::time::Instant::now(),
            target: target_id.clone(),
            current_target: target_id.clone(),
            event_phase: EventPhase::Target,
            propagation_path: propagation_path.clone(),
        };

        // Handle event at target
        if let Some(component) = self.components.get_mut(&target_id) {
            if component.can_handle_event(&event) {
                let result = component.handle_event(&event, &context);
                results.push(result);

                // If event was consumed, stop propagation
                if let Some(last_result) = results.last() {
                    if last_result.propagation == EventPropagation::Consume {
                        return results;
                    }
                }
            }
        }

        // Bubble up to parent components (simplified - would need component tree)
        // In a real implementation, this would traverse up the component hierarchy

        results
    }

    /// Queue an event for later processing
    pub fn queue_event(&mut self, event: InputEvent, target: ComponentId) {
        if self.event_queue.len() < self.max_queue_size {
            let context = EventContext {
                timestamp: std::time::Instant::now(),
                target: target.clone(),
                current_target: target.clone(),
                event_phase: EventPhase::Target,
                propagation_path: vec![target],
            };
            self.event_queue.push_back((event, context));
        } else {
            tracing::warn!("Event queue full, dropping event");
        }
    }

    /// Process queued events
    pub async fn process_queue(&mut self) -> Vec<Vec<EventResult>> {
        let mut all_results = Vec::new();

        while let Some((event, context)) = self.event_queue.pop_front() {
            let results = self.dispatch_event(event, context.target).await;
            all_results.push(results);
        }

        all_results
    }

    /// Get queue size
    pub fn queue_size(&self) -> usize {
        self.event_queue.len()
    }

    /// Clear event queue
    pub fn clear_queue(&mut self) {
        self.event_queue.clear();
    }

    /// Convert AppMessage to ComponentEvent
    pub fn app_message_to_event(message: &AppMessage) -> Option<ComponentEvent> {
        match message {
            AppMessage::KeyPress(key) => {
                let key_code = match key.code {
                    crossterm::event::KeyCode::Char(c) => crate::event::KeyCode::Char(c),
                    crossterm::event::KeyCode::Enter => crate::event::KeyCode::Enter,
                    crossterm::event::KeyCode::Esc => crate::event::KeyCode::Esc,
                    crossterm::event::KeyCode::Tab => crate::event::KeyCode::Tab,
                    crossterm::event::KeyCode::Backspace => crate::event::KeyCode::Backspace,
                    crossterm::event::KeyCode::Delete => crate::event::KeyCode::Delete,
                    crossterm::event::KeyCode::Up => crate::event::KeyCode::Up,
                    crossterm::event::KeyCode::Down => crate::event::KeyCode::Down,
                    crossterm::event::KeyCode::Left => crate::event::KeyCode::Left,
                    crossterm::event::KeyCode::Right => crate::event::KeyCode::Right,
                    crossterm::event::KeyCode::Home => crate::event::KeyCode::Home,
                    crossterm::event::KeyCode::End => crate::event::KeyCode::End,
                    crossterm::event::KeyCode::PageUp => crate::event::KeyCode::PageUp,
                    crossterm::event::KeyCode::PageDown => crate::event::KeyCode::PageDown,
                    crossterm::event::KeyCode::F(n) => match n {
                        1 => crate::event::KeyCode::F1,
                        2 => crate::event::KeyCode::F2,
                        3 => crate::event::KeyCode::F3,
                        4 => crate::event::KeyCode::F4,
                        5 => crate::event::KeyCode::F5,
                        6 => crate::event::KeyCode::F6,
                        7 => crate::event::KeyCode::F7,
                        8 => crate::event::KeyCode::F8,
                        9 => crate::event::KeyCode::F9,
                        10 => crate::event::KeyCode::F10,
                        11 => crate::event::KeyCode::F11,
                        12 => crate::event::KeyCode::F12,
                        _ => return None,
                    },
                    _ => return None,
                };

                let modifiers = crate::event::KeyModifiers {
                    ctrl: key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL),
                    alt: key.modifiers.contains(crossterm::event::KeyModifiers::ALT),
                    shift: key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT),
                };

                Some(ComponentEvent::Keyboard(crate::event::KeyEvent {
                    code: key_code,
                    modifiers,
                }))
            }
            AppMessage::MouseEvent(mouse) => {
                let button = match mouse.kind {
                    crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        Some(crate::event::MouseButton::Left)
                    }
                    crossterm::event::MouseEventKind::Down(
                        crossterm::event::MouseButton::Right,
                    ) => Some(crate::event::MouseButton::Right),
                    crossterm::event::MouseEventKind::Down(
                        crossterm::event::MouseButton::Middle,
                    ) => Some(crate::event::MouseButton::Middle),
                    _ => None,
                };

                if let Some(button) = button {
                    Some(ComponentEvent::Mouse(crate::event::MouseEvent {
                        x: mouse.column,
                        y: mouse.row,
                        button,
                    }))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro to implement EventComponent for a type
#[macro_export]
macro_rules! impl_event_component {
    ($type:ty) => {
        impl EventComponent for $type {
            fn handle_event(
                &mut self,
                event: &ComponentEvent,
                _context: &EventContext,
            ) -> EventResult {
                // Default implementation - delegate to existing update method if available
                // This is a simplified implementation
                let handled = match event {
                    ComponentEvent::Keyboard(key_event) => {
                        // Convert back to AppMessage for compatibility
                        let message = match key_event.key {
                            crate::event::KeyCode::Char(c) => {
                                AppMessage::KeyPress(crossterm::event::KeyEvent::new(
                                    crossterm::event::KeyCode::Char(c),
                                    // Convert modifiers back
                                    if key_event.modifiers.ctrl {
                                        crossterm::event::KeyModifiers::CONTROL
                                    } else if key_event.modifiers.alt {
                                        crossterm::event::KeyModifiers::ALT
                                    } else if key_event.modifiers.shift {
                                        crossterm::event::KeyModifiers::SHIFT
                                    } else {
                                        crossterm::event::KeyModifiers::empty()
                                    },
                                ))
                            }
                            // Add other key mappings as needed
                            _ => {
                                return EventResult {
                                    propagation: EventPropagation::Continue,
                                    handled: false,
                                    data: None,
                                }
                            }
                        };

                        self.update(&message, &AppModel::default()) // Simplified - would need real model
                    }
                    _ => false,
                };

                EventResult {
                    propagation: if handled {
                        EventPropagation::Consume
                    } else {
                        EventPropagation::Continue
                    },
                    handled,
                    data: None,
                }
            }
        }
    };
}

/// Mode indicator component
#[derive(Debug, Clone)]
pub struct ModeIndicator {
    /// Current mode
    pub mode: AppMode,
    /// Show keyboard shortcut
    pub show_shortcut: bool,
    /// Show mode capabilities
    pub show_capabilities: bool,
}

impl ModeIndicator {
    /// Create a new mode indicator
    pub fn new(mode: AppMode) -> Self {
        Self {
            mode,
            show_shortcut: true,
            show_capabilities: false,
        }
    }

    /// Get the display text for the mode
    pub fn display_text(&self) -> String {
        if self.show_shortcut {
            format!("[{}] {}", self.mode.shortcut(), self.mode.display_name())
        } else {
            format!("[{}]", self.mode.display_name())
        }
    }

    /// Get the short display text
    pub fn short_text(&self) -> &'static str {
        self.mode.display_name()
    }

    /// Get the capabilities for the current mode
    pub fn get_capabilities(&self) -> Vec<&'static str> {
        match self.mode {
            AppMode::Chat => vec!["QuestionAnswering", "FreeformChat"],
            AppMode::Command => vec!["CodeGeneration", "FileOperations", "CommandExecution"],
            AppMode::Diff => vec!["CodeModification", "FileOperations"],
            AppMode::Mcp => vec!["ToolExecution", "ServerManagement"],
            AppMode::Provider => vec!["ProviderManagement", "StatusMonitoring"],
            AppMode::Session => vec!["SessionManagement", "Sharing"],
            AppMode::Help => vec!["QuestionAnswering"],
        }
    }

    /// Get capabilities display text
    pub fn capabilities_text(&self) -> String {
        let caps = self.get_capabilities();
        format!("Capabilities: {}", caps.join(", "))
    }

    /// Update the mode
    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    /// Toggle shortcut display
    pub fn toggle_shortcut_display(&mut self) {
        self.show_shortcut = !self.show_shortcut;
    }

    /// Toggle capabilities display
    pub fn toggle_capabilities_display(&mut self) {
        self.show_capabilities = !self.show_capabilities;
    }

    /// Enable capabilities display
    pub fn show_capabilities_enabled(&mut self) {
        self.show_capabilities = true;
    }

    /// Disable capabilities display
    pub fn hide_capabilities_enabled(&mut self) {
        self.show_capabilities = false;
    }
}

impl Default for ModeIndicator {
    fn default() -> Self {
        Self::new(AppMode::Chat)
    }
}

/// Mode selection menu for switching between modes
#[derive(Debug, Clone)]
pub struct ModeSelectionMenu {
    /// Available modes
    pub modes: Vec<AppMode>,
    /// Currently selected mode index
    pub selected: usize,
    /// Whether the menu is open
    pub open: bool,
    /// Whether to show confirmation dialog
    pub show_confirmation: bool,
    /// Previous mode (for cancellation)
    pub previous_mode: AppMode,
}

impl ModeSelectionMenu {
    /// Create a new mode selection menu
    pub fn new() -> Self {
        Self {
            modes: vec![
                AppMode::Chat,
                AppMode::Command,
                AppMode::Diff,
                AppMode::Mcp,
                AppMode::Provider,
                AppMode::Help,
            ],
            selected: 0,
            open: false,
            show_confirmation: false,
            previous_mode: AppMode::Chat,
        }
    }

    /// Open the mode selection menu
    pub fn open(&mut self, current_mode: AppMode) {
        self.open = true;
        self.previous_mode = current_mode;
        // Find and select the current mode
        if let Some(pos) = self.modes.iter().position(|&m| m == current_mode) {
            self.selected = pos;
        }
    }

    /// Close the mode selection menu
    pub fn close(&mut self) {
        self.open = false;
        self.show_confirmation = false;
    }

    /// Get the currently selected mode
    pub fn selected_mode(&self) -> AppMode {
        self.modes
            .get(self.selected)
            .copied()
            .unwrap_or(AppMode::Chat)
    }

    /// Move selection to next mode
    pub fn select_next(&mut self) {
        if self.selected < self.modes.len().saturating_sub(1) {
            self.selected += 1;
        } else {
            self.selected = 0;
        }
    }

    /// Move selection to previous mode
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.modes.len().saturating_sub(1);
        }
    }

    /// Confirm mode switch
    pub fn confirm_switch(&mut self) -> AppMode {
        let mode = self.selected_mode();
        self.close();
        mode
    }

    /// Cancel mode switch
    pub fn cancel_switch(&mut self) {
        self.close();
    }

    /// Get mode descriptions for display
    pub fn get_mode_descriptions(&self) -> Vec<(&AppMode, &'static str)> {
        self.modes
            .iter()
            .map(|mode| {
                let desc = match mode {
                    AppMode::Chat => "Chat with the AI assistant",
                    AppMode::Command => "Execute commands and generate code",
                    AppMode::Diff => "Review and apply code changes",
                    AppMode::Mcp => "Manage MCP servers and tools",
                    AppMode::Provider => "Configure AI providers",
                    AppMode::Session => "Manage and share sessions",
                    AppMode::Help => "Get help and documentation",
                };
                (mode, desc)
            })
            .collect()
    }

    /// Get keyboard shortcuts for mode switching
    pub fn get_shortcuts(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Ctrl+1", "Chat Mode"),
            ("Ctrl+2", "Command Mode"),
            ("Ctrl+3", "Diff Mode"),
            ("Ctrl+4", "Help Mode"),
        ]
    }
}

impl Default for ModeSelectionMenu {
    fn default() -> Self {
        Self::new()
    }
}

/// Menu item
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Item label
    pub label: String,
    /// Item description
    pub description: Option<String>,
}

impl MenuItem {
    /// Create a new menu item
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Menu widget
pub struct MenuWidget {
    /// Menu items
    pub items: Vec<MenuItem>,
    /// Selected item index
    pub selected: usize,
    /// Whether menu is open
    pub open: bool,
    /// Menu title
    pub title: Option<String>,
    /// Scroll offset for large menus
    pub scroll: usize,
}

impl MenuWidget {
    /// Create a new menu widget
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            open: false,
            title: None,
            scroll: 0,
        }
    }

    /// Create a menu with a title
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            open: false,
            title: Some(title.into()),
            scroll: 0,
        }
    }

    /// Add a menu item
    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    /// Add multiple items
    pub fn add_items(&mut self, items: Vec<MenuItem>) {
        self.items.extend(items);
    }

    /// Select next item
    pub fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
            self.ensure_visible(10); // Assume 10 visible items
        }
    }

    /// Select previous item
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.ensure_visible(10);
        }
    }

    /// Jump to first item
    pub fn select_first(&mut self) {
        self.selected = 0;
        self.scroll = 0;
    }

    /// Jump to last item
    pub fn select_last(&mut self) {
        self.selected = self.items.len().saturating_sub(1);
        self.ensure_visible(10);
    }

    /// Ensure selected item is visible
    fn ensure_visible(&mut self, visible_height: usize) {
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + visible_height {
            self.scroll = self.selected.saturating_sub(visible_height - 1);
        }
    }

    /// Get selected item
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected)
    }

    /// Get selected item index
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Open the menu
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Close the menu
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Toggle menu open state
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Get visible items based on scroll
    pub fn visible_items(&self, height: usize) -> Vec<(usize, &MenuItem)> {
        self.items
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(height)
            .collect()
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
        self.selected = 0;
        self.scroll = 0;
    }

    /// Get total item count
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if menu is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for MenuWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// List widget
pub struct ListWidget {
    /// List items
    pub items: Vec<String>,
    /// Selected item index
    pub selected: Option<usize>,
    /// Filter text
    pub filter: String,
    /// Multi-select enabled
    pub multi_select: bool,
    /// Selected items (for multi-select)
    pub selected_items: std::collections::HashSet<usize>,
    /// Scroll offset
    pub scroll: usize,
}

impl ListWidget {
    /// Create a new list widget
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            filter: String::new(),
            multi_select: false,
            selected_items: std::collections::HashSet::new(),
            scroll: 0,
        }
    }

    /// Enable multi-select mode
    pub fn with_multi_select(mut self) -> Self {
        self.multi_select = true;
        self
    }

    /// Add an item
    pub fn add_item(&mut self, item: impl Into<String>) {
        self.items.push(item.into());
    }

    /// Add multiple items
    pub fn add_items(&mut self, items: Vec<String>) {
        self.items.extend(items);
    }

    /// Set filter
    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.filter = filter.into();
        self.scroll = 0; // Reset scroll when filtering
    }

    /// Clear filter
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.scroll = 0;
    }

    /// Get filtered items
    pub fn filtered_items(&self) -> Vec<(usize, &String)> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.to_lowercase().contains(&self.filter.to_lowercase()))
            .collect()
    }

    /// Get visible items based on scroll
    pub fn visible_items(&self, height: usize) -> Vec<(usize, &String)> {
        self.filtered_items()
            .into_iter()
            .skip(self.scroll)
            .take(height)
            .collect()
    }

    /// Select next item
    pub fn select_next(&mut self) {
        let filtered = self.filtered_items();
        if filtered.is_empty() {
            return;
        }

        match self.selected {
            None => {
                self.selected = Some(filtered[0].0);
                self.scroll = 0;
            }
            Some(idx) => {
                if let Some(pos) = filtered.iter().position(|(i, _)| *i == idx) {
                    if pos < filtered.len() - 1 {
                        self.selected = Some(filtered[pos + 1].0);
                    }
                }
            }
        }
    }

    /// Select previous item
    pub fn select_prev(&mut self) {
        let filtered = self.filtered_items();
        if filtered.is_empty() {
            return;
        }

        match self.selected {
            None => {}
            Some(idx) => {
                if let Some(pos) = filtered.iter().position(|(i, _)| *i == idx) {
                    if pos > 0 {
                        self.selected = Some(filtered[pos - 1].0);
                    }
                }
            }
        }
    }

    /// Toggle selection for current item (multi-select)
    pub fn toggle_selection(&mut self) {
        if self.multi_select {
            if let Some(idx) = self.selected {
                if self.selected_items.contains(&idx) {
                    self.selected_items.remove(&idx);
                } else {
                    self.selected_items.insert(idx);
                }
            }
        }
    }

    /// Select all items
    pub fn select_all(&mut self) {
        if self.multi_select {
            let indices: Vec<usize> = self
                .filtered_items()
                .into_iter()
                .map(|(idx, _)| idx)
                .collect();
            for idx in indices {
                self.selected_items.insert(idx);
            }
        }
    }

    /// Deselect all items
    pub fn deselect_all(&mut self) {
        self.selected_items.clear();
    }

    /// Get selected item
    pub fn selected_item(&self) -> Option<&String> {
        self.selected.and_then(|idx| self.items.get(idx))
    }

    /// Get all selected items (multi-select)
    pub fn get_selected_items(&self) -> Vec<&String> {
        self.selected_items
            .iter()
            .filter_map(|idx| self.items.get(*idx))
            .collect()
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
        self.selected = None;
        self.selected_items.clear();
        self.scroll = 0;
    }

    /// Get total item count
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if list is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for ListWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Dialog type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    /// Input dialog
    Input,
    /// Confirmation dialog
    Confirm,
    /// Message dialog
    Message,
}

/// Dialog result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    /// Dialog was confirmed
    Confirmed,
    /// Dialog was cancelled
    Cancelled,
    /// Dialog is still open
    Pending,
}

/// Dialog widget
pub struct DialogWidget {
    /// Dialog type
    pub dialog_type: DialogType,
    /// Dialog title
    pub title: String,
    /// Dialog message
    pub message: String,
    /// Input value (for input dialogs)
    pub input: String,
    /// Cursor position
    pub cursor: usize,
    /// Dialog result
    pub result: DialogResult,
    /// Validation function (for input dialogs)
    pub validator: Option<fn(&str) -> bool>,
    /// Error message (if validation fails)
    pub error_message: Option<String>,
    /// Confirmation state (for confirm dialogs)
    pub confirmed: Option<bool>,
}

impl DialogWidget {
    /// Create a new dialog widget
    pub fn new(
        dialog_type: DialogType,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            dialog_type,
            title: title.into(),
            message: message.into(),
            input: String::new(),
            cursor: 0,
            result: DialogResult::Pending,
            validator: None,
            error_message: None,
            confirmed: None,
        }
    }

    /// Set a validator function
    pub fn with_validator(mut self, validator: fn(&str) -> bool) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Insert character
    pub fn insert_char(&mut self, ch: char) {
        if ch.is_ascii_graphic() || ch == ' ' {
            self.input.insert(self.cursor, ch);
            self.cursor += 1;
            self.error_message = None;
        }
    }

    /// Backspace
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.input.remove(self.cursor - 1);
            self.cursor -= 1;
            self.error_message = None;
        }
    }

    /// Delete character at cursor
    pub fn delete(&mut self) {
        if self.cursor < self.input.len() {
            self.input.remove(self.cursor);
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
        }
    }

    /// Move cursor to start
    pub fn cursor_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor = self.input.len();
    }

    /// Get input value
    pub fn get_input(&self) -> String {
        self.input.clone()
    }

    /// Validate input
    pub fn validate(&mut self) -> bool {
        if let Some(validator) = self.validator {
            if validator(&self.input) {
                self.error_message = None;
                true
            } else {
                self.error_message = Some("Invalid input".to_string());
                false
            }
        } else {
            true
        }
    }

    /// Confirm dialog
    pub fn confirm(&mut self) {
        match self.dialog_type {
            DialogType::Input => {
                if self.validate() {
                    self.result = DialogResult::Confirmed;
                }
            }
            DialogType::Confirm => {
                self.confirmed = Some(true);
                self.result = DialogResult::Confirmed;
            }
            DialogType::Message => {
                self.result = DialogResult::Confirmed;
            }
        }
    }

    /// Cancel dialog
    pub fn cancel(&mut self) {
        if self.dialog_type == DialogType::Confirm {
            self.confirmed = Some(false);
        }
        self.result = DialogResult::Cancelled;
    }

    /// Check if dialog is confirmed
    pub fn is_confirmed(&self) -> bool {
        self.result == DialogResult::Confirmed
    }

    /// Check if dialog is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.result == DialogResult::Cancelled
    }

    /// Check if dialog is pending
    pub fn is_pending(&self) -> bool {
        self.result == DialogResult::Pending
    }

    /// Clear input
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.error_message = None;
    }
}

/// Split direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    /// Vertical split (left/right)
    Vertical,
    /// Horizontal split (top/bottom)
    Horizontal,
}

/// Split view widget
pub struct SplitViewWidget {
    /// Left/top panel content
    pub left_content: String,
    /// Right/bottom panel content
    pub right_content: String,
    /// Split ratio (0-100)
    pub split_ratio: u8,
    /// Split direction
    pub direction: SplitDirection,
    /// Active panel (0 = left/top, 1 = right/bottom)
    pub active_panel: usize,
    /// Left/top panel scroll
    pub left_scroll: usize,
    /// Right/bottom panel scroll
    pub right_scroll: usize,
}

impl SplitViewWidget {
    /// Create a new split view widget
    pub fn new() -> Self {
        Self {
            left_content: String::new(),
            right_content: String::new(),
            split_ratio: 50,
            direction: SplitDirection::Vertical,
            active_panel: 0,
            left_scroll: 0,
            right_scroll: 0,
        }
    }

    /// Create a horizontal split view
    pub fn horizontal() -> Self {
        Self {
            left_content: String::new(),
            right_content: String::new(),
            split_ratio: 50,
            direction: SplitDirection::Horizontal,
            active_panel: 0,
            left_scroll: 0,
            right_scroll: 0,
        }
    }

    /// Set left/top content
    pub fn set_left(&mut self, content: impl Into<String>) {
        self.left_content = content.into();
    }

    /// Set right/bottom content
    pub fn set_right(&mut self, content: impl Into<String>) {
        self.right_content = content.into();
    }

    /// Adjust split ratio
    pub fn adjust_split(&mut self, delta: i8) {
        let new_ratio = (self.split_ratio as i16 + delta as i16).clamp(20, 80) as u8;
        self.split_ratio = new_ratio;
    }

    /// Switch active panel
    pub fn switch_panel(&mut self) {
        self.active_panel = 1 - self.active_panel;
    }

    /// Get active panel content
    pub fn active_content(&self) -> &str {
        if self.active_panel == 0 {
            &self.left_content
        } else {
            &self.right_content
        }
    }

    /// Get active panel scroll
    pub fn active_scroll(&self) -> usize {
        if self.active_panel == 0 {
            self.left_scroll
        } else {
            self.right_scroll
        }
    }

    /// Scroll active panel up
    pub fn scroll_up(&mut self) {
        if self.active_panel == 0 {
            if self.left_scroll > 0 {
                self.left_scroll -= 1;
            }
        } else if self.right_scroll > 0 {
            self.right_scroll -= 1;
        }
    }

    /// Scroll active panel down
    pub fn scroll_down(&mut self) {
        if self.active_panel == 0 {
            self.left_scroll += 1;
        } else {
            self.right_scroll += 1;
        }
    }
}

impl Default for SplitViewWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Tab widget
pub struct TabWidget {
    /// Tab titles
    pub tabs: Vec<String>,
    /// Active tab index
    pub active: usize,
    /// Tab content
    pub content: Vec<String>,
    /// Scroll offset for tab bar
    pub scroll: usize,
}

impl TabWidget {
    /// Create a new tab widget
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
            content: Vec::new(),
            scroll: 0,
        }
    }

    /// Add a tab
    pub fn add_tab(&mut self, title: impl Into<String>) {
        self.tabs.push(title.into());
        self.content.push(String::new());
    }

    /// Add a tab with content
    pub fn add_tab_with_content(&mut self, title: impl Into<String>, content: impl Into<String>) {
        self.tabs.push(title.into());
        self.content.push(content.into());
    }

    /// Select next tab
    pub fn select_next(&mut self) {
        if self.active < self.tabs.len().saturating_sub(1) {
            self.active += 1;
            self.ensure_visible(10);
        }
    }

    /// Select previous tab
    pub fn select_prev(&mut self) {
        if self.active > 0 {
            self.active -= 1;
            self.ensure_visible(10);
        }
    }

    /// Select tab by index
    pub fn select_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active = index;
            self.ensure_visible(10);
        }
    }

    /// Ensure active tab is visible
    fn ensure_visible(&mut self, visible_width: usize) {
        if self.active < self.scroll {
            self.scroll = self.active;
        } else if self.active >= self.scroll + visible_width {
            self.scroll = self.active.saturating_sub(visible_width - 1);
        }
    }

    /// Get active tab title
    pub fn active_tab(&self) -> Option<&String> {
        self.tabs.get(self.active)
    }

    /// Get active tab content
    pub fn active_content(&self) -> Option<&String> {
        self.content.get(self.active)
    }

    /// Set content for active tab
    pub fn set_active_content(&mut self, content: impl Into<String>) {
        if let Some(c) = self.content.get_mut(self.active) {
            *c = content.into();
        }
    }

    /// Get visible tabs
    pub fn visible_tabs(&self, width: usize) -> Vec<(usize, &String)> {
        self.tabs
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(width)
            .collect()
    }

    /// Close tab
    pub fn close_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs.remove(index);
            self.content.remove(index);

            if self.active >= self.tabs.len() && self.active > 0 {
                self.active -= 1;
            }
        }
    }

    /// Close active tab
    pub fn close_active_tab(&mut self) {
        self.close_tab(self.active);
    }

    /// Get tab count
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Check if tabs are empty
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Clear all tabs
    pub fn clear(&mut self) {
        self.tabs.clear();
        self.content.clear();
        self.active = 0;
        self.scroll = 0;
    }
}

impl Default for TabWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Vim keybinding mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    /// Normal mode (navigation)
    Normal,
    /// Insert mode (text input)
    Insert,
    /// Visual mode (selection)
    Visual,
    /// Command mode (commands)
    Command,
}

/// Vim keybindings configuration
pub struct VimKeybindings {
    /// Whether vim mode is enabled
    pub enabled: bool,
    /// Current vim mode
    pub mode: VimMode,
    /// Command buffer for command mode
    pub command_buffer: String,
}

impl VimKeybindings {
    /// Create a new vim keybindings configuration
    pub fn new() -> Self {
        Self {
            enabled: false,
            mode: VimMode::Normal,
            command_buffer: String::new(),
        }
    }

    /// Enable vim mode
    pub fn enable(&mut self) {
        self.enabled = true;
        self.mode = VimMode::Normal;
    }

    /// Disable vim mode
    pub fn disable(&mut self) {
        self.enabled = false;
        self.mode = VimMode::Normal;
        self.command_buffer.clear();
    }

    /// Toggle vim mode
    pub fn toggle(&mut self) {
        if self.enabled {
            self.disable();
        } else {
            self.enable();
        }
    }

    /// Enter insert mode
    pub fn enter_insert(&mut self) {
        if self.enabled {
            self.mode = VimMode::Insert;
        }
    }

    /// Enter normal mode
    pub fn enter_normal(&mut self) {
        if self.enabled {
            self.mode = VimMode::Normal;
            self.command_buffer.clear();
        }
    }

    /// Enter visual mode
    pub fn enter_visual(&mut self) {
        if self.enabled {
            self.mode = VimMode::Visual;
        }
    }

    /// Enter command mode
    pub fn enter_command(&mut self) {
        if self.enabled {
            self.mode = VimMode::Command;
            self.command_buffer.clear();
        }
    }

    /// Add character to command buffer
    pub fn add_to_command(&mut self, ch: char) {
        self.command_buffer.push(ch);
    }

    /// Clear command buffer
    pub fn clear_command(&mut self) {
        self.command_buffer.clear();
    }

    /// Get command buffer
    pub fn get_command(&self) -> &str {
        &self.command_buffer
    }

    /// Check if in normal mode
    pub fn is_normal(&self) -> bool {
        self.enabled && self.mode == VimMode::Normal
    }

    /// Check if in insert mode
    pub fn is_insert(&self) -> bool {
        self.enabled && self.mode == VimMode::Insert
    }

    /// Check if in visual mode
    pub fn is_visual(&self) -> bool {
        self.enabled && self.mode == VimMode::Visual
    }

    /// Check if in command mode
    pub fn is_command(&self) -> bool {
        self.enabled && self.mode == VimMode::Command
    }
}

impl Default for VimKeybindings {
    fn default() -> Self {
        Self::new()
    }
}
