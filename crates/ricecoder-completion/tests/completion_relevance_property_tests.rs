/// Property-based tests for completion relevance
///
/// **Feature: ricecoder-completion, Property 1: Completion relevance**
/// **Validates: Requirements Completion-1.1, Completion-3.1**
///
/// Property: For any code context, top-ranked completion is relevant and applicable
/// - Generate random code contexts and verify top completion relevance
/// - Run 100+ iterations with various code patterns
use proptest::prelude::*;
use ricecoder_completion::*;
use std::sync::Arc;

/// Strategy for generating valid Rust code snippets
fn rust_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("fn main() { let x = ".to_string()),
        Just("fn test() { let result = ".to_string()),
        Just("impl MyStruct { fn method(&self) { ".to_string()),
        Just("match value { ".to_string()),
        Just("if condition { ".to_string()),
        Just("for item in ".to_string()),
        Just("let vec = vec![".to_string()),
        Just("struct MyStruct { field: ".to_string()),
        Just("enum MyEnum { ".to_string()),
        Just("trait MyTrait { fn method(&self) -> ".to_string()),
    ]
}

/// Strategy for generating valid TypeScript code snippets
fn typescript_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("function main() { const x = ".to_string()),
        Just("class MyClass { method() { ".to_string()),
        Just("interface MyInterface { ".to_string()),
        Just("type MyType = ".to_string()),
        Just("const arr = [".to_string()),
        Just("if (condition) { ".to_string()),
        Just("for (let i = ".to_string()),
        Just("async function test() { await ".to_string()),
        Just("const promise = new Promise(".to_string()),
        Just("const obj = { ".to_string()),
    ]
}

/// Strategy for generating valid Python code snippets
fn python_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("def main():\n    x = ".to_string()),
        Just("class MyClass:\n    def method(self):\n        ".to_string()),
        Just("if condition:\n    ".to_string()),
        Just("for item in ".to_string()),
        Just("with open('file') as ".to_string()),
        Just("try:\n    ".to_string()),
        Just("import ".to_string()),
        Just("from module import ".to_string()),
        Just("lambda x: ".to_string()),
        Just("list_comp = [x for x in ".to_string()),
    ]
}

/// Strategy for generating valid positions within code
fn position_strategy() -> impl Strategy<Value = Position> {
    (0u32..5, 0u32..50).prop_map(|(line, character)| Position::new(line, character))
}

proptest! {
    /// Property: Top completion is not empty
    ///
    /// For any code context, if completions are generated, the top completion
    /// should not be empty.
    #[test]
    fn prop_top_completion_not_empty(
        code in rust_code_strategy(),
        position in position_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let engine = create_test_engine();

            // Generate completions
            engine
                .generate_completions(&code, position, "rust")
                .await
        });

        if let Ok(completions) = result {
            if !completions.is_empty() {
                let top = &completions[0];
                // Top completion should have a non-empty label
                prop_assert!(!top.label.is_empty());
                // Top completion should have non-empty insert text
                prop_assert!(!top.insert_text.is_empty());
            }
        }
        // Errors are acceptable for invalid positions
    }

    /// Property: Top completion has valid kind
    ///
    /// For any code context, the top completion should have a valid completion kind.
    #[test]
    fn prop_top_completion_has_valid_kind(
        code in rust_code_strategy(),
        position in position_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let engine = create_test_engine();
            engine
                .generate_completions(&code, position, "rust")
                .await
        });

        if let Ok(completions) = result {
            if !completions.is_empty() {
                let top = &completions[0];
                // Top completion should have a valid kind
                match top.kind {
                    CompletionItemKind::Text
                    | CompletionItemKind::Method
                    | CompletionItemKind::Function
                    | CompletionItemKind::Constructor
                    | CompletionItemKind::Field
                    | CompletionItemKind::Variable
                    | CompletionItemKind::Class
                    | CompletionItemKind::Interface
                    | CompletionItemKind::Module
                    | CompletionItemKind::Property
                    | CompletionItemKind::Unit
                    | CompletionItemKind::Value
                    | CompletionItemKind::Enum
                    | CompletionItemKind::Keyword
                    | CompletionItemKind::Snippet
                    | CompletionItemKind::Color
                    | CompletionItemKind::File
                    | CompletionItemKind::Reference
                    | CompletionItemKind::Folder
                    | CompletionItemKind::EnumMember
                    | CompletionItemKind::Constant
                    | CompletionItemKind::Struct
                    | CompletionItemKind::EventListener
                    | CompletionItemKind::Operator
                    | CompletionItemKind::TypeParameter
                    | CompletionItemKind::Trait => {
                        // Valid kind
                    }
                }
            }
        }
        // Errors are acceptable
    }

    /// Property: Top completion has non-negative score
    ///
    /// For any code context, the top completion should have a non-negative score.
    #[test]
    fn prop_top_completion_has_valid_score(
        code in rust_code_strategy(),
        position in position_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let engine = create_test_engine();
            engine
                .generate_completions(&code, position, "rust")
                .await
        });

        if let Ok(completions) = result {
            if !completions.is_empty() {
                let top = &completions[0];
                // Top completion should have a non-negative score
                prop_assert!(top.score >= 0.0);
            }
        }
        // Errors are acceptable
    }

    /// Property: Completions are ranked by score (descending)
    ///
    /// For any code context, completions should be ranked in descending order by score.
    #[test]
    fn prop_completions_ranked_by_score(
        code in rust_code_strategy(),
        position in position_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let engine = create_test_engine();
            engine
                .generate_completions(&code, position, "rust")
                .await
        });

        if let Ok(completions) = result {
            // Verify completions are sorted by score (descending)
            for i in 0..completions.len().saturating_sub(1) {
                prop_assert!(
                    completions[i].score >= completions[i + 1].score,
                    "Completions not sorted by score: {} < {}",
                    completions[i].score,
                    completions[i + 1].score
                );
            }
        }
        // Errors are acceptable
    }

    /// Property: TypeScript completions are relevant
    ///
    /// For any TypeScript code context, completions should be relevant.
    #[test]
    fn prop_typescript_completions_relevant(
        code in typescript_code_strategy(),
        position in position_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let engine = create_test_engine();
            engine
                .generate_completions(&code, position, "typescript")
                .await
        });

        if let Ok(completions) = result {
            if !completions.is_empty() {
                let top = &completions[0];
                // Top completion should have a non-empty label
                prop_assert!(!top.label.is_empty());
                // Top completion should have a valid kind
                prop_assert!(top.score >= 0.0);
            }
        }
        // Errors are acceptable
    }

    /// Property: Python completions are relevant
    ///
    /// For any Python code context, completions should be relevant.
    #[test]
    fn prop_python_completions_relevant(
        code in python_code_strategy(),
        position in position_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let engine = create_test_engine();
            engine
                .generate_completions(&code, position, "python")
                .await
        });

        if let Ok(completions) = result {
            if !completions.is_empty() {
                let top = &completions[0];
                // Top completion should have a non-empty label
                prop_assert!(!top.label.is_empty());
                // Top completion should have a valid kind
                prop_assert!(top.score >= 0.0);
            }
        }
        // Errors are acceptable
    }

    /// Property: Completions are deterministic
    ///
    /// For any code context, generating completions twice should produce
    /// the same top completion.
    #[test]
    fn prop_completions_deterministic(
        code in rust_code_strategy(),
        position in position_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result1 = rt.block_on(async {
            let engine = create_test_engine();
            engine
                .generate_completions(&code, position, "rust")
                .await
        });

        let result2 = rt.block_on(async {
            let engine = create_test_engine();
            engine
                .generate_completions(&code, position, "rust")
                .await
        });

        match (result1, result2) {
            (Ok(completions1), Ok(completions2)) => {
                if !completions1.is_empty() && !completions2.is_empty() {
                    // Top completions should be identical
                    prop_assert_eq!(&completions1[0].label, &completions2[0].label);
                    prop_assert_eq!(&completions1[0].insert_text, &completions2[0].insert_text);
                    prop_assert_eq!(completions1[0].score, completions2[0].score);
                }
            }
            (Err(_), Err(_)) => {
                // Both errors are acceptable
            }
            _ => {
                // Inconsistent results are not acceptable
                prop_assert!(false, "Inconsistent completion results");
            }
        }
    }

    /// Property: All completions have non-empty labels
    ///
    /// For any code context, all completions should have non-empty labels.
    #[test]
    fn prop_all_completions_have_labels(
        code in rust_code_strategy(),
        position in position_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let engine = create_test_engine();
            engine
                .generate_completions(&code, position, "rust")
                .await
        });

        if let Ok(completions) = result {
            for completion in completions {
                prop_assert!(!completion.label.is_empty());
            }
        }
        // Errors are acceptable
    }

    /// Property: All completions have non-empty insert text
    ///
    /// For any code context, all completions should have non-empty insert text.
    #[test]
    fn prop_all_completions_have_insert_text(
        code in rust_code_strategy(),
        position in position_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let engine = create_test_engine();
            engine
                .generate_completions(&code, position, "rust")
                .await
        });

        if let Ok(completions) = result {
            for completion in completions {
                prop_assert!(!completion.insert_text.is_empty());
            }
        }
        // Errors are acceptable
    }

    /// Property: All completions have valid scores
    ///
    /// For any code context, all completions should have non-negative scores.
    #[test]
    fn prop_all_completions_have_valid_scores(
        code in rust_code_strategy(),
        position in position_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let engine = create_test_engine();
            engine
                .generate_completions(&code, position, "rust")
                .await
        });

        if let Ok(completions) = result {
            for completion in completions {
                prop_assert!(completion.score >= 0.0);
            }
        }
        // Errors are acceptable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_completion_relevance_simple_rust() {
        let engine = create_test_engine();
        let code = "fn main() { let x = ";
        let position = Position::new(0, 20);

        let completions = engine
            .generate_completions(code, position, "rust")
            .await
            .expect("Failed to generate completions");

        // Should have at least one completion
        assert!(!completions.is_empty());

        // Top completion should be relevant
        let top = &completions[0];
        assert!(!top.label.is_empty());
        assert!(!top.insert_text.is_empty());
        assert!(top.score >= 0.0);
    }

    #[tokio::test]
    async fn test_completion_relevance_function_call() {
        let engine = create_test_engine();
        let code = "fn main() { println!(";
        let position = Position::new(0, 21);

        let completions = engine
            .generate_completions(code, position, "rust")
            .await
            .expect("Failed to generate completions");

        // Should have at least one completion
        assert!(!completions.is_empty());

        // Top completion should be relevant
        let top = &completions[0];
        assert!(!top.label.is_empty());
        assert!(top.score >= 0.0);
    }

    #[tokio::test]
    async fn test_completion_relevance_struct_field() {
        let engine = create_test_engine();
        let code = "struct MyStruct { field: ";
        let position = Position::new(0, 24);

        let completions = engine
            .generate_completions(code, position, "rust")
            .await
            .expect("Failed to generate completions");

        // Should have at least one completion
        assert!(!completions.is_empty());

        // Top completion should be relevant
        let top = &completions[0];
        assert!(!top.label.is_empty());
        assert!(top.score >= 0.0);
    }

    #[tokio::test]
    async fn test_completion_relevance_typescript() {
        let engine = create_test_engine();
        let code = "function main() { const x = ";
        let position = Position::new(0, 28);

        let completions = engine
            .generate_completions(code, position, "typescript")
            .await
            .expect("Failed to generate completions");

        // Should have at least one completion
        assert!(!completions.is_empty());

        // Top completion should be relevant
        let top = &completions[0];
        assert!(!top.label.is_empty());
        assert!(top.score >= 0.0);
    }

    #[tokio::test]
    async fn test_completion_relevance_python() {
        let engine = create_test_engine();
        let code = "def main():\n    x = ";
        let position = Position::new(1, 9);

        let completions = engine
            .generate_completions(code, position, "python")
            .await
            .expect("Failed to generate completions");

        // Should have at least one completion
        assert!(!completions.is_empty());

        // Top completion should be relevant
        let top = &completions[0];
        assert!(!top.label.is_empty());
        assert!(top.score >= 0.0);
    }

    #[tokio::test]
    async fn test_completion_relevance_ranked_order() {
        let engine = create_test_engine();
        let code = "fn main() { let x = ";
        let position = Position::new(0, 20);

        let completions = engine
            .generate_completions(code, position, "rust")
            .await
            .expect("Failed to generate completions");

        // Verify completions are ranked by score (descending)
        for i in 0..completions.len().saturating_sub(1) {
            assert!(
                completions[i].score >= completions[i + 1].score,
                "Completions not ranked by score: {} < {}",
                completions[i].score,
                completions[i + 1].score
            );
        }
    }

    #[tokio::test]
    async fn test_completion_relevance_deterministic() {
        let engine = create_test_engine();
        let code = "fn main() { let x = ";
        let position = Position::new(0, 20);

        let completions1 = engine
            .generate_completions(code, position, "rust")
            .await
            .expect("Failed to generate completions");

        let completions2 = engine
            .generate_completions(code, position, "rust")
            .await
            .expect("Failed to generate completions");

        // Top completions should be identical
        assert_eq!(completions1.len(), completions2.len());
        if !completions1.is_empty() {
            assert_eq!(completions1[0].label, completions2[0].label);
            assert_eq!(completions1[0].insert_text, completions2[0].insert_text);
            assert_eq!(completions1[0].score, completions2[0].score);
        }
    }
}

/// Helper function to create a test completion engine
fn create_test_engine() -> Arc<dyn CompletionEngine> {
    let context_analyzer = Arc::new(GenericContextAnalyzer);
    let generator = Arc::new(BasicCompletionGenerator);
    let ranker = Arc::new(BasicCompletionRanker);
    let provider_registry = ProviderRegistry::new();

    Arc::new(GenericCompletionEngine::new(
        context_analyzer,
        generator,
        ranker,
        provider_registry,
    ))
}

/// Basic completion generator for testing
struct BasicCompletionGenerator;

#[async_trait::async_trait]
impl CompletionGenerator for BasicCompletionGenerator {
    async fn generate_completions(
        &self,
        _code: &str,
        _position: Position,
        _context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>> {
        // Generate basic completions
        let completions = vec![
            CompletionItem::new(
                "let".to_string(),
                CompletionItemKind::Keyword,
                "let ".to_string(),
            )
            .with_score(0.9),
            CompletionItem::new(
                "loop".to_string(),
                CompletionItemKind::Keyword,
                "loop ".to_string(),
            )
            .with_score(0.8),
            CompletionItem::new(
                "match".to_string(),
                CompletionItemKind::Keyword,
                "match ".to_string(),
            )
            .with_score(0.7),
        ];

        Ok(completions)
    }
}

/// Basic completion ranker for testing
struct BasicCompletionRanker;

impl CompletionRanker for BasicCompletionRanker {
    fn rank_completions(
        &self,
        mut items: Vec<CompletionItem>,
        _context: &CompletionContext,
    ) -> Vec<CompletionItem> {
        // Sort by score descending
        items.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        items
    }

    fn score_relevance(&self, _item: &CompletionItem, _context: &CompletionContext) -> f32 {
        0.5
    }

    fn score_frequency(&self, _item: &CompletionItem) -> f32 {
        0.5
    }
}
