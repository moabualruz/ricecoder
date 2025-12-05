//! Architectural pattern detection

use crate::codebase_scanner::ScanResult;
use crate::models::{DetectedPattern, PatternCategory};
use crate::ResearchError;
use std::collections::HashMap;

/// Detects architectural patterns in a codebase
pub struct ArchitecturalPatternDetector;

impl ArchitecturalPatternDetector {
    /// Detects layered architecture pattern
    ///
    /// Layered architecture is characterized by:
    /// - Domain layer (business logic)
    /// - Application layer (use cases, services)
    /// - Infrastructure layer (persistence, external services)
    /// - Interfaces layer (API, CLI, UI)
    pub fn detect_layered_architecture(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let mut layer_indicators = HashMap::new();

        for file in &scan_result.files {
            let path_str = file.path.to_string_lossy().to_lowercase();

            if path_str.contains("domain") {
                *layer_indicators.entry("domain").or_insert(0) += 1;
            }
            if path_str.contains("application") {
                *layer_indicators.entry("application").or_insert(0) += 1;
            }
            if path_str.contains("infrastructure") {
                *layer_indicators.entry("infrastructure").or_insert(0) += 1;
            }
            if path_str.contains("interface") {
                *layer_indicators.entry("interface").or_insert(0) += 1;
            }
        }

        let layer_count = layer_indicators.len();
        if layer_count >= 2 {
            let confidence = (layer_count as f32) / 4.0;
            let locations = scan_result
                .files
                .iter()
                .filter(|f| {
                    let path_str = f.path.to_string_lossy().to_lowercase();
                    path_str.contains("domain")
                        || path_str.contains("application")
                        || path_str.contains("infrastructure")
                        || path_str.contains("interface")
                })
                .map(|f| f.path.clone())
                .collect();

            return Ok(Some(DetectedPattern {
                name: "Layered Architecture".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations,
                description: format!(
                    "Layered architecture detected with {} layers: {}",
                    layer_count,
                    layer_indicators
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            }));
        }

        Ok(None)
    }

    /// Detects microservices pattern
    ///
    /// Microservices architecture is characterized by:
    /// - Multiple service modules/directories
    /// - Service-specific configuration files
    /// - Independent deployment units
    pub fn detect_microservices_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let mut service_modules = HashMap::new();

        for file in &scan_result.files {
            let path_str = file.path.to_string_lossy().to_lowercase();

            // Look for service-related patterns in path components
            let is_service_path = file.path.components().any(|c| {
                let name = c.as_os_str().to_string_lossy().to_lowercase();
                name.contains("service")
            });

            if is_service_path {
                if let Some(parent) = file.path.parent() {
                    *service_modules.entry(parent.to_path_buf()).or_insert(0) += 1;
                }
            }

            // Look for service configuration files
            if path_str.ends_with("service.yaml")
                || path_str.ends_with("service.yml")
                || path_str.ends_with("service.toml")
                || path_str.ends_with("service.json")
            {
                if let Some(parent) = file.path.parent() {
                    *service_modules.entry(parent.to_path_buf()).or_insert(0) += 1;
                }
            }
        }

        if service_modules.len() >= 2 {
            let confidence = 0.65;
            let locations: Vec<_> = service_modules.keys().cloned().collect();

            return Ok(Some(DetectedPattern {
                name: "Microservices Pattern".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations,
                description: format!(
                    "Microservices architecture detected with {} service modules",
                    service_modules.len()
                ),
            }));
        }

        Ok(None)
    }

    /// Detects event-driven architecture pattern
    ///
    /// Event-driven architecture is characterized by:
    /// - Event definitions and handlers
    /// - Pub/sub or message broker patterns
    /// - Event sourcing patterns
    pub fn detect_event_driven_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let mut event_indicators = 0;
        let mut event_files = Vec::new();

        for file in &scan_result.files {
            let file_name = file
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            if file_name.contains("event") {
                event_indicators += 2;
                event_files.push(file.path.clone());
            }
            if file_name.contains("handler") {
                event_indicators += 1;
                event_files.push(file.path.clone());
            }
            if file_name.contains("listener") {
                event_indicators += 1;
                event_files.push(file.path.clone());
            }
            if file_name.contains("subscriber") {
                event_indicators += 1;
                event_files.push(file.path.clone());
            }
            if file_name.contains("publisher") {
                event_indicators += 1;
                event_files.push(file.path.clone());
            }
        }

        if event_indicators >= 3 {
            let confidence = (event_indicators as f32 / 10.0).min(0.95);
            event_files.sort();
            event_files.dedup();

            return Ok(Some(DetectedPattern {
                name: "Event-Driven Pattern".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations: event_files,
                description: "Event-driven architecture detected with event handlers, listeners, and publishers".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects monolithic architecture pattern
    ///
    /// Monolithic architecture is characterized by:
    /// - Single entry point
    /// - Tightly coupled modules
    /// - No service separation
    pub fn detect_monolithic_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let mut entry_points = Vec::new();
        let mut service_count = 0;

        for file in &scan_result.files {
            let file_name = file.path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Look for entry points
            if file_name == "main.rs"
                || file_name == "main.py"
                || file_name == "main.go"
                || file_name == "main.java"
                || file_name == "main.ts"
                || file_name == "main.js"
                || file_name == "index.ts"
                || file_name == "index.js"
            {
                entry_points.push(file.path.clone());
            }

            // Count service modules
            if file
                .path
                .to_string_lossy()
                .to_lowercase()
                .contains("service")
            {
                service_count += 1;
            }
        }

        // Monolithic if single entry point and few services
        if entry_points.len() == 1 && service_count < 3 {
            let confidence = 0.6;

            return Ok(Some(DetectedPattern {
                name: "Monolithic Architecture".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations: entry_points,
                description: "Monolithic architecture detected with single entry point and tightly coupled modules".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects serverless architecture pattern
    ///
    /// Serverless architecture is characterized by:
    /// - Function definitions (Lambda, Cloud Functions, etc.)
    /// - Serverless configuration files
    /// - Event-driven function triggers
    pub fn detect_serverless_pattern(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let mut serverless_indicators = 0;
        let mut serverless_files = Vec::new();

        for file in &scan_result.files {
            let file_name = file
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Look for serverless configuration
            if file_name == "serverless.yml"
                || file_name == "serverless.yaml"
                || file_name == "serverless.json"
                || file_name == "sam.yaml"
                || file_name == "sam.yml"
            {
                serverless_indicators += 3;
                serverless_files.push(file.path.clone());
            }

            // Look for function definitions
            if file_name.contains("lambda") || file_name.contains("function") {
                serverless_indicators += 1;
                serverless_files.push(file.path.clone());
            }
        }

        if serverless_indicators >= 2 {
            let confidence = (serverless_indicators as f32 / 6.0).min(0.9);
            serverless_files.sort();
            serverless_files.dedup();

            return Ok(Some(DetectedPattern {
                name: "Serverless Pattern".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations: serverless_files,
                description: "Serverless architecture detected with function definitions and serverless configuration".to_string(),
            }));
        }

        Ok(None)
    }

    /// Detects plugin architecture pattern
    ///
    /// Plugin architecture is characterized by:
    /// - Plugin directories/modules
    /// - Plugin interface definitions
    /// - Plugin loader/registry
    pub fn detect_plugin_architecture(
        scan_result: &ScanResult,
    ) -> Result<Option<DetectedPattern>, ResearchError> {
        let mut plugin_indicators = 0;
        let mut plugin_files = Vec::new();

        for file in &scan_result.files {
            let path_str = file.path.to_string_lossy().to_lowercase();
            let file_name = file
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            if path_str.contains("plugin") {
                plugin_indicators += 1;
                plugin_files.push(file.path.clone());
            }
            if file_name.contains("loader") || file_name.contains("registry") {
                plugin_indicators += 1;
                plugin_files.push(file.path.clone());
            }
        }

        if plugin_indicators >= 2 {
            let confidence = (plugin_indicators as f32 / 5.0).min(0.85);
            plugin_files.sort();
            plugin_files.dedup();

            return Ok(Some(DetectedPattern {
                name: "Plugin Architecture".to_string(),
                category: PatternCategory::Architectural,
                confidence,
                locations: plugin_files,
                description: "Plugin architecture detected with plugin modules and loader/registry"
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
    fn test_detect_layered_architecture() {
        let scan_result = create_test_scan_result(vec![
            ("src/domain/entity.rs", Some(Language::Rust)),
            ("src/application/service.rs", Some(Language::Rust)),
            ("src/infrastructure/repository.rs", Some(Language::Rust)),
        ]);

        let pattern =
            ArchitecturalPatternDetector::detect_layered_architecture(&scan_result).unwrap();
        assert!(pattern.is_some());
        let pattern = pattern.unwrap();
        assert_eq!(pattern.name, "Layered Architecture");
        assert!(pattern.confidence > 0.5);
    }

    #[test]
    fn test_detect_microservices_pattern() {
        let scan_result = create_test_scan_result(vec![
            ("services/user-service/main.rs", Some(Language::Rust)),
            ("services/order-service/main.rs", Some(Language::Rust)),
        ]);

        let pattern =
            ArchitecturalPatternDetector::detect_microservices_pattern(&scan_result).unwrap();
        assert!(pattern.is_some());
    }

    #[test]
    fn test_detect_event_driven_pattern() {
        let scan_result = create_test_scan_result(vec![
            ("src/events/user_event.rs", Some(Language::Rust)),
            ("src/handlers/user_handler.rs", Some(Language::Rust)),
            (
                "src/listeners/notification_listener.rs",
                Some(Language::Rust),
            ),
        ]);

        let pattern =
            ArchitecturalPatternDetector::detect_event_driven_pattern(&scan_result).unwrap();
        assert!(pattern.is_some());
    }

    #[test]
    fn test_detect_monolithic_pattern() {
        let scan_result = create_test_scan_result(vec![
            ("src/main.rs", Some(Language::Rust)),
            ("src/lib.rs", Some(Language::Rust)),
        ]);

        let pattern =
            ArchitecturalPatternDetector::detect_monolithic_pattern(&scan_result).unwrap();
        assert!(pattern.is_some());
    }
}
