//! 60 FPS rendering pipeline for RiceCoder TUI
//!
//! This module implements a high-performance rendering system with virtual DOM diffing,
//! targeted re-renders, and virtualization for optimal performance.

use crate::model::{AppModel, StateChange};
use ratatui::prelude::*;
use ratatui::widgets::Borders;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

/// Virtual DOM node representing a UI component
#[derive(Clone, Debug, PartialEq)]
pub enum VirtualNode {
    /// Text content
    Text(String),
    /// Block widget with borders and title
    Block {
        title: Option<String>,
        borders: Borders,
        children: Vec<VirtualNode>,
    },
    /// Paragraph widget
    Paragraph {
        text: String,
        alignment: Alignment,
    },
    /// List widget
    List {
        items: Vec<String>,
        selected: Option<usize>,
    },
    /// Table widget
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        selected_row: Option<usize>,
    },
    /// Custom component with ID for diffing
    Component {
        id: String,
        component_type: ComponentType,
        props: HashMap<String, String>,
        children: Vec<VirtualNode>,
    },
}

/// Component types for virtual DOM
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ComponentType {
    ChatWidget,
    CommandPalette,
    FilePicker,
    StatusBar,
    MessageList,
    CodeEditor,
    DiffView,
    HelpDialog,
}

/// Render operation for targeted updates
#[derive(Clone, Debug)]
pub enum RenderOperation {
    /// Full screen re-render
    FullRender,
    /// Update specific component
    UpdateComponent {
        component_id: String,
        new_node: VirtualNode,
    },
    /// Update text content
    UpdateText {
        component_id: String,
        new_text: String,
    },
    /// Update selection
    UpdateSelection {
        component_id: String,
        new_selection: usize,
    },
    /// Add new component
    AddComponent {
        parent_id: String,
        component: VirtualNode,
    },
    /// Remove component
    RemoveComponent {
        component_id: String,
    },
}

/// Render batch for efficient updates
#[derive(Clone, Debug)]
pub struct RenderBatch {
    pub operations: Vec<RenderOperation>,
    pub priority: RenderPriority,
    pub timestamp: Instant,
}

/// Render priority levels
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Virtual scrolling state
#[derive(Clone, Debug)]
pub struct VirtualScroll {
    /// Total number of items
    pub total_items: usize,
    /// Number of visible items
    pub visible_items: usize,
    /// Current scroll offset
    pub offset: usize,
    /// Item height in lines
    pub item_height: usize,
    /// Cached rendered items
    pub cache: HashMap<usize, VirtualNode>,
}

/// Virtual list component for large datasets
pub struct VirtualList<T> {
    /// All items in the list
    items: Vec<T>,
    /// Current scroll state
    scroll: VirtualScroll,
    /// Render function for items
    render_fn: Box<dyn Fn(&T, usize) -> VirtualNode + Send + Sync>,
    /// Cache size limit
    cache_limit: usize,
}

impl<T> VirtualList<T>
where
    T: Clone + Send + Sync,
{
    pub fn new<F>(items: Vec<T>, visible_items: usize, render_fn: F) -> Self
    where
        F: Fn(&T, usize) -> VirtualNode + Send + Sync + 'static,
    {
        let total_items = items.len();
        Self {
            items,
            scroll: VirtualScroll {
                total_items,
                visible_items,
                offset: 0,
                item_height: 1,
                cache: HashMap::new(),
            },
            render_fn: Box::new(render_fn),
            cache_limit: visible_items * 2, // Cache 2x visible items
        }
    }

    /// Get visible items for rendering
    pub fn visible_items(&self) -> Vec<VirtualNode> {
        let start = self.scroll.offset;
        let end = (start + self.scroll.visible_items).min(self.scroll.total_items);

        (start..end)
            .map(|index| {
                if let Some(cached) = self.scroll.cache.get(&index) {
                    cached.clone()
                } else {
                    let item = &self.items[index];
                    let node = (self.render_fn)(item, index);
                    // Cache the rendered node
                    if self.scroll.cache.len() < self.cache_limit {
                        self.scroll.cache.insert(index, node.clone());
                    }
                    node
                }
            })
            .collect()
    }

    /// Scroll to specific position
    pub fn scroll_to(&mut self, offset: usize) {
        let max_offset = self.scroll.total_items.saturating_sub(self.scroll.visible_items);
        self.scroll.offset = offset.min(max_offset);
        self.evict_cache();
    }

    /// Scroll by delta
    pub fn scroll_by(&mut self, delta: isize) {
        let new_offset = if delta > 0 {
            self.scroll.offset.saturating_add(delta as usize)
        } else {
            self.scroll.offset.saturating_sub(delta.unsigned_abs())
        };
        self.scroll_to(new_offset);
    }

    /// Update items and invalidate cache
    pub fn update_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.scroll.total_items = self.items.len();
        self.scroll.cache.clear();
        // Adjust offset if it's now out of bounds
        self.scroll_to(self.scroll.offset);
    }

    /// Evict old cache entries
    fn evict_cache(&mut self) {
        if self.scroll.cache.len() >= self.scroll.cache_limit {
            // Remove items that are far from current viewport
            let current_start = self.scroll.offset;
            let current_end = current_start + self.scroll.visible_items;

            self.scroll.cache.retain(|&index, _| {
                index >= current_start.saturating_sub(self.scroll.visible_items)
                    && index <= current_end + self.scroll.visible_items
            });
        }
    }

    /// Get current scroll position
    pub fn scroll_position(&self) -> (usize, usize) {
        (self.scroll.offset, self.scroll.total_items)
    }

    /// Check if scrolling is needed
    pub fn needs_scroll(&self) -> bool {
        self.scroll.total_items > self.scroll.visible_items
    }
}

/// Virtual DOM renderer with diffing
pub struct VirtualRenderer {
    /// Current virtual DOM tree
    current_tree: Option<VirtualNode>,
    /// Previous virtual DOM tree for diffing
    previous_tree: Option<VirtualNode>,
    /// Render batch queue
    batch_queue: Vec<RenderBatch>,
    /// Performance metrics
    metrics: RenderMetrics,
}

/// Render performance metrics
#[derive(Clone, Debug, Default)]
pub struct RenderMetrics {
    pub total_renders: u64,
    pub average_render_time: Duration,
    pub skipped_renders: u64,
    pub batched_operations: u64,
}

impl VirtualRenderer {
    pub fn new() -> Self {
        Self {
            current_tree: None,
            previous_tree: None,
            batch_queue: Vec::new(),
            metrics: RenderMetrics::default(),
        }
    }

    /// Render virtual DOM tree and return operations
    pub fn render(&mut self, new_tree: VirtualNode) -> Vec<RenderOperation> {
        let start_time = Instant::now();

        self.previous_tree = self.current_tree.clone();
        self.current_tree = Some(new_tree.clone());

        let operations = if let Some(prev) = &self.previous_tree {
            self.diff_trees(prev, &new_tree)
        } else {
            vec![RenderOperation::FullRender]
        };

        // Update metrics
        let render_time = start_time.elapsed();
        self.metrics.total_renders += 1;
        self.metrics.average_render_time = if self.metrics.total_renders == 1 {
            render_time
        } else {
            (self.metrics.average_render_time + render_time) / 2
        };

        operations
    }

    /// Diff two virtual DOM trees and return minimal operations
    fn diff_trees(&self, old_tree: &VirtualNode, new_tree: &VirtualNode) -> Vec<RenderOperation> {
        let mut operations = Vec::new();

        match (old_tree, new_tree) {
            // Same nodes - no change
            (a, b) if a == b => {
                self.metrics.skipped_renders += 1;
                return vec![];
            }

            // Text content changed
            (VirtualNode::Text(old_text), VirtualNode::Text(new_text)) if old_text != new_text => {
                operations.push(RenderOperation::UpdateText {
                    component_id: "root".to_string(),
                    new_text: new_text.clone(),
                });
            }

            // Component with same ID but different props
            (VirtualNode::Component { id: old_id, props: old_props, .. },
             VirtualNode::Component { id: new_id, props: new_props, .. }) if old_id == new_id => {
                if old_props != new_props {
                    operations.push(RenderOperation::UpdateComponent {
                        component_id: old_id.clone(),
                        new_node: new_tree.clone(),
                    });
                }
            }

            // Different component types - full update
            _ => {
                operations.push(RenderOperation::FullRender);
            }
        }

        operations
    }

    /// Batch render operations for efficiency
    pub fn batch_operations(&mut self, operations: Vec<RenderOperation>, priority: RenderPriority) {
        let batch = RenderBatch {
            operations,
            priority,
            timestamp: Instant::now(),
        };

        self.batch_queue.push(batch);
        self.metrics.batched_operations += operations.len() as u64;
    }

    /// Process pending render batches
    pub fn process_batches(&mut self) -> Vec<RenderBatch> {
        // Sort by priority (highest first)
        self.batch_queue.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Return batches older than 16ms (for 60 FPS)
        let cutoff = Instant::now() - Duration::from_millis(16);
        let (ready, pending): (Vec<_>, Vec<_>) = self.batch_queue
            .drain(..)
            .partition(|batch| batch.timestamp <= cutoff);

        self.batch_queue = pending;
        ready
    }

    /// Get render metrics
    pub fn metrics(&self) -> &RenderMetrics {
        &self.metrics
    }

    /// Clear render cache
    pub fn clear_cache(&mut self) {
        self.previous_tree = None;
        self.current_tree = None;
        self.batch_queue.clear();
    }
}

/// Lazy loading manager for large datasets
pub struct LazyLoader<T> {
    /// All available items
    all_items: Vec<T>,
    /// Currently loaded items
    loaded_items: Vec<T>,
    /// Loading state
    is_loading: bool,
    /// Current loading progress (0.0 to 1.0)
    progress: f32,
    /// Batch size for loading
    batch_size: usize,
    /// Load function
    load_fn: Box<dyn Fn(Vec<T>) + Send + Sync>,
}

impl<T> LazyLoader<T>
where
    T: Clone + Send + Sync,
{
    pub fn new<F>(all_items: Vec<T>, batch_size: usize, load_fn: F) -> Self
    where
        F: Fn(Vec<T>) + Send + Sync + 'static,
    {
        Self {
            all_items,
            loaded_items: Vec::new(),
            is_loading: false,
            progress: 0.0,
            batch_size,
            load_fn: Box::new(load_fn),
        }
    }

    /// Load next batch of items
    pub async fn load_next_batch(&mut self) {
        if self.is_loading || self.loaded_items.len() >= self.all_items.len() {
            return;
        }

        self.is_loading = true;
        let start_idx = self.loaded_items.len();
        let end_idx = (start_idx + self.batch_size).min(self.all_items.len());

        let batch: Vec<T> = self.all_items[start_idx..end_idx].to_vec();
        self.loaded_items.extend(batch.clone());

        self.progress = self.loaded_items.len() as f32 / self.all_items.len() as f32;

        // Call load function
        (self.load_fn)(batch);

        self.is_loading = false;
    }

    /// Get currently loaded items
    pub fn loaded_items(&self) -> &[T] {
        &self.loaded_items
    }

    /// Get loading progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Check if loading is in progress
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    /// Check if all items are loaded
    pub fn is_fully_loaded(&self) -> bool {
        self.loaded_items.len() >= self.all_items.len()
    }

    /// Force load all remaining items
    pub async fn load_all(&mut self) {
        if self.loaded_items.len() < self.all_items.len() {
            let remaining: Vec<T> = self.all_items[self.loaded_items.len()..].to_vec();
            self.loaded_items.extend(remaining.clone());
            self.progress = 1.0;
            (self.load_fn)(remaining);
        }
    }
}

/// Memory-efficient data structures for large datasets
pub mod memory_efficient {
    use std::collections::VecDeque;

    /// Circular buffer for recent items with automatic cleanup
    pub struct CircularBuffer<T> {
        buffer: VecDeque<T>,
        capacity: usize,
    }

    impl<T> CircularBuffer<T> {
        pub fn new(capacity: usize) -> Self {
            Self {
                buffer: VecDeque::with_capacity(capacity),
                capacity,
            }
        }

        pub fn push(&mut self, item: T) {
            if self.buffer.len() >= self.capacity {
                self.buffer.pop_front();
            }
            self.buffer.push_back(item);
        }

        pub fn iter(&self) -> impl Iterator<Item = &T> {
            self.buffer.iter()
        }

        pub fn len(&self) -> usize {
            self.buffer.len()
        }

        pub fn clear(&mut self) {
            self.buffer.clear();
        }
    }

    /// LRU cache with size limits
    pub struct LruCache<K, V> {
        cache: std::collections::HashMap<K, V>,
        order: VecDeque<K>,
        capacity: usize,
    }

    impl<K, V> LruCache<K, V>
    where
        K: Clone + Eq + std::hash::Hash,
    {
        pub fn new(capacity: usize) -> Self {
            Self {
                cache: std::collections::HashMap::new(),
                order: VecDeque::new(),
                capacity,
            }
        }

        pub fn get(&mut self, key: &K) -> Option<&V> {
            if self.cache.contains_key(key) {
                // Move to front
                self.order.retain(|k| k != key);
                self.order.push_back(key.clone());
                self.cache.get(key)
            } else {
                None
            }
        }

        pub fn put(&mut self, key: K, value: V) {
            if self.cache.len() >= self.capacity {
                if let Some(oldest) = self.order.pop_front() {
                    self.cache.remove(&oldest);
                }
            }

            self.order.retain(|k| k != &key);
            self.order.push_back(key.clone());
            self.cache.insert(key, value);
        }

        pub fn len(&self) -> usize {
            self.cache.len()
        }
    }
}

