//! Event dispatcher for triggering hooks

pub mod event;

pub use event::DefaultEventDispatcher;

use crate::error::Result;
use crate::types::Event;

/// Trait for dispatching events to hooks
///
/// The EventDispatcher is responsible for:
/// 1. Receiving events from the system
/// 2. Querying the HookRegistry for hooks matching the event type
/// 3. Routing events to matching hooks
/// 4. Passing event context to hooks
/// 5. Handling hook execution and error recovery
///
/// # Examples
///
/// ```ignore
/// let dispatcher = DefaultEventDispatcher::new(registry, executor);
/// let event = Event {
///     event_type: "file_saved".to_string(),
///     context: EventContext { ... },
///     timestamp: "2024-01-01T12:00:00Z".to_string(),
/// };
/// dispatcher.dispatch_event(event)?;
/// ```
pub trait EventDispatcher: Send + Sync {
    /// Dispatch an event to matching hooks
    ///
    /// This method:
    /// 1. Queries the registry for hooks matching the event type
    /// 2. Filters enabled hooks
    /// 3. Executes each hook with the event context
    /// 4. Logs errors but continues with other hooks
    /// 5. Returns success if at least one hook executed
    fn dispatch_event(&self, event: Event) -> Result<()>;
}
