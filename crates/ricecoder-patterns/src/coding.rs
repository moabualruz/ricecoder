//! Design pattern and coding convention detection

use std::{collections::HashMap, path::Path};

#[cfg(feature = "parsing")]
use ricecoder_parsers::{ASTNode, CodeParser, NodeType, SyntaxTree};
#[cfg(feature = "parsing")]
use walkdir;

use crate::{
    error::{PatternError, PatternResult},
    models::{DesignPattern, DetectedPattern, PatternCategory, PatternLocation},
};

/// Detector for design patterns and coding conventions
#[derive(Debug)]
pub struct CodingPatternDetector {
    /// Parser for code analysis (optional)
    #[cfg(feature = "parsing")]
    parser: std::sync::Arc<dyn CodeParser>,
}

impl CodingPatternDetector {
    /// Create a new coding pattern detector
    #[cfg(feature = "parsing")]
    pub fn new() -> Self {
        // This would need a parser - in practice, pass it in
        panic!("CodingPatternDetector::new() requires a parser");
    }

    /// Create a new detector with a parser
    #[cfg(feature = "parsing")]
    pub fn with_parser(parser: std::sync::Arc<dyn CodeParser>) -> Self {
        Self { parser }
    }

    /// Create a new coding pattern detector (without parsing)
    #[cfg(not(feature = "parsing"))]
    pub fn new() -> Self {
        Self {}
    }

    /// Detect design patterns in a codebase
    #[cfg(feature = "parsing")]
    pub async fn detect_design_patterns(&self, root: &Path) -> PatternResult<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        // Walk through source files
        let walker = walkdir::WalkDir::new(root.join("src"))
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"));

        for entry in walker {
            let path = entry.path();
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(tree) = self.parser.parse(&content).await {
                    let file_patterns = self.detect_in_tree(&tree, path).await?;
                    patterns.extend(file_patterns);
                }
            }
        }

        Ok(patterns)
    }

    /// Detect design patterns in a codebase (without parsing)
    #[cfg(not(feature = "parsing"))]
    pub async fn detect_design_patterns(
        &self,
        _root: &Path,
    ) -> PatternResult<Vec<DetectedPattern>> {
        // Return empty patterns when parsing is not available
        Ok(Vec::new())
    }

    /// Detect patterns in a syntax tree
    #[cfg(feature = "parsing")]
    pub async fn detect_in_tree(
        &self,
        tree: &SyntaxTree,
        file_path: &Path,
    ) -> PatternResult<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        // Detect factory pattern
        if let Some(pattern) = self.detect_factory_pattern(tree, file_path).await? {
            patterns.push(pattern);
        }

        // Detect observer pattern
        if let Some(pattern) = self.detect_observer_pattern(tree, file_path).await? {
            patterns.push(pattern);
        }

        // Detect repository pattern
        if let Some(pattern) = self.detect_repository_pattern(tree, file_path).await? {
            patterns.push(pattern);
        }

        Ok(patterns)
    }

    /// Detect factory pattern
    #[cfg(feature = "parsing")]
    async fn detect_factory_pattern(
        &self,
        tree: &SyntaxTree,
        file_path: &Path,
    ) -> PatternResult<Option<DetectedPattern>> {
        // Look for functions that create objects based on parameters
        let mut factory_functions = Vec::new();

        self.walk_tree(&tree.root, &mut |node| {
            if let NodeType::Function = node.node_type {
                if node.text.contains("create")
                    || node.text.contains("build")
                    || node.text.contains("make")
                    || node.text.contains("new")
                {
                    // Check if it has conditional logic for different types
                    if self.has_conditional_logic(node) {
                        factory_functions.push(node.clone());
                    }
                }
            }
        });

        if !factory_functions.is_empty() {
            Ok(Some(DetectedPattern {
                name: "Factory Pattern".to_string(),
                category: PatternCategory::Design,
                confidence: 0.8,
                locations: factory_functions
                    .into_iter()
                    .map(|node| PatternLocation {
                        file: file_path.to_string_lossy().to_string(),
                        line: node.position.line,
                        column: node.position.column,
                        snippet: node.text.chars().take(50).collect(),
                    })
                    .collect(),
                metadata: HashMap::from([
                    ("pattern_type".to_string(), serde_json::json!("design")),
                    (
                        "functions_found".to_string(),
                        serde_json::json!(factory_functions.len()),
                    ),
                ]),
            }))
        } else {
            Ok(None)
        }
    }

    /// Detect observer pattern
    #[cfg(feature = "parsing")]
    async fn detect_observer_pattern(
        &self,
        tree: &SyntaxTree,
        file_path: &Path,
    ) -> PatternResult<Option<DetectedPattern>> {
        // Look for observer-related patterns
        let mut observer_indicators = Vec::new();

        self.walk_tree(&tree.root, &mut |node| {
            let text = node.text.to_lowercase();
            if text.contains("subscribe")
                || text.contains("notify")
                || text.contains("observer")
                || text.contains("listener")
            {
                observer_indicators.push(node.clone());
            }
        });

        if observer_indicators.len() >= 2 {
            Ok(Some(DetectedPattern {
                name: "Observer Pattern".to_string(),
                category: PatternCategory::Design,
                confidence: 0.7,
                locations: observer_indicators
                    .into_iter()
                    .map(|node| PatternLocation {
                        file: file_path.to_string_lossy().to_string(),
                        line: node.position.line,
                        column: node.position.column,
                        snippet: node.text.chars().take(50).collect(),
                    })
                    .collect(),
                metadata: HashMap::from([
                    ("pattern_type".to_string(), serde_json::json!("design")),
                    (
                        "indicators_found".to_string(),
                        serde_json::json!(observer_indicators.len()),
                    ),
                ]),
            }))
        } else {
            Ok(None)
        }
    }

    /// Detect repository pattern
    #[cfg(feature = "parsing")]
    async fn detect_repository_pattern(
        &self,
        tree: &SyntaxTree,
        file_path: &Path,
    ) -> PatternResult<Option<DetectedPattern>> {
        // Look for repository-related patterns
        let mut repository_indicators = Vec::new();

        self.walk_tree(&tree.root, &mut |node| {
            let text = node.text.to_lowercase();
            if text.contains("repository")
                || text.contains("save")
                || text.contains("find")
                || text.contains("delete")
            {
                repository_indicators.push(node.clone());
            }
        });

        if repository_indicators.len() >= 3 {
            Ok(Some(DetectedPattern {
                name: "Repository Pattern".to_string(),
                category: PatternCategory::Design,
                confidence: 0.75,
                locations: repository_indicators
                    .into_iter()
                    .map(|node| PatternLocation {
                        file: file_path.to_string_lossy().to_string(),
                        line: node.position.line,
                        column: node.position.column,
                        snippet: node.text.chars().take(50).collect(),
                    })
                    .collect(),
                metadata: HashMap::from([
                    ("pattern_type".to_string(), serde_json::json!("design")),
                    (
                        "indicators_found".to_string(),
                        serde_json::json!(repository_indicators.len()),
                    ),
                ]),
            }))
        } else {
            Ok(None)
        }
    }

    /// Detect coding conventions in a codebase
    pub async fn detect_conventions(&self, _root: &Path) -> PatternResult<Vec<DetectedPattern>> {
        // This would analyze naming conventions, documentation, etc.
        // For now, return empty vec
        Ok(Vec::new())
    }

    /// Helper to walk the AST tree
    #[cfg(feature = "parsing")]
    fn walk_tree<F>(&self, node: &ASTNode, visitor: &mut F)
    where
        F: FnMut(&ASTNode),
    {
        visitor(node);
        for child in &node.children {
            self.walk_tree(child, visitor);
        }
    }

    /// Check if a node has conditional logic
    #[cfg(feature = "parsing")]
    fn has_conditional_logic(&self, node: &ASTNode) -> bool {
        for child in &node.children {
            match child.node_type {
                NodeType::If | NodeType::Match => return true,
                _ => {}
            }
            if self.has_conditional_logic(child) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ricecoder_parsers::error::ParserError;

    use super::*;

    // Mock parser for testing
    struct MockParser;

    impl Parser for MockParser {
        fn parse(&self, _content: &str) -> Result<SyntaxTree, ParserError> {
            Ok(SyntaxTree {
                root: ASTNode {
                    node_type: NodeType::Root,
                    text: "".to_string(),
                    children: vec![],
                    position: ricecoder_parsers::Position { line: 1, column: 1 },
                    range: ricecoder_parsers::Range {
                        start: ricecoder_parsers::Position { line: 1, column: 1 },
                        end: ricecoder_parsers::Position { line: 1, column: 1 },
                    },
                },
            })
        }
    }

    #[test]
    fn test_coding_pattern_detector_creation() {
        let parser = Arc::new(MockParser);
        let detector = CodingPatternDetector::with_parser(parser);
        // Just test that it can be created
        assert!(true);
    }
}
