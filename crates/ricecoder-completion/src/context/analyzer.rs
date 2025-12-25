// Context analyzer trait

use async_trait::async_trait;
use crate::types::*;

/// Context analyzer trait for analyzing code context
///
/// Implementations analyze code context to determine available symbols, scopes, and expected types.
/// This information is used by the completion generator to provide relevant suggestions.
///
/// # Example
///
/// ```ignore
/// use ricecoder_completion::context::*;
/// use ricecoder_completion::types::*;
///
/// let analyzer = GenericContextAnalyzer;
/// let context = analyzer.analyze_context(
///     "fn main() { let x = ",
///     Position::new(0, 20),
///     "rust",
/// ).await?;
///
/// println!("Scope: {:?}", context.scope);
/// println!("Available symbols: {:?}", context.available_symbols);
/// ```
#[async_trait]
pub trait ContextAnalyzer: Send + Sync {
    /// Analyze the code context at the given position
    ///
    /// # Arguments
    ///
    /// * `code` - The source code to analyze
    /// * `position` - The cursor position where context is requested
    /// * `language` - The programming language identifier
    ///
    /// # Returns
    ///
    /// A `CompletionContext` containing scope, available symbols, and expected type information.
    ///
    /// # Errors
    ///
    /// Returns `CompletionError` if:
    /// - The language is not supported
    /// - Code parsing fails
    /// - Context analysis fails
    async fn analyze_context(
        &self,
        code: &str,
        position: Position,
        language: &str,
    ) -> CompletionResult<CompletionContext>;

    /// Get available symbols in the given context
    ///
    /// # Arguments
    ///
    /// * `context` - The completion context
    /// * `code` - The source code
    ///
    /// # Returns
    ///
    /// A vector of symbols available in the given context.
    fn get_available_symbols(&self, context: &CompletionContext, code: &str) -> Vec<Symbol>;

    /// Infer the expected type at the given context
    ///
    /// # Arguments
    ///
    /// * `context` - The completion context
    ///
    /// # Returns
    ///
    /// The expected type at the cursor position, or `None` if it cannot be inferred.
    fn infer_expected_type(&self, context: &CompletionContext) -> Option<Type>;
}
