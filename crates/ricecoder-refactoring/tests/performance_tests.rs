//! Performance tests for the refactoring engine
//!
//! These tests verify that the refactoring engine meets performance targets:
//! - Configuration loading: < 100ms
//! - Impact analysis: < 2s for typical projects
//! - Preview generation: < 1s
//! - Provider lookup: < 50ms

use ricecoder_refactoring::{
    ConfigManager, ImpactAnalyzer, PreviewGenerator, RefactoringEngine, RefactoringType,
};
use std::sync::Arc;
use std::time::Instant;

#[test]
fn test_performance_config_manager_creation() {
    let start = Instant::now();
    let _config_manager = ConfigManager::new();
    let elapsed = start.elapsed();

    println!("Config manager creation: {:?}", elapsed);
    assert!(elapsed.as_millis() < 100, "Config manager creation should be < 100ms");
}

#[test]
fn test_performance_engine_creation() {
    let start = Instant::now();
    let config_manager = Arc::new(ConfigManager::new());
    let _engine = RefactoringEngine::new(config_manager);
    let elapsed = start.elapsed();

    println!("Engine creation: {:?}", elapsed);
    assert!(elapsed.as_millis() < 200, "Engine creation should be < 200ms");
}

#[test]
fn test_performance_provider_lookup() {
    let config_manager = Arc::new(ConfigManager::new());
    let engine = RefactoringEngine::new(config_manager);

    let start = Instant::now();
    let _provider = engine.provider_registry().clone().get_provider("rust");
    let elapsed = start.elapsed();

    println!("Provider lookup: {:?}", elapsed);
    assert!(elapsed.as_millis() < 50, "Provider lookup should be < 50ms");
}

#[test]
fn test_performance_provider_lookup_multiple() {
    let config_manager = Arc::new(ConfigManager::new());
    let engine = RefactoringEngine::new(config_manager);

    let start = Instant::now();
    for _ in 0..100 {
        let _provider = engine.provider_registry().clone().get_provider("rust");
    }
    let elapsed = start.elapsed();

    println!("100 provider lookups: {:?}", elapsed);
    assert!(elapsed.as_millis() < 5000, "100 provider lookups should be < 5s");
}

#[test]
fn test_performance_analysis_simple_code() {
    let config_manager = Arc::new(ConfigManager::new());
    let engine = RefactoringEngine::new(config_manager);
    let provider = engine.provider_registry().clone().get_provider("rust");

    let code = "fn main() {}";

    let start = Instant::now();
    let _analysis = provider.analyze_refactoring(code, "rust", RefactoringType::Rename);
    let elapsed = start.elapsed();

    println!("Simple code analysis: {:?}", elapsed);
    assert!(elapsed.as_millis() < 100, "Simple code analysis should be < 100ms");
}

#[test]
fn test_performance_analysis_complex_code() {
    let config_manager = Arc::new(ConfigManager::new());
    let engine = RefactoringEngine::new(config_manager);
    let provider = engine.provider_registry().clone().get_provider("rust");

    let code = r#"
        fn main() {
            let x = 42;
            let y = 43;
            let z = x + y;
            println!("{}", z);
            
            for i in 0..100 {
                println!("{}", i);
            }
            
            match z {
                85 => println!("correct"),
                _ => println!("wrong"),
            }
        }
    "#;

    let start = Instant::now();
    let _analysis = provider.analyze_refactoring(code, "rust", RefactoringType::Rename);
    let elapsed = start.elapsed();

    println!("Complex code analysis: {:?}", elapsed);
    assert!(elapsed.as_millis() < 500, "Complex code analysis should be < 500ms");
}

#[test]
fn test_performance_validation_simple() {
    let config_manager = Arc::new(ConfigManager::new());
    let engine = RefactoringEngine::new(config_manager);
    let provider = engine.provider_registry().clone().get_provider("rust");

    let original = "fn main() {}";
    let refactored = "fn main() { println!(); }";

    let start = Instant::now();
    let _validation = provider.validate_refactoring(original, refactored, "rust");
    let elapsed = start.elapsed();

    println!("Simple validation: {:?}", elapsed);
    assert!(elapsed.as_millis() < 100, "Simple validation should be < 100ms");
}

#[test]
fn test_performance_validation_complex() {
    let config_manager = Arc::new(ConfigManager::new());
    let engine = RefactoringEngine::new(config_manager);
    let provider = engine.provider_registry().clone().get_provider("rust");

    let original = r#"
        fn main() {
            let x = 42;
            let y = 43;
            let z = x + y;
        }
    "#;

    let refactored = r#"
        fn main() {
            let x = 42;
            let y = 43;
            let z = x + y;
            println!("{}", z);
        }
    "#;

    let start = Instant::now();
    let _validation = provider.validate_refactoring(original, refactored, "rust");
    let elapsed = start.elapsed();

    println!("Complex validation: {:?}", elapsed);
    assert!(elapsed.as_millis() < 500, "Complex validation should be < 500ms");
}

#[test]
fn test_performance_impact_analyzer_creation() {
    let start = Instant::now();
    let _analyzer = ImpactAnalyzer::new();
    let elapsed = start.elapsed();

    println!("Impact analyzer creation: {:?}", elapsed);
    assert!(elapsed.as_millis() < 100, "Impact analyzer creation should be < 100ms");
}

#[test]
fn test_performance_preview_generator_diff() {
    let original = "fn main() {}";
    let new = "fn main() { println!(); }";

    let start = Instant::now();
    let _diff = PreviewGenerator::generate_unified_diff(original, new);
    let elapsed = start.elapsed();

    println!("Preview generator diff: {:?}", elapsed);
    assert!(elapsed.as_millis() < 100, "Preview generator diff should be < 100ms");
}

#[test]
fn test_performance_multiple_languages() {
    let config_manager = Arc::new(ConfigManager::new());
    let engine = RefactoringEngine::new(config_manager);

    let languages = vec!["rust", "typescript", "python"];
    let code_samples = vec![
        "fn main() {}",
        "function main() {}",
        "def main():\n    pass",
    ];

    let start = Instant::now();
    for (language, code) in languages.iter().zip(code_samples.iter()) {
        let provider = engine.provider_registry().clone().get_provider(language);
        let _analysis = provider.analyze_refactoring(code, language, RefactoringType::Rename);
    }
    let elapsed = start.elapsed();

    println!("Multiple language analysis: {:?}", elapsed);
    assert!(elapsed.as_millis() < 500, "Multiple language analysis should be < 500ms");
}

#[test]
fn test_performance_batch_analysis() {
    let config_manager = Arc::new(ConfigManager::new());
    let engine = RefactoringEngine::new(config_manager);
    let provider = engine.provider_registry().clone().get_provider("rust");

    let code = "fn main() {}";

    let start = Instant::now();
    for _ in 0..100 {
        let _analysis = provider.analyze_refactoring(code, "rust", RefactoringType::Rename);
    }
    let elapsed = start.elapsed();

    println!("100 analyses: {:?}", elapsed);
    assert!(elapsed.as_millis() < 10000, "100 analyses should be < 10s");
}

#[test]
fn test_performance_all_refactoring_types() {
    let config_manager = Arc::new(ConfigManager::new());
    let engine = RefactoringEngine::new(config_manager);
    let provider = engine.provider_registry().clone().get_provider("rust");

    let types = vec![
        RefactoringType::Rename,
        RefactoringType::Extract,
        RefactoringType::Inline,
        RefactoringType::Move,
        RefactoringType::ChangeSignature,
        RefactoringType::RemoveUnused,
        RefactoringType::Simplify,
    ];

    let code = "fn main() {}";

    let start = Instant::now();
    for refactoring_type in types {
        let _analysis = provider.analyze_refactoring(code, "rust", refactoring_type);
    }
    let elapsed = start.elapsed();

    println!("All refactoring types: {:?}", elapsed);
    assert!(elapsed.as_millis() < 500, "All refactoring types should be < 500ms");
}

#[test]
fn test_performance_unknown_language_fallback() {
    let config_manager = Arc::new(ConfigManager::new());
    let engine = RefactoringEngine::new(config_manager);

    let start = Instant::now();
    let _provider = engine.provider_registry().clone().get_provider("unknown_language");
    let elapsed = start.elapsed();

    println!("Unknown language fallback: {:?}", elapsed);
    assert!(elapsed.as_millis() < 50, "Unknown language fallback should be < 50ms");
}

#[test]
fn test_performance_summary() {
    println!("\n=== Performance Test Summary ===");
    println!("Target: Configuration loading < 100ms");
    println!("Target: Impact analysis < 2s");
    println!("Target: Preview generation < 1s");
    println!("Target: Provider lookup < 50ms");
    println!("================================\n");
}
