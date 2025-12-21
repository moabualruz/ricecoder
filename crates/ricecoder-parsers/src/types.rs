//! Common AST and syntax tree types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Position in source code (0-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    /// Create a new position
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Create position at start of file
    pub fn zero() -> Self {
        Self::new(0, 0)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::zero()
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.column + 1) // 1-based for display
    }
}

/// Range in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    /// Create a new range
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a range for a single position
    pub fn point(position: Position) -> Self {
        Self::new(position, position)
    }

    /// Check if this range contains a position
    pub fn contains(&self, position: Position) -> bool {
        (position.line > self.start.line
            || (position.line == self.start.line && position.column >= self.start.column))
            && (position.line < self.end.line
                || (position.line == self.end.line && position.column <= self.end.column))
    }

    /// Check if this range overlaps with another range
    pub fn overlaps(&self, other: &Range) -> bool {
        self.contains(other.start)
            || self.contains(other.end)
            || other.contains(self.start)
            || other.contains(self.end)
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

/// Node types in the AST
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    // Common node types
    Program,
    Function,
    Class,
    Method,
    Variable,
    Constant,
    Type,
    Module,
    Import,
    Export,

    // Control flow
    IfStatement,
    ForLoop,
    WhileLoop,
    TryCatch,
    SwitchCase,

    // Expressions
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    Assignment,
    Return,

    // Literals
    StringLiteral,
    NumberLiteral,
    BooleanLiteral,
    ArrayLiteral,
    ObjectLiteral,

    // Comments and documentation
    Comment,
    DocComment,

    // Language-specific or custom types
    Custom(String),
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Program => write!(f, "program"),
            NodeType::Function => write!(f, "function"),
            NodeType::Class => write!(f, "class"),
            NodeType::Method => write!(f, "method"),
            NodeType::Variable => write!(f, "variable"),
            NodeType::Constant => write!(f, "constant"),
            NodeType::Type => write!(f, "type"),
            NodeType::Module => write!(f, "module"),
            NodeType::Import => write!(f, "import"),
            NodeType::Export => write!(f, "export"),
            NodeType::IfStatement => write!(f, "if_statement"),
            NodeType::ForLoop => write!(f, "for_loop"),
            NodeType::WhileLoop => write!(f, "while_loop"),
            NodeType::TryCatch => write!(f, "try_catch"),
            NodeType::SwitchCase => write!(f, "switch_case"),
            NodeType::BinaryExpression => write!(f, "binary_expression"),
            NodeType::UnaryExpression => write!(f, "unary_expression"),
            NodeType::CallExpression => write!(f, "call_expression"),
            NodeType::Assignment => write!(f, "assignment"),
            NodeType::Return => write!(f, "return"),
            NodeType::StringLiteral => write!(f, "string_literal"),
            NodeType::NumberLiteral => write!(f, "number_literal"),
            NodeType::BooleanLiteral => write!(f, "boolean_literal"),
            NodeType::ArrayLiteral => write!(f, "array_literal"),
            NodeType::ObjectLiteral => write!(f, "object_literal"),
            NodeType::Comment => write!(f, "comment"),
            NodeType::DocComment => write!(f, "doc_comment"),
            NodeType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Abstract Syntax Tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTNode {
    /// Node type
    pub node_type: NodeType,
    /// Node range in source
    pub range: Range,
    /// Node text content
    pub text: String,
    /// Child nodes
    pub children: Vec<ASTNode>,
    /// Node properties/metadata
    pub properties: HashMap<String, serde_json::Value>,
}

impl ASTNode {
    /// Create a new AST node
    pub fn new(node_type: NodeType, range: Range, text: String) -> Self {
        Self {
            node_type,
            range,
            text,
            children: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: ASTNode) {
        self.children.push(child);
    }

    /// Get child nodes of a specific type
    pub fn children_of_type(&self, node_type: &NodeType) -> Vec<&ASTNode> {
        self.children
            .iter()
            .filter(|child| &child.node_type == node_type)
            .collect()
    }

    /// Find the first child of a specific type
    pub fn first_child_of_type(&self, node_type: &NodeType) -> Option<&ASTNode> {
        self.children
            .iter()
            .find(|child| &child.node_type == node_type)
    }

    /// Check if node contains a position
    pub fn contains_position(&self, position: Position) -> bool {
        self.range.contains(position)
    }

    /// Get all descendant nodes (depth-first)
    pub fn descendants(&self) -> Vec<&ASTNode> {
        let mut result = Vec::new();
        self.collect_descendants(&mut result);
        result
    }

    fn collect_descendants<'a>(&'a self, result: &mut Vec<&'a ASTNode>) {
        for child in &self.children {
            result.push(child);
            child.collect_descendants(result);
        }
    }

    /// Find nodes by type
    pub fn find_by_type(&self, node_type: &NodeType) -> Vec<&ASTNode> {
        let mut result = Vec::new();
        if &self.node_type == node_type {
            result.push(self);
        }
        for child in &self.children {
            result.extend(child.find_by_type(node_type));
        }
        result
    }

    /// Get node depth in the tree
    pub fn depth(&self) -> usize {
        self.children
            .iter()
            .map(|child| child.depth() + 1)
            .max()
            .unwrap_or(0)
    }
}

/// Parsed syntax tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxTree {
    /// Root node of the AST
    pub root: ASTNode,
    /// Source code language
    pub language: String,
    /// Source file path (if available)
    pub file_path: Option<String>,
    /// Parse warnings
    pub warnings: Vec<crate::error::ParserWarning>,
    /// Parse metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SyntaxTree {
    /// Create a new syntax tree
    pub fn new(root: ASTNode, language: String) -> Self {
        Self {
            root,
            language,
            file_path: None,
            warnings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the file path
    pub fn with_file_path(mut self, path: String) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: crate::error::ParserWarning) {
        self.warnings.push(warning);
    }

    /// Get all nodes of a specific type
    pub fn find_nodes(&self, node_type: &NodeType) -> Vec<&ASTNode> {
        self.root.find_by_type(node_type)
    }

    /// Find node at position
    pub fn node_at_position(&self, position: Position) -> Option<&ASTNode> {
        self.find_node_at_position(&self.root, position)
    }

    fn find_node_at_position<'a>(
        &'a self,
        node: &'a ASTNode,
        position: Position,
    ) -> Option<&'a ASTNode> {
        // Check if position is in this node
        if node.contains_position(position) {
            // Check children first (more specific)
            for child in &node.children {
                if let Some(found) = self.find_node_at_position(child, position) {
                    return Some(found);
                }
            }
            // No child contains the position, so this node does
            Some(node)
        } else {
            None
        }
    }

    /// Get tree statistics
    pub fn stats(&self) -> TreeStats {
        let total_nodes = self.count_nodes(&self.root);
        let max_depth = self.root.depth();
        let node_types = self.count_node_types(&self.root);

        TreeStats {
            total_nodes,
            max_depth,
            node_types,
            warnings_count: self.warnings.len(),
        }
    }

    fn count_nodes(&self, node: &ASTNode) -> usize {
        1 + node
            .children
            .iter()
            .map(|child| self.count_nodes(child))
            .sum::<usize>()
    }

    fn count_node_types(&self, node: &ASTNode) -> HashMap<NodeType, usize> {
        let mut counts = HashMap::new();
        self.count_node_types_recursive(node, &mut counts);
        counts
    }

    fn count_node_types_recursive(&self, node: &ASTNode, counts: &mut HashMap<NodeType, usize>) {
        *counts.entry(node.node_type.clone()).or_insert(0) += 1;
        for child in &node.children {
            self.count_node_types_recursive(child, counts);
        }
    }
}

/// Syntax tree statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeStats {
    pub total_nodes: usize,
    pub max_depth: usize,
    pub node_types: HashMap<NodeType, usize>,
    pub warnings_count: usize,
}

impl std::fmt::Display for TreeStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Syntax Tree Statistics:")?;
        writeln!(f, "  Total nodes: {}", self.total_nodes)?;
        writeln!(f, "  Maximum depth: {}", self.max_depth)?;
        writeln!(f, "  Warnings: {}", self.warnings_count)?;
        writeln!(f, "  Node types:")?;
        for (node_type, count) in &self.node_types {
            writeln!(f, "    {}: {}", node_type, count)?;
        }
        Ok(())
    }
}
