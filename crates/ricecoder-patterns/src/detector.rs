//! Main pattern detector coordinating all pattern detection

use std::path::Path;
#[cfg(feature = "parsing")]
use std::sync::Arc;

#[cfg(feature = "parsing")]
use ricecoder_parsers::{CodeParser, SyntaxTree};

use crate::{
    architectural::ArchitecturalPatternDetector,
    coding::CodingPatternDetector,
    error::{PatternError, PatternResult},
    models::{DetectedPattern, PatternDetectionConfig},
};

/// Main pattern detector that coordinates all pattern detection activities
#[derive(Debug)]
pub struct PatternDetector {
    /// Architectural pattern detector
    architectural_detector: ArchitecturalPatternDetector,
    /// Coding pattern detector
    coding_detector: CodingPatternDetector,
    /// Configuration
    config: PatternDetectionConfig,
    /// Parser for code analysis (optional)
    #[cfg(feature = "parsing")]
    parser: Arc<dyn CodeParser>,
}

impl PatternDetector {
    /// Create a new pattern detector with default configuration
    #[cfg(feature = "parsing")]
    pub fn new(parser: Arc<dyn CodeParser>) -> Self {
        Self {
            architectural_detector: ArchitecturalPatternDetector::new(),
            coding_detector: CodingPatternDetector::with_parser(Arc::clone(&parser)),
            config: PatternDetectionConfig::default(),
            parser,
        }
    }

    /// Create a new pattern detector with custom configuration
    #[cfg(feature = "parsing")]
    pub fn with_config(parser: Arc<dyn CodeParser>, config: PatternDetectionConfig) -> Self {
        Self {
            architectural_detector: ArchitecturalPatternDetector::new(),
            coding_detector: CodingPatternDetector::with_parser(Arc::clone(&parser)),
            config,
            parser,
        }
    }

    /// Create a new pattern detector without parsing capabilities
    #[cfg(not(feature = "parsing"))]
    pub fn new() -> Self {
        Self {
            architectural_detector: ArchitecturalPatternDetector::new(),
            coding_detector: CodingPatternDetector::new(),
            config: PatternDetectionConfig::default(),
        }
    }

    /// Create a new pattern detector with custom configuration (no parsing)
    #[cfg(not(feature = "parsing"))]
    pub fn with_config(config: PatternDetectionConfig) -> Self {
        Self {
            architectural_detector: ArchitecturalPatternDetector::new(),
            coding_detector: CodingPatternDetector::new(),
            config,
        }
    }

    /// Detect patterns in a codebase
    ///
    /// Analyzes the entire codebase at the given root path and returns
    /// all detected patterns above the confidence threshold.
    ///
    /// # Arguments
    ///
    /// * `root` - Root path of the codebase to analyze
    ///
    /// # Returns
    ///
    /// A vector of detected patterns, or a PatternError
    pub async fn detect(&self, root: &Path) -> PatternResult<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        // Detect architectural patterns
        if self.config.detect_architectural {
            let arch_patterns = self.architectural_detector.detect(root).await?;
            patterns.extend(arch_patterns);
        }

        // Detect design patterns
        if self.config.detect_design {
            let design_patterns = self.coding_detector.detect_design_patterns(root).await?;
            patterns.extend(design_patterns);
        }

        // Detect coding conventions
        if self.config.detect_conventions {
            let convention_patterns = self.coding_detector.detect_conventions(root).await?;
            patterns.extend(convention_patterns);
        }

        // Filter by confidence and limit
        let mut filtered_patterns: Vec<_> = patterns
            .into_iter()
            .filter(|p| p.confidence >= self.config.min_confidence)
            .take(self.config.max_patterns)
            .collect();

        // Sort by confidence (highest first)
        filtered_patterns.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(filtered_patterns)
    }

    /// Detect patterns in a specific file
    ///
    /// Analyzes a single file and returns detected patterns.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to analyze
    ///
    /// # Returns
    ///
    /// A vector of detected patterns in the file, or a PatternError
    #[cfg(feature = "parsing")]
    pub async fn detect_in_file(&self, file_path: &Path) -> PatternResult<Vec<DetectedPattern>> {
        // Parse the file
        let content = std::fs::read_to_string(file_path).map_err(|e| PatternError::Io(e))?;

        let tree = self
            .parser
            .parse(&content)
            .await
            .map_err(|e| PatternError::Parsing(e.to_string()))?;

        // Detect patterns in the syntax tree
        let mut patterns = Vec::new();

        // Design patterns in this file
        if self.config.detect_design {
            let design_patterns = self
                .coding_detector
                .detect_in_tree(&tree, file_path)
                .await?;
            patterns.extend(design_patterns);
        }

        // Filter and sort
        let mut filtered_patterns: Vec<_> = patterns
            .into_iter()
            .filter(|p| p.confidence >= self.config.min_confidence)
            .collect();

        filtered_patterns.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(filtered_patterns)
    }

    /// Detect patterns in a specific file (without parsing)
    #[cfg(not(feature = "parsing"))]
    pub async fn detect_in_file(&self, _file_path: &Path) -> PatternResult<Vec<DetectedPattern>> {
        // Return empty patterns when parsing is not available
        Ok(Vec::new())
    }

    /// Get the current configuration
    pub fn config(&self) -> &PatternDetectionConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: PatternDetectionConfig) {
        self.config = config;
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        // This would need a default parser - in practice, this should be provided
        panic!("PatternDetector::default() requires a parser - use PatternDetector::new() instead");
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ricecoder_parsers::error::ParserError;

    use super::*;

    // Mock parser for testing
    struct MockParser;

    impl CodeParser for MockParser {
        async fn parse(&self, _content: &str) -> Result<SyntaxTree, ParserError> {
            Ok(SyntaxTree {
                root: ricecoder_parsers::ASTNode {
                    node_type: ricecoder_parsers::NodeType::Root,
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
    fn test_pattern_detector_creation() {
        let parser = Arc::new(MockParser);
        let detector = PatternDetector::new(parser);
        assert!(detector.config().min_confidence >= 0.0);
        assert!(detector.config().min_confidence <= 1.0);
    }

    #[test]
    fn test_pattern_detector_with_config() {
        let parser = Arc::new(MockParser);
        let config = PatternDetectionConfig {
            min_confidence: 0.8,
            max_patterns: 10,
            detect_architectural: false,
            detect_design: true,
            detect_conventions: false,
            detect_anti_patterns: false,
        };
        let detector = PatternDetector::with_config(parser, config.clone());
        assert_eq!(detector.config().min_confidence, config.min_confidence);
        assert_eq!(detector.config().max_patterns, config.max_patterns);
    }
}
