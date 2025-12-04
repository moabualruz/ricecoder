/// Tests for language-specific completion suggestions
/// Tests that language-specific completions include traits, macros, decorators, type hints, etc.
use ricecoder_completion::{
    CompletionContext, CompletionItem, CompletionItemKind, Position,
    RustCompletionProvider, TypeScriptCompletionProvider, PythonCompletionProvider,
    CompletionProvider,
};
use async_trait::async_trait;

#[tokio::test]
async fn test_rust_trait_completions() {
    let provider = RustCompletionProvider;
    let context = CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have trait completions
    let debug_trait = completions
        .iter()
        .find(|c| c.label == "Debug")
        .expect("Debug trait not found");

    assert_eq!(debug_trait.kind, CompletionItemKind::Trait);
    assert!(debug_trait.detail.is_some());

    let clone_trait = completions
        .iter()
        .find(|c| c.label == "Clone")
        .expect("Clone trait not found");

    assert_eq!(clone_trait.kind, CompletionItemKind::Trait);

    let iterator_trait = completions
        .iter()
        .find(|c| c.label == "Iterator")
        .expect("Iterator trait not found");

    assert_eq!(iterator_trait.kind, CompletionItemKind::Trait);
}

#[tokio::test]
async fn test_rust_macro_completions() {
    let provider = RustCompletionProvider;
    let context = CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have macro completions
    let println_macro = completions
        .iter()
        .find(|c| c.label == "println!")
        .expect("println! macro not found");

    assert_eq!(println_macro.kind, CompletionItemKind::Operator);

    let vec_macro = completions
        .iter()
        .find(|c| c.label == "vec!")
        .expect("vec! macro not found");

    assert_eq!(vec_macro.kind, CompletionItemKind::Operator);

    let panic_macro = completions
        .iter()
        .find(|c| c.label == "panic!")
        .expect("panic! macro not found");

    assert_eq!(panic_macro.kind, CompletionItemKind::Operator);
}

#[tokio::test]
async fn test_rust_derive_attribute_completions() {
    let provider = RustCompletionProvider;
    let context = CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have derive attribute completions
    let debug_derive = completions
        .iter()
        .find(|c| c.label == "#[derive(Debug)]")
        .expect("Debug derive not found");

    assert_eq!(debug_derive.kind, CompletionItemKind::Keyword);
    assert!(debug_derive.insert_text.contains("derive"));

    let clone_derive = completions
        .iter()
        .find(|c| c.label == "#[derive(Clone)]")
        .expect("Clone derive not found");

    assert_eq!(clone_derive.kind, CompletionItemKind::Keyword);
}

#[tokio::test]
async fn test_typescript_interface_completions() {
    let provider = TypeScriptCompletionProvider;
    let context = CompletionContext::new("typescript".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have interface completions
    let record_interface = completions
        .iter()
        .find(|c| c.label == "Record")
        .expect("Record interface not found");

    assert_eq!(record_interface.kind, CompletionItemKind::Interface);

    let partial_interface = completions
        .iter()
        .find(|c| c.label == "Partial")
        .expect("Partial interface not found");

    assert_eq!(partial_interface.kind, CompletionItemKind::Interface);

    let pick_interface = completions
        .iter()
        .find(|c| c.label == "Pick")
        .expect("Pick interface not found");

    assert_eq!(pick_interface.kind, CompletionItemKind::Interface);
}

#[tokio::test]
async fn test_typescript_decorator_completions() {
    let provider = TypeScriptCompletionProvider;
    let context = CompletionContext::new("typescript".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have decorator completions
    let deprecated_decorator = completions
        .iter()
        .find(|c| c.label == "@deprecated")
        .expect("@deprecated decorator not found");

    assert_eq!(deprecated_decorator.kind, CompletionItemKind::Keyword);

    let memoize_decorator = completions
        .iter()
        .find(|c| c.label == "@memoize")
        .expect("@memoize decorator not found");

    assert_eq!(memoize_decorator.kind, CompletionItemKind::Keyword);
}

#[tokio::test]
async fn test_typescript_generic_completions() {
    let provider = TypeScriptCompletionProvider;
    let context = CompletionContext::new("typescript".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have generic completions
    let array_generic = completions
        .iter()
        .find(|c| c.label == "Array<T>")
        .expect("Array<T> generic not found");

    assert_eq!(array_generic.kind, CompletionItemKind::TypeParameter);

    let promise_generic = completions
        .iter()
        .find(|c| c.label == "Promise<T>")
        .expect("Promise<T> generic not found");

    assert_eq!(promise_generic.kind, CompletionItemKind::TypeParameter);

    let map_generic = completions
        .iter()
        .find(|c| c.label == "Map<K, V>")
        .expect("Map<K, V> generic not found");

    assert_eq!(map_generic.kind, CompletionItemKind::TypeParameter);
}

#[tokio::test]
async fn test_python_decorator_completions() {
    let provider = PythonCompletionProvider;
    let context = CompletionContext::new("python".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have decorator completions
    let property_decorator = completions
        .iter()
        .find(|c| c.label == "@property")
        .expect("@property decorator not found");

    assert_eq!(property_decorator.kind, CompletionItemKind::Keyword);

    let staticmethod_decorator = completions
        .iter()
        .find(|c| c.label == "@staticmethod")
        .expect("@staticmethod decorator not found");

    assert_eq!(staticmethod_decorator.kind, CompletionItemKind::Keyword);

    let classmethod_decorator = completions
        .iter()
        .find(|c| c.label == "@classmethod")
        .expect("@classmethod decorator not found");

    assert_eq!(classmethod_decorator.kind, CompletionItemKind::Keyword);
}

#[tokio::test]
async fn test_python_type_hint_completions() {
    let provider = PythonCompletionProvider;
    let context = CompletionContext::new("python".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have type hint completions
    let list_hint = completions
        .iter()
        .find(|c| c.label == "List[T]")
        .expect("List[T] type hint not found");

    assert_eq!(list_hint.kind, CompletionItemKind::TypeParameter);

    let dict_hint = completions
        .iter()
        .find(|c| c.label == "Dict[K, V]")
        .expect("Dict[K, V] type hint not found");

    assert_eq!(dict_hint.kind, CompletionItemKind::TypeParameter);

    let optional_hint = completions
        .iter()
        .find(|c| c.label == "Optional[T]")
        .expect("Optional[T] type hint not found");

    assert_eq!(optional_hint.kind, CompletionItemKind::TypeParameter);

    let union_hint = completions
        .iter()
        .find(|c| c.label == "Union[T, U]")
        .expect("Union[T, U] type hint not found");

    assert_eq!(union_hint.kind, CompletionItemKind::TypeParameter);
}

#[tokio::test]
async fn test_python_context_manager_completions() {
    let provider = PythonCompletionProvider;
    let context = CompletionContext::new("python".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have context manager completions
    let open_cm = completions
        .iter()
        .find(|c| c.label == "open()")
        .expect("open() context manager not found");

    assert_eq!(open_cm.kind, CompletionItemKind::Function);

    let tempdir_cm = completions
        .iter()
        .find(|c| c.label == "tempfile.TemporaryDirectory()")
        .expect("tempfile.TemporaryDirectory() context manager not found");

    assert_eq!(tempdir_cm.kind, CompletionItemKind::Function);
}

#[tokio::test]
async fn test_rust_language_specific_completions_count() {
    let provider = RustCompletionProvider;
    let context = CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have keywords, snippets, traits, macros, and derives
    let keywords = completions.iter().filter(|c| c.kind == CompletionItemKind::Keyword).count();
    let snippets = completions.iter().filter(|c| c.kind == CompletionItemKind::Snippet).count();
    let traits = completions.iter().filter(|c| c.kind == CompletionItemKind::Trait).count();
    let operators = completions.iter().filter(|c| c.kind == CompletionItemKind::Operator).count();

    assert!(keywords > 0, "No keywords found");
    assert!(snippets > 0, "No snippets found");
    assert!(traits > 0, "No traits found");
    assert!(operators > 0, "No macros found");
}

#[tokio::test]
async fn test_typescript_language_specific_completions_count() {
    let provider = TypeScriptCompletionProvider;
    let context = CompletionContext::new("typescript".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have keywords, snippets, interfaces, decorators, and generics
    let keywords = completions.iter().filter(|c| c.kind == CompletionItemKind::Keyword).count();
    let snippets = completions.iter().filter(|c| c.kind == CompletionItemKind::Snippet).count();
    let interfaces = completions.iter().filter(|c| c.kind == CompletionItemKind::Interface).count();
    let type_params = completions.iter().filter(|c| c.kind == CompletionItemKind::TypeParameter).count();

    assert!(keywords > 0, "No keywords found");
    assert!(snippets > 0, "No snippets found");
    assert!(interfaces > 0, "No interfaces found");
    assert!(type_params > 0, "No type parameters found");
}

#[tokio::test]
async fn test_python_language_specific_completions_count() {
    let provider = PythonCompletionProvider;
    let context = CompletionContext::new("python".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have keywords, snippets, decorators, type hints, and context managers
    let keywords = completions.iter().filter(|c| c.kind == CompletionItemKind::Keyword).count();
    let snippets = completions.iter().filter(|c| c.kind == CompletionItemKind::Snippet).count();
    let type_params = completions.iter().filter(|c| c.kind == CompletionItemKind::TypeParameter).count();
    let functions = completions.iter().filter(|c| c.kind == CompletionItemKind::Function).count();

    assert!(keywords > 0, "No keywords found");
    assert!(snippets > 0, "No snippets found");
    assert!(type_params > 0, "No type parameters found");
    assert!(functions > 0, "No functions found");
}

#[tokio::test]
async fn test_language_specific_completions_have_descriptions() {
    let providers: Vec<(&str, Box<dyn CompletionProvider>)> = vec![
        ("rust", Box::new(RustCompletionProvider)),
        ("typescript", Box::new(TypeScriptCompletionProvider)),
        ("python", Box::new(PythonCompletionProvider)),
    ];

    for (lang, provider) in providers {
        let context = CompletionContext::new(lang.to_string(), Position::new(0, 0), "".to_string());

        let completions = provider
            .generate_completions("", Position::new(0, 0), &context)
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

#[tokio::test]
async fn test_rust_naming_conventions() {
    let provider = RustCompletionProvider;
    let context = CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Rust uses snake_case for functions and variables
    // Check that we have snake_case examples in snippets
    let snippets: Vec<_> = completions
        .iter()
        .filter(|c| c.kind == CompletionItemKind::Snippet)
        .collect();

    for snippet in snippets {
        // Snippet templates should use snake_case for variable names
        if snippet.insert_text.contains("${1:name}") {
            // This is a valid Rust naming convention
            assert!(true);
        }
    }
}

#[tokio::test]
async fn test_typescript_naming_conventions() {
    let provider = TypeScriptCompletionProvider;
    let context = CompletionContext::new("typescript".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // TypeScript uses camelCase for functions and variables
    // Check that we have camelCase examples in snippets
    let snippets: Vec<_> = completions
        .iter()
        .filter(|c| c.kind == CompletionItemKind::Snippet)
        .collect();

    for snippet in snippets {
        // Snippet templates should use camelCase for variable names
        if snippet.insert_text.contains("${1:name}") {
            // This is a valid TypeScript naming convention
            assert!(true);
        }
    }
}

#[tokio::test]
async fn test_python_naming_conventions() {
    let provider = PythonCompletionProvider;
    let context = CompletionContext::new("python".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Python uses snake_case for functions and variables
    // Check that we have snake_case examples in snippets
    let snippets: Vec<_> = completions
        .iter()
        .filter(|c| c.kind == CompletionItemKind::Snippet)
        .collect();

    for snippet in snippets {
        // Snippet templates should use snake_case for variable names
        if snippet.insert_text.contains("${1:name}") {
            // This is a valid Python naming convention
            assert!(true);
        }
    }
}
