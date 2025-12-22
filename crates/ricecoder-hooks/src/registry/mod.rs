//! Hook registry for storing and managing hooks
//!
//! The hook registry is responsible for storing, retrieving, and managing hooks.
//! It provides a central place to register hooks that will be triggered on specific events.
//!
//! # Examples
//!
//! ```ignore
//! use ricecoder_hooks::registry::InMemoryHookRegistry;
//! use ricecoder_hooks::types::Hook;
//! use ricecoder_hooks::HookRegistry;
//!
//! let mut registry = InMemoryHookRegistry::new();
//!
//! // Register a hook
//! let hook = Hook {
//!     id: "hook-1".to_string(),
//!     name: "Format on save".to_string(),
//!     event: "file_modified".to_string(),
//!     enabled: true,
//!     // ... other fields
//! };
//!
//! let hook_id = registry.register_hook(hook)?;
//! println!("Registered hook: {}", hook_id);
//!
//! // List all hooks
//! let hooks = registry.list_hooks()?;
//! println!("Total hooks: {}", hooks.len());
//!
//! // List hooks for a specific event
//! let file_hooks = registry.list_hooks_for_event("file_modified")?;
//! println!("Hooks for file_modified: {}", file_hooks.len());
//!
//! // Enable/disable a hook
//! registry.disable_hook(&hook_id)?;
//! registry.enable_hook(&hook_id)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod storage;

pub use storage::InMemoryHookRegistry;

use crate::{error::Result, types::Hook};

/// Trait for managing hooks
///
/// The `HookRegistry` trait defines the interface for storing and managing hooks.
/// Implementations must support registering, unregistering, querying, and enabling/disabling hooks.
///
/// # Thread Safety
///
/// All implementations must be thread-safe (`Send + Sync`) to support concurrent access.
pub trait HookRegistry: Send + Sync {
    /// Register a new hook
    ///
    /// Stores a hook in the registry and returns its unique ID.
    /// The hook will be assigned a unique ID if not already set.
    ///
    /// # Arguments
    ///
    /// * `hook` - The hook to register
    ///
    /// # Returns
    ///
    /// The unique ID of the registered hook
    ///
    /// # Errors
    ///
    /// Returns an error if the hook is invalid or registration fails
    fn register_hook(&mut self, hook: Hook) -> Result<String>;

    /// Unregister a hook by ID
    ///
    /// Removes a hook from the registry.
    ///
    /// # Arguments
    ///
    /// * `hook_id` - The ID of the hook to unregister
    ///
    /// # Errors
    ///
    /// Returns an error if the hook is not found
    fn unregister_hook(&self, hook_id: &str) -> Result<()>;

    /// Get a hook by ID
    ///
    /// Retrieves a hook from the registry by its ID.
    ///
    /// # Arguments
    ///
    /// * `hook_id` - The ID of the hook to retrieve
    ///
    /// # Returns
    ///
    /// The hook with the specified ID
    ///
    /// # Errors
    ///
    /// Returns an error if the hook is not found
    fn get_hook(&self, hook_id: &str) -> Result<Hook>;

    /// List all hooks
    ///
    /// Returns all hooks in the registry.
    ///
    /// # Returns
    ///
    /// A vector of all hooks
    ///
    /// # Errors
    ///
    /// Returns an error if listing fails
    fn list_hooks(&self) -> Result<Vec<Hook>>;

    /// List hooks for a specific event
    ///
    /// Returns all hooks registered for a specific event type.
    /// Only enabled hooks are returned.
    ///
    /// # Arguments
    ///
    /// * `event` - The event type to filter by
    ///
    /// # Returns
    ///
    /// A vector of hooks for the specified event
    ///
    /// # Errors
    ///
    /// Returns an error if listing fails
    fn list_hooks_for_event(&self, event: &str) -> Result<Vec<Hook>>;

    /// Enable a hook
    ///
    /// Enables a hook so it will be triggered when its event occurs.
    ///
    /// # Arguments
    ///
    /// * `hook_id` - The ID of the hook to enable
    ///
    /// # Errors
    ///
    /// Returns an error if the hook is not found
    fn enable_hook(&mut self, hook_id: &str) -> Result<()>;

    /// Disable a hook
    ///
    /// Disables a hook so it will not be triggered when its event occurs.
    ///
    /// # Arguments
    ///
    /// * `hook_id` - The ID of the hook to disable
    ///
    /// # Errors
    ///
    /// Returns an error if the hook is not found
    fn disable_hook(&mut self, hook_id: &str) -> Result<()>;
}
