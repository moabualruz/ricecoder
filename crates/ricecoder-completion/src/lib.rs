/// RiceCoder Completion Engine
///
/// A language-agnostic code completion engine with pluggable providers for language-specific behavior.
///
/// # Architecture
///
/// The completion engine follows a layered architecture:
///
/// 1. **Configuration Layer**: Load and manage language-specific completion configurations
/// 2. **Context Analysis Layer**: Analyze code context to determine available symbols and expected types
/// 3. **Completion Generation Layer**: Generate completion suggestions (generic or language-specific)
/// 4. **Ranking Layer**: Rank completions by relevance, frequency, and recency
///
/// # Language Support
///
/// The engine supports multiple languages through a pluggable provider system:
///
/// - **Rust**: Full support with Rust-specific keywords and patterns
/// - **TypeScript**: Full support with TypeScript-specific keywords and patterns
/// - **Python**: Full support with Python-specific keywords and patterns
/// - **Generic**: Fallback for unconfigured languages using text-based completion
///
/// # Core Components
///
/// ## CompletionEngine
/// The main trait that orchestrates the completion process. Implementations:
/// - [`GenericCompletionEngine`]: Language-agnostic engine with pluggable providers
///
/// ## ContextAnalyzer
/// Analyzes code context to determine available symbols and expected types. Implementations:
/// - [`GenericContextAnalyzer`]: Basic text-based context analysis
/// - [`TreeSitterContextAnalyzer`]: Tree-sitter based scope and symbol detection
///
/// ## CompletionGenerator
/// Generates completion suggestions. Implementations:
/// - Built-in generic generator for text-based completions
/// - Language-specific providers (Rust, TypeScript, Python)
///
/// ## CompletionRanker
/// Ranks completions by relevance, frequency, and recency. Implementations:
/// - [`BasicCompletionRanker`]: Prefix matching and fuzzy matching
/// - [`AdvancedCompletionRanker`]: Advanced scoring with frequency and recency
///
/// ## CompletionProvider
/// Pluggable providers for language-specific behavior. Implementations:
/// - [`RustCompletionProvider`]: Rust-specific completions
/// - [`TypeScriptCompletionProvider`]: TypeScript-specific completions
/// - [`PythonCompletionProvider`]: Python-specific completions
/// - [`GenericTextProvider`]: Generic text-based completions
///
/// # Ghost Text
///
/// Ghost text displays inline suggestions in a lighter color. Components:
/// - [`GhostTextGenerator`]: Generates ghost text from completions
/// - [`GhostTextRenderer`]: Renders ghost text in the editor
/// - [`GhostTextStateManager`]: Manages ghost text state and keyboard handling
///
/// # Configuration
///
/// Configuration is loaded from YAML/JSON files and supports:
/// - Language-specific keywords and snippets
/// - Ranking weights for relevance, frequency, and recency
/// - Provider references for language-specific behavior
///
/// # Example: Basic Usage
///
/// ```ignore
/// use ricecoder_completion::engine::*;
/// use ricecoder_completion::types::*;
/// use ricecoder_completion::context::*;
/// use ricecoder_completion::ranker::*;
/// use std::sync::Arc;
///
/// // Create components
/// let context_analyzer = Arc::new(GenericContextAnalyzer);
/// let generator = Arc::new(BasicCompletionGenerator);
/// let ranker = Arc::new(BasicCompletionRanker::default_weights());
/// let mut registry = ProviderRegistry::new();
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
///
/// # Example: With Language-Specific Provider
///
/// ```ignore
/// use ricecoder_completion::providers::RustCompletionProvider;
/// use std::sync::Arc;
///
/// // Register Rust provider
/// let mut registry = ProviderRegistry::new();
/// registry.register(Arc::new(RustCompletionProvider));
///
/// // Now Rust code will use language-specific completions
/// ```
///
/// # Example: Ghost Text
///
/// ```ignore
/// use ricecoder_completion::ghost_text::*;
/// use ricecoder_completion::types::*;
///
/// let generator = BasicGhostTextGenerator;
/// let completion = CompletionItem::new(
///     "println!".to_string(),
///     CompletionItemKind::Macro,
///     "println!(\"{}\", {})".to_string(),
/// );
///
/// let ghost_text = generator.generate_ghost_text(&completion, Position::new(0, 5))?;
/// ```
pub mod config;
pub mod context;
pub mod engine;
pub mod ghost_text;
pub mod ghost_text_state;
pub mod history;
pub mod language;
pub mod providers;
pub mod ranker;
pub mod types;

// Re-export public types and traits
pub use config::{ConfigFormat, ConfigLoader, LanguageConfigRegistry};
pub use context::{ContextAnalyzer, GenericContextAnalyzer, TreeSitterContextAnalyzer};
pub use engine::{
    CompletionEngine, CompletionGenerator, CompletionProvider, CompletionRanker,
    GenericCompletionEngine, ProviderRegistry,
};
pub use ghost_text::{
    BasicGhostTextGenerator, BasicGhostTextRenderer, GhostTextGenerator, GhostTextRenderer,
    GhostTextStyle,
};
pub use ghost_text_state::{
    BasicGhostTextKeyHandler, BasicGhostTextStateManager, GhostTextKeyHandler,
    GhostTextState, GhostTextStateManager, PartialAcceptanceMode,
};
pub use history::{CompletionHistory, CompletionUsage};
pub use language::{Language, LanguageDetector};
pub use providers::{
    CompletionProviderFactory, DartCompletionProvider, GenericTextProvider,
    GoCompletionProvider, JavaCompletionProvider, KotlinCompletionProvider,
    PythonCompletionProvider, RustCompletionProvider, TypeScriptCompletionProvider,
};
pub use ranker::{AdvancedCompletionRanker, BasicCompletionRanker};
pub use types::*;

// Re-export storage integration
pub use ricecoder_storage::get_builtin_completion_configs;
pub use ricecoder_storage::get_completion_config;
