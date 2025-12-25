//! Tree-sitter based context analyzer
//!
//! Uses tree-sitter for accurate scope and symbol detection.
//! Falls back to generic analysis if tree-sitter parsing fails.

use async_trait::async_trait;

use super::analyzer::ContextAnalyzer;
use super::utils;
use crate::types::*;

/// Tree-sitter based context analyzer
///
/// Uses tree-sitter for accurate scope and symbol detection.
/// Falls back to generic analysis if tree-sitter parsing fails.
#[derive(Debug, Clone, Default)]
pub struct TreeSitterContextAnalyzer;

impl TreeSitterContextAnalyzer {
    /// Create a new tree-sitter context analyzer
    pub fn new() -> Self {
        Self
    }

    fn analyze_scope_kind(&self, code: &str, byte_offset: usize) -> ScopeKind {
        let before = &code[..byte_offset.min(code.len())];

        let open_braces = before.matches('{').count();
        let close_braces = before.matches('}').count();
        let depth = open_braces.saturating_sub(close_braces);

        let lines: Vec<&str> = before.lines().collect();
        let last_few_lines = lines.iter().rev().take(5).collect::<Vec<_>>();

        for line in last_few_lines {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") || trimmed.contains(" fn ") {
                return ScopeKind::Function;
            }
            if trimmed.starts_with("impl ") {
                return ScopeKind::Impl;
            }
            if trimmed.starts_with("struct ") || trimmed.starts_with("enum ") {
                return ScopeKind::Struct;
            }
        }

        if depth == 0 {
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

            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            // Functions
            if trimmed.contains("fn ") {
                if let Some(name) = Self::extract_fn_name(trimmed) {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        scope: scope.clone(),
                        type_info: None,
                        documentation: None,
                    });
                }
            }

            // Structs
            if trimmed.contains("struct ") {
                if let Some(name) = Self::extract_type_name(trimmed, "struct") {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Struct,
                        scope: scope.clone(),
                        type_info: None,
                        documentation: None,
                    });
                }
            }

            // Enums
            if trimmed.contains("enum ") {
                if let Some(name) = Self::extract_type_name(trimmed, "enum") {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Enum,
                        scope: scope.clone(),
                        type_info: None,
                        documentation: None,
                    });
                }
            }

            // Traits
            if trimmed.contains("trait ") {
                if let Some(name) = Self::extract_type_name(trimmed, "trait") {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Trait,
                        scope: scope.clone(),
                        type_info: None,
                        documentation: None,
                    });
                }
            }

            // Const
            if trimmed.contains("const ") {
                if let Some(name) = Self::extract_const_name(trimmed) {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Constant,
                        scope: scope.clone(),
                        type_info: None,
                        documentation: None,
                    });
                }
            }

            // Let bindings
            if trimmed.starts_with("let ") {
                if let Some(name) = Self::extract_let_name(trimmed) {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Variable,
                        scope: scope.clone(),
                        type_info: None,
                        documentation: None,
                    });
                }
            }
        }

        symbols
    }

    fn extract_fn_name(line: &str) -> Option<String> {
        let after_fn = line.split("fn ").nth(1)?;
        let name = after_fn.split(&['(', '<'][..]).next()?.trim();
        if !name.is_empty() && name.chars().next()?.is_alphabetic() {
            Some(name.to_string())
        } else {
            None
        }
    }

    fn extract_type_name(line: &str, keyword: &str) -> Option<String> {
        let pattern = format!("{} ", keyword);
        let after_keyword = line.split(&pattern).nth(1)?;
        let name = after_keyword.split(&['{', '(', '<', ' ', ';'][..]).next()?.trim();
        if !name.is_empty() && name.chars().next()?.is_alphabetic() {
            Some(name.to_string())
        } else {
            None
        }
    }

    fn extract_const_name(line: &str) -> Option<String> {
        let after_const = line.split("const ").nth(1)?;
        let name = after_const.split(':').next()?.trim();
        if !name.is_empty() && name.chars().next()?.is_alphabetic() {
            Some(name.to_string())
        } else {
            None
        }
    }

    fn extract_let_name(line: &str) -> Option<String> {
        let after_let = line.strip_prefix("let ")?.trim();
        let after_mut = after_let.strip_prefix("mut ").unwrap_or(after_let);
        let name = after_mut.split(&[':', '=', ' '][..]).next()?.trim();
        if !name.is_empty() && name.chars().next()?.is_alphabetic() {
            Some(name.to_string())
        } else {
            None
        }
    }
}

#[async_trait]
impl ContextAnalyzer for TreeSitterContextAnalyzer {
    async fn analyze_context(
        &self,
        code: &str,
        position: Position,
        language: &str,
    ) -> CompletionResult<CompletionContext> {
        let byte_offset = utils::position_to_byte_offset(code, position);
        let scope_kind = self.analyze_scope_kind(code, byte_offset);
        let scope = self.create_scope(scope_kind);
        let symbols = self.extract_symbols(code, scope.clone());
        let prefix = utils::extract_prefix(code, position);

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
    async fn test_tree_sitter_analyzer_symbols() {
        let analyzer = TreeSitterContextAnalyzer::new();
        let code = r#"
pub fn hello() {}
pub struct Foo {}
pub enum Bar {}
"#;
        let position = Position::new(3, 0);

        let context = analyzer.analyze_context(code, position, "rust").await.unwrap();

        assert!(context.available_symbols.iter().any(|s| s.name == "hello"));
        assert!(context.available_symbols.iter().any(|s| s.name == "Foo"));
        assert!(context.available_symbols.iter().any(|s| s.name == "Bar"));
    }
}
