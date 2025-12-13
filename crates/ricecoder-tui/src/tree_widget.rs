//! Tree widget for file and project navigation
//!
//! This module provides a tree-based navigation widget for displaying file hierarchies,
//! project structures, and other hierarchical data with expand/collapse support.

use std::collections::HashMap;

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
pub struct TreeWidget {
    /// All nodes in the tree (stored in a map for quick access)
    nodes: HashMap<String, TreeNode>,
    /// Root node ID
    root_id: String,
    /// Currently selected node ID
    selected: Option<String>,
    /// Flat list of visible nodes (for rendering)
    visible_nodes: Vec<String>,
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

        let mut widget = Self {
            nodes,
            root_id: root_id.clone(),
            selected: None,
            visible_nodes: vec![root_id],
            title: "Navigation".to_string(),
            show_borders: true,
        };

        widget.rebuild_visible_nodes();
        widget
    }

    /// Add a node to the tree
    pub fn add_node(&mut self, parent_id: &str, node: TreeNode) {
        let node_id = node.id.clone();
        self.nodes.insert(node_id.clone(), node);

        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.add_child(node_id);
        }

        self.rebuild_visible_nodes();
    }

    /// Remove a node from the tree
    pub fn remove_node(&mut self, node_id: &str) {
        if let Some(node) = self.nodes.remove(node_id) {
            // Remove from parent
            for parent in self.nodes.values_mut() {
                parent.remove_child(node_id);
            }

            // Remove children
            for child_id in node.children {
                self.remove_node(&child_id);
            }
        }

        self.rebuild_visible_nodes();
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
            self.selected = Some(id);
        }
    }

    /// Deselect the current node
    pub fn deselect(&mut self) {
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
        }
        self.rebuild_visible_nodes();
    }

    /// Expand a node
    pub fn expand_node(&mut self, id: &str) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.expand();
        }
        self.rebuild_visible_nodes();
    }

    /// Collapse a node
    pub fn collapse_node(&mut self, id: &str) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.collapse();
        }
        self.rebuild_visible_nodes();
    }

    /// Expand all nodes
    pub fn expand_all(&mut self) {
        for node in self.nodes.values_mut() {
            if node.is_dir {
                node.expanded = true;
            }
        }
        self.rebuild_visible_nodes();
    }

    /// Collapse all nodes
    pub fn collapse_all(&mut self) {
        for node in self.nodes.values_mut() {
            node.expanded = false;
        }
        self.rebuild_visible_nodes();
    }

    /// Select the next visible node
    pub fn select_next(&mut self) {
        if let Some(selected) = &self.selected {
            if let Some(pos) = self.visible_nodes.iter().position(|id| id == selected) {
                if pos < self.visible_nodes.len() - 1 {
                    self.selected = Some(self.visible_nodes[pos + 1].clone());
                }
            }
        } else if !self.visible_nodes.is_empty() {
            self.selected = Some(self.visible_nodes[0].clone());
        }
    }

    /// Select the previous visible node
    pub fn select_prev(&mut self) {
        if let Some(selected) = &self.selected {
            if let Some(pos) = self.visible_nodes.iter().position(|id| id == selected) {
                if pos > 0 {
                    self.selected = Some(self.visible_nodes[pos - 1].clone());
                }
            }
        }
    }

    /// Get visible nodes
    pub fn visible_nodes(&self) -> &[String] {
        &self.visible_nodes
    }

    /// Get visible node count
    pub fn visible_count(&self) -> usize {
        self.visible_nodes.len()
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
        self.visible_nodes.clear();
        self.selected = None;

        let root = TreeNode::new(self.root_id.clone(), "Root", true);
        self.nodes.insert(self.root_id.clone(), root);
        self.visible_nodes.push(self.root_id.clone());
    }

    /// Rebuild the visible nodes list based on expansion state
    fn rebuild_visible_nodes(&mut self) {
        self.visible_nodes.clear();
        self.collect_visible_nodes(&self.root_id.clone(), 0);
    }

    /// Recursively collect visible nodes
    fn collect_visible_nodes(&mut self, node_id: &str, depth: usize) {
        self.visible_nodes.push(node_id.to_string());

        if let Some(node) = self.nodes.get(node_id) {
            if node.expanded {
                let children = node.children.clone();
                for child_id in children {
                    self.collect_visible_nodes(&child_id, depth + 1);
                }
            }
        }
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
}

impl Default for TreeWidget {
    fn default() -> Self {
        Self::new("root", "Root")
    }
}


