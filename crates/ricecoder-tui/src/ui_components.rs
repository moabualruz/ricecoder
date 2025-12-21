//! UI component types and utilities
//!
//! This module provides types for advanced UI functionality including
//! optimistic updates, loading states, and virtual rendering.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Optimistic update manager for responsive UI
pub struct OptimisticUpdater {
    /// Pending optimistic updates
    pending_updates: Arc<RwLock<HashMap<String, OptimisticUpdate>>>,
    /// Rollback strategies
    rollback_strategies: HashMap<String, Box<dyn Fn() + Send + Sync>>,
}

impl std::fmt::Debug for OptimisticUpdater {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptimisticUpdater")
            .field("pending_updates", &self.pending_updates)
            .field(
                "rollback_strategies",
                &format!("<{} strategies>", self.rollback_strategies.len()),
            )
            .finish()
    }
}

impl Clone for OptimisticUpdater {
    fn clone(&self) -> Self {
        Self {
            pending_updates: Arc::clone(&self.pending_updates),
            rollback_strategies: HashMap::new(), // Can't clone Box<dyn Fn>, so start empty
        }
    }
}

impl OptimisticUpdater {
    /// Create a new optimistic updater
    pub fn new() -> Self {
        Self {
            pending_updates: Arc::new(RwLock::new(HashMap::new())),
            rollback_strategies: HashMap::new(),
        }
    }

    /// Apply an optimistic update
    pub async fn apply_update(&self, id: String, update: OptimisticUpdate) {
        let mut updates = self.pending_updates.write().await;
        updates.insert(id, update);
    }

    /// Confirm an optimistic update
    pub async fn confirm_update(&self, id: &str) {
        let mut updates = self.pending_updates.write().await;
        updates.remove(id);
    }

    /// Rollback an optimistic update
    pub async fn rollback_update(&self, id: &str) {
        let mut updates = self.pending_updates.write().await;
        if let Some(update) = updates.remove(id) {
            // Execute rollback if available
            if let Some(rollback) = self.rollback_strategies.get(id) {
                rollback();
            }
        }
    }
}

/// Optimistic update data
#[derive(Debug, Clone)]
pub struct OptimisticUpdate {
    /// Update identifier
    pub id: String,
    /// Update data
    pub data: serde_json::Value,
    /// Timestamp
    pub timestamp: std::time::Instant,
}

/// Loading state manager
#[derive(Debug, Clone)]
pub struct LoadingManager {
    /// Active loading states
    active_loaders: Arc<RwLock<HashMap<String, LoadingState>>>,
}

impl LoadingManager {
    /// Create a new loading manager
    pub fn new() -> Self {
        Self {
            active_loaders: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a loading operation
    pub async fn start_loading(&self, id: String, message: String) {
        let mut loaders = self.active_loaders.write().await;
        loaders.insert(
            id,
            LoadingState {
                message,
                start_time: std::time::Instant::now(),
                progress: None,
            },
        );
    }

    /// Update loading progress
    pub async fn update_progress(&self, id: &str, progress: f32) {
        let mut loaders = self.active_loaders.write().await;
        if let Some(loader) = loaders.get_mut(id) {
            loader.progress = Some(progress);
        }
    }

    /// Finish a loading operation
    pub async fn finish_loading(&self, id: &str) {
        let mut loaders = self.active_loaders.write().await;
        loaders.remove(id);
    }

    /// Complete a loading operation (alias for finish_loading)
    pub async fn complete_loading(&self, id: &str) {
        self.finish_loading(id).await;
    }

    /// Get loading state
    pub async fn get_loading_state(&self, id: &str) -> Option<LoadingState> {
        let loaders = self.active_loaders.read().await;
        loaders.get(id).cloned()
    }
}

/// Loading state information
#[derive(Debug, Clone)]
pub struct LoadingState {
    /// Loading message
    pub message: String,
    /// Start time
    pub start_time: std::time::Instant,
    /// Progress (0.0 to 1.0)
    pub progress: Option<f32>,
}

/// Virtual renderer for efficient rendering
#[derive(Debug, Clone)]
pub struct VirtualRenderer {
    /// Viewport dimensions
    viewport: (u16, u16),
    /// Virtual DOM root
    root: Option<VirtualNode>,
    /// Render cache
    cache: HashMap<String, ratatui::buffer::Buffer>,
}

impl VirtualRenderer {
    /// Create a new virtual renderer
    pub fn new(viewport: (u16, u16)) -> Self {
        Self {
            viewport,
            root: None,
            cache: HashMap::new(),
        }
    }

    /// Set the root virtual node
    pub fn set_root(&mut self, root: VirtualNode) {
        self.root = Some(root);
    }

    /// Render to buffer
    pub fn render(&self) -> ratatui::buffer::Buffer {
        let rect = ratatui::layout::Rect::new(0, 0, self.viewport.0, self.viewport.1);
        let mut buffer = ratatui::buffer::Buffer::empty(rect);

        if let Some(ref root) = self.root {
            self.render_node(root, &mut buffer, 0, 0);
        }

        buffer
    }

    /// Render a virtual node
    fn render_node(
        &self,
        node: &VirtualNode,
        buffer: &mut ratatui::buffer::Buffer,
        x: u16,
        y: u16,
    ) {
        // Simplified rendering logic
        match &node.content {
            VirtualContent::Text(text) => {
                for (i, ch) in text.chars().enumerate() {
                    if (x + i as u16) < self.viewport.0 && y < self.viewport.1 {
                        buffer.get_mut(x + i as u16, y).set_char(ch);
                    }
                }
            }
            VirtualContent::Container(children) => {
                let mut current_y = y;
                for child in children {
                    self.render_node(child, buffer, x, current_y);
                    current_y += 1; // Simple vertical stacking
                }
            }
        }
    }
}

/// Virtual DOM node
#[derive(Debug, Clone)]
pub struct VirtualNode {
    /// Node identifier
    pub id: String,
    /// Node content
    pub content: VirtualContent,
    /// Node style
    pub style: VirtualStyle,
}

/// Virtual node content
#[derive(Debug, Clone)]
pub enum VirtualContent {
    /// Text content
    Text(String),
    /// Container with children
    Container(Vec<VirtualNode>),
}

/// Virtual node style
#[derive(Debug, Clone, Default)]
pub struct VirtualStyle {
    /// Foreground color
    pub fg: Option<ratatui::style::Color>,
    /// Background color
    pub bg: Option<ratatui::style::Color>,
    /// Text modifiers
    pub modifiers: ratatui::style::Modifier,
}

/// Virtual list for efficient large list rendering
#[derive(Debug, Clone)]
pub struct VirtualList<T> {
    /// All items
    items: Vec<T>,
    /// Visible range
    visible_range: (usize, usize),
    /// Item height
    item_height: u16,
    /// Total height
    total_height: u16,
}

impl<T> VirtualList<T> {
    /// Create a new virtual list
    pub fn new(items: Vec<T>, item_height: u16) -> Self {
        let total_height = items.len() as u16 * item_height;
        Self {
            items,
            visible_range: (0, 0),
            item_height,
            total_height,
        }
    }

    /// Set visible range
    pub fn set_visible_range(&mut self, start: usize, end: usize) {
        self.visible_range = (start, end);
    }

    /// Get visible items
    pub fn visible_items(&self) -> &[T] {
        let (start, end) = self.visible_range;
        let end = end.min(self.items.len());
        &self.items[start..end]
    }

    /// Get total height
    pub fn total_height(&self) -> u16 {
        self.total_height
    }

    /// Get current scroll position (offset, total_items)
    pub fn scroll_position(&self) -> (usize, usize) {
        (self.visible_range.0, self.items.len())
    }

    /// Get number of visible items
    pub fn visible_items_count(&self) -> usize {
        let (start, end) = self.visible_range;
        end.saturating_sub(start)
    }

    /// Scroll by a delta amount
    pub fn scroll_by(&mut self, delta: isize) {
        let (start, end) = self.visible_range;
        let new_start = if delta > 0 {
            start.saturating_add(delta as usize)
        } else {
            start.saturating_sub((-delta) as usize)
        };

        // Ensure we don't scroll past the bounds
        let new_start = new_start.min(self.items.len().saturating_sub(end - start));
        let new_end = new_start + (end - start);

        self.visible_range = (new_start, new_end);
    }

    /// Update items in the list
    pub fn update_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.total_height = self.items.len() as u16 * self.item_height;
    }
}

/// Lazy loader for on-demand content loading
pub struct LazyLoader<T> {
    /// Loading function
    loader: Arc<dyn Fn() -> T + Send + Sync>,
    /// Cached result
    cache: Arc<RwLock<Option<T>>>,
    /// Loading state
    loading: Arc<RwLock<bool>>,
}

impl<T> std::fmt::Debug for LazyLoader<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyLoader")
            .field("loader", &"<function>")
            .field("cache", &"<cache>")
            .field("loading", &self.loading)
            .finish()
    }
}

impl<T> Clone for LazyLoader<T> {
    fn clone(&self) -> Self {
        Self {
            loader: Arc::clone(&self.loader),
            cache: Arc::clone(&self.cache),
            loading: Arc::clone(&self.loading),
        }
    }
}

impl<T> LazyLoader<T>
where
    T: Clone + Send + Sync,
{
    /// Create a new lazy loader
    pub fn new<F>(loader: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            loader: Arc::new(loader),
            cache: Arc::new(RwLock::new(None)),
            loading: Arc::new(RwLock::new(false)),
        }
    }

    /// Get the value, loading it if necessary
    pub async fn get(&self) -> T {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(ref value) = *cache {
                return value.clone();
            }
        }

        // Check if already loading
        {
            let loading = self.loading.read().await;
            if *loading {
                // Wait for loading to complete
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    let cache = self.cache.read().await;
                    if let Some(ref value) = *cache {
                        return value.clone();
                    }
                }
            }
        }

        // Start loading
        {
            let mut loading = self.loading.write().await;
            *loading = true;
        }

        // Load the value
        let value = (self.loader)();

        // Cache and return
        {
            let mut cache = self.cache.write().await;
            *cache = Some(value.clone());
        }

        {
            let mut loading = self.loading.write().await;
            *loading = false;
        }

        value
    }

    /// Check if loading is in progress
    pub async fn is_loading(&self) -> bool {
        *self.loading.read().await
    }

    /// Check if the value is fully loaded (available in cache)
    pub async fn is_fully_loaded(&self) -> bool {
        self.cache.read().await.is_some()
    }
}
