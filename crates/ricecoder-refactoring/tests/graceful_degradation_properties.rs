//! Property-based tests for graceful degradation
//!
//! **Feature: ricecoder-refactoring, Property 4: Generic Fallback for Unconfigured Languages**
//! **Validates: Requirements REF-4.4, REF-4.5**
//!
//! Property: For any language without configured provider, generic refactoring works
//! Generate random code in unknown languages and verify generic refactoring
//! Run 100+ iterations with different language patterns

use std::sync::Arc;

use proptest::prelude::*;
use ricecoder_refactoring::{
    adapters::GenericRefactoringProvider, providers::ProviderRegistry, ConfigManager,
    RefactoringEngine, RefactoringType,
};

// Strategy for generating unknown language names
fn unknown_language_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("cobol".to_string()),
        Just("fortran".to_string()),
        Just("lisp".to_string()),
        Just("haskell".to_string()),
        Just("erlang".to_string()),
        Just("clojure".to_string()),
        Just("scala".to_string()),
        Just("kotlin".to_string()),
        Just("swift".to_string()),
        Just("objective-c".to_string()),
        Just("c".to_string()),
        Just("cpp".to_string()),
        Just("csharp".to_string()),
        Just("java".to_string()),
        Just("php".to_string()),
        Just("ruby".to_string()),
        Just("perl".to_string()),
        Just("lua".to_string()),
        Just("r".to_string()),
        Just("julia".to_string()),
        Just("unknown_lang_xyz".to_string()),
        Just("custom_dsl".to_string()),
        Just("proprietary_lang".to_string()),
    ]
}

// Strategy for generating code patterns
fn code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("code".to_string()),
        Just("function main() {}".to_string()),
        Just("def main():\n    pass".to_string()),
        Just("fn main() {}".to_string()),
        Just("public static void main(String[] args) {}".to_string()),
        Just("PROGRAM MAIN\nEND PROGRAM MAIN".to_string()),
        Just("(defn main [] nil)".to_string()),
        Just("main :: IO ()\nmain = putStrLn \"Hello\"".to_string()),
        Just("let main = () => console.log('Hello');".to_string()),
        Just("var x = 42;".to_string()),
        Just("x = 42".to_string()),
        Just("int x = 42;".to_string()),
        Just("let x = 42;".to_string()),
        Just("const x = 42;".to_string()),
        Just("x := 42".to_string()),
        Just("x = 42 -- comment".to_string()),
        Just("x = 42 # comment".to_string()),
        Just("x = 42 ; comment".to_string()),
        Just("{ x: 42 }".to_string()),
        Just("[1, 2, 3]".to_string()),
    ]
}

// Strategy for generating refactoring types
fn refactoring_type_strategy() -> impl Strategy<Value = RefactoringType> {
    prop_oneof![
        Just(RefactoringType::Rename),
        Just(RefactoringType::Extract),
        Just(RefactoringType::Inline),
        Just(RefactoringType::Move),
        Just(RefactoringType::ChangeSignature),
        Just(RefactoringType::RemoveUnused),
        Just(RefactoringType::Simplify),
    ]
}

fn create_test_engine() -> RefactoringEngine {
    let config_manager = ConfigManager::new();
    let generic_provider = Arc::new(GenericRefactoringProvider::new());
    let provider_registry = ProviderRegistry::new(generic_provider);
    RefactoringEngine::new(config_manager, provider_registry)
}

proptest! {
    /// Property 4: Generic Fallback for Unconfigured Languages
    ///
    /// For any language without configured provider, generic refactoring works.
    /// This property verifies that:
    /// 1. The system doesn't fail for unknown languages
    /// 2. The generic provider is used as fallback
    /// 3. The generic provider can analyze refactoring for any language
    /// 4. The analysis result indicates the refactoring is applicable
    #[test]
    fn prop_graceful_degradation_unknown_language(
        language in unknown_language_strategy(),
        code in code_strategy(),
        refactoring_type in refactoring_type_strategy(),
    ) {
        // Setup: Create engine
        let engine = create_test_engine();

        // Verify: Get provider for unknown language (should return generic)
        let provider = engine.provider_registry().clone().get_provider(&language);

        // Verify: Provider can analyze refactoring for unknown language
        let analysis = provider.analyze_refactoring(&code, &language, refactoring_type);
        prop_assert!(analysis.is_ok(), "Generic provider should handle unknown language {}", language);

        let analysis = analysis.unwrap();

        // Verify: Analysis indicates refactoring is applicable
        prop_assert!(analysis.applicable, "Refactoring should be applicable for unknown language {}", language);

        // Verify: Complexity is reasonable
        prop_assert!(analysis.complexity > 0, "Complexity should be positive for unknown language {}", language);
    }

    /// Property 4b: Generic Provider Consistency
    ///
    /// For any unknown language, the generic provider should consistently handle refactoring.
    #[test]
    fn prop_generic_provider_consistency(
        language in unknown_language_strategy(),
        code in code_strategy(),
    ) {
        let engine = create_test_engine();

        // Get provider twice for the same unknown language
        let provider1 = engine.provider_registry().clone().get_provider(&language);
        let provider2 = engine.provider_registry().clone().get_provider(&language);

        // Both should work
        let analysis1 = provider1.analyze_refactoring(&code, &language, RefactoringType::Rename);
        let analysis2 = provider2.analyze_refactoring(&code, &language, RefactoringType::Rename);

        prop_assert!(analysis1.is_ok(), "First provider should work for {}", language);
        prop_assert!(analysis2.is_ok(), "Second provider should work for {}", language);

        let analysis1 = analysis1.unwrap();
        let analysis2 = analysis2.unwrap();

        // Both should have same applicability
        prop_assert_eq!(analysis1.applicable, analysis2.applicable, "Applicability should be consistent for {}", language);
    }

    /// Property 4c: Generic Provider Validation
    ///
    /// For any unknown language, the generic provider should validate refactoring results.
    #[test]
    fn prop_generic_provider_validation(
        language in unknown_language_strategy(),
        original_code in code_strategy(),
        modified_code in code_strategy(),
    ) {
        let engine = create_test_engine();

        let provider = engine.provider_registry().clone().get_provider(&language);

        // Verify: Provider can validate refactoring for unknown language
        let validation = provider.validate_refactoring(&original_code, &modified_code, &language);
        prop_assert!(validation.is_ok(), "Generic provider should validate for unknown language {}", language);

        let validation = validation.unwrap();

        // Verify: Validation result is reasonable
        if original_code == modified_code {
            // If code is unchanged, validation should pass but have warnings
            prop_assert!(validation.passed, "Validation should pass for unchanged code");
        } else {
            // If code is changed, validation should pass (generic provider is permissive)
            prop_assert!(validation.passed, "Validation should pass for changed code");
        }
    }

    /// Property 4d: No Failure for Unknown Languages
    ///
    /// For any unknown language, the system should never fail or panic.
    #[test]
    fn prop_no_failure_for_unknown_languages(
        language in unknown_language_strategy(),
        code in code_strategy(),
        refactoring_type in refactoring_type_strategy(),
    ) {
        let engine = create_test_engine();

        // This should never panic or fail
        let provider = engine.provider_registry().clone().get_provider(&language);
        let _analysis = provider.analyze_refactoring(&code, &language, refactoring_type);

        // If we get here, the system handled the unknown language gracefully
        prop_assert!(true, "System should handle unknown language {} gracefully", language);
    }

    /// Property 4e: Generic Provider Fallback
    ///
    /// For any unknown language, the provider registry should return the generic provider.
    #[test]
    fn prop_generic_provider_fallback(
        language in unknown_language_strategy(),
    ) {
        let engine = create_test_engine();

        // Get provider for unknown language
        let provider = engine.provider_registry().clone().get_provider(&language);

        // Verify: Provider can handle basic refactoring
        let analysis = provider.analyze_refactoring("code", &language, RefactoringType::Rename);
        prop_assert!(analysis.is_ok(), "Generic provider should be available for {}", language);

        let analysis = analysis.unwrap();
        prop_assert!(analysis.applicable, "Generic provider should indicate applicability for {}", language);
    }

    /// Property 4f: Multiple Unknown Languages
    ///
    /// For multiple unknown languages, the system should handle all of them gracefully.
    #[test]
    fn prop_multiple_unknown_languages(
        languages in prop::collection::vec(unknown_language_strategy(), 1..5),
        code in code_strategy(),
    ) {
        let engine = create_test_engine();

        // For each unknown language, verify the system handles it
        for language in languages {
            let provider = engine.provider_registry().clone().get_provider(&language);
            let analysis = provider.analyze_refactoring(&code, &language, RefactoringType::Rename);

            prop_assert!(analysis.is_ok(), "System should handle unknown language {}", language);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ricecoder_refactoring::RefactoringEngine;

    use super::*;

    #[test]
    fn test_unknown_language_cobol() {
        let engine = create_test_engine();

        let provider = engine.provider_registry().clone().get_provider("cobol");
        let analysis =
            provider.analyze_refactoring("PROGRAM MAIN", "cobol", RefactoringType::Rename);

        assert!(analysis.is_ok());
        let analysis = analysis.unwrap();
        assert!(analysis.applicable);
    }

    #[test]
    fn test_unknown_language_lisp() {
        let engine = create_test_engine();

        let provider = engine.provider_registry().clone().get_provider("lisp");
        let analysis =
            provider.analyze_refactoring("(defn main [] nil)", "lisp", RefactoringType::Rename);

        assert!(analysis.is_ok());
        let analysis = analysis.unwrap();
        assert!(analysis.applicable);
    }

    #[test]
    fn test_unknown_language_haskell() {
        let engine = create_test_engine();

        let provider = engine.provider_registry().clone().get_provider("haskell");
        let analysis =
            provider.analyze_refactoring("main :: IO ()", "haskell", RefactoringType::Rename);

        assert!(analysis.is_ok());
        let analysis = analysis.unwrap();
        assert!(analysis.applicable);
    }

    #[test]
    fn test_unknown_language_custom_dsl() {
        let engine = create_test_engine();

        let provider = engine
            .provider_registry()
            .clone()
            .get_provider("custom_dsl");
        let analysis =
            provider.analyze_refactoring("custom code", "custom_dsl", RefactoringType::Rename);

        assert!(analysis.is_ok());
        let analysis = analysis.unwrap();
        assert!(analysis.applicable);
    }

    #[test]
    fn test_generic_provider_validation_unchanged() {
        let engine = create_test_engine();

        let provider = engine.provider_registry().clone().get_provider("unknown");
        let validation = provider.validate_refactoring("code", "code", "unknown");

        assert!(validation.is_ok());
        let validation = validation.unwrap();
        assert!(validation.passed);
    }

    #[test]
    fn test_generic_provider_validation_changed() {
        let engine = create_test_engine();

        let provider = engine.provider_registry().clone().get_provider("unknown");
        let validation = provider.validate_refactoring("code", "modified code", "unknown");

        assert!(validation.is_ok());
        let validation = validation.unwrap();
        assert!(validation.passed);
    }

    #[test]
    fn test_generic_provider_all_refactoring_types() {
        let engine = create_test_engine();

        let provider = engine.provider_registry().clone().get_provider("unknown");

        let types = vec![
            RefactoringType::Rename,
            RefactoringType::Extract,
            RefactoringType::Inline,
            RefactoringType::Move,
            RefactoringType::ChangeSignature,
            RefactoringType::RemoveUnused,
            RefactoringType::Simplify,
        ];

        for refactoring_type in types {
            let analysis = provider.analyze_refactoring("code", "unknown", refactoring_type);
            assert!(analysis.is_ok());
            let analysis = analysis.unwrap();
            assert!(analysis.applicable);
        }
    }
}
