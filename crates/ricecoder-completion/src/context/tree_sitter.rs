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
            if trimmed.starts_with("class ") {
                return ScopeKind::Class;
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

    fn extract_symbols(
        &self,
        code: &str,
        scope: Scope,
        language: &str,
        position: Position,
    ) -> Vec<Symbol> {
        let mut symbols = utils::builtin_symbols(language);
        if let Some(parameters) = Self::extract_enclosing_function_parameters(
            code,
            language,
            utils::position_to_byte_offset(code, position),
        ) {
            symbols.extend(parameters.into_iter().map(|name| Symbol {
                name,
                kind: SymbolKind::Parameter,
                scope: scope.clone(),
                type_info: Some("parameter".to_string()),
                documentation: Some("Function parameter".to_string()),
            }));
        }

        for (line_number, line) in code.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            // Functions
            if trimmed.contains("fn ")
                || trimmed.starts_with("function ")
                || trimmed.starts_with("def ")
            {
                if let Some(name) = Self::extract_fn_name(trimmed).filter(|name| {
                    if line_number as u32 != position.line {
                        return true;
                    }
                    let Some(name_start) = line.find(name) else {
                        return true;
                    };
                    let name_end = name_start + name.chars().count();
                    !(position.character as usize >= name_start
                        && (position.character as usize) < name_end)
                }) {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        scope: scope.clone(),
                        type_info: Some("function".to_string()),
                        documentation: Some("Function declared in the current source".to_string()),
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
            if language == "rust" && trimmed.contains("const ") {
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
            if language == "rust" && trimmed.starts_with("let ") {
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

            if matches!(
                language,
                "typescript" | "ts" | "tsx" | "javascript" | "js" | "jsx"
            ) {
                if let Some(name) = Self::extract_js_variable_name(trimmed) {
                    let kind = if trimmed.starts_with("const ") {
                        SymbolKind::Constant
                    } else {
                        SymbolKind::Variable
                    };
                    symbols.push(Symbol {
                        name,
                        kind,
                        scope: scope.clone(),
                        type_info: None,
                        documentation: None,
                    });
                }
            }

            if matches!(language, "python" | "py") {
                if let Some(name) = Self::extract_python_variable_name(trimmed) {
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
        let after_fn = line
            .split("fn ")
            .nth(1)
            .or_else(|| line.strip_prefix("function "))
            .or_else(|| line.strip_prefix("def "))?;
        after_fn.contains('(').then_some(())?;
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
        let name = after_keyword
            .split(&['{', '(', '<', ' ', ';'][..])
            .next()?
            .trim();
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

    fn extract_js_variable_name(line: &str) -> Option<String> {
        let rest = line
            .strip_prefix("const ")
            .or_else(|| line.strip_prefix("let "))
            .or_else(|| line.strip_prefix("var "))?;
        let name = rest.split(&[':', '=', ';', ' '][..]).next()?.trim();
        (!name.is_empty()).then(|| name.to_string())
    }

    fn extract_python_variable_name(line: &str) -> Option<String> {
        let name = line.split('=').next()?.trim();
        if name.is_empty()
            || line.starts_with("def ")
            || line.starts_with("class ")
            || line.starts_with("return ")
        {
            return None;
        }
        name.chars()
            .all(|ch| ch.is_alphanumeric() || ch == '_')
            .then(|| name.to_string())
    }

    fn extract_enclosing_function_parameters(
        code: &str,
        language: &str,
        byte_offset: usize,
    ) -> Option<Vec<String>> {
        fn find_functions<'tree>(
            node: tree_sitter::Node<'tree>,
            code: &str,
            byte_offset: usize,
            language: &str,
        ) -> Option<Vec<tree_sitter::Node<'tree>>> {
            if byte_offset < node.start_byte() || byte_offset > node.end_byte() {
                return None;
            }

            let is_function = match language {
                "rust" => node.kind() == "function_item",
                "typescript" | "ts" | "tsx" | "javascript" | "js" | "jsx" => matches!(
                    node.kind(),
                    "function_declaration"
                        | "function_expression"
                        | "generator_function"
                        | "generator_function_declaration"
                        | "generator_function_expression"
                        | "method_definition"
                        | "arrow_function"
                ),
                "python" | "py" => node.kind() == "function_definition",
                _ => false,
            };
            let cursor_in_body = node.child_by_field_name("body").is_some_and(|body| {
                let cursor_at_body_end = byte_offset == body.end_byte()
                    && (matches!(language, "python" | "py")
                        || !body
                            .utf8_text(code.as_bytes())
                            .unwrap_or_default()
                            .trim_end()
                            .ends_with('}'));
                body.start_byte() <= byte_offset
                    && (byte_offset < body.end_byte() || cursor_at_body_end)
            });

            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                if let Some(mut functions) = find_functions(child, code, byte_offset, language) {
                    if is_function && cursor_in_body {
                        functions.push(node);
                    }
                    return Some(functions);
                }
            }

            (is_function && cursor_in_body).then(|| vec![node])
        }

        let grammar = match language {
            "rust" => tree_sitter_rust::LANGUAGE.into(),
            "typescript" | "ts" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            "tsx" | "jsx" | "javascript" | "js" => tree_sitter_typescript::LANGUAGE_TSX.into(),
            "python" | "py" => tree_sitter_python::LANGUAGE.into(),
            _ => return None,
        };
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&grammar).ok()?;
        let tree = parser.parse(code, None)?;
        let mut functions = find_functions(tree.root_node(), code, byte_offset, language)?;
        if language == "rust" {
            functions.truncate(1);
        }

        let mut parameters = Vec::new();
        for function in functions {
            let Some(parameter_node) = function
                .child_by_field_name("parameters")
                .or_else(|| function.child_by_field_name("parameter"))
            else {
                continue;
            };
            let Some(parameter_text) = parameter_node.utf8_text(code.as_bytes()).ok() else {
                continue;
            };
            let function_parameters = if parameter_node.kind() == "identifier" {
                vec![parameter_text.to_string()]
            } else if let Some(parameters) = Self::extract_parameters(parameter_text, language) {
                parameters
            } else {
                continue;
            };
            for parameter in function_parameters {
                if !parameters.contains(&parameter) {
                    parameters.push(parameter);
                }
            }
        }
        Some(parameters)
    }

    fn extract_parameters(line: &str, language: &str) -> Option<Vec<String>> {
        let start = line.find('(')?;
        let end = line[start + 1..].find(')')? + start + 1;
        Some(
            line[start + 1..end]
                .split(',')
                .filter_map(|parameter| {
                    let parameter = parameter.trim();
                    let name = match language {
                        "rust" => parameter.split(':').next()?.trim(),
                        "typescript" | "ts" | "tsx" | "javascript" | "js" | "jsx" => {
                            parameter.split(':').next()?.trim()
                        }
                        "python" | "py" => parameter.split(':').next()?.trim(),
                        _ => return None,
                    };
                    (!name.is_empty() && name != "self" && name != "this").then(|| name.to_string())
                })
                .collect(),
        )
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
        if utils::builtin_symbols(language).is_empty() {
            return Err(CompletionError::UnsupportedLanguage(language.to_string()));
        }
        let byte_offset = utils::position_to_byte_offset(code, position);
        let scope_kind = self.analyze_scope_kind(code, byte_offset);
        let scope = self.create_scope(scope_kind);
        let symbols = self.extract_symbols(code, scope.clone(), language, position);
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

        let context = analyzer
            .analyze_context(code, position, "rust")
            .await
            .unwrap();

        assert!(context.available_symbols.iter().any(|s| s.name == "hello"));
        assert!(context.available_symbols.iter().any(|s| s.name == "Foo"));
        assert!(context.available_symbols.iter().any(|s| s.name == "Bar"));
    }

    #[tokio::test]
    async fn test_only_enclosing_function_parameters_are_available() {
        let analyzer = TreeSitterContextAnalyzer::new();
        let code = r#"
fn first(first_param: i32) {
    let _ = first_param;
}

fn second(second_param: i32) {
    let _ = second_param;
}
"#;
        let position = Position::new(2, 10);

        let context = analyzer
            .analyze_context(code, position, "rust")
            .await
            .unwrap();

        assert!(context
            .available_symbols
            .iter()
            .any(|symbol| symbol.name == "first_param"));
        assert!(!context
            .available_symbols
            .iter()
            .any(|symbol| symbol.name == "second_param"));
    }

    #[tokio::test]
    async fn enclosing_function_parameters_include_lexical_ancestors() {
        let analyzer = TreeSitterContextAnalyzer::new();
        let code = r#"function outer(outer_param) {
    function inner(inner_param) {
        return inner_param;
    }
}"#;
        let position = Position::new(2, 16);

        let context = analyzer
            .analyze_context(code, position, "javascript")
            .await
            .unwrap();

        assert!(context
            .available_symbols
            .iter()
            .any(|symbol| symbol.name == "inner_param"));
        assert!(context
            .available_symbols
            .iter()
            .any(|symbol| symbol.name == "outer_param"));
    }

    #[tokio::test]
    async fn python_function_parameters_are_available_at_the_body_end() {
        let analyzer = TreeSitterContextAnalyzer::new();
        let code = r#"def first(first_param):
    return first_param"#;
        let position = Position::new(1, "    return first_param".len() as u32);

        let context = analyzer
            .analyze_context(code, position, "python")
            .await
            .unwrap();

        assert!(context
            .available_symbols
            .iter()
            .any(|symbol| symbol.name == "first_param"));
    }

    #[tokio::test]
    async fn generator_function_parameters_are_available() {
        let analyzer = TreeSitterContextAnalyzer::new();
        let code = r#"function* generate(item) {
    yield item;
}"#;
        let position = Position::new(1, 14);

        let context = analyzer
            .analyze_context(code, position, "javascript")
            .await
            .unwrap();

        assert!(context
            .available_symbols
            .iter()
            .any(|symbol| symbol.name == "item"));
    }

    #[tokio::test]
    async fn single_parameter_arrow_functions_expose_the_parameter() {
        let analyzer = TreeSitterContextAnalyzer::new();
        let code = "const identity = value => value;";
        let position = Position::new(0, 28);

        let context = analyzer
            .analyze_context(code, position, "javascript")
            .await
            .unwrap();

        assert!(context
            .available_symbols
            .iter()
            .any(|symbol| symbol.name == "value"));
    }
}
