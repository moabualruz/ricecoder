//! Architectural pattern detection

use crate::error::{PatternError, PatternResult};
use crate::models::{ArchitecturalPattern, DetectedPattern, PatternCategory, PatternLocation};
use std::collections::HashMap;
use std::path::Path;

/// Detector for architectural patterns in codebases
#[derive(Debug)]
pub struct ArchitecturalPatternDetector;

impl ArchitecturalPatternDetector {
    /// Create a new architectural pattern detector
    pub fn new() -> Self {
        ArchitecturalPatternDetector
    }

    /// Detect architectural patterns in a codebase
    ///
    /// Analyzes the project structure and code organization to identify
    /// architectural patterns like layered architecture, microservices, etc.
    ///
    /// # Arguments
    ///
    /// * `root` - Root path of the codebase
    ///
    /// # Returns
    ///
    /// A vector of detected architectural patterns
    pub async fn detect(&self, root: &Path) -> PatternResult<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        // Check for layered architecture
        if let Some(pattern) = self.detect_layered_architecture(root).await? {
            patterns.push(pattern);
        }

        // Check for microservices
        if let Some(pattern) = self.detect_microservices_pattern(root).await? {
            patterns.push(pattern);
        }

        // Check for event-driven
        if let Some(pattern) = self.detect_event_driven_pattern(root).await? {
            patterns.push(pattern);
        }

        // Check for monolithic
        if let Some(pattern) = self.detect_monolithic_pattern(root).await? {
            patterns.push(pattern);
        }

        Ok(patterns)
    }

    /// Detect layered architecture pattern
    async fn detect_layered_architecture(&self, root: &Path) -> PatternResult<Option<DetectedPattern>> {
        // Look for common layered architecture directory structure
        let layer_dirs = ["presentation", "application", "domain", "infrastructure", "interfaces"];

        let mut found_layers = Vec::new();
        for layer in &layer_dirs {
            if root.join("src").join(layer).exists() {
                found_layers.push(*layer);
            }
        }

        if found_layers.len() >= 3 {
            // Found at least 3 layers, likely layered architecture
            let confidence = (found_layers.len() as f64) / (layer_dirs.len() as f64);

            Ok(Some(DetectedPattern {
                name: "Layered Architecture".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations: vec![PatternLocation {
                    file: "project structure".to_string(),
                    line: 1,
                    column: 1,
                    snippet: format!("Found layers: {:?}", found_layers),
                }],
                metadata: HashMap::from([
                    ("pattern_type".to_string(), serde_json::json!("architectural")),
                    ("layers".to_string(), serde_json::json!(found_layers)),
                ]),
            }))
        } else {
            Ok(None)
        }
    }

    /// Detect microservices architecture pattern
    async fn detect_microservices_pattern(&self, root: &Path) -> PatternResult<Option<DetectedPattern>> {
        // Look for services directory or multiple Cargo.toml files
        let services_dir = root.join("services");
        let has_services_dir = services_dir.exists() && services_dir.is_dir();

        // Count Cargo.toml files in subdirectories
        let mut cargo_count = 0;
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                if entry.path().join("Cargo.toml").exists() {
                    cargo_count += 1;
                }
            }
        }

        if has_services_dir || cargo_count > 1 {
            let confidence = if has_services_dir { 0.9 } else { 0.7 };

            Ok(Some(DetectedPattern {
                name: "Microservices Pattern".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations: vec![PatternLocation {
                    file: "project structure".to_string(),
                    line: 1,
                    column: 1,
                    snippet: format!("Found {} service components", cargo_count),
                }],
                metadata: HashMap::from([
                    ("pattern_type".to_string(), serde_json::json!("architectural")),
                    ("service_count".to_string(), serde_json::json!(cargo_count)),
                ]),
            }))
        } else {
            Ok(None)
        }
    }

    /// Detect event-driven architecture pattern
    async fn detect_event_driven_pattern(&self, root: &Path) -> PatternResult<Option<DetectedPattern>> {
        // Look for event-related patterns
        let event_indicators = ["events", "handlers", "listeners", "pubsub", "message"];

        let mut found_indicators = Vec::new();
        for indicator in &event_indicators {
            if root.join("src").join(indicator).exists() {
                found_indicators.push(*indicator);
            }
        }

        if found_indicators.len() >= 2 {
            let confidence = (found_indicators.len() as f64) / (event_indicators.len() as f64);

            Ok(Some(DetectedPattern {
                name: "Event-Driven Pattern".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations: vec![PatternLocation {
                    file: "project structure".to_string(),
                    line: 1,
                    column: 1,
                    snippet: format!("Found event indicators: {:?}", found_indicators),
                }],
                metadata: HashMap::from([
                    ("pattern_type".to_string(), serde_json::json!("architectural")),
                    ("indicators".to_string(), serde_json::json!(found_indicators)),
                ]),
            }))
        } else {
            Ok(None)
        }
    }

    /// Detect monolithic architecture pattern
    async fn detect_monolithic_pattern(&self, root: &Path) -> PatternResult<Option<DetectedPattern>> {
        // Monolithic is the default if no other patterns are detected
        // Look for single main application structure
        let has_single_src = root.join("src").join("main.rs").exists() ||
                           root.join("src").join("lib.rs").exists();
        let has_single_cargo = root.join("Cargo.toml").exists();

        if has_single_src && has_single_cargo {
            // Check if it's not microservices or layered
            let services_dir = root.join("services");
            let has_layers = root.join("src").join("domain").exists() &&
                           root.join("src").join("application").exists();

            if !services_dir.exists() && !has_layers {
                Ok(Some(DetectedPattern {
                    name: "Monolithic Architecture".to_string(),
                    category: PatternCategory::Architectural,
                    confidence: 0.8,
                    locations: vec![PatternLocation {
                        file: "project structure".to_string(),
                        line: 1,
                        column: 1,
                        snippet: "Single application structure detected".to_string(),
                    }],
                    metadata: HashMap::from([
                        ("pattern_type".to_string(), serde_json::json!("architectural")),
                    ]),
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

impl Default for ArchitecturalPatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_detect_layered_architecture() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create layered structure
        std::fs::create_dir_all(root.join("src/domain")).unwrap();
        std::fs::create_dir_all(root.join("src/application")).unwrap();
        std::fs::create_dir_all(root.join("src/infrastructure")).unwrap();

        let detector = ArchitecturalPatternDetector::new();
        let result = detector.detect_layered_architecture(root).await.unwrap();

        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.name, "Layered Architecture");
        assert!(pattern.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_detect_microservices_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create services structure
        std::fs::create_dir_all(root.join("services/user-service")).unwrap();
        std::fs::create_dir_all(root.join("services/order-service")).unwrap();

        let detector = ArchitecturalPatternDetector::new();
        let result = detector.detect_microservices_pattern(root).await.unwrap();

        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.name, "Microservices Pattern");
    }

    #[tokio::test]
    async fn test_detect_monolithic_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create monolithic structure
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let detector = ArchitecturalPatternDetector::new();
        let result = detector.detect_monolithic_pattern(root).await.unwrap();

        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.name, "Monolithic Architecture");
    }
}