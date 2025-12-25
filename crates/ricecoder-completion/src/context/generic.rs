//! Generic text-based context analyzer
//!
//! Provides basic context analysis without requiring tree-sitter or LSP.
//! Used as a fallback when more sophisticated analyzers are unavailable.

use async_trait::async_trait;

use super::analyzer::ContextAnalyzer;
use super::utils;
use crate::types::*;

/// Generic text-based context analyzer
///
/// Provides basic context analysis without requiring tree-sitter or LSP.
/// Used as a fallback when more sophisticated analyzers are unavailable.
#[derive(Debug, Clone, Default)]
pub struct GenericContextAnalyzer;

impl GenericContextAnalyzer {
    /// Create a new generic context analyzer
    pub fn new() -> Self {
        Self
    }

    fn detect_scope_kind(&self, code: &str, byte_offset: usize) -> ScopeKind {
        let before = &code[..byte_offset.min(code.len())];

        // Count open braces to estimate nesting
        let open_braces = before.matches('{').count();
        let close_braces = before.matches('}').count();
        let depth = open_braces.saturating_sub(close_braces);

        // Simple heuristics for scope detection
        if before.contains("fn ") && depth > 0 {
            ScopeKind::Function
        } else if before.contains("impl ") {
            ScopeKind::Impl
        } else if before.contains("struct ") || before.contains("enum ") {
            ScopeKind::Struct
        } else if before.contains("mod ") {
            ScopeKind::Module
        } else if depth == 0 {
            ScopeKind::Global
        } else {
            ScopeKind::Block
        }
    }

    fn create_scope(&self, kind: ScopeKind) -> Scope {
        Scope {
            kind,
            name: None,
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
        }
    }

    fn extract_symbols(&self, code: &str, scope: Scope) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        for line in code.lines() {
            let trimmed = line.trim();

            // Function declarations
            if let Some(name) = Self::extract_fn_name(trimmed) {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Function,
                    scope: scope.clone(),
                    type_info: None,
                    documentation: None,
                });
            }

            // Variable declarations
            if let Some(name) = Self::extract_let_name(trimmed) {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Variable,
                    scope: scope.clone(),
                    type_info: None,
                    documentation: None,
                });
            }

            // Struct declarations
            if let Some(name) = Self::extract_struct_name(trimmed) {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Struct,
                    scope: scope.clone(),
                    type_info: None,
                    documentation: None,
                });
            }
        }

        symbols
    }

    fn extract_fn_name(line: &str) -> Option<String> {
        if line.starts_with("fn ") || line.contains(" fn ") {
            let after_fn = line.split("fn ").nth(1)?;
            let name = after_fn.split('(').next()?.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        None
    }

    fn extract_let_name(line: &str) -> Option<String> {
        if line.starts_with("let ") {
            let after_let = line.strip_prefix("let ")?.trim();
            let after_mut = after_let.strip_prefix("mut ").unwrap_or(after_let);
            let name = after_mut.split(&[':', '=', ' '][..]).next()?.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        None
    }

    fn extract_struct_name(line: &str) -> Option<String> {
        if line.starts_with("struct ") || line.contains(" struct ") {
            let after_struct = line.split("struct ").nth(1)?;
            let name = after_struct.split(&['{', '(', '<', ' '][..]).next()?.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        None
    }
}

#[async_trait]
impl ContextAnalyzer for GenericContextAnalyzer {
    async fn analyze_context(
        &self,
        code: &str,
        position: Position,
        language: &str,
    ) -> CompletionResult<CompletionContext> {
        let byte_offset = utils::position_to_byte_offset(code, position);
        let prefix = utils::extract_prefix(code, position);
        let scope_kind = self.detect_scope_kind(code, byte_offset);
        let scope = self.create_scope(scope_kind);
        let symbols = self.extract_symbols(code, scope.clone());

        Ok(CompletionContext {
            prefix,
            position,
            scope,
            language: language.to_string(),
            available_symbols: symbols,
            expected_type: None,
        })
    }

    fn get_available_symbols(&self, context: &CompletionContext, _code: &str) -> Vec<Symbol> {
        context.available_symbols.clone()
    }

    fn infer_expected_type(&self, _context: &CompletionContext) -> Option<Type> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generic_analyzer_basic() {
        let analyzer = GenericContextAnalyzer::new();
        let code = "fn main() { let x = }";
        let position = Position::new(0, 20);

        let context = analyzer.analyze_context(code, position, "rust").await.unwrap();

        assert_eq!(context.scope.kind, ScopeKind::Function);
    }
}
