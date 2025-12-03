//! Pattern detection for identifying coding and architectural patterns in codebases

use crate::models::{DetectedPattern, PatternCategory};
use crate::codebase_scanner::ScanResult;
use crate::ResearchError;
use std::collections::HashMap;

/// Detects coding patterns and architectural patterns in the codebase
#[derive(Debug, Clone)]
pub struct PatternDetector {
    /// Minimum confidence threshold for pattern detection (0.0 to 1.0)
    pub confidence_threshold: f32,
}

impl PatternDetector {
    /// Creates a new PatternDetector with default settings
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.5,
        }
    }

    /// Creates a new PatternDetector with a custom confidence threshold
    pub fn with_threshold(confidence_threshold: f32) -> Self {
        Self {
            confidence_threshold: confidence_threshold.clamp(0.0, 1.0),
        }
    }

    /// Detects patterns in the provided scan result
    ///
    /// # Arguments
    ///
    /// * `scan_result` - The result of a codebase scan containing files and symbols
    ///
    /// # Returns
    ///
    /// A vector of detected patterns, or an error if detection fails
    pub fn detect(&self, scan_result: &ScanResult) -> Result<Vec<DetectedPattern>, ResearchError> {
        let mut patterns = Vec::new();

        // Detect architectural patterns
        let arch_patterns = self.detect_architectural_patterns(scan_result)?;
        patterns.extend(arch_patterns);

        // Detect coding patterns
        let coding_patterns = self.detect_coding_patterns(scan_result)?;
        patterns.extend(coding_patterns);

        // Filter patterns by confidence threshold
        patterns.retain(|p| p.confidence >= self.confidence_threshold);

        // Sort by confidence (highest first)
        patterns.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        Ok(patterns)
    }

    /// Detects architectural patterns in the codebase
    fn detect_architectural_patterns(&self, scan_result: &ScanResult) -> Result<Vec<DetectedPattern>, ResearchError> {
        let mut patterns = Vec::new();

        // Detect layered architecture
        if let Some(pattern) = self.detect_layered_architecture(scan_result)? {
            patterns.push(pattern);
        }

        // Detect microservices pattern
        if let Some(pattern) = self.detect_microservices_pattern(scan_result)? {
            patterns.push(pattern);
        }

        // Detect event-driven pattern
        if let Some(pattern) = self.detect_event_driven_pattern(scan_result)? {
            patterns.push(pattern);
        }

        // Detect monolithic pattern
        if let Some(pattern) = self.detect_monolithic_pattern(scan_result)? {
            patterns.push(pattern);
        }

        Ok(patterns)
    }

    /// Detects coding patterns in the codebase
    fn detect_coding_patterns(&self, scan_result: &ScanResult) -> Result<Vec<DetectedPattern>, ResearchError> {
        let mut patterns = Vec::new();

        // Detect factory pattern
        if let Some(pattern) = self.detect_factory_pattern(scan_result)? {
            patterns.push(pattern);
        }

        // Detect observer pattern
        if let Some(pattern) = self.detect_observer_pattern(scan_result)? {
            patterns.push(pattern);
        }

        // Detect strategy pattern
        if let Some(pattern) = self.detect_strategy_pattern(scan_result)? {
            patterns.push(pattern);
        }

        // Detect singleton pattern
        if let Some(pattern) = self.detect_singleton_pattern(scan_result)? {
            patterns.push(pattern);
        }

        Ok(patterns)
    }

    /// Detects layered architecture pattern
    fn detect_layered_architecture(&self, scan_result: &ScanResult) -> Result<Option<DetectedPattern>, ResearchError> {
        // Check for common layered architecture directory structure
        let has_domain = scan_result.files.iter().any(|f| {
            f.path.components().any(|c| c.as_os_str().to_string_lossy().contains("domain"))
        });

        let has_application = scan_result.files.iter().any(|f| {
            f.path.components().any(|c| c.as_os_str().to_string_lossy().contains("application"))
        });

        let has_infrastructure = scan_result.files.iter().any(|f| {
            f.path.components().any(|c| c.as_os_str().to_string_lossy().contains("infrastructure"))
        });

        let has_interfaces = scan_result.files.iter().any(|f| {
            f.path.components().any(|c| c.as_os_str().to_string_lossy().contains("interfaces"))
        });

        // Calculate confidence based on how many layers are present
        let layer_count = [has_domain, has_application, has_infrastructure, has_interfaces]
            .iter()
            .filter(|&&x| x)
            .count();

        if layer_count >= 2 {
            let confidence = (layer_count as f32) / 4.0;
            let locations = scan_result.files.iter()
                .filter(|f| {
                    f.path.components().any(|c| {
                        let name = c.as_os_str().to_string_lossy();
                        name.contains("domain") || name.contains("application") || 
                        name.contains("infrastructure") || name.contains("interfaces")
                    })
                })
                .map(|f| f.path.clone())
                .collect();

            return Ok(Some(DetectedPattern {
                name: "Layered Architecture".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations,
                description: "Project uses layered architecture with domain, application, infrastructure, and/or interfaces layers".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects microservices pattern
    fn detect_microservices_pattern(&self, scan_result: &ScanResult) -> Result<Option<DetectedPattern>, ResearchError> {
        // Check for multiple service directories or modules
        let mut service_dirs = HashMap::new();
        for file in &scan_result.files {
            if let Some(parent) = file.path.parent() {
                let parent_name = parent.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                
                if parent_name.contains("service") || parent_name.contains("services") {
                    *service_dirs.entry(parent.to_path_buf()).or_insert(0) += 1;
                }
            }
        }

        if service_dirs.len() >= 2 {
            let confidence = 0.6;
            let locations: Vec<_> = service_dirs.keys().cloned().collect();

            return Ok(Some(DetectedPattern {
                name: "Microservices Pattern".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations,
                description: "Project appears to use microservices architecture with multiple service modules".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects event-driven pattern
    fn detect_event_driven_pattern(&self, scan_result: &ScanResult) -> Result<Option<DetectedPattern>, ResearchError> {
        // Check for event-related naming patterns
        let event_count = scan_result.files.iter().filter(|f| {
            let name = f.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            name.contains("event") || name.contains("handler") || name.contains("listener")
        }).count();

        if event_count >= 3 {
            let confidence = 0.65;
            let locations = scan_result.files.iter()
                .filter(|f| {
                    let name = f.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    name.contains("event") || name.contains("handler") || name.contains("listener")
                })
                .map(|f| f.path.clone())
                .collect();

            return Ok(Some(DetectedPattern {
                name: "Event-Driven Pattern".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations,
                description: "Project uses event-driven architecture with event handlers and listeners".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects monolithic pattern
    fn detect_monolithic_pattern(&self, scan_result: &ScanResult) -> Result<Option<DetectedPattern>, ResearchError> {
        // Check if project has a single main entry point and no service separation
        let has_main = scan_result.files.iter().any(|f| {
            f.path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n == "main.rs" || n == "main.py" || n == "main.go" || n == "main.java")
                .unwrap_or(false)
        });

        let service_count = scan_result.files.iter().filter(|f| {
            f.path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains("service"))
                .unwrap_or(false)
        }).count();

        if has_main && service_count < 2 {
            let confidence = 0.55;
            let locations = vec![scan_result.files.first().map(|f| f.path.clone()).unwrap_or_default()];

            return Ok(Some(DetectedPattern {
                name: "Monolithic Architecture".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations,
                description: "Project appears to be monolithic with a single entry point".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects factory pattern
    fn detect_factory_pattern(&self, scan_result: &ScanResult) -> Result<Option<DetectedPattern>, ResearchError> {
        // Check for factory-related naming
        let factory_count = scan_result.files.iter().filter(|f| {
            f.path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_lowercase().contains("factory"))
                .unwrap_or(false)
        }).count();

        if factory_count >= 1 {
            let confidence = 0.7;
            let locations = scan_result.files.iter()
                .filter(|f| {
                    f.path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_lowercase().contains("factory"))
                        .unwrap_or(false)
                })
                .map(|f| f.path.clone())
                .collect();

            return Ok(Some(DetectedPattern {
                name: "Factory Pattern".to_string(),
                category: PatternCategory::Design,
                confidence,
                locations,
                description: "Project uses factory pattern for object creation".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects observer pattern
    fn detect_observer_pattern(&self, scan_result: &ScanResult) -> Result<Option<DetectedPattern>, ResearchError> {
        // Check for observer-related naming
        let observer_count = scan_result.files.iter().filter(|f| {
            f.path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| {
                    let lower = n.to_lowercase();
                    lower.contains("observer") || lower.contains("listener") || lower.contains("subscriber")
                })
                .unwrap_or(false)
        }).count();

        if observer_count >= 1 {
            let confidence = 0.65;
            let locations = scan_result.files.iter()
                .filter(|f| {
                    f.path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| {
                            let lower = n.to_lowercase();
                            lower.contains("observer") || lower.contains("listener") || lower.contains("subscriber")
                        })
                        .unwrap_or(false)
                })
                .map(|f| f.path.clone())
                .collect();

            return Ok(Some(DetectedPattern {
                name: "Observer Pattern".to_string(),
                category: PatternCategory::Design,
                confidence,
                locations,
                description: "Project uses observer pattern for event notification".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects strategy pattern
    fn detect_strategy_pattern(&self, scan_result: &ScanResult) -> Result<Option<DetectedPattern>, ResearchError> {
        // Check for strategy-related naming
        let strategy_count = scan_result.files.iter().filter(|f| {
            f.path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_lowercase().contains("strategy"))
                .unwrap_or(false)
        }).count();

        if strategy_count >= 1 {
            let confidence = 0.7;
            let locations = scan_result.files.iter()
                .filter(|f| {
                    f.path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_lowercase().contains("strategy"))
                        .unwrap_or(false)
                })
                .map(|f| f.path.clone())
                .collect();

            return Ok(Some(DetectedPattern {
                name: "Strategy Pattern".to_string(),
                category: PatternCategory::Design,
                confidence,
                locations,
                description: "Project uses strategy pattern for algorithm selection".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects singleton pattern
    fn detect_singleton_pattern(&self, scan_result: &ScanResult) -> Result<Option<DetectedPattern>, ResearchError> {
        // Check for singleton-related naming
        let singleton_count = scan_result.files.iter().filter(|f| {
            f.path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_lowercase().contains("singleton"))
                .unwrap_or(false)
        }).count();

        if singleton_count >= 1 {
            let confidence = 0.75;
            let locations = scan_result.files.iter()
                .filter(|f| {
                    f.path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_lowercase().contains("singleton"))
                        .unwrap_or(false)
                })
                .map(|f| f.path.clone())
                .collect();

            return Ok(Some(DetectedPattern {
                name: "Singleton Pattern".to_string(),
                category: PatternCategory::Design,
                confidence,
                locations,
                description: "Project uses singleton pattern for single instance management".to_string(),
            }));
        }

        Ok(None)
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_detector_creation() {
        let detector = PatternDetector::new();
        assert_eq!(detector.confidence_threshold, 0.5);
    }

    #[test]
    fn test_pattern_detector_with_threshold() {
        let detector = PatternDetector::with_threshold(0.8);
        assert_eq!(detector.confidence_threshold, 0.8);
    }

    #[test]
    fn test_pattern_detector_threshold_clamping() {
        let detector_low = PatternDetector::with_threshold(-0.5);
        assert_eq!(detector_low.confidence_threshold, 0.0);

        let detector_high = PatternDetector::with_threshold(1.5);
        assert_eq!(detector_high.confidence_threshold, 1.0);
    }

    #[test]
    fn test_pattern_detector_default() {
        let detector = PatternDetector::default();
        assert_eq!(detector.confidence_threshold, 0.5);
    }
}
