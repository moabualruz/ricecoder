//! Syntax tree traversal utilities and visitor patterns

use crate::error::{ParserError, ParserResult};
use crate::types::{ASTNode, NodeType, Position, Range};
use std::collections::HashMap;

/// Result of a visitor operation
pub type VisitorResult = ParserResult<VisitAction>;

/// Action to take after visiting a node
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisitAction {
    /// Continue traversal normally
    Continue,
    /// Skip children of this node
    SkipChildren,
    /// Stop traversal entirely
    Stop,
}

/// Node visitor trait for traversing AST nodes
#[async_trait::async_trait]
pub trait NodeVisitor: Send + Sync {
    /// Called before visiting a node's children
    async fn pre_visit(&mut self, node: &ASTNode) -> VisitorResult;

    /// Called after visiting a node's children
    async fn post_visit(&mut self, node: &ASTNode) -> VisitorResult;

    /// Called when visiting a leaf node
    async fn visit_leaf(&mut self, node: &ASTNode) -> VisitorResult {
        self.pre_visit(node).await
    }
}

/// Tree walker for traversing syntax trees
pub struct TreeWalker<V: NodeVisitor> {
    visitor: V,
}

impl<V: NodeVisitor> TreeWalker<V> {
    /// Create a new tree walker with a visitor
    pub fn new(visitor: V) -> Self {
        Self { visitor }
    }

    /// Walk the syntax tree starting from the root
    pub async fn walk(&mut self, root: &ASTNode) -> ParserResult<()> {
        self.walk_iterative(root).await
    }

    /// Iterative tree walking to avoid async recursion
    async fn walk_iterative(&mut self, root: &ASTNode) -> ParserResult<()> {
        let mut stack = vec![(root, false)]; // (node, post_visit_done)

        while let Some((node, post_visit_done)) = stack.pop() {
            if post_visit_done {
                // Post-visit
                match self.visitor.post_visit(node).await? {
                    VisitAction::Stop => return Ok(()),
                    _ => {}
                }
            } else {
                // Pre-visit
                match self.visitor.pre_visit(node).await? {
                    VisitAction::Stop => return Ok(()),
                    VisitAction::SkipChildren => {
                        // Skip children, do post-visit
                        stack.push((node, true));
                    }
                    VisitAction::Continue => {
                        // Push post-visit marker, then children (in reverse order)
                        stack.push((node, true));

                        // Push children in reverse order so they're visited left-to-right
                        for child in node.children.iter().rev() {
                            stack.push((child, false));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Utility functions for tree traversal
pub struct TraversalUtils;

impl TraversalUtils {
    /// Find all nodes matching a predicate
    pub fn find_nodes<F>(root: &ASTNode, predicate: F) -> Vec<&ASTNode>
    where
        F: Fn(&ASTNode) -> bool,
    {
        let mut result = Vec::new();
        Self::collect_nodes(root, &predicate, &mut result);
        result
    }

    /// Find the first node matching a predicate
    pub fn find_first<F>(root: &ASTNode, predicate: F) -> Option<&ASTNode>
    where
        F: Fn(&ASTNode) -> bool,
    {
        Self::find_nodes(root, predicate).into_iter().next()
    }

    /// Find nodes by type
    pub fn find_by_type<'a>(root: &'a ASTNode, node_type: &NodeType) -> Vec<&'a ASTNode> {
        Self::find_nodes(root, |node| &node.node_type == node_type)
    }

    /// Find nodes containing a position
    pub fn nodes_at_position(root: &ASTNode, position: Position) -> Vec<&ASTNode> {
        Self::find_nodes(root, |node| node.contains_position(position))
    }

    /// Find the most specific node at a position
    pub fn most_specific_node_at(root: &ASTNode, position: Position) -> Option<&ASTNode> {
        let candidates = Self::nodes_at_position(root, position);
        candidates
            .into_iter()
            .min_by_key(|node| node.range.end.line - node.range.start.line)
    }

    /// Get the path from root to a specific node
    pub fn node_path<'a>(root: &'a ASTNode, target: &ASTNode) -> Option<Vec<&'a ASTNode>> {
        Self::find_path(root, target, &mut Vec::new())
    }

    /// Get all nodes within a range
    pub fn nodes_in_range(root: &ASTNode, range: Range) -> Vec<&ASTNode> {
        Self::find_nodes(root, |node| node.range.overlaps(&range))
    }

    /// Count nodes by type
    pub fn count_by_type(root: &ASTNode) -> HashMap<NodeType, usize> {
        let mut counts = HashMap::new();
        Self::count_nodes_recursive(root, &mut counts);
        counts
    }

    /// Get tree depth
    pub fn tree_depth(root: &ASTNode) -> usize {
        root.children
            .iter()
            .map(|child| Self::tree_depth(child) + 1)
            .max()
            .unwrap_or(0)
    }

    /// Extract text from a range of nodes
    pub fn extract_text(nodes: &[&ASTNode]) -> String {
        let mut texts: Vec<&str> = nodes.iter().map(|node| node.text.as_str()).collect();
        texts.sort_by_key(|text| text.as_ptr()); // Sort by memory address for consistent ordering
        texts.join("")
    }

    fn collect_nodes<'a, F>(node: &'a ASTNode, predicate: &F, result: &mut Vec<&'a ASTNode>)
    where
        F: Fn(&ASTNode) -> bool,
    {
        if predicate(node) {
            result.push(node);
        }

        for child in &node.children {
            Self::collect_nodes(child, predicate, result);
        }
    }

    fn find_path<'a>(
        current: &'a ASTNode,
        target: &ASTNode,
        path: &mut Vec<&'a ASTNode>,
    ) -> Option<Vec<&'a ASTNode>> {
        path.push(current);

        if std::ptr::eq(current, target) {
            return Some(path.clone());
        }

        for child in &current.children {
            if let Some(found_path) = Self::find_path(child, target, path) {
                return Some(found_path);
            }
        }

        path.pop();
        None
    }

    fn count_nodes_recursive(node: &ASTNode, counts: &mut HashMap<NodeType, usize>) {
        *counts.entry(node.node_type.clone()).or_insert(0) += 1;

        for child in &node.children {
            Self::count_nodes_recursive(child, counts);
        }
    }
}

/// Common visitor implementations

/// Node counter visitor
pub struct NodeCounter {
    counts: HashMap<NodeType, usize>,
}

impl NodeCounter {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    pub fn counts(&self) -> &HashMap<NodeType, usize> {
        &self.counts
    }

    pub fn into_counts(self) -> HashMap<NodeType, usize> {
        self.counts
    }
}

#[async_trait::async_trait]
impl NodeVisitor for NodeCounter {
    async fn pre_visit(&mut self, node: &ASTNode) -> VisitorResult {
        *self.counts.entry(node.node_type.clone()).or_insert(0) += 1;
        Ok(VisitAction::Continue)
    }

    async fn post_visit(&mut self, _node: &ASTNode) -> VisitorResult {
        Ok(VisitAction::Continue)
    }
}

/// Node finder visitor
pub struct NodeFinder<F> {
    predicate: F,
    found_nodes: Vec<ASTNode>,
}

impl<F> NodeFinder<F>
where
    F: Fn(&ASTNode) -> bool,
{
    pub fn new(predicate: F) -> Self {
        Self {
            predicate,
            found_nodes: Vec::new(),
        }
    }

    pub fn found_nodes(&self) -> &[ASTNode] {
        &self.found_nodes
    }

    pub fn into_found_nodes(self) -> Vec<ASTNode> {
        self.found_nodes
    }
}

#[async_trait::async_trait]
impl<F> NodeVisitor for NodeFinder<F>
where
    F: Fn(&ASTNode) -> bool + Send + Sync,
{
    async fn pre_visit(&mut self, node: &ASTNode) -> VisitorResult {
        if (self.predicate)(node) {
            self.found_nodes.push(node.clone());
        }
        Ok(VisitAction::Continue)
    }

    async fn post_visit(&mut self, _node: &ASTNode) -> VisitorResult {
        Ok(VisitAction::Continue)
    }
}

/// Position-based node finder
pub struct PositionFinder {
    position: Position,
    found_node: Option<ASTNode>,
}

impl PositionFinder {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            found_node: None,
        }
    }

    pub fn found_node(self) -> Option<ASTNode> {
        self.found_node
    }
}

#[async_trait::async_trait]
impl NodeVisitor for PositionFinder {
    async fn pre_visit(&mut self, node: &ASTNode) -> VisitorResult {
        if node.contains_position(self.position) {
            // Check if this is more specific than what we found
            let is_better = match &self.found_node {
                None => true,
                Some(existing) => {
                    let existing_size = (existing.range.end.line - existing.range.start.line)
                        + (existing.range.end.column - existing.range.start.column);
                    let current_size = (node.range.end.line - node.range.start.line)
                        + (node.range.end.column - node.range.start.column);
                    current_size < existing_size
                }
            };

            if is_better {
                self.found_node = Some(node.clone());
            }
        }
        Ok(VisitAction::Continue)
    }

    async fn post_visit(&mut self, _node: &ASTNode) -> VisitorResult {
        Ok(VisitAction::Continue)
    }
}

/// Tree structure analyzer
pub struct TreeAnalyzer {
    depth: usize,
    max_depth: usize,
    node_count: usize,
    leaf_count: usize,
    branch_factor_sum: usize,
    branch_factor_count: usize,
}

impl TreeAnalyzer {
    pub fn new() -> Self {
        Self {
            depth: 0,
            max_depth: 0,
            node_count: 0,
            leaf_count: 0,
            branch_factor_sum: 0,
            branch_factor_count: 0,
        }
    }

    pub fn analyze(&mut self, root: &ASTNode) {
        self.analyze_node(root, 0);
    }

    pub fn stats(&self) -> TreeAnalysisStats {
        let avg_branch_factor = if self.branch_factor_count > 0 {
            self.branch_factor_sum as f64 / self.branch_factor_count as f64
        } else {
            0.0
        };

        TreeAnalysisStats {
            total_nodes: self.node_count,
            leaf_nodes: self.leaf_count,
            max_depth: self.max_depth,
            average_branch_factor: avg_branch_factor,
        }
    }

    fn analyze_node(&mut self, node: &ASTNode, current_depth: usize) {
        self.node_count += 1;
        self.max_depth = self.max_depth.max(current_depth);

        let child_count = node.children.len();

        if child_count == 0 {
            self.leaf_count += 1;
        } else {
            self.branch_factor_sum += child_count;
            self.branch_factor_count += 1;
        }

        for child in &node.children {
            self.analyze_node(child, current_depth + 1);
        }
    }
}

/// Tree analysis statistics
#[derive(Debug, Clone)]
pub struct TreeAnalysisStats {
    pub total_nodes: usize,
    pub leaf_nodes: usize,
    pub max_depth: usize,
    pub average_branch_factor: f64,
}

impl std::fmt::Display for TreeAnalysisStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Tree Analysis:")?;
        writeln!(f, "  Total nodes: {}", self.total_nodes)?;
        writeln!(f, "  Leaf nodes: {}", self.leaf_nodes)?;
        writeln!(f, "  Maximum depth: {}", self.max_depth)?;
        writeln!(
            f,
            "  Average branch factor: {:.2}",
            self.average_branch_factor
        )?;
        Ok(())
    }
}
