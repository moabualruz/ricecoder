//! Tree widget wrapper for tui-tree-widget
//!
//! This module provides a wrapper around the `tui-tree-widget` crate to maintain
//! API compatibility while leveraging the external library's implementation.

use std::collections::HashMap;
use tui_tree_widget::{Tree, TreeItem as TuiTreeItem, TreeState as TuiTreeState};

/// Tree node representing a file or directory
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// Node identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Whether this is a directory/folder
    pub is_dir: bool,
    /// Child node IDs
    pub children: Vec<String>,
    /// Whether the node is expanded
    pub expanded: bool,
    /// Depth in the tree (for indentation)
    pub depth: usize,
}

impl TreeNode {
    /// Create a new tree node
    pub fn new(id: impl Into<String>, name: impl Into<String>, is_dir: bool) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            is_dir,
            children: Vec::new(),
            expanded: false,
            depth: 0,
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child_id: String) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Remove a child node
    pub fn remove_child(&mut self, child_id: &str) {
        self.children.retain(|id| id != child_id);
    }

    /// Toggle expansion state
    pub fn toggle_expanded(&mut self) {
        if self.is_dir {
            self.expanded = !self.expanded;
        }
    }

    /// Expand the node
    pub fn expand(&mut self) {
        if self.is_dir {
            self.expanded = true;
        }
    }

    /// Collapse the node
    pub fn collapse(&mut self) {
        self.expanded = false;
    }

    /// Check if node has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// Tree widget for hierarchical navigation
/// 
/// This is a wrapper around `tui-tree-widget::Tree` that maintains compatibility
/// with the existing RiceCoder TUI API.
pub struct TreeWidget {
    /// All nodes in the tree (stored in a map for quick access)
    nodes: HashMap<String, TreeNode>,
    /// Root node ID
    root_id: String,
    /// Tree state for rendering
    state: TuiTreeState<String>,
    /// Currently selected node ID
    selected: Option<String>,
    /// Title for the widget
    title: String,
    /// Whether to show borders
    show_borders: bool,
}

impl TreeWidget {
    /// Create a new tree widget
    pub fn new(root_id: impl Into<String>, root_name: impl Into<String>) -> Self {
        let root_id = root_id.into();
        let mut nodes = HashMap::new();

        let root = TreeNode::new(root_id.clone(), root_name, true);
        nodes.insert(root_id.clone(), root);

        Self {
            nodes,
            root_id,
            state: TuiTreeState::default(),
            selected: None,
            title: "Navigation".to_string(),
            show_borders: true,
        }
    }

    /// Add a node to the tree
    pub fn add_node(&mut self, parent_id: &str, mut node: TreeNode) {
        let node_id = node.id.clone();
        
        // Set depth based on parent
        if let Some(parent) = self.nodes.get(parent_id) {
            node.depth = parent.depth + 1;
        }
        
        self.nodes.insert(node_id.clone(), node);

        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.add_child(node_id);
        }
    }

    /// Remove a node from the tree
    pub fn remove_node(&mut self, node_id: &str) {
        if let Some(node) = self.nodes.remove(node_id) {
            // Remove from parent
            for parent in self.nodes.values_mut() {
                parent.remove_child(node_id);
            }

            // Remove children recursively
            for child_id in node.children {
                self.remove_node(&child_id);
            }
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&TreeNode> {
        self.nodes.get(id)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut TreeNode> {
        self.nodes.get_mut(id)
    }

    /// Select a node
    pub fn select(&mut self, id: String) {
        if self.nodes.contains_key(&id) {
            self.state.select(vec![id.clone()]);
            self.selected = Some(id);
        }
    }

    /// Deselect the current node
    pub fn deselect(&mut self) {
        self.state.select(Vec::new());
        self.selected = None;
    }

    /// Get the selected node ID
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Get the selected node
    pub fn selected_node(&self) -> Option<&TreeNode> {
        self.selected.as_ref().and_then(|id| self.nodes.get(id))
    }

    /// Toggle expansion of a node
    pub fn toggle_node(&mut self, id: &str) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.toggle_expanded();
            
            // Update tui-tree-widget state
            if node.expanded {
                self.state.open(vec![id.to_string()]);
            } else {
                self.state.close(&[id.to_string()]);
            }
        }
    }

    /// Expand a node
    pub fn expand_node(&mut self, id: &str) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.expand();
            self.state.open(vec![id.to_string()]);
        }
    }

    /// Collapse a node
    pub fn collapse_node(&mut self, id: &str) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.collapse();
            self.state.close(&[id.to_string()]);
        }
    }

    /// Expand all nodes
    pub fn expand_all(&mut self) {
        for node in self.nodes.values_mut() {
            if node.is_dir {
                node.expanded = true;
            }
        }
    }

    /// Collapse all nodes
    pub fn collapse_all(&mut self) {
        for node in self.nodes.values_mut() {
            node.expanded = false;
        }
        self.state.close_all();
    }

    /// Select the next visible node
    pub fn select_next(&mut self) {
        let items = self.build_tree_items();
        self.state.key_down(&items);
    }

    /// Select the previous visible node
    pub fn select_prev(&mut self) {
        let items = self.build_tree_items();
        self.state.key_up(&items);
    }

    /// Get visible nodes (IDs of currently visible items)
    pub fn visible_nodes(&self) -> Vec<String> {
        // Build tree and get flattened view
        let tree_items = self.build_tree_items();
        let flattened = self.state.flatten(&tree_items);
        flattened
            .into_iter()
            .filter_map(|f| f.identifier.first().map(|s| s.clone()))
            .collect()
    }

    /// Get visible node count
    pub fn visible_count(&self) -> usize {
        self.visible_nodes().len()
    }

    /// Set the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Set whether to show borders
    pub fn set_show_borders(&mut self, show: bool) {
        self.show_borders = show;
    }

    /// Clear the tree
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.state = TuiTreeState::default();
        self.selected = None;

        let root = TreeNode::new(self.root_id.clone(), "Root", true);
        self.nodes.insert(self.root_id.clone(), root);
    }

    /// Get the display text for a node (with indentation and icon)
    pub fn get_display_text(&self, node_id: &str) -> String {
        if let Some(node) = self.nodes.get(node_id) {
            let indent = "  ".repeat(node.depth);
            let icon = if node.is_dir {
                if node.expanded {
                    "▼"
                } else {
                    "▶"
                }
            } else {
                "•"
            };

            format!("{}{} {}", indent, icon, node.name)
        } else {
            String::new()
        }
    }

    /// Get all nodes
    pub fn nodes(&self) -> &HashMap<String, TreeNode> {
        &self.nodes
    }

    /// Get the root node ID
    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    /// Build tui-tree-widget TreeItems from our nodes
    fn build_tree_items(&self) -> Vec<TuiTreeItem<'static, String>> {
        self.build_tree_items_recursive(&self.root_id)
    }

    /// Recursively build tree items
    fn build_tree_items_recursive(&self, node_id: &str) -> Vec<TuiTreeItem<'static, String>> {
        if let Some(node) = self.nodes.get(node_id) {
            let mut children = Vec::new();
            
            for child_id in &node.children {
                let mut child_items = self.build_tree_items_recursive(child_id);
                children.append(&mut child_items);
            }

            let text = self.get_display_text(node_id);
            let item = if children.is_empty() {
                TuiTreeItem::new_leaf(node_id.to_string(), text)
            } else {
                match TuiTreeItem::new(node_id.to_string(), text, children) {
                    Ok(item) => item,
                    Err(_) => return Vec::new(),
                }
            };

            vec![item]
        } else {
            Vec::new()
        }
    }
}

impl Default for TreeWidget {
    fn default() -> Self {
        Self::new("root", "Root")
    }
}
