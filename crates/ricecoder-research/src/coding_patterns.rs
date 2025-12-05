//! Coding pattern detection

use crate::codebase_scanner::ScanResult;
use crate::models::{DetectedPattern, PatternCategory};
use crate::ResearchError;

/// Detects coding patterns and design patterns in a codebase
pub struct CodingPatternDetector;

impl CodingPatternDetector {
    /// Detects factory pattern
    ///
    /// Factory pattern is characterized by:
    /// - Factory classes/functions
    /// - Object creation abstraction
    /// - Multiple concrete implementations
    pub fn detect_factory_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let factory_count = scan_result
            .files
            .iter()
            .filter(|f| {
                f.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_lowercase().contains("factory"))
                    .unwrap_or(false)
            })
            .count();

        if factory_count >= 1 {
            let confidence = 0.75;
            let locations = scan_result
                .files
                .iter()
                .filter(|f| {
                    f.path
                        .file_name()
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
                description: "Factory pattern detected for object creation abstraction".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects observer pattern
    ///
    /// Observer pattern is characterized by:
    /// - Observer/listener interfaces
    /// - Subject/publisher classes
    /// - Event notification mechanism
    pub fn detect_observer_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let observer_count = scan_result
            .files
            .iter()
            .filter(|f| {
                f.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| {
                        let lower = n.to_lowercase();
                        lower.contains("observer")
                            || lower.contains("listener")
                            || lower.contains("subscriber")
                    })
                    .unwrap_or(false)
            })
            .count();

        if observer_count >= 1 {
            let confidence = 0.7;
            let locations = scan_result
                .files
                .iter()
                .filter(|f| {
                    f.path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| {
                            let lower = n.to_lowercase();
                            lower.contains("observer")
                                || lower.contains("listener")
                                || lower.contains("subscriber")
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
                description: "Observer pattern detected for event notification".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects strategy pattern
    ///
    /// Strategy pattern is characterized by:
    /// - Strategy interfaces/traits
    /// - Multiple concrete strategies
    /// - Context that uses strategies
    pub fn detect_strategy_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let strategy_count = scan_result
            .files
            .iter()
            .filter(|f| {
                f.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_lowercase().contains("strategy"))
                    .unwrap_or(false)
            })
            .count();

        if strategy_count >= 1 {
            let confidence = 0.75;
            let locations = scan_result
                .files
                .iter()
                .filter(|f| {
                    f.path
                        .file_name()
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
                description: "Strategy pattern detected for algorithm selection".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects singleton pattern
    ///
    /// Singleton pattern is characterized by:
    /// - Singleton class definitions
    /// - Single instance management
    /// - Global access point
    pub fn detect_singleton_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let singleton_count = scan_result
            .files
            .iter()
            .filter(|f| {
                f.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_lowercase().contains("singleton"))
                    .unwrap_or(false)
            })
            .count();

        if singleton_count >= 1 {
            let confidence = 0.8;
            let locations = scan_result
                .files
                .iter()
                .filter(|f| {
                    f.path
                        .file_name()
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
                description: "Singleton pattern detected for single instance management"
                    .to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects decorator pattern
    ///
    /// Decorator pattern is characterized by:
    /// - Decorator classes/functions
    /// - Wrapping/composition of objects
    /// - Dynamic behavior addition
    pub fn detect_decorator_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let decorator_count = scan_result
            .files
            .iter()
            .filter(|f| {
                f.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_lowercase().contains("decorator"))
                    .unwrap_or(false)
            })
            .count();

        if decorator_count >= 1 {
            let confidence = 0.75;
            let locations = scan_result
                .files
                .iter()
                .filter(|f| {
                    f.path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_lowercase().contains("decorator"))
                        .unwrap_or(false)
                })
                .map(|f| f.path.clone())
                .collect();

            return Ok(Some(DetectedPattern {
                name: "Decorator Pattern".to_string(),
                category: PatternCategory::Design,
                confidence,
                locations,
                description: "Decorator pattern detected for dynamic behavior addition".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects adapter pattern
    ///
    /// Adapter pattern is characterized by:
    /// - Adapter classes
    /// - Interface conversion
    /// - Incompatible interface bridging
    pub fn detect_adapter_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let adapter_count = scan_result
            .files
            .iter()
            .filter(|f| {
                f.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_lowercase().contains("adapter"))
                    .unwrap_or(false)
            })
            .count();

        if adapter_count >= 1 {
            let confidence = 0.75;
            let locations = scan_result
                .files
                .iter()
                .filter(|f| {
                    f.path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_lowercase().contains("adapter"))
                        .unwrap_or(false)
                })
                .map(|f| f.path.clone())
                .collect();

            return Ok(Some(DetectedPattern {
                name: "Adapter Pattern".to_string(),
                category: PatternCategory::Design,
                confidence,
                locations,
                description: "Adapter pattern detected for interface conversion".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects builder pattern
    ///
    /// Builder pattern is characterized by:
    /// - Builder classes
    /// - Fluent interface
    /// - Complex object construction
    pub fn detect_builder_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let builder_count = scan_result
            .files
            .iter()
            .filter(|f| {
                f.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_lowercase().contains("builder"))
                    .unwrap_or(false)
            })
            .count();

        if builder_count >= 1 {
            let confidence = 0.75;
            let locations = scan_result
                .files
                .iter()
                .filter(|f| {
                    f.path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_lowercase().contains("builder"))
                        .unwrap_or(false)
                })
                .map(|f| f.path.clone())
                .collect();

            return Ok(Some(DetectedPattern {
                name: "Builder Pattern".to_string(),
                category: PatternCategory::Design,
                confidence,
                locations,
                description: "Builder pattern detected for complex object construction".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects repository pattern
    ///
    /// Repository pattern is characterized by:
    /// - Repository classes/interfaces
    /// - Data access abstraction
    /// - CRUD operations
    pub fn detect_repository_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let repository_count = scan_result
            .files
            .iter()
            .filter(|f| {
                f.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_lowercase().contains("repository"))
                    .unwrap_or(false)
            })
            .count();

        if repository_count >= 1 {
            let confidence = 0.8;
            let locations = scan_result
                .files
                .iter()
                .filter(|f| {
                    f.path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_lowercase().contains("repository"))
                        .unwrap_or(false)
                })
                .map(|f| f.path.clone())
                .collect();

            return Ok(Some(DetectedPattern {
                name: "Repository Pattern".to_string(),
                category: PatternCategory::Design,
                confidence,
                locations,
                description: "Repository pattern detected for data access abstraction".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects service locator pattern
    ///
    /// Service locator pattern is characterized by:
    /// - Service locator classes
    /// - Service registry
    /// - Dependency lookup
    pub fn detect_service_locator_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let mut locator_indicators = 0;
        let mut locator_files = Vec::new();

        for file in &scan_result.files {
            let file_name = file
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            if file_name.contains("locator") {
                locator_indicators += 2;
                locator_files.push(file.path.clone());
            }
            if file_name.contains("registry") {
                locator_indicators += 1;
                locator_files.push(file.path.clone());
            }
            if file_name.contains("container") {
                locator_indicators += 1;
                locator_files.push(file.path.clone());
            }
        }

        if locator_indicators >= 2 {
            let confidence = (locator_indicators as f32 / 5.0).min(0.85);
            locator_files.sort();
            locator_files.dedup();

            return Ok(Some(DetectedPattern {
                name: "Service Locator Pattern".to_string(),
                category: PatternCategory::Design,
                confidence,
                locations: locator_files,
                description: "Service locator pattern detected for dependency lookup".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects dependency injection pattern
    ///
    /// Dependency injection pattern is characterized by:
    /// - Dependency injection containers
    /// - Constructor/setter injection
    /// - Inversion of control
    pub fn detect_dependency_injection_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let mut di_indicators = 0;
        let mut di_files = Vec::new();

        for file in &scan_result.files {
            let file_name = file
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            if file_name.contains("inject") {
                di_indicators += 2;
                di_files.push(file.path.clone());
            }
            if file_name.contains("container") {
                di_indicators += 1;
                di_files.push(file.path.clone());
            }
            if file_name.contains("provider") {
                di_indicators += 1;
                di_files.push(file.path.clone());
            }
        }

        if di_indicators >= 2 {
            let confidence = (di_indicators as f32 / 5.0).min(0.85);
            di_files.sort();
            di_files.dedup();

            return Ok(Some(DetectedPattern {
                name: "Dependency Injection Pattern".to_string(),
                category: PatternCategory::Design,
                confidence,
                locations: di_files,
                description: "Dependency injection pattern detected for inversion of control"
                    .to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects middleware pattern
    ///
    /// Middleware pattern is characterized by:
    /// - Middleware classes/functions
    /// - Request/response processing chain
    /// - Cross-cutting concerns
    pub fn detect_middleware_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let middleware_count = scan_result
            .files
            .iter()
            .filter(|f| {
                f.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_lowercase().contains("middleware"))
                    .unwrap_or(false)
            })
            .count();

        if middleware_count >= 1 {
            let confidence = 0.8;
            let locations = scan_result
                .files
                .iter()
                .filter(|f| {
                    f.path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_lowercase().contains("middleware"))
                        .unwrap_or(false)
                })
                .map(|f| f.path.clone())
                .collect();

            return Ok(Some(DetectedPattern {
                name: "Middleware Pattern".to_string(),
                category: PatternCategory::Design,
                confidence,
                locations,
                description: "Middleware pattern detected for request/response processing"
                    .to_string(),
            }));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codebase_scanner::FileMetadata;
    use crate::models::Language;
    use std::path::PathBuf;

    fn create_test_scan_result(files: Vec<(&str, Option<Language>)>) -> ScanResult {
        let files = files
            .into_iter()
            .map(|(path, lang)| FileMetadata {
                path: PathBuf::from(path),
                language: lang,
                size: 100,
                is_test: false,
            })
            .collect();

        ScanResult {
            files,
            languages: vec![],
            frameworks: vec![],
            source_dirs: vec![],
            test_dirs: vec![],
        }
    }

    #[test]
    fn test_detect_factory_pattern() {
        let scan_result = create_test_scan_result(vec![("src/factory.rs", Some(Language::Rust))]);

        let pattern = CodingPatternDetector::detect_factory_pattern(&scan_result).unwrap();
        assert!(pattern.is_some());
        let pattern = pattern.unwrap();
        assert_eq!(pattern.name, "Factory Pattern");
    }

    #[test]
    fn test_detect_observer_pattern() {
        let scan_result = create_test_scan_result(vec![("src/observer.rs", Some(Language::Rust))]);

        let pattern = CodingPatternDetector::detect_observer_pattern(&scan_result).unwrap();
        assert!(pattern.is_some());
    }

    #[test]
    fn test_detect_strategy_pattern() {
        let scan_result = create_test_scan_result(vec![("src/strategy.rs", Some(Language::Rust))]);

        let pattern = CodingPatternDetector::detect_strategy_pattern(&scan_result).unwrap();
        assert!(pattern.is_some());
    }

    #[test]
    fn test_detect_repository_pattern() {
        let scan_result =
            create_test_scan_result(vec![("src/repository.rs", Some(Language::Rust))]);

        let pattern = CodingPatternDetector::detect_repository_pattern(&scan_result).unwrap();
        assert!(pattern.is_some());
    }
}
