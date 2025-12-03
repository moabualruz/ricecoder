//! Property-based tests for pattern detection stability
//! **Feature: ricecoder-research, Property 2: Pattern Detection Stability**
//! **Validates: Requirements 1.3, 1.4**

use proptest::prelude::*;
use ricecoder_research::{PatternDetector, CodebaseScanner, ArchitecturalPatternDetector, CodingPatternDetector};
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Generators for property testing
// ============================================================================

/// Create a layered architecture project structure
fn create_layered_architecture_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    std::fs::create_dir_all(root.join("src/domain")).ok();
    std::fs::create_dir_all(root.join("src/application")).ok();
    std::fs::create_dir_all(root.join("src/infrastructure")).ok();
    std::fs::create_dir_all(root.join("src/interfaces")).ok();
    
    std::fs::write(root.join("src/domain/entity.rs"), "pub struct Entity;").ok();
    std::fs::write(root.join("src/application/service.rs"), "pub struct Service;").ok();
    std::fs::write(root.join("src/infrastructure/repository.rs"), "pub struct Repository;").ok();
    std::fs::write(root.join("src/interfaces/api.rs"), "pub struct Api;").ok();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").ok();
    
    root.to_path_buf()
}

/// Create a microservices project structure
fn create_microservices_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    std::fs::create_dir_all(root.join("services/user-service/src")).ok();
    std::fs::create_dir_all(root.join("services/order-service/src")).ok();
    
    std::fs::write(root.join("services/user-service/Cargo.toml"), "[package]\nname = \"user-service\"\n").ok();
    std::fs::write(root.join("services/order-service/Cargo.toml"), "[package]\nname = \"order-service\"\n").ok();
    std::fs::write(root.join("services/user-service/src/main.rs"), "fn main() {}").ok();
    std::fs::write(root.join("services/order-service/src/main.rs"), "fn main() {}").ok();
    
    root.to_path_buf()
}

/// Create an event-driven project structure
fn create_event_driven_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    std::fs::create_dir_all(root.join("src")).ok();
    
    std::fs::write(root.join("src/events.rs"), "pub struct Event;").ok();
    std::fs::write(root.join("src/event_handler.rs"), "pub struct EventHandler;").ok();
    std::fs::write(root.join("src/event_listener.rs"), "pub struct EventListener;").ok();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").ok();
    
    root.to_path_buf()
}

/// Create a project with design patterns
fn create_design_patterns_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    std::fs::create_dir_all(root.join("src")).ok();
    
    std::fs::write(root.join("src/factory.rs"), "pub struct Factory;").ok();
    std::fs::write(root.join("src/observer.rs"), "pub struct Observer;").ok();
    std::fs::write(root.join("src/strategy.rs"), "pub struct Strategy;").ok();
    std::fs::write(root.join("src/repository.rs"), "pub struct Repository;").ok();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").ok();
    
    root.to_path_buf()
}

/// Create a monolithic project structure
fn create_monolithic_project(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path();
    std::fs::create_dir_all(root.join("src")).ok();
    
    std::fs::write(root.join("src/main.rs"), "fn main() {}").ok();
    std::fs::write(root.join("src/lib.rs"), "pub mod utils;").ok();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").ok();
    
    root.to_path_buf()
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    /// Property: Pattern detection is stable for unchanged code
    /// For any codebase, detecting patterns multiple times should produce consistent results
    #[test]
    fn prop_pattern_detection_stability_layered(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_layered_architecture_project(&temp_dir);

        let scan_result1 = CodebaseScanner::scan(&root).unwrap();
        let scan_result2 = CodebaseScanner::scan(&root).unwrap();

        let detector = PatternDetector::new();
        let patterns1 = detector.detect(&scan_result1).unwrap();
        let patterns2 = detector.detect(&scan_result2).unwrap();

        // Same number of patterns detected
        prop_assert_eq!(patterns1.len(), patterns2.len());

        // Same pattern names in same order
        for (p1, p2) in patterns1.iter().zip(patterns2.iter()) {
            prop_assert_eq!(&p1.name, &p2.name);
            prop_assert_eq!(p1.category, p2.category);
            // Confidence should be very close (within floating point precision)
            prop_assert!((p1.confidence - p2.confidence).abs() < 0.001);
        }
    }

    /// Property: Architectural pattern detection is stable
    /// For any codebase with architectural patterns, detecting them multiple times should be consistent
    #[test]
    fn prop_architectural_pattern_stability_microservices(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_microservices_project(&temp_dir);

        let scan_result1 = CodebaseScanner::scan(&root).unwrap();
        let scan_result2 = CodebaseScanner::scan(&root).unwrap();

        let pattern1 = ArchitecturalPatternDetector::detect_microservices_pattern(&scan_result1).unwrap();
        let pattern2 = ArchitecturalPatternDetector::detect_microservices_pattern(&scan_result2).unwrap();

        // Both should detect or both should not detect
        prop_assert_eq!(pattern1.is_some(), pattern2.is_some());

        if let (Some(p1), Some(p2)) = (pattern1, pattern2) {
            prop_assert_eq!(&p1.name, &p2.name);
            prop_assert_eq!(p1.category, p2.category);
            prop_assert!((p1.confidence - p2.confidence).abs() < 0.001);
        }
    }

    /// Property: Event-driven pattern detection is stable
    /// For any event-driven codebase, detecting patterns multiple times should be consistent
    #[test]
    fn prop_event_driven_pattern_stability(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_event_driven_project(&temp_dir);

        let scan_result1 = CodebaseScanner::scan(&root).unwrap();
        let scan_result2 = CodebaseScanner::scan(&root).unwrap();

        let pattern1 = ArchitecturalPatternDetector::detect_event_driven_pattern(&scan_result1).unwrap();
        let pattern2 = ArchitecturalPatternDetector::detect_event_driven_pattern(&scan_result2).unwrap();

        // Both should detect or both should not detect
        prop_assert_eq!(pattern1.is_some(), pattern2.is_some());

        if let (Some(p1), Some(p2)) = (pattern1, pattern2) {
            prop_assert_eq!(&p1.name, &p2.name);
            prop_assert_eq!(p1.category, p2.category);
            prop_assert!((p1.confidence - p2.confidence).abs() < 0.001);
        }
    }

    /// Property: Design pattern detection is stable
    /// For any codebase with design patterns, detecting them multiple times should be consistent
    #[test]
    fn prop_design_pattern_stability(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_design_patterns_project(&temp_dir);

        let scan_result1 = CodebaseScanner::scan(&root).unwrap();
        let scan_result2 = CodebaseScanner::scan(&root).unwrap();

        let factory1 = CodingPatternDetector::detect_factory_pattern(&scan_result1).unwrap();
        let factory2 = CodingPatternDetector::detect_factory_pattern(&scan_result2).unwrap();

        // Both should detect or both should not detect
        prop_assert_eq!(factory1.is_some(), factory2.is_some());

        if let (Some(p1), Some(p2)) = (factory1, factory2) {
            prop_assert_eq!(&p1.name, &p2.name);
            prop_assert_eq!(p1.category, p2.category);
            prop_assert!((p1.confidence - p2.confidence).abs() < 0.001);
        }
    }

    /// Property: Repository pattern detection is stable
    /// For any codebase with repository pattern, detecting it multiple times should be consistent
    #[test]
    fn prop_repository_pattern_stability(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_design_patterns_project(&temp_dir);

        let scan_result1 = CodebaseScanner::scan(&root).unwrap();
        let scan_result2 = CodebaseScanner::scan(&root).unwrap();

        let repo1 = CodingPatternDetector::detect_repository_pattern(&scan_result1).unwrap();
        let repo2 = CodingPatternDetector::detect_repository_pattern(&scan_result2).unwrap();

        // Both should detect or both should not detect
        prop_assert_eq!(repo1.is_some(), repo2.is_some());

        if let (Some(p1), Some(p2)) = (repo1, repo2) {
            prop_assert_eq!(&p1.name, &p2.name);
            prop_assert_eq!(p1.category, p2.category);
            prop_assert!((p1.confidence - p2.confidence).abs() < 0.001);
        }
    }

    /// Property: Monolithic pattern detection is stable
    /// For any monolithic codebase, detecting patterns multiple times should be consistent
    #[test]
    fn prop_monolithic_pattern_stability(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_monolithic_project(&temp_dir);

        let scan_result1 = CodebaseScanner::scan(&root).unwrap();
        let scan_result2 = CodebaseScanner::scan(&root).unwrap();

        let pattern1 = ArchitecturalPatternDetector::detect_monolithic_pattern(&scan_result1).unwrap();
        let pattern2 = ArchitecturalPatternDetector::detect_monolithic_pattern(&scan_result2).unwrap();

        // Both should detect or both should not detect
        prop_assert_eq!(pattern1.is_some(), pattern2.is_some());

        if let (Some(p1), Some(p2)) = (pattern1, pattern2) {
            prop_assert_eq!(&p1.name, &p2.name);
            prop_assert_eq!(p1.category, p2.category);
            prop_assert!((p1.confidence - p2.confidence).abs() < 0.001);
        }
    }

    /// Property: Layered architecture detection is stable
    /// For any layered architecture codebase, detecting patterns multiple times should be consistent
    #[test]
    fn prop_layered_architecture_stability(_seed in 0u32..1) {
        let temp_dir = TempDir::new().unwrap();
        let root = create_layered_architecture_project(&temp_dir);

        let scan_result1 = CodebaseScanner::scan(&root).unwrap();
        let scan_result2 = CodebaseScanner::scan(&root).unwrap();

        let pattern1 = ArchitecturalPatternDetector::detect_layered_architecture(&scan_result1).unwrap();
        let pattern2 = ArchitecturalPatternDetector::detect_layered_architecture(&scan_result2).unwrap();

        // Both should detect or both should not detect
        prop_assert_eq!(pattern1.is_some(), pattern2.is_some());

        if let (Some(p1), Some(p2)) = (pattern1, pattern2) {
            prop_assert_eq!(&p1.name, &p2.name);
            prop_assert_eq!(p1.category, p2.category);
            prop_assert!((p1.confidence - p2.confidence).abs() < 0.001);
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[test]
fn test_pattern_detector_creation() {
    let detector = PatternDetector::new();
    assert_eq!(detector.confidence_threshold, 0.5);
}

#[test]
fn test_pattern_detector_with_custom_threshold() {
    let detector = PatternDetector::with_threshold(0.8);
    assert_eq!(detector.confidence_threshold, 0.8);
}

#[test]
fn test_layered_architecture_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_layered_architecture_project(&temp_dir);

    let scan_result = CodebaseScanner::scan(&root).unwrap();
    let pattern = ArchitecturalPatternDetector::detect_layered_architecture(&scan_result).unwrap();
    
    assert!(pattern.is_some());
    let pattern = pattern.unwrap();
    assert_eq!(pattern.name, "Layered Architecture");
}

#[test]
fn test_microservices_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_microservices_project(&temp_dir);

    let scan_result = CodebaseScanner::scan(&root).unwrap();
    let pattern = ArchitecturalPatternDetector::detect_microservices_pattern(&scan_result).unwrap();
    
    assert!(pattern.is_some());
    let pattern = pattern.unwrap();
    assert_eq!(pattern.name, "Microservices Pattern");
}

#[test]
fn test_event_driven_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_event_driven_project(&temp_dir);

    let scan_result = CodebaseScanner::scan(&root).unwrap();
    let pattern = ArchitecturalPatternDetector::detect_event_driven_pattern(&scan_result).unwrap();
    
    assert!(pattern.is_some());
    let pattern = pattern.unwrap();
    assert_eq!(pattern.name, "Event-Driven Pattern");
}

#[test]
fn test_factory_pattern_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_design_patterns_project(&temp_dir);

    let scan_result = CodebaseScanner::scan(&root).unwrap();
    let pattern = CodingPatternDetector::detect_factory_pattern(&scan_result).unwrap();
    
    assert!(pattern.is_some());
    let pattern = pattern.unwrap();
    assert_eq!(pattern.name, "Factory Pattern");
}

#[test]
fn test_observer_pattern_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_design_patterns_project(&temp_dir);

    let scan_result = CodebaseScanner::scan(&root).unwrap();
    let pattern = CodingPatternDetector::detect_observer_pattern(&scan_result).unwrap();
    
    assert!(pattern.is_some());
    let pattern = pattern.unwrap();
    assert_eq!(pattern.name, "Observer Pattern");
}

#[test]
fn test_repository_pattern_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_design_patterns_project(&temp_dir);

    let scan_result = CodebaseScanner::scan(&root).unwrap();
    let pattern = CodingPatternDetector::detect_repository_pattern(&scan_result).unwrap();
    
    assert!(pattern.is_some());
    let pattern = pattern.unwrap();
    assert_eq!(pattern.name, "Repository Pattern");
}

#[test]
fn test_monolithic_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = create_monolithic_project(&temp_dir);

    let scan_result = CodebaseScanner::scan(&root).unwrap();
    let pattern = ArchitecturalPatternDetector::detect_monolithic_pattern(&scan_result).unwrap();
    
    assert!(pattern.is_some());
    let pattern = pattern.unwrap();
    assert_eq!(pattern.name, "Monolithic Architecture");
}
