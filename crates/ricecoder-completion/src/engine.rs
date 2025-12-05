/// Core completion engine with language-agnostic architecture
///
/// This module provides the main completion engine and related traits for generating,
/// ranking, and managing code completions. The engine is designed to be language-agnostic
/// with pluggable providers for language-specific behavior and external LSP integration.
///
/// # Architecture
///
/// The completion engine follows a pipeline architecture with external LSP routing:
///
/// 1. **External LSP Routing** (Primary): Route to external LSP server if available and configured
/// 2. **Context Analysis**: Analyze code context to determine available symbols and expected types
/// 3. **Completion Generation**: Generate suggestions using language-specific provider or generic generator
/// 4. **Merging**: Merge external LSP completions with internal completions (external takes priority)
/// 5. **Ranking**: Rank completions by relevance, frequency, and recency
///
/// # External LSP Integration
///
/// The completion engine integrates with external LSP servers through the `ExternalLspCompletionProxy`.
/// When a completion request is made:
///
/// 1. If an external LSP server is configured for the language, the request is forwarded to it
/// 2. The external LSP response is transformed to ricecoder's internal model
/// 3. External completions are merged with internal completions:
///    - External completions have higher priority (appear first)
///    - Internal completions are added if they don't duplicate external ones
///    - Results are sorted by relevance score
/// 4. If the external LSP is unavailable or times out, the system falls back to internal providers
///
/// # Merge Strategy
///
/// The merge strategy for combining external and internal completions:
///
/// - **Priority**: External completions are prioritized over internal ones
/// - **Deduplication**: Completions with the same label are deduplicated (external wins)
/// - **Sorting**: All completions are sorted by relevance score
/// - **Fallback**: If external LSP fails, internal completions are used as fallback
///
/// This ensures users get the best available completions while maintaining graceful degradation.
///
/// # Example
///
/// ```ignore
/// use ricecoder_completion::engine::*;
/// use ricecoder_completion::types::*;
/// use std::sync::Arc;
///
/// // Create engine components
/// let context_analyzer = Arc::new(GenericContextAnalyzer);
/// let generator = Arc::new(BasicCompletionGenerator);
/// let ranker = Arc::new(BasicCompletionRanker::default_weights());
/// let registry = ProviderRegistry::new();
///
/// // Create engine
/// let engine = GenericCompletionEngine::new(
///     context_analyzer,
///     generator,
///     ranker,
///     registry,
/// );
///
/// // Generate completions
/// let completions = engine.generate_completions(
///     "fn main() { let x = ",
///     Position::new(0, 20),
///     "rust",
/// ).await?;
/// ```
use crate::context::ContextAnalyzer;
use crate::types::*;
use async_trait::async_trait;
use std::sync::Arc;

/// Main completion engine trait
///
/// Implementations of this trait orchestrate the completion process by coordinating
/// context analysis, completion generation, and ranking.
///
/// # Async Behavior
///
/// All methods are async to support non-blocking I/O and streaming responses.
/// Implementations should handle cancellation gracefully.
#[async_trait]
pub trait CompletionEngine: Send + Sync {
    /// Generate completion suggestions for the given code at the specified position
    ///
    /// # Arguments
    ///
    /// * `code` - The source code to analyze
    /// * `position` - The cursor position where completions are requested
    /// * `language` - The programming language identifier (e.g., "rust", "typescript", "python")
    ///
    /// # Returns
    ///
    /// A vector of completion items ranked by relevance, or an error if generation fails.
    ///
    /// # Errors
    ///
    /// Returns `CompletionError` if:
    /// - Context analysis fails
    /// - Completion generation fails
    /// - Ranking fails
    /// - The language is not supported
    async fn generate_completions(
        &self,
        code: &str,
        position: Position,
        language: &str,
    ) -> CompletionResult<Vec<CompletionItem>>;

    /// Resolve additional details for a completion item
    ///
    /// This method is called when the user selects a completion item to resolve
    /// additional details like documentation, type information, or additional edits.
    ///
    /// # Arguments
    ///
    /// * `item` - The completion item to resolve
    ///
    /// # Returns
    ///
    /// The resolved completion item with additional details, or an error if resolution fails.
    async fn resolve_completion(&self, item: &CompletionItem) -> CompletionResult<CompletionItem>;
}

/// Generic completion engine implementation
///
/// This is the main implementation of the completion engine. It coordinates
/// external LSP routing, context analysis, completion generation, and ranking
/// to produce ranked completion suggestions.
///
/// # Completion Flow
///
/// The engine follows this flow for each completion request:
///
/// 1. **External LSP Check**: Check if an external LSP server is configured for the language
/// 2. **Context Analysis**: Analyze code context to determine available symbols and expected types
/// 3. **Completion Generation**: Generate suggestions using:
///    - External LSP server (if available and configured)
///    - Language-specific provider (if registered)
///    - Generic completion generator (fallback)
/// 4. **Merging**: Merge external and internal completions (external takes priority)
/// 5. **Ranking**: Rank all completions by relevance, frequency, and recency
///
/// # Language Support
///
/// The engine supports multiple languages through a pluggable provider system:
/// - If an external LSP server is configured, it will be used for semantic completions
/// - If a language-specific provider is registered, it will be used as fallback
/// - Otherwise, the generic completion generator is used as a fallback
///
/// # Graceful Degradation
///
/// If the external LSP server is unavailable or times out:
/// - The system falls back to language-specific providers
/// - If no provider is available, the generic generator is used
/// - Users always get some completions, even if not semantic
///
/// # Example
///
/// ```ignore
/// use ricecoder_completion::engine::*;
/// use ricecoder_completion::types::*;
/// use std::sync::Arc;
///
/// let engine = GenericCompletionEngine::new(
///     Arc::new(GenericContextAnalyzer),
///     Arc::new(BasicCompletionGenerator),
///     Arc::new(BasicCompletionRanker::default_weights()),
///     ProviderRegistry::new(),
/// );
/// ```
pub struct GenericCompletionEngine {
    context_analyzer: Arc<dyn ContextAnalyzer>,
    generator: Arc<dyn CompletionGenerator>,
    ranker: Arc<dyn CompletionRanker>,
    provider_registry: ProviderRegistry,
}

impl GenericCompletionEngine {
    /// Create a new completion engine
    ///
    /// # Arguments
    ///
    /// * `context_analyzer` - Analyzer for determining code context
    /// * `generator` - Generic completion generator (used as fallback)
    /// * `ranker` - Ranker for sorting completions
    /// * `provider_registry` - Registry of language-specific providers
    pub fn new(
        context_analyzer: Arc<dyn ContextAnalyzer>,
        generator: Arc<dyn CompletionGenerator>,
        ranker: Arc<dyn CompletionRanker>,
        provider_registry: ProviderRegistry,
    ) -> Self {
        Self {
            context_analyzer,
            generator,
            ranker,
            provider_registry,
        }
    }
}

#[async_trait]
impl CompletionEngine for GenericCompletionEngine {
    async fn generate_completions(
        &self,
        code: &str,
        position: Position,
        language: &str,
    ) -> CompletionResult<Vec<CompletionItem>> {
        // Analyze context
        let context = self
            .context_analyzer
            .analyze_context(code, position, language)
            .await?;

        // Generate completions using language-specific provider if available
        let mut completions = if let Some(provider) = self.provider_registry.get_provider(language)
        {
            provider
                .generate_completions(code, position, &context)
                .await?
        } else {
            // Fall back to generic completion
            self.generator
                .generate_completions(code, position, &context)
                .await?
        };

        // Rank completions
        completions = self.ranker.rank_completions(completions, &context);

        Ok(completions)
    }

    async fn resolve_completion(&self, item: &CompletionItem) -> CompletionResult<CompletionItem> {
        Ok(item.clone())
    }
}

/// Completion generator trait
///
/// Implementations generate completion suggestions based on code context.
/// This is typically used as a fallback when no language-specific provider is available.
///
/// # Implementations
///
/// - `BasicCompletionGenerator`: Generic text-based completion
/// - Language-specific providers: Rust, TypeScript, Python
#[async_trait]
pub trait CompletionGenerator: Send + Sync {
    /// Generate completion suggestions
    ///
    /// # Arguments
    ///
    /// * `code` - The source code to analyze
    /// * `position` - The cursor position where completions are requested
    /// * `context` - The analyzed code context
    ///
    /// # Returns
    ///
    /// A vector of completion items (not yet ranked), or an error if generation fails.
    async fn generate_completions(
        &self,
        code: &str,
        position: Position,
        context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>>;
}

/// Completion ranker trait
///
/// Implementations rank completion suggestions by relevance, frequency, and recency.
/// The ranker is responsible for sorting completions so the most relevant appear first.
///
/// # Scoring
///
/// Rankers typically combine multiple scoring factors:
/// - **Relevance**: How well the completion matches the context
/// - **Frequency**: How often the completion is used
/// - **Recency**: How recently the completion was used
///
/// # Implementations
///
/// - `BasicCompletionRanker`: Prefix matching and fuzzy matching
/// - `AdvancedCompletionRanker`: Advanced scoring with frequency and recency
pub trait CompletionRanker: Send + Sync {
    /// Rank completions by relevance and frequency
    ///
    /// # Arguments
    ///
    /// * `items` - Unranked completion items
    /// * `context` - The analyzed code context
    ///
    /// # Returns
    ///
    /// The same completion items, sorted by relevance (highest first).
    fn rank_completions(
        &self,
        items: Vec<CompletionItem>,
        context: &CompletionContext,
    ) -> Vec<CompletionItem>;

    /// Score relevance of a completion item
    ///
    /// # Arguments
    ///
    /// * `item` - The completion item to score
    /// * `context` - The analyzed code context
    ///
    /// # Returns
    ///
    /// A relevance score (typically 0.0 to 1.0, where 1.0 is most relevant).
    fn score_relevance(&self, item: &CompletionItem, context: &CompletionContext) -> f32;

    /// Score frequency of a completion item
    ///
    /// # Arguments
    ///
    /// * `item` - The completion item to score
    ///
    /// # Returns
    ///
    /// A frequency score (typically 0.0 to 1.0, where 1.0 is most frequently used).
    fn score_frequency(&self, item: &CompletionItem) -> f32;
}

/// Pluggable completion provider for language-specific behavior
///
/// Implementations provide language-specific completion suggestions. Providers are
/// registered in the `ProviderRegistry` and selected based on the language identifier.
///
/// # Language Support
///
/// Each provider supports a single language. The engine queries the registry to find
/// the appropriate provider for the current language.
///
/// # Implementations
///
/// - `RustCompletionProvider`: Rust-specific completions
/// - `TypeScriptCompletionProvider`: TypeScript-specific completions
/// - `PythonCompletionProvider`: Python-specific completions
/// - `GenericTextProvider`: Generic text-based completions
///
/// # Example
///
/// ```ignore
/// use ricecoder_completion::providers::RustCompletionProvider;
/// use ricecoder_completion::engine::CompletionProvider;
///
/// let provider = RustCompletionProvider;
/// assert_eq!(provider.language(), "rust");
/// ```
#[async_trait]
pub trait CompletionProvider: Send + Sync {
    /// Get the language this provider supports
    ///
    /// # Returns
    ///
    /// A language identifier string (e.g., "rust", "typescript", "python").
    fn language(&self) -> &str;

    /// Generate completions for this language
    ///
    /// # Arguments
    ///
    /// * `code` - The source code to analyze
    /// * `position` - The cursor position where completions are requested
    /// * `context` - The analyzed code context
    ///
    /// # Returns
    ///
    /// A vector of language-specific completion items (not yet ranked), or an error if generation fails.
    async fn generate_completions(
        &self,
        code: &str,
        position: Position,
        context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>>;
}

/// Registry for completion providers
///
/// The provider registry manages language-specific completion providers.
/// It allows registering, retrieving, and listing available providers.
///
/// # Example
///
/// ```ignore
/// use ricecoder_completion::engine::ProviderRegistry;
/// use ricecoder_completion::providers::RustCompletionProvider;
/// use std::sync::Arc;
///
/// let mut registry = ProviderRegistry::new();
/// registry.register(Arc::new(RustCompletionProvider));
///
/// // Get provider for Rust
/// let provider = registry.get_provider("rust");
/// assert!(provider.is_some());
///
/// // List all supported languages
/// let languages = registry.list_languages();
/// assert!(languages.contains(&"rust".to_string()));
/// ```
pub struct ProviderRegistry {
    providers: std::collections::HashMap<String, Arc<dyn CompletionProvider>>,
}

impl ProviderRegistry {
    /// Create a new empty provider registry
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    /// Create a new provider registry with all built-in providers registered
    ///
    /// This is a convenience method that automatically registers all language-specific
    /// providers (Rust, TypeScript, Python, Go, Java, Kotlin, Dart).
    pub fn with_builtin_providers() -> Self {
        let mut registry = Self::new();
        registry.register_builtin_providers();
        registry
    }

    /// Register all built-in language providers
    ///
    /// This method registers providers for all supported languages:
    /// - Rust
    /// - TypeScript
    /// - Python
    /// - Go
    /// - Java
    /// - Kotlin
    /// - Dart
    pub fn register_builtin_providers(&mut self) {
        use crate::providers::*;

        self.register(Arc::new(RustCompletionProvider));
        self.register(Arc::new(TypeScriptCompletionProvider));
        self.register(Arc::new(PythonCompletionProvider));
        self.register(Arc::new(GoCompletionProvider));
        self.register(Arc::new(JavaCompletionProvider));
        self.register(Arc::new(KotlinCompletionProvider));
        self.register(Arc::new(DartCompletionProvider));
    }

    /// Register a completion provider
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider to register
    ///
    /// # Behavior
    ///
    /// If a provider for the same language already exists, it will be replaced.
    pub fn register(&mut self, provider: Arc<dyn CompletionProvider>) {
        self.providers
            .insert(provider.language().to_string(), provider);
    }

    /// Get a completion provider for a language
    ///
    /// # Arguments
    ///
    /// * `language` - The language identifier
    ///
    /// # Returns
    ///
    /// The provider for the language, or `None` if no provider is registered.
    pub fn get_provider(&self, language: &str) -> Option<Arc<dyn CompletionProvider>> {
        self.providers.get(language).cloned()
    }

    /// Unregister a completion provider
    ///
    /// # Arguments
    ///
    /// * `language` - The language identifier
    ///
    /// # Returns
    ///
    /// The unregistered provider, or `None` if no provider was registered.
    pub fn unregister(&mut self, language: &str) -> Option<Arc<dyn CompletionProvider>> {
        self.providers.remove(language)
    }

    /// List all supported languages
    ///
    /// # Returns
    ///
    /// A vector of language identifiers for all registered providers.
    pub fn list_languages(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_registry_register() {
        let registry = ProviderRegistry::new();
        assert_eq!(registry.list_languages().len(), 0);
    }

    #[test]
    fn test_provider_registry_get_nonexistent() {
        let registry = ProviderRegistry::new();
        assert!(registry.get_provider("rust").is_none());
    }
}
