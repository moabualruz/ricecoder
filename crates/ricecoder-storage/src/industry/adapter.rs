//! Industry file adapter trait and implementations
//!
//! This module defines the interface for reading and converting configuration files
//! from other AI coding tools (Cursor, Claude, Windsurf, etc.) into RiceCoder's
//! internal configuration format.

use crate::config::Config;
use crate::error::StorageResult;
use std::path::Path;

/// Trait for adapting industry-standard configuration files to RiceCoder format
pub trait IndustryFileAdapter: Send + Sync {
    /// Get the name of this adapter (e.g., "cursor", "claude", "windsurf")
    fn name(&self) -> &'static str;

    /// Check if this adapter can handle files in the given directory
    fn can_handle(&self, project_root: &Path) -> bool;

    /// Read and convert industry-standard config to RiceCoder Config
    fn read_config(&self, project_root: &Path) -> StorageResult<Config>;

    /// Get the priority of this adapter (higher = higher priority)
    /// Used when multiple adapters can handle the same directory
    fn priority(&self) -> u32 {
        0
    }
}

/// File detection result
#[derive(Debug, Clone)]
pub struct FileDetectionResult {
    /// Name of the adapter that can handle this file
    pub adapter_name: String,
    /// Priority of the adapter
    pub priority: u32,
    /// Path to the detected file
    pub file_path: std::path::PathBuf,
}

/// Industry file detector
pub struct IndustryFileDetector {
    adapters: Vec<Box<dyn IndustryFileAdapter>>,
}

impl IndustryFileDetector {
    /// Create a new detector with the given adapters
    pub fn new(adapters: Vec<Box<dyn IndustryFileAdapter>>) -> Self {
        IndustryFileDetector { adapters }
    }

    /// Detect which industry files exist in the project root
    pub fn detect_files(&self, project_root: &Path) -> Vec<FileDetectionResult> {
        let mut results = Vec::new();

        for adapter in &self.adapters {
            if adapter.can_handle(project_root) {
                // For now, we just record that this adapter can handle the directory
                // The actual file path detection is done by each adapter
                results.push(FileDetectionResult {
                    adapter_name: adapter.name().to_string(),
                    priority: adapter.priority(),
                    file_path: project_root.to_path_buf(),
                });
            }
        }

        // Sort by priority (highest first)
        results.sort_by(|a, b| b.priority.cmp(&a.priority));
        results
    }

    /// Get the highest priority adapter that can handle the project
    pub fn get_best_adapter(&self, project_root: &Path) -> Option<&dyn IndustryFileAdapter> {
        self.adapters
            .iter()
            .filter(|adapter| adapter.can_handle(project_root))
            .max_by_key(|adapter| adapter.priority())
            .map(|adapter| adapter.as_ref())
    }

    /// Register a new adapter
    pub fn register_adapter(&mut self, adapter: Box<dyn IndustryFileAdapter>) {
        self.adapters.push(adapter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAdapter {
        name: &'static str,
        priority: u32,
        can_handle: bool,
    }

    impl IndustryFileAdapter for MockAdapter {
        fn name(&self) -> &'static str {
            self.name
        }

        fn can_handle(&self, _project_root: &Path) -> bool {
            self.can_handle
        }

        fn read_config(&self, _project_root: &Path) -> StorageResult<Config> {
            Ok(Config::default())
        }

        fn priority(&self) -> u32 {
            self.priority
        }
    }

    #[test]
    fn test_detector_sorts_by_priority() {
        let adapters: Vec<Box<dyn IndustryFileAdapter>> = vec![
            Box::new(MockAdapter {
                name: "low",
                priority: 1,
                can_handle: true,
            }),
            Box::new(MockAdapter {
                name: "high",
                priority: 10,
                can_handle: true,
            }),
            Box::new(MockAdapter {
                name: "medium",
                priority: 5,
                can_handle: true,
            }),
        ];

        let detector = IndustryFileDetector::new(adapters);
        let results = detector.detect_files(Path::new("."));

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].adapter_name, "high");
        assert_eq!(results[1].adapter_name, "medium");
        assert_eq!(results[2].adapter_name, "low");
    }

    #[test]
    fn test_detector_filters_by_can_handle() {
        let adapters: Vec<Box<dyn IndustryFileAdapter>> = vec![
            Box::new(MockAdapter {
                name: "yes",
                priority: 1,
                can_handle: true,
            }),
            Box::new(MockAdapter {
                name: "no",
                priority: 10,
                can_handle: false,
            }),
        ];

        let detector = IndustryFileDetector::new(adapters);
        let results = detector.detect_files(Path::new("."));

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].adapter_name, "yes");
    }

    #[test]
    fn test_get_best_adapter() {
        let adapters: Vec<Box<dyn IndustryFileAdapter>> = vec![
            Box::new(MockAdapter {
                name: "low",
                priority: 1,
                can_handle: true,
            }),
            Box::new(MockAdapter {
                name: "high",
                priority: 10,
                can_handle: true,
            }),
        ];

        let detector = IndustryFileDetector::new(adapters);
        let best = detector.get_best_adapter(Path::new("."));

        assert!(best.is_some());
        assert_eq!(best.unwrap().name(), "high");
    }

    #[test]
    fn test_get_best_adapter_respects_can_handle() {
        let adapters: Vec<Box<dyn IndustryFileAdapter>> = vec![
            Box::new(MockAdapter {
                name: "low",
                priority: 1,
                can_handle: true,
            }),
            Box::new(MockAdapter {
                name: "high",
                priority: 10,
                can_handle: false,
            }),
        ];

        let detector = IndustryFileDetector::new(adapters);
        let best = detector.get_best_adapter(Path::new("."));

        assert!(best.is_some());
        assert_eq!(best.unwrap().name(), "low");
    }
}
