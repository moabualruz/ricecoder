/// Property-based tests for language-specific completions
/// **Feature: ricecoder-completion, Property 4: Language-specific completions**
/// **Validates: Requirements Completion-4.1, Completion-4.2**
/// Property: For any supported language, completions follow language conventions
use ricecoder_completion::{
    CompletionContext, CompletionItemKind, Position,
    RustCompletionProvider, TypeScriptCompletionProvider, PythonCompletionProvider,
    CompletionProvider,
};

/// Property: For any Rust code position, Rust completions follow Rust naming conventions
/// Rust uses snake_case for functions and variables, PascalCase for types
#[tokio::test]
async fn prop_rust_completions_follow_conventions() {
    // Test multiple positions to verify property holds across different contexts
    for line in 0..5 {
        for char in 0..5 {
            let pos = Position::new(line, char);
            let provider = RustCompletionProvider;
            let context = CompletionContext::new("rust".to_string(), pos, "".to_string());

            let completions = provider
                .generate_completions("", pos, &context)
                .await
                .expect("Failed to generate completions");

            // All completions should be present
            assert!(!completions.is_empty(), "No completions generated for Rust at pos ({}, {})", line, char);

            // Should have keywords
            let has_keywords = completions.iter().any(|c| c.kind == CompletionItemKind::Keyword);
            assert!(has_keywords, "No keywords found in Rust completions at pos ({}, {})", line, char);

            // Should have snippets
            let has_snippets = completions.iter().any(|c| c.kind == CompletionItemKind::Snippet);
            assert!(has_snippets, "No snippets found in Rust completions at pos ({}, {})", line, char);

            // Should have traits
            let has_traits = completions.iter().any(|c| c.kind == CompletionItemKind::Trait);
            assert!(has_traits, "No traits found in Rust completions at pos ({}, {})", line, char);

            // All completions should have labels
            for completion in &completions {
                assert!(!completion.label.is_empty(), "Empty completion label at pos ({}, {})", line, char);
                assert!(!completion.insert_text.is_empty(), "Empty insert text at pos ({}, {})", line, char);
            }

            // Rust-specific completions should be present
            let has_debug_trait = completions.iter().any(|c| c.label == "Debug");
            assert!(has_debug_trait, "Debug trait not found in Rust completions at pos ({}, {})", line, char);

            let has_println_macro = completions.iter().any(|c| c.label == "println!");
            assert!(has_println_macro, "println! macro not found in Rust completions at pos ({}, {})", line, char);

            let has_derive = completions.iter().any(|c| c.label.contains("derive"));
            assert!(has_derive, "Derive attributes not found in Rust completions at pos ({}, {})", line, char);
        }
    }
}

/// Property: For any TypeScript code position, TypeScript completions follow TypeScript conventions
/// TypeScript uses camelCase for functions and variables, PascalCase for types
#[tokio::test]
async fn prop_typescript_completions_follow_conventions() {
    // Test multiple positions to verify property holds across different contexts
    for line in 0..5 {
        for char in 0..5 {
            let pos = Position::new(line, char);
            let provider = TypeScriptCompletionProvider;
            let context = CompletionContext::new("typescript".to_string(), pos, "".to_string());

            let completions = provider
                .generate_completions("", pos, &context)
                .await
                .expect("Failed to generate completions");

            // All completions should be present
            assert!(!completions.is_empty(), "No completions generated for TypeScript at pos ({}, {})", line, char);

            // Should have keywords
            let has_keywords = completions.iter().any(|c| c.kind == CompletionItemKind::Keyword);
            assert!(has_keywords, "No keywords found in TypeScript completions at pos ({}, {})", line, char);

            // Should have snippets
            let has_snippets = completions.iter().any(|c| c.kind == CompletionItemKind::Snippet);
            assert!(has_snippets, "No snippets found in TypeScript completions at pos ({}, {})", line, char);

            // Should have interfaces
            let has_interfaces = completions.iter().any(|c| c.kind == CompletionItemKind::Interface);
            assert!(has_interfaces, "No interfaces found in TypeScript completions at pos ({}, {})", line, char);

            // All completions should have labels
            for completion in &completions {
                assert!(!completion.label.is_empty(), "Empty completion label at pos ({}, {})", line, char);
                assert!(!completion.insert_text.is_empty(), "Empty insert text at pos ({}, {})", line, char);
            }

            // TypeScript-specific completions should be present
            let has_record = completions.iter().any(|c| c.label == "Record");
            assert!(has_record, "Record interface not found in TypeScript completions at pos ({}, {})", line, char);

            let has_decorator = completions.iter().any(|c| c.label.starts_with("@"));
            assert!(has_decorator, "Decorators not found in TypeScript completions at pos ({}, {})", line, char);

            let has_generic = completions.iter().any(|c| c.label.contains("<"));
            assert!(has_generic, "Generic types not found in TypeScript completions at pos ({}, {})", line, char);
        }
    }
}

/// Property: For any Python code position, Python completions follow Python conventions
/// Python uses snake_case for functions and variables, PascalCase for types
#[tokio::test]
async fn prop_python_completions_follow_conventions() {
    // Test multiple positions to verify property holds across different contexts
    for line in 0..5 {
        for char in 0..5 {
            let pos = Position::new(line, char);
            let provider = PythonCompletionProvider;
            let context = CompletionContext::new("python".to_string(), pos, "".to_string());

            let completions = provider
                .generate_completions("", pos, &context)
                .await
                .expect("Failed to generate completions");

            // All completions should be present
            assert!(!completions.is_empty(), "No completions generated for Python at pos ({}, {})", line, char);

            // Should have keywords
            let has_keywords = completions.iter().any(|c| c.kind == CompletionItemKind::Keyword);
            assert!(has_keywords, "No keywords found in Python completions at pos ({}, {})", line, char);

            // Should have snippets
            let has_snippets = completions.iter().any(|c| c.kind == CompletionItemKind::Snippet);
            assert!(has_snippets, "No snippets found in Python completions at pos ({}, {})", line, char);

            // Should have type parameters (type hints)
            let has_type_hints = completions.iter().any(|c| c.kind == CompletionItemKind::TypeParameter);
            assert!(has_type_hints, "No type hints found in Python completions at pos ({}, {})", line, char);

            // All completions should have labels
            for completion in &completions {
                assert!(!completion.label.is_empty(), "Empty completion label at pos ({}, {})", line, char);
                assert!(!completion.insert_text.is_empty(), "Empty insert text at pos ({}, {})", line, char);
            }

            // Python-specific completions should be present
            let has_property = completions.iter().any(|c| c.label == "@property");
            assert!(has_property, "@property decorator not found in Python completions at pos ({}, {})", line, char);

            let has_list_hint = completions.iter().any(|c| c.label == "List[T]");
            assert!(has_list_hint, "List[T] type hint not found in Python completions at pos ({}, {})", line, char);

            let has_context_manager = completions.iter().any(|c| c.label == "open()");
            assert!(has_context_manager, "open() context manager not found in Python completions at pos ({}, {})", line, char);
        }
    }
}

/// Property: All language-specific completions have descriptions
#[tokio::test]
async fn prop_all_language_specific_completions_have_descriptions() {
    let providers: Vec<(&str, Box<dyn CompletionProvider>)> = vec![
        ("rust", Box::new(RustCompletionProvider)),
        ("typescript", Box::new(TypeScriptCompletionProvider)),
        ("python", Box::new(PythonCompletionProvider)),
    ];

    for (lang, provider) in providers {
        let pos = Position::new(0, 0);
        let context = CompletionContext::new(lang.to_string(), pos, "".to_string());

        let completions = provider
            .generate_completions("", pos, &context)
            .await
            .expect(&format!("Failed to generate completions for {}", lang));

        // All language-specific completions should have descriptions
        for completion in &completions {
            if completion.kind == CompletionItemKind::Trait
                || completion.kind == CompletionItemKind::Interface
                || completion.kind == CompletionItemKind::TypeParameter
                || completion.kind == CompletionItemKind::Operator
            {
                assert!(
                    completion.detail.is_some(),
                    "Completion {} in {} has no description",
                    completion.label,
                    lang
                );
            }
        }
    }
}

/// Property: All snippets have placeholder syntax
#[tokio::test]
async fn prop_all_snippets_have_placeholders() {
    let providers: Vec<(&str, Box<dyn CompletionProvider>)> = vec![
        ("rust", Box::new(RustCompletionProvider)),
        ("typescript", Box::new(TypeScriptCompletionProvider)),
        ("python", Box::new(PythonCompletionProvider)),
    ];

    for (lang, provider) in providers {
        let pos = Position::new(0, 0);
        let context = CompletionContext::new(lang.to_string(), pos, "".to_string());

        let completions = provider
            .generate_completions("", pos, &context)
            .await
            .expect(&format!("Failed to generate completions for {}", lang));

        // All snippets should have placeholder syntax
        for completion in &completions {
            if completion.kind == CompletionItemKind::Snippet {
                assert!(
                    completion.insert_text.contains("${"),
                    "Snippet {} in {} missing placeholders",
                    completion.label,
                    lang
                );
            }
        }
    }
}

/// Property: All completions have non-empty labels and insert text
#[tokio::test]
async fn prop_all_completions_have_valid_labels() {
    let providers: Vec<(&str, Box<dyn CompletionProvider>)> = vec![
        ("rust", Box::new(RustCompletionProvider)),
        ("typescript", Box::new(TypeScriptCompletionProvider)),
        ("python", Box::new(PythonCompletionProvider)),
    ];

    for (lang, provider) in providers {
        let pos = Position::new(0, 0);
        let context = CompletionContext::new(lang.to_string(), pos, "".to_string());

        let completions = provider
            .generate_completions("", pos, &context)
            .await
            .expect(&format!("Failed to generate completions for {}", lang));

        for completion in &completions {
            assert!(!completion.label.is_empty(), "Empty label in {}", lang);
            assert!(!completion.insert_text.is_empty(), "Empty insert_text in {}", lang);
        }
    }
}

/// Property: Completions have reasonable scores
#[tokio::test]
async fn prop_completions_have_reasonable_scores() {
    let providers: Vec<(&str, Box<dyn CompletionProvider>)> = vec![
        ("rust", Box::new(RustCompletionProvider)),
        ("typescript", Box::new(TypeScriptCompletionProvider)),
        ("python", Box::new(PythonCompletionProvider)),
    ];

    for (lang, provider) in providers {
        let pos = Position::new(0, 0);
        let context = CompletionContext::new(lang.to_string(), pos, "".to_string());

        let completions = provider
            .generate_completions("", pos, &context)
            .await
            .expect(&format!("Failed to generate completions for {}", lang));

        for completion in &completions {
            assert!(completion.score >= 0.0, "Negative score in {}", lang);
            assert!(completion.score <= 1.0, "Score > 1.0 in {}", lang);
        }
    }
}
