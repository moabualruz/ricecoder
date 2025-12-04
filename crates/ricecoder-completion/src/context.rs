/// Context analysis for code completion
///
/// This module provides context analysis for code completion. It analyzes code context
/// to determine available symbols, scopes, and expected types.
///
/// # Implementations
///
/// - [`GenericContextAnalyzer`]: Basic text-based context analysis
/// - [`TreeSitterContextAnalyzer`]: Tree-sitter based scope and symbol detection
use crate::types::*;
use async_trait::async_trait;
use tree_sitter::{Language, Parser};

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

/// Tree-sitter based context analyzer for scope detection
///
/// This analyzer uses tree-sitter to parse code and detect scopes, available symbols,
/// and expected types. It supports Rust, TypeScript, and Python.
///
/// # Supported Languages
///
/// - **Rust**: Full support with scope detection and symbol extraction
/// - **TypeScript/JavaScript**: Full support with scope detection and symbol extraction
/// - **Python**: Full support with scope detection and symbol extraction
///
/// # Example
///
/// ```ignore
/// use ricecoder_completion::context::TreeSitterContextAnalyzer;
/// use ricecoder_completion::types::Position;
///
/// let analyzer = TreeSitterContextAnalyzer;
/// let context = analyzer.analyze_context(
///     "fn main() { let x = ",
///     Position::new(0, 20),
///     "rust",
/// ).await?;
/// ```
pub struct TreeSitterContextAnalyzer;

impl TreeSitterContextAnalyzer {
    /// Get the tree-sitter language for the given language identifier
    fn get_language(language: &str) -> Option<Language> {
        match language {
            "rust" => Some(tree_sitter_rust::language()),
            "typescript" | "ts" | "tsx" | "javascript" | "js" | "jsx" => {
                Some(tree_sitter_typescript::language_typescript())
            }
            "python" | "py" => Some(tree_sitter_python::language()),
            _ => None,
        }
    }

    /// Get built-in symbols for a language
    fn get_builtin_symbols(language: &str) -> Vec<Symbol> {
        match language {
            "rust" => vec![
                Symbol {
                    name: "String".to_string(),
                    kind: SymbolKind::Type,
                    scope: Scope {
                        kind: ScopeKind::Global,
                        name: None,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    },
                    type_info: Some("std::string::String".to_string()),
                    documentation: Some("A UTF-8 encoded string".to_string()),
                },
                Symbol {
                    name: "Vec".to_string(),
                    kind: SymbolKind::Type,
                    scope: Scope {
                        kind: ScopeKind::Global,
                        name: None,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    },
                    type_info: Some("std::vec::Vec".to_string()),
                    documentation: Some("A growable array".to_string()),
                },
                Symbol {
                    name: "Option".to_string(),
                    kind: SymbolKind::Type,
                    scope: Scope {
                        kind: ScopeKind::Global,
                        name: None,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    },
                    type_info: Some("std::option::Option".to_string()),
                    documentation: Some("An optional value".to_string()),
                },
                Symbol {
                    name: "Result".to_string(),
                    kind: SymbolKind::Type,
                    scope: Scope {
                        kind: ScopeKind::Global,
                        name: None,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    },
                    type_info: Some("std::result::Result".to_string()),
                    documentation: Some("A result type for error handling".to_string()),
                },
            ],
            "typescript" | "javascript" => vec![
                Symbol {
                    name: "Array".to_string(),
                    kind: SymbolKind::Type,
                    scope: Scope {
                        kind: ScopeKind::Global,
                        name: None,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    },
                    type_info: Some("Array".to_string()),
                    documentation: Some("A JavaScript array".to_string()),
                },
                Symbol {
                    name: "Object".to_string(),
                    kind: SymbolKind::Type,
                    scope: Scope {
                        kind: ScopeKind::Global,
                        name: None,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    },
                    type_info: Some("Object".to_string()),
                    documentation: Some("A JavaScript object".to_string()),
                },
                Symbol {
                    name: "Promise".to_string(),
                    kind: SymbolKind::Type,
                    scope: Scope {
                        kind: ScopeKind::Global,
                        name: None,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    },
                    type_info: Some("Promise".to_string()),
                    documentation: Some("A promise for async operations".to_string()),
                },
                Symbol {
                    name: "Map".to_string(),
                    kind: SymbolKind::Type,
                    scope: Scope {
                        kind: ScopeKind::Global,
                        name: None,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    },
                    type_info: Some("Map".to_string()),
                    documentation: Some("A key-value map".to_string()),
                },
            ],
            "python" => vec![
                Symbol {
                    name: "list".to_string(),
                    kind: SymbolKind::Type,
                    scope: Scope {
                        kind: ScopeKind::Global,
                        name: None,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    },
                    type_info: Some("list".to_string()),
                    documentation: Some("A Python list".to_string()),
                },
                Symbol {
                    name: "dict".to_string(),
                    kind: SymbolKind::Type,
                    scope: Scope {
                        kind: ScopeKind::Global,
                        name: None,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    },
                    type_info: Some("dict".to_string()),
                    documentation: Some("A Python dictionary".to_string()),
                },
                Symbol {
                    name: "str".to_string(),
                    kind: SymbolKind::Type,
                    scope: Scope {
                        kind: ScopeKind::Global,
                        name: None,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    },
                    type_info: Some("str".to_string()),
                    documentation: Some("A Python string".to_string()),
                },
                Symbol {
                    name: "int".to_string(),
                    kind: SymbolKind::Type,
                    scope: Scope {
                        kind: ScopeKind::Global,
                        name: None,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    },
                    type_info: Some("int".to_string()),
                    documentation: Some("A Python integer".to_string()),
                },
            ],
            _ => Vec::new(),
        }
    }

    /// Collect symbols from imported modules
    fn collect_imported_symbols(code: &str, language: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        match language {
            "rust" => {
                // Look for use statements
                for line in code.lines() {
                    if line.trim().starts_with("use ") {
                        // Extract the imported item
                        if let Some(imported) = Self::extract_rust_import(line) {
                            symbols.push(Symbol {
                                name: imported.clone(),
                                kind: SymbolKind::Module,
                                scope: Scope {
                                    kind: ScopeKind::Global,
                                    name: None,
                                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                                },
                                type_info: None,
                                documentation: None,
                            });
                        }
                    }
                }
            }
            "typescript" | "javascript" => {
                // Look for import statements
                for line in code.lines() {
                    if line.trim().starts_with("import ") {
                        // Extract the imported items
                        if let Some(imported) = Self::extract_typescript_import(line) {
                            symbols.push(Symbol {
                                name: imported.clone(),
                                kind: SymbolKind::Module,
                                scope: Scope {
                                    kind: ScopeKind::Global,
                                    name: None,
                                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                                },
                                type_info: None,
                                documentation: None,
                            });
                        }
                    }
                }
            }
            "python" => {
                // Look for import statements
                for line in code.lines() {
                    if line.trim().starts_with("import ") || line.trim().starts_with("from ") {
                        // Extract the imported items
                        if let Some(imported) = Self::extract_python_import(line) {
                            symbols.push(Symbol {
                                name: imported.clone(),
                                kind: SymbolKind::Module,
                                scope: Scope {
                                    kind: ScopeKind::Global,
                                    name: None,
                                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                                },
                                type_info: None,
                                documentation: None,
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        symbols
    }

    /// Extract Rust import name from use statement
    fn extract_rust_import(line: &str) -> Option<String> {
        let line = line.trim();
        let import_part = line.strip_prefix("use ")?;
        let import_part = import_part.trim_end_matches(';');

        // Handle simple imports like "use std::vec::Vec;"
        if let Some(last_colon) = import_part.rfind("::") {
            let name = &import_part[last_colon + 2..];
            if !name.is_empty() && !name.contains('{') {
                return Some(name.to_string());
            }
        }

        // Handle glob imports like "use std::*;"
        if import_part.ends_with("::*") {
            return Some("*".to_string());
        }

        // Handle simple module imports
        if !import_part.contains("::") && !import_part.contains('{') {
            return Some(import_part.to_string());
        }

        None
    }

    /// Extract TypeScript import name from import statement
    fn extract_typescript_import(line: &str) -> Option<String> {
        let line = line.trim();
        let line = line.strip_prefix("import ")?;

        // Handle "import { name } from 'module'"
        if let Some(start) = line.find('{') {
            if let Some(end) = line.find('}') {
                let names = &line[start + 1..end];
                // Get the first imported name
                if let Some(name) = names.split(',').next() {
                    return Some(name.trim().to_string());
                }
            }
        }

        // Handle "import name from 'module'"
        if let Some(from_pos) = line.find(" from ") {
            let import_part = &line[..from_pos];
            return Some(import_part.trim().to_string());
        }

        None
    }

    /// Extract Python import name from import statement
    fn extract_python_import(line: &str) -> Option<String> {
        let line = line.trim();

        if let Some(import_part) = line.strip_prefix("import ") {
            // Handle "import module" or "import module as alias"
            if let Some(as_pos) = import_part.find(" as ") {
                return Some(import_part[as_pos + 4..].trim().to_string());
            }
            return Some(import_part.split(',').next()?.trim().to_string());
        }

        if let Some(rest) = line.strip_prefix("from ") {
            // Handle "from module import name"
            if let Some(import_pos) = rest.find(" import ") {
                let import_part = &rest[import_pos + 8..];
                // Get the first imported name
                if let Some(name) = import_part.split(',').next() {
                    return Some(name.trim().to_string());
                }
            }
        }

        None
    }

    /// Collect symbols from the current scope with full code access
    fn collect_scope_symbols_with_code(context: &CompletionContext, code: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        match context.language.as_str() {
            "rust" => {
                symbols.extend(Self::collect_rust_scope_symbols(code, context));
            }
            "typescript" | "javascript" => {
                symbols.extend(Self::collect_typescript_scope_symbols(code, context));
            }
            "python" => {
                symbols.extend(Self::collect_python_scope_symbols(code, context));
            }
            _ => {}
        }

        symbols
    }

    /// Collect Rust scope symbols (variables, functions, types)
    fn collect_rust_scope_symbols(code: &str, context: &CompletionContext) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let byte_offset = Self::position_to_byte_offset(code, context.position);
        let code_before = &code[..byte_offset];

        // Look for variable declarations: let, const, mut
        for line in code_before.lines().rev() {
            let trimmed = line.trim();

            // Variable declaration: let x = ...
            if trimmed.starts_with("let ") || trimmed.starts_with("let mut ") {
                if let Some(name) = Self::extract_rust_variable_name(trimmed) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Variable,
                        scope: context.scope.clone(),
                        type_info: None,
                        documentation: None,
                    });
                }
            }

            // Const declaration: const X = ...
            if trimmed.starts_with("const ") {
                if let Some(name) = Self::extract_rust_variable_name(trimmed) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Constant,
                        scope: context.scope.clone(),
                        type_info: None,
                        documentation: None,
                    });
                }
            }

            // Function parameter (if in function scope)
            if context.scope.kind == ScopeKind::Function && trimmed.starts_with("fn ") {
                if let Some(params) = Self::extract_rust_function_params(trimmed) {
                    for param in params {
                        symbols.push(Symbol {
                            name: param.clone(),
                            kind: SymbolKind::Parameter,
                            scope: context.scope.clone(),
                            type_info: None,
                            documentation: None,
                        });
                    }
                }
                break; // Stop at function definition
            }
        }

        symbols
    }

    /// Extract Rust variable name from declaration
    fn extract_rust_variable_name(line: &str) -> Option<String> {
        let line = if let Some(stripped) = line.strip_prefix("let mut ") {
            stripped
        } else if let Some(stripped) = line.strip_prefix("let ") {
            stripped
        } else if let Some(stripped) = line.strip_prefix("const ") {
            stripped
        } else {
            return None;
        };

        // Get the name before : or =
        let name = if let Some(colon_pos) = line.find(':') {
            &line[..colon_pos]
        } else if let Some(eq_pos) = line.find('=') {
            &line[..eq_pos]
        } else {
            line
        };

        let name = name.trim();
        if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            Some(name.to_string())
        } else {
            None
        }
    }

    /// Extract Rust function parameters
    fn extract_rust_function_params(line: &str) -> Option<Vec<String>> {
        if !line.starts_with("fn ") {
            return None;
        }

        // Find the parameter list
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let params_str = &line[start + 1..end];
                let mut params = Vec::new();

                for param in params_str.split(',') {
                    let param = param.trim();
                    if !param.is_empty() {
                        // Extract name before :
                        if let Some(colon_pos) = param.find(':') {
                            let name = param[..colon_pos].trim();
                            if !name.is_empty() {
                                params.push(name.to_string());
                            }
                        }
                    }
                }

                return Some(params);
            }
        }

        None
    }

    /// Collect TypeScript scope symbols
    fn collect_typescript_scope_symbols(code: &str, context: &CompletionContext) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let byte_offset = Self::position_to_byte_offset(code, context.position);
        let code_before = &code[..byte_offset];

        // Look for variable declarations: let, const, var
        for line in code_before.lines().rev() {
            let trimmed = line.trim();

            // Variable declaration: let x = ...
            if trimmed.starts_with("let ") || trimmed.starts_with("const ") || trimmed.starts_with("var ") {
                if let Some(name) = Self::extract_typescript_variable_name(trimmed) {
                    let kind = if trimmed.starts_with("const ") {
                        SymbolKind::Constant
                    } else {
                        SymbolKind::Variable
                    };

                    symbols.push(Symbol {
                        name: name.clone(),
                        kind,
                        scope: context.scope.clone(),
                        type_info: None,
                        documentation: None,
                    });
                }
            }

            // Function parameter (if in function scope)
            if context.scope.kind == ScopeKind::Function && (trimmed.starts_with("function ") || trimmed.contains("=>")) {
                if let Some(params) = Self::extract_typescript_function_params(trimmed) {
                    for param in params {
                        symbols.push(Symbol {
                            name: param.clone(),
                            kind: SymbolKind::Parameter,
                            scope: context.scope.clone(),
                            type_info: None,
                            documentation: None,
                        });
                    }
                }
                break; // Stop at function definition
            }
        }

        symbols
    }

    /// Extract TypeScript variable name from declaration
    fn extract_typescript_variable_name(line: &str) -> Option<String> {
        let line = if let Some(stripped) = line.strip_prefix("let ") {
            stripped
        } else if let Some(stripped) = line.strip_prefix("const ") {
            stripped
        } else if let Some(stripped) = line.strip_prefix("var ") {
            stripped
        } else {
            return None;
        };

        // Get the name before : or =
        let name = if let Some(colon_pos) = line.find(':') {
            &line[..colon_pos]
        } else if let Some(eq_pos) = line.find('=') {
            &line[..eq_pos]
        } else {
            line
        };

        let name = name.trim();
        if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
            Some(name.to_string())
        } else {
            None
        }
    }

    /// Extract TypeScript function parameters
    fn extract_typescript_function_params(line: &str) -> Option<Vec<String>> {
        // Find the parameter list
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let params_str = &line[start + 1..end];
                let mut params = Vec::new();

                for param in params_str.split(',') {
                    let param = param.trim();
                    if !param.is_empty() {
                        // Extract name before : or =
                        let name = if let Some(colon_pos) = param.find(':') {
                            &param[..colon_pos]
                        } else if let Some(eq_pos) = param.find('=') {
                            &param[..eq_pos]
                        } else {
                            param
                        };

                        let name = name.trim();
                        if !name.is_empty() {
                            params.push(name.to_string());
                        }
                    }
                }

                return Some(params);
            }
        }

        None
    }

    /// Collect Python scope symbols
    fn collect_python_scope_symbols(code: &str, context: &CompletionContext) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let byte_offset = Self::position_to_byte_offset(code, context.position);
        let code_before = &code[..byte_offset];

        // Look for variable assignments
        for line in code_before.lines().rev() {
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Variable assignment: x = ...
            if let Some(eq_pos) = trimmed.find('=') {
                // Make sure it's not ==, !=, <=, >=
                if eq_pos > 0 && eq_pos < trimmed.len() - 1 {
                    let before = &trimmed[..eq_pos];
                    let after = &trimmed[eq_pos + 1..eq_pos + 2];

                    if after != "=" && !before.ends_with('!') && !before.ends_with('<') && !before.ends_with('>') {
                        if let Some(name) = Self::extract_python_variable_name(before) {
                            symbols.push(Symbol {
                                name: name.clone(),
                                kind: SymbolKind::Variable,
                                scope: context.scope.clone(),
                                type_info: None,
                                documentation: None,
                            });
                        }
                    }
                }
            }

            // Function parameter (if in function scope)
            if context.scope.kind == ScopeKind::Function && trimmed.starts_with("def ") {
                if let Some(params) = Self::extract_python_function_params(trimmed) {
                    for param in params {
                        symbols.push(Symbol {
                            name: param.clone(),
                            kind: SymbolKind::Parameter,
                            scope: context.scope.clone(),
                            type_info: None,
                            documentation: None,
                        });
                    }
                }
                break; // Stop at function definition
            }
        }

        symbols
    }

    /// Extract Python variable name from assignment
    fn extract_python_variable_name(line: &str) -> Option<String> {
        let name = line.trim();
        if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            Some(name.to_string())
        } else {
            None
        }
    }

    /// Extract Python function parameters
    fn extract_python_function_params(line: &str) -> Option<Vec<String>> {
        if !line.starts_with("def ") {
            return None;
        }

        // Find the parameter list
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let params_str = &line[start + 1..end];
                let mut params = Vec::new();

                for param in params_str.split(',') {
                    let param = param.trim();
                    if !param.is_empty() && param != "self" {
                        // Extract name before : or =
                        let name = if let Some(colon_pos) = param.find(':') {
                            &param[..colon_pos]
                        } else if let Some(eq_pos) = param.find('=') {
                            &param[..eq_pos]
                        } else {
                            param
                        };

                        let name = name.trim();
                        if !name.is_empty() {
                            params.push(name.to_string());
                        }
                    }
                }

                return Some(params);
            }
        }

        None
    }





    /// Parse code using tree-sitter
    fn parse_code(code: &str, language: &str) -> CompletionResult<tree_sitter::Tree> {
        let lang = Self::get_language(language).ok_or_else(|| {
            CompletionError::UnsupportedLanguage(format!(
                "Language {} not supported by tree-sitter",
                language
            ))
        })?;

        let mut parser = Parser::new();
        parser.set_language(lang).map_err(|_| {
            CompletionError::ContextAnalysisError("Failed to set parser language".to_string())
        })?;

        parser.parse(code, None).ok_or_else(|| {
            CompletionError::ContextAnalysisError("Failed to parse code".to_string())
        })
    }

    /// Detect the scope at the given position
    fn detect_scope(
        code: &str,
        position: Position,
        tree: &tree_sitter::Tree,
        language: &str,
    ) -> Scope {
        let byte_offset = Self::position_to_byte_offset(code, position);
        let root = tree.root_node();

        // Find the deepest node at the position
        let mut current_node = root;
        let mut cursor = root.walk();

        // Walk down the tree to find the deepest node at the position
        loop {
            let mut found_child = false;
            for child in current_node.children(&mut cursor) {
                if child.start_byte() <= byte_offset && byte_offset <= child.end_byte() {
                    current_node = child;
                    found_child = true;
                    break;
                }
            }
            if !found_child {
                break;
            }
        }

        // Walk up the tree to find scope-defining nodes
        let mut scope_kind = ScopeKind::Global;
        let mut scope_name = None;

        let mut node = current_node;
        loop {
            let node_type = node.kind();

            match language {
                "rust" => {
                    match node_type {
                        "function_item" => {
                            scope_kind = ScopeKind::Function;
                            scope_name = Self::extract_name_rust(node, code);
                            break;
                        }
                        "struct_item" => {
                            scope_kind = ScopeKind::Struct;
                            scope_name = Self::extract_name_rust(node, code);
                            break;
                        }
                        "impl_item" => {
                            scope_kind = ScopeKind::Impl;
                            scope_name = Self::extract_name_rust(node, code);
                            break;
                        }
                        "mod_item" => {
                            scope_kind = ScopeKind::Module;
                            scope_name = Self::extract_name_rust(node, code);
                            break;
                        }
                        "block" => {
                            scope_kind = ScopeKind::Block;
                        }
                        _ => {}
                    }
                }
                "typescript" | "javascript" => {
                    match node_type {
                        "function_declaration" | "function" => {
                            scope_kind = ScopeKind::Function;
                            scope_name = Self::extract_name_typescript(node, code);
                            break;
                        }
                        "class_declaration" => {
                            scope_kind = ScopeKind::Class;
                            scope_name = Self::extract_name_typescript(node, code);
                            break;
                        }
                        "method_definition" => {
                            scope_kind = ScopeKind::Function;
                            scope_name = Self::extract_name_typescript(node, code);
                            break;
                        }
                        "block" => {
                            scope_kind = ScopeKind::Block;
                        }
                        _ => {}
                    }
                }
                "python" => {
                    match node_type {
                        "function_definition" => {
                            scope_kind = ScopeKind::Function;
                            scope_name = Self::extract_name_python(node, code);
                            break;
                        }
                        "class_definition" => {
                            scope_kind = ScopeKind::Class;
                            scope_name = Self::extract_name_python(node, code);
                            break;
                        }
                        "block" => {
                            scope_kind = ScopeKind::Block;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            if let Some(parent) = node.parent() {
                node = parent;
            } else {
                break;
            }
        }

        Scope {
            kind: scope_kind,
            name: scope_name,
            range: Range::new(
                Self::byte_offset_to_position(code, current_node.start_byte()),
                Self::byte_offset_to_position(code, current_node.end_byte()),
            ),
        }
    }

    /// Extract the name from a Rust node
    fn extract_name_rust(node: tree_sitter::Node, code: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                if let Ok(text) = child.utf8_text(code.as_bytes()) {
                    return Some(text.to_string());
                }
            }
        }
        None
    }

    /// Extract the name from a TypeScript node
    fn extract_name_typescript(node: tree_sitter::Node, code: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                if let Ok(text) = child.utf8_text(code.as_bytes()) {
                    return Some(text.to_string());
                }
            }
        }
        None
    }

    /// Extract the name from a Python node
    fn extract_name_python(node: tree_sitter::Node, code: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                if let Ok(text) = child.utf8_text(code.as_bytes()) {
                    return Some(text.to_string());
                }
            }
        }
        None
    }

    /// Convert a Position to a byte offset in the code
    fn position_to_byte_offset(code: &str, position: Position) -> usize {
        let mut byte_offset = 0;
        let mut current_line = 0;
        let mut current_char = 0;

        for ch in code.chars() {
            if current_line == position.line && current_char == position.character {
                return byte_offset;
            }

            byte_offset += ch.len_utf8();

            if ch == '\n' {
                current_line += 1;
                current_char = 0;
            } else {
                current_char += 1;
            }
        }

        byte_offset
    }

    /// Convert a byte offset to a Position in the code
    fn byte_offset_to_position(code: &str, byte_offset: usize) -> Position {
        let mut current_line = 0;
        let mut current_char = 0;
        let mut current_byte = 0;

        for ch in code.chars() {
            if current_byte >= byte_offset {
                return Position::new(current_line, current_char);
            }

            current_byte += ch.len_utf8();

            if ch == '\n' {
                current_line += 1;
                current_char = 0;
            } else {
                current_char += 1;
            }
        }

        Position::new(current_line, current_char)
    }

    /// Extract the prefix (partial word) at the cursor position
    fn extract_prefix(code: &str, position: Position) -> String {
        let byte_offset = Self::position_to_byte_offset(code, position);
        let mut prefix = String::new();

        // Walk backwards from the position to find the start of the word
        for ch in code[..byte_offset].chars().rev() {
            if ch.is_alphanumeric() || ch == '_' {
                prefix.insert(0, ch);
            } else {
                break;
            }
        }

        prefix
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
        // Parse the code
        let tree = Self::parse_code(code, language)?;

        // Detect the scope at the position
        let scope = Self::detect_scope(code, position, &tree, language);

        // Extract the prefix
        let prefix = Self::extract_prefix(code, position);

        // Create the context
        let mut context = CompletionContext::new(language.to_string(), position, prefix);
        context.scope = scope;

        // Collect available symbols
        context.available_symbols = self.get_available_symbols(&context, code);

        // Infer expected type at the cursor position
        context.expected_type = self.infer_expected_type(&context);

        Ok(context)
    }

    fn get_available_symbols(&self, context: &CompletionContext, code: &str) -> Vec<Symbol> {
        // Collect symbols from the current scope
        let mut symbols = Vec::new();

        // Add built-in symbols for the language
        symbols.extend(Self::get_builtin_symbols(&context.language));

        // Add symbols from imported modules
        symbols.extend(Self::collect_imported_symbols(code, &context.language));

        // Add symbols from the current scope
        symbols.extend(Self::collect_scope_symbols_with_code(context, code));

        symbols
    }

    fn infer_expected_type(&self, context: &CompletionContext) -> Option<Type> {
        Self::infer_type_from_context(context)
    }
}

impl TreeSitterContextAnalyzer {
    /// Infer the expected type at the given context
    fn infer_type_from_context(context: &CompletionContext) -> Option<Type> {
        // Try to infer type from assignment context
        if let Some(type_info) = Self::infer_from_assignment(context) {
            return Some(type_info);
        }

        // Try to infer type from function parameter context
        if let Some(type_info) = Self::infer_from_function_param(context) {
            return Some(type_info);
        }

        // Try to infer type from return type context
        if let Some(type_info) = Self::infer_from_return_type(context) {
            return Some(type_info);
        }

        None
    }

    /// Infer type from assignment context (e.g., "let x: Type = ")
    fn infer_from_assignment(context: &CompletionContext) -> Option<Type> {
        match context.language.as_str() {
            "rust" => Self::infer_rust_assignment_type(context),
            "typescript" | "javascript" => Self::infer_typescript_assignment_type(context),
            "python" => Self::infer_python_assignment_type(context),
            _ => None,
        }
    }

    /// Infer Rust assignment type from type annotations
    fn infer_rust_assignment_type(context: &CompletionContext) -> Option<Type> {
        // Look for patterns like "let x: Type = " or "let x: Type<T> = "
        // We need to find the line with the assignment and extract the type annotation
        
        // For now, we'll use a simple pattern matching approach
        // In a full implementation, this would use the AST
        
        // Check if we're in an assignment context by looking at the scope
        if context.scope.kind == ScopeKind::Function || context.scope.kind == ScopeKind::Block {
            // Try to infer from variable declarations with type annotations
            // This is a simplified implementation
            
            // Common Rust types that might be inferred
            if context.prefix.contains("vec") || context.prefix.contains("Vec") {
                return Some(Type::new("Vec".to_string()).array());
            }
            if context.prefix.contains("string") || context.prefix.contains("String") {
                return Some(Type::new("String".to_string()));
            }
            if context.prefix.contains("option") || context.prefix.contains("Option") {
                return Some(Type::new("Option".to_string()).optional());
            }
            if context.prefix.contains("result") || context.prefix.contains("Result") {
                return Some(Type::new("Result".to_string()));
            }
        }
        
        None
    }

    /// Infer TypeScript assignment type from type annotations
    fn infer_typescript_assignment_type(context: &CompletionContext) -> Option<Type> {
        // Look for patterns like "let x: Type = " or "const x: Type = "
        
        // Common TypeScript types that might be inferred
        if context.prefix.contains("array") || context.prefix.contains("Array") {
            return Some(Type::new("Array".to_string()).array());
        }
        if context.prefix.contains("string") || context.prefix.contains("String") {
            return Some(Type::new("string".to_string()));
        }
        if context.prefix.contains("number") || context.prefix.contains("Number") {
            return Some(Type::new("number".to_string()));
        }
        if context.prefix.contains("promise") || context.prefix.contains("Promise") {
            return Some(Type::new("Promise".to_string()));
        }
        if context.prefix.contains("map") || context.prefix.contains("Map") {
            return Some(Type::new("Map".to_string()));
        }
        
        None
    }

    /// Infer Python assignment type from type hints
    fn infer_python_assignment_type(context: &CompletionContext) -> Option<Type> {
        // Look for patterns like "x: Type = " or "x: Type[T] = "
        
        // Common Python types that might be inferred
        if context.prefix.contains("list") || context.prefix.contains("List") {
            return Some(Type::new("list".to_string()).array());
        }
        if context.prefix.contains("dict") || context.prefix.contains("Dict") {
            return Some(Type::new("dict".to_string()));
        }
        if context.prefix.contains("str") || context.prefix.contains("String") {
            return Some(Type::new("str".to_string()));
        }
        if context.prefix.contains("int") || context.prefix.contains("Integer") {
            return Some(Type::new("int".to_string()));
        }
        if context.prefix.contains("tuple") || context.prefix.contains("Tuple") {
            return Some(Type::new("tuple".to_string()).array());
        }
        
        None
    }

    /// Infer type from function parameter context
    fn infer_from_function_param(context: &CompletionContext) -> Option<Type> {
        match context.language.as_str() {
            "rust" => Self::infer_rust_function_param_type(context),
            "typescript" | "javascript" => Self::infer_typescript_function_param_type(context),
            "python" => Self::infer_python_function_param_type(context),
            _ => None,
        }
    }

    /// Infer Rust function parameter type
    fn infer_rust_function_param_type(context: &CompletionContext) -> Option<Type> {
        // If we're in a function scope, try to infer parameter types
        if context.scope.kind == ScopeKind::Function {
            // Look for common parameter patterns
            if context.prefix.contains("&str") {
                return Some(Type::new("&str".to_string()));
            }
            if context.prefix.contains("&mut") {
                return Some(Type::new("&mut".to_string()));
            }
            if context.prefix.contains("&") {
                return Some(Type::new("&".to_string()));
            }
        }
        
        None
    }

    /// Infer TypeScript function parameter type
    fn infer_typescript_function_param_type(context: &CompletionContext) -> Option<Type> {
        // If we're in a function scope, try to infer parameter types
        if context.scope.kind == ScopeKind::Function {
            // Look for common parameter patterns
            if context.prefix.contains("string") {
                return Some(Type::new("string".to_string()));
            }
            if context.prefix.contains("number") {
                return Some(Type::new("number".to_string()));
            }
            if context.prefix.contains("boolean") {
                return Some(Type::new("boolean".to_string()));
            }
            if context.prefix.contains("any") {
                return Some(Type::new("any".to_string()));
            }
        }
        
        None
    }

    /// Infer Python function parameter type
    fn infer_python_function_param_type(context: &CompletionContext) -> Option<Type> {
        // If we're in a function scope, try to infer parameter types
        if context.scope.kind == ScopeKind::Function {
            // Look for common parameter patterns
            if context.prefix.contains("str") {
                return Some(Type::new("str".to_string()));
            }
            if context.prefix.contains("int") {
                return Some(Type::new("int".to_string()));
            }
            if context.prefix.contains("float") {
                return Some(Type::new("float".to_string()));
            }
            if context.prefix.contains("bool") {
                return Some(Type::new("bool".to_string()));
            }
        }
        
        None
    }

    /// Infer type from return type context
    fn infer_from_return_type(context: &CompletionContext) -> Option<Type> {
        match context.language.as_str() {
            "rust" => Self::infer_rust_return_type(context),
            "typescript" | "javascript" => Self::infer_typescript_return_type(context),
            "python" => Self::infer_python_return_type(context),
            _ => None,
        }
    }

    /// Infer Rust return type
    fn infer_rust_return_type(context: &CompletionContext) -> Option<Type> {
        // If we're in a function scope, try to infer return type
        if context.scope.kind == ScopeKind::Function {
            // Look for common return type patterns
            if context.prefix.contains("Result") {
                return Some(Type::new("Result".to_string()));
            }
            if context.prefix.contains("Option") {
                return Some(Type::new("Option".to_string()).optional());
            }
            if context.prefix.contains("Vec") {
                return Some(Type::new("Vec".to_string()).array());
            }
            if context.prefix.contains("String") {
                return Some(Type::new("String".to_string()));
            }
        }
        
        None
    }

    /// Infer TypeScript return type
    fn infer_typescript_return_type(context: &CompletionContext) -> Option<Type> {
        // If we're in a function scope, try to infer return type
        if context.scope.kind == ScopeKind::Function {
            // Look for common return type patterns
            if context.prefix.contains("Promise") {
                return Some(Type::new("Promise".to_string()));
            }
            if context.prefix.contains("Array") {
                return Some(Type::new("Array".to_string()).array());
            }
            if context.prefix.contains("string") {
                return Some(Type::new("string".to_string()));
            }
            if context.prefix.contains("number") {
                return Some(Type::new("number".to_string()));
            }
        }
        
        None
    }

    /// Infer Python return type
    fn infer_python_return_type(context: &CompletionContext) -> Option<Type> {
        // If we're in a function scope, try to infer return type
        if context.scope.kind == ScopeKind::Function {
            // Look for common return type patterns
            if context.prefix.contains("list") {
                return Some(Type::new("list".to_string()).array());
            }
            if context.prefix.contains("dict") {
                return Some(Type::new("dict".to_string()));
            }
            if context.prefix.contains("str") {
                return Some(Type::new("str".to_string()));
            }
            if context.prefix.contains("int") {
                return Some(Type::new("int".to_string()));
            }
        }
        
        None
    }

    /// Infer variable type from assignments and declarations
    pub fn infer_variable_type(code: &str, position: Position, language: &str) -> Option<Type> {
        let byte_offset = Self::position_to_byte_offset(code, position);
        let code_before = &code[..byte_offset];

        match language {
            "rust" => Self::infer_rust_variable_type(code_before),
            "typescript" | "javascript" => Self::infer_typescript_variable_type(code_before),
            "python" => Self::infer_python_variable_type(code_before),
            _ => None,
        }
    }

    /// Infer Rust variable type from assignments
    fn infer_rust_variable_type(code_before: &str) -> Option<Type> {
        // Look for the most recent variable declaration with type annotation
        for line in code_before.lines().rev() {
            let trimmed = line.trim();
            
            // Pattern: let x: Type = ...
            if (trimmed.starts_with("let ") || trimmed.starts_with("let mut ")) && trimmed.contains(':') {
                if let Some(type_str) = Self::extract_rust_type_annotation(trimmed) {
                    return Some(Self::parse_rust_type(&type_str));
                }
            }
            
            // Pattern: const X: Type = ...
            if trimmed.starts_with("const ") && trimmed.contains(':') {
                if let Some(type_str) = Self::extract_rust_type_annotation(trimmed) {
                    return Some(Self::parse_rust_type(&type_str));
                }
            }
        }
        
        None
    }

    /// Extract Rust type annotation from a line
    fn extract_rust_type_annotation(line: &str) -> Option<String> {
        if let Some(colon_pos) = line.find(':') {
            if let Some(eq_pos) = line.find('=') {
                if colon_pos < eq_pos {
                    let type_str = &line[colon_pos + 1..eq_pos];
                    return Some(type_str.trim().to_string());
                }
            } else {
                let type_str = &line[colon_pos + 1..];
                return Some(type_str.trim().to_string());
            }
        }
        None
    }

    /// Parse Rust type string into Type struct
    fn parse_rust_type(type_str: &str) -> Type {
        let type_str = type_str.trim();
        
        // Handle Option<T>
        if type_str.starts_with("Option<") {
            let inner = &type_str[7..type_str.len() - 1];
            return Type::new(inner.to_string()).optional();
        }
        
        // Handle Vec<T>
        if type_str.starts_with("Vec<") {
            let inner = &type_str[4..type_str.len() - 1];
            return Type::new(inner.to_string()).array();
        }
        
        // Handle Result<T, E>
        if type_str.starts_with("Result<") {
            let inner = &type_str[7..type_str.len() - 1];
            return Type::new(inner.to_string());
        }
        
        // Handle references
        if let Some(inner) = type_str.strip_prefix("&") {
            return Type::new(inner.to_string());
        }
        
        // Simple type
        Type::new(type_str.to_string())
    }

    /// Infer TypeScript variable type from assignments
    fn infer_typescript_variable_type(code_before: &str) -> Option<Type> {
        // Look for the most recent variable declaration with type annotation
        for line in code_before.lines().rev() {
            let trimmed = line.trim();
            
            // Pattern: let x: Type = ...
            if (trimmed.starts_with("let ") || trimmed.starts_with("const ") || trimmed.starts_with("var ")) && trimmed.contains(':') {
                if let Some(type_str) = Self::extract_typescript_type_annotation(trimmed) {
                    return Some(Self::parse_typescript_type(&type_str));
                }
            }
        }
        
        None
    }

    /// Extract TypeScript type annotation from a line
    fn extract_typescript_type_annotation(line: &str) -> Option<String> {
        if let Some(colon_pos) = line.find(':') {
            if let Some(eq_pos) = line.find('=') {
                if colon_pos < eq_pos {
                    let type_str = &line[colon_pos + 1..eq_pos];
                    return Some(type_str.trim().to_string());
                }
            } else {
                let type_str = &line[colon_pos + 1..];
                return Some(type_str.trim().to_string());
            }
        }
        None
    }

    /// Parse TypeScript type string into Type struct
    fn parse_typescript_type(type_str: &str) -> Type {
        let type_str = type_str.trim();
        
        // Handle Array<T>
        if type_str.starts_with("Array<") {
            let inner = &type_str[6..type_str.len() - 1];
            return Type::new(inner.to_string()).array();
        }
        
        // Handle T[]
        if let Some(inner) = type_str.strip_suffix("[]") {
            return Type::new(inner.to_string()).array();
        }
        
        // Handle Promise<T>
        if type_str.starts_with("Promise<") {
            let inner = &type_str[8..type_str.len() - 1];
            return Type::new(inner.to_string());
        }
        
        // Simple type
        Type::new(type_str.to_string())
    }

    /// Infer Python variable type from assignments
    fn infer_python_variable_type(code_before: &str) -> Option<Type> {
        // Look for the most recent variable declaration with type hint
        for line in code_before.lines().rev() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            // Pattern: x: Type = ...
            if trimmed.contains(':') && trimmed.contains('=') {
                if let Some(type_str) = Self::extract_python_type_annotation(trimmed) {
                    return Some(Self::parse_python_type(&type_str));
                }
            }
        }
        
        None
    }

    /// Extract Python type annotation from a line
    fn extract_python_type_annotation(line: &str) -> Option<String> {
        if let Some(colon_pos) = line.find(':') {
            if let Some(eq_pos) = line.find('=') {
                if colon_pos < eq_pos {
                    let type_str = &line[colon_pos + 1..eq_pos];
                    return Some(type_str.trim().to_string());
                }
            }
        }
        None
    }

    /// Parse Python type string into Type struct
    fn parse_python_type(type_str: &str) -> Type {
        let type_str = type_str.trim();
        
        // Handle List[T]
        if type_str.starts_with("List[") {
            let inner = &type_str[5..type_str.len() - 1];
            return Type::new(inner.to_string()).array();
        }
        
        // Handle list
        if type_str == "list" {
            return Type::new("list".to_string()).array();
        }
        
        // Handle Dict[K, V]
        if type_str.starts_with("Dict[") {
            let inner = &type_str[5..type_str.len() - 1];
            return Type::new(inner.to_string());
        }
        
        // Handle Optional[T]
        if type_str.starts_with("Optional[") {
            let inner = &type_str[9..type_str.len() - 1];
            return Type::new(inner.to_string()).optional();
        }
        
        // Handle Tuple[T, ...]
        if type_str.starts_with("Tuple[") {
            let inner = &type_str[6..type_str.len() - 1];
            return Type::new(inner.to_string()).array();
        }
        
        // Simple type
        Type::new(type_str.to_string())
    }

    /// Infer function return type from signature
    pub fn infer_function_return_type(code: &str, position: Position, language: &str) -> Option<Type> {
        let byte_offset = Self::position_to_byte_offset(code, position);
        let code_before = &code[..byte_offset];

        match language {
            "rust" => Self::infer_rust_function_return_type(code_before),
            "typescript" | "javascript" => Self::infer_typescript_function_return_type(code_before),
            "python" => Self::infer_python_function_return_type(code_before),
            _ => None,
        }
    }

    /// Infer Rust function return type from signature
    fn infer_rust_function_return_type(code_before: &str) -> Option<Type> {
        // Look for the most recent function definition
        for line in code_before.lines().rev() {
            let trimmed = line.trim();
            
            // Pattern: fn name(...) -> Type { ... }
            if trimmed.starts_with("fn ") && trimmed.contains("->") {
                if let Some(return_type) = Self::extract_rust_return_type(trimmed) {
                    return Some(Self::parse_rust_type(&return_type));
                }
            }
        }
        
        None
    }

    /// Extract Rust return type from function signature
    fn extract_rust_return_type(line: &str) -> Option<String> {
        if let Some(arrow_pos) = line.find("->") {
            if let Some(brace_pos) = line.find('{') {
                let return_type = &line[arrow_pos + 2..brace_pos];
                return Some(return_type.trim().to_string());
            } else {
                let return_type = &line[arrow_pos + 2..];
                return Some(return_type.trim().to_string());
            }
        }
        None
    }

    /// Infer TypeScript function return type from signature
    fn infer_typescript_function_return_type(code_before: &str) -> Option<Type> {
        // Look for the most recent function definition
        for line in code_before.lines().rev() {
            let trimmed = line.trim();
            
            // Pattern: function name(...): Type { ... } or async function name(...): Type { ... } or (...): Type => ...
            if (trimmed.starts_with("function ") || trimmed.starts_with("async function ") || trimmed.contains("=>")) && trimmed.contains(':') {
                if let Some(return_type) = Self::extract_typescript_return_type(trimmed) {
                    return Some(Self::parse_typescript_type(&return_type));
                }
            }
        }
        
        None
    }

    /// Extract TypeScript return type from function signature
    fn extract_typescript_return_type(line: &str) -> Option<String> {
        // Look for pattern: ): Type { or ): Type =>
        if let Some(paren_pos) = line.rfind(')') {
            let after_paren = &line[paren_pos..];
            
            // Look for : Type pattern
            if let Some(colon_pos) = after_paren.find(':') {
                let after_colon = &after_paren[colon_pos + 1..];
                
                // Find the end of the type (either { or =>)
                let end_pos = if let Some(brace_pos) = after_colon.find('{') {
                    brace_pos
                } else if let Some(arrow_pos) = after_colon.find("=>") {
                    arrow_pos
                } else {
                    // No brace or arrow found, take until end of line
                    after_colon.len()
                };
                
                let return_type = &after_colon[..end_pos];
                return Some(return_type.trim().to_string());
            }
        }
        None
    }

    /// Infer Python function return type from signature
    fn infer_python_function_return_type(code_before: &str) -> Option<Type> {
        // Look for the most recent function definition
        for line in code_before.lines().rev() {
            let trimmed = line.trim();
            
            // Pattern: def name(...) -> Type: ...
            if trimmed.starts_with("def ") && trimmed.contains("->") {
                if let Some(return_type) = Self::extract_python_return_type(trimmed) {
                    return Some(Self::parse_python_type(&return_type));
                }
            }
        }
        
        None
    }

    /// Extract Python return type from function signature
    fn extract_python_return_type(line: &str) -> Option<String> {
        if let Some(arrow_pos) = line.find("->") {
            if let Some(colon_pos) = line.find(':') {
                if arrow_pos < colon_pos {
                    let return_type = &line[arrow_pos + 2..colon_pos];
                    return Some(return_type.trim().to_string());
                }
            }
        }
        None
    }
}

/// Generic context analyzer (fallback for unsupported languages)
pub struct GenericContextAnalyzer;

impl GenericContextAnalyzer {
    /// Collect symbols from imported modules using text patterns
    fn collect_imported_symbols_generic(code: &str, language: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        match language {
            "rust" => {
                // Look for use statements
                for line in code.lines() {
                    if line.trim().starts_with("use ") {
                        if let Some(imported) = TreeSitterContextAnalyzer::extract_rust_import(line) {
                            symbols.push(Symbol {
                                name: imported.clone(),
                                kind: SymbolKind::Module,
                                scope: Scope {
                                    kind: ScopeKind::Global,
                                    name: None,
                                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                                },
                                type_info: None,
                                documentation: None,
                            });
                        }
                    }
                }
            }
            "typescript" | "javascript" => {
                // Look for import statements
                for line in code.lines() {
                    if line.trim().starts_with("import ") {
                        if let Some(imported) = TreeSitterContextAnalyzer::extract_typescript_import(line) {
                            symbols.push(Symbol {
                                name: imported.clone(),
                                kind: SymbolKind::Module,
                                scope: Scope {
                                    kind: ScopeKind::Global,
                                    name: None,
                                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                                },
                                type_info: None,
                                documentation: None,
                            });
                        }
                    }
                }
            }
            "python" => {
                // Look for import statements
                for line in code.lines() {
                    if line.trim().starts_with("import ") || line.trim().starts_with("from ") {
                        if let Some(imported) = TreeSitterContextAnalyzer::extract_python_import(line) {
                            symbols.push(Symbol {
                                name: imported.clone(),
                                kind: SymbolKind::Module,
                                scope: Scope {
                                    kind: ScopeKind::Global,
                                    name: None,
                                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                                },
                                type_info: None,
                                documentation: None,
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        symbols
    }

    /// Collect symbols from the current scope using text patterns
    fn collect_scope_symbols_generic(context: &CompletionContext, code: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        match context.language.as_str() {
            "rust" => {
                symbols.extend(TreeSitterContextAnalyzer::collect_rust_scope_symbols(code, context));
            }
            "typescript" | "javascript" => {
                symbols.extend(TreeSitterContextAnalyzer::collect_typescript_scope_symbols(code, context));
            }
            "python" => {
                symbols.extend(TreeSitterContextAnalyzer::collect_python_scope_symbols(code, context));
            }
            _ => {}
        }

        symbols
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
        // Extract the prefix using simple text-based approach
        let prefix = Self::extract_prefix_generic(code, position);

        // Create the context
        let mut context = CompletionContext::new(language.to_string(), position, prefix);

        // Detect scope using simple text patterns
        context.scope = Self::detect_scope_generic(code, position);

        // Collect available symbols
        context.available_symbols = self.get_available_symbols(&context, code);

        // Infer expected type at the cursor position
        context.expected_type = self.infer_expected_type(&context);

        Ok(context)
    }

    fn get_available_symbols(&self, context: &CompletionContext, code: &str) -> Vec<Symbol> {
        // Collect symbols from the current scope
        let mut symbols = Vec::new();

        // Add built-in symbols for the language
        symbols.extend(Self::get_builtin_symbols(&context.language));

        // Add symbols from imported modules
        symbols.extend(Self::collect_imported_symbols_generic(code, &context.language));

        // Add symbols from the current scope
        symbols.extend(Self::collect_scope_symbols_generic(context, code));

        symbols
    }

    fn infer_expected_type(&self, context: &CompletionContext) -> Option<Type> {
        Self::infer_type_from_context_generic(context)
    }
}

impl GenericContextAnalyzer {
    /// Infer the expected type at the given context using text patterns
    fn infer_type_from_context_generic(context: &CompletionContext) -> Option<Type> {
        // Try to infer type from assignment context
        if let Some(type_info) = Self::infer_from_assignment_generic(context) {
            return Some(type_info);
        }

        // Try to infer type from function parameter context
        if let Some(type_info) = Self::infer_from_function_param_generic(context) {
            return Some(type_info);
        }

        // Try to infer type from return type context
        if let Some(type_info) = Self::infer_from_return_type_generic(context) {
            return Some(type_info);
        }

        None
    }

    /// Infer type from assignment context using text patterns
    fn infer_from_assignment_generic(context: &CompletionContext) -> Option<Type> {
        match context.language.as_str() {
            "rust" => TreeSitterContextAnalyzer::infer_rust_assignment_type(context),
            "typescript" | "javascript" => TreeSitterContextAnalyzer::infer_typescript_assignment_type(context),
            "python" => TreeSitterContextAnalyzer::infer_python_assignment_type(context),
            _ => None,
        }
    }

    /// Infer type from function parameter context using text patterns
    fn infer_from_function_param_generic(context: &CompletionContext) -> Option<Type> {
        match context.language.as_str() {
            "rust" => TreeSitterContextAnalyzer::infer_rust_function_param_type(context),
            "typescript" | "javascript" => TreeSitterContextAnalyzer::infer_typescript_function_param_type(context),
            "python" => TreeSitterContextAnalyzer::infer_python_function_param_type(context),
            _ => None,
        }
    }

    /// Infer type from return type context using text patterns
    fn infer_from_return_type_generic(context: &CompletionContext) -> Option<Type> {
        match context.language.as_str() {
            "rust" => TreeSitterContextAnalyzer::infer_rust_return_type(context),
            "typescript" | "javascript" => TreeSitterContextAnalyzer::infer_typescript_return_type(context),
            "python" => TreeSitterContextAnalyzer::infer_python_return_type(context),
            _ => None,
        }
    }
    /// Extract the prefix using simple text-based approach
    fn extract_prefix_generic(code: &str, position: Position) -> String {
        let byte_offset = TreeSitterContextAnalyzer::position_to_byte_offset(code, position);
        let mut prefix = String::new();

        // Walk backwards from the position to find the start of the word
        for ch in code[..byte_offset].chars().rev() {
            if ch.is_alphanumeric() || ch == '_' {
                prefix.insert(0, ch);
            } else {
                break;
            }
        }

        prefix
    }

    /// Detect scope using simple text patterns
    fn detect_scope_generic(code: &str, position: Position) -> Scope {
        let byte_offset = TreeSitterContextAnalyzer::position_to_byte_offset(code, position);
        let code_before = &code[..byte_offset];

        // Count braces to determine if we're in a block
        let open_braces = code_before.matches('{').count();
        let close_braces = code_before.matches('}').count();
        let in_block = open_braces > close_braces;

        // Try to detect function or class definitions
        let scope_kind = if code_before.contains("fn ") || code_before.contains("function ") {
            ScopeKind::Function
        } else if code_before.contains("class ") || code_before.contains("struct ") {
            ScopeKind::Class
        } else if in_block {
            ScopeKind::Block
        } else {
            ScopeKind::Global
        };

        Scope {
            kind: scope_kind,
            name: None,
            range: Range::new(Position::new(0, 0), position),
        }
    }

    /// Get built-in symbols for a language
    fn get_builtin_symbols(language: &str) -> Vec<Symbol> {
        TreeSitterContextAnalyzer::get_builtin_symbols(language)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_to_byte_offset() {
        let code = "hello\nworld";
        let pos = Position::new(1, 2);
        let offset = TreeSitterContextAnalyzer::position_to_byte_offset(code, pos);
        assert_eq!(offset, 8); // "hello\nwo"
    }

    #[test]
    fn test_byte_offset_to_position() {
        let code = "hello\nworld";
        let pos = TreeSitterContextAnalyzer::byte_offset_to_position(code, 8);
        assert_eq!(pos, Position::new(1, 2));
    }

    #[test]
    fn test_extract_prefix() {
        let code = "let my_var = ";
        let pos = Position::new(0, 13);
        let prefix = TreeSitterContextAnalyzer::extract_prefix(code, pos);
        assert_eq!(prefix, "");
    }

    #[test]
    fn test_extract_prefix_with_word() {
        let code = "let my_var = my";
        let pos = Position::new(0, 15);
        let prefix = TreeSitterContextAnalyzer::extract_prefix(code, pos);
        assert_eq!(prefix, "my");
    }

    #[test]
    fn test_generic_extract_prefix() {
        let code = "let my_var = my";
        let pos = Position::new(0, 15);
        let prefix = GenericContextAnalyzer::extract_prefix_generic(code, pos);
        assert_eq!(prefix, "my");
    }

    #[test]
    fn test_generic_detect_scope_global() {
        let code = "let x = 5;";
        let pos = Position::new(0, 5);
        let scope = GenericContextAnalyzer::detect_scope_generic(code, pos);
        assert_eq!(scope.kind, ScopeKind::Global);
    }

    #[test]
    fn test_generic_detect_scope_function() {
        let code = "fn main() { let x = 5; }";
        let pos = Position::new(0, 15);
        let scope = GenericContextAnalyzer::detect_scope_generic(code, pos);
        assert_eq!(scope.kind, ScopeKind::Function);
    }

    #[test]
    fn test_generic_detect_scope_block() {
        let code = "{ let x = 5; }";
        let pos = Position::new(0, 10);
        let scope = GenericContextAnalyzer::detect_scope_generic(code, pos);
        assert_eq!(scope.kind, ScopeKind::Block);
    }

    // Type inference tests
    #[test]
    fn test_infer_rust_variable_type_simple() {
        let code = "let x: String = ";
        let ty = TreeSitterContextAnalyzer::infer_rust_variable_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "String");
    }

    #[test]
    fn test_infer_rust_variable_type_vec() {
        let code = "let items: Vec<i32> = ";
        let ty = TreeSitterContextAnalyzer::infer_rust_variable_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "i32");
        assert!(ty.is_array);
    }

    #[test]
    fn test_infer_rust_variable_type_option() {
        let code = "let maybe: Option<String> = ";
        let ty = TreeSitterContextAnalyzer::infer_rust_variable_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "String");
        assert!(ty.is_optional);
    }

    #[test]
    fn test_infer_rust_variable_type_reference() {
        let code = "let reference: &str = ";
        let ty = TreeSitterContextAnalyzer::infer_rust_variable_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "str");
    }

    #[test]
    fn test_infer_typescript_variable_type_simple() {
        let code = "let x: string = ";
        let ty = TreeSitterContextAnalyzer::infer_typescript_variable_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "string");
    }

    #[test]
    fn test_infer_typescript_variable_type_array() {
        let code = "let items: number[] = ";
        let ty = TreeSitterContextAnalyzer::infer_typescript_variable_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "number");
        assert!(ty.is_array);
    }

    #[test]
    fn test_infer_typescript_variable_type_array_generic() {
        let code = "let items: Array<string> = ";
        let ty = TreeSitterContextAnalyzer::infer_typescript_variable_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "string");
        assert!(ty.is_array);
    }

    #[test]
    fn test_infer_python_variable_type_simple() {
        let code = "x: str = ";
        let ty = TreeSitterContextAnalyzer::infer_python_variable_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "str");
    }

    #[test]
    fn test_infer_python_variable_type_list() {
        let code = "items: List[int] = ";
        let ty = TreeSitterContextAnalyzer::infer_python_variable_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "int");
        assert!(ty.is_array);
    }

    #[test]
    fn test_infer_python_variable_type_optional() {
        let code = "maybe: Optional[str] = ";
        let ty = TreeSitterContextAnalyzer::infer_python_variable_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "str");
        assert!(ty.is_optional);
    }

    #[test]
    fn test_infer_rust_function_return_type() {
        let code = "fn get_name() -> String { ";
        let ty = TreeSitterContextAnalyzer::infer_rust_function_return_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "String");
    }

    #[test]
    fn test_infer_rust_function_return_type_result() {
        let code = "fn process() -> Result<i32, String> { ";
        let ty = TreeSitterContextAnalyzer::infer_rust_function_return_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "i32, String");
    }

    #[test]
    fn test_infer_typescript_function_return_type() {
        let code = "function getName(): string { ";
        let ty = TreeSitterContextAnalyzer::infer_typescript_function_return_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "string");
    }

    #[test]
    fn test_infer_typescript_function_return_type_promise() {
        let code = "async function fetch(): Promise<string> { ";
        let ty = TreeSitterContextAnalyzer::infer_typescript_function_return_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "string");
    }

    #[test]
    fn test_infer_python_function_return_type() {
        let code = "def get_name() -> str: ";
        let ty = TreeSitterContextAnalyzer::infer_python_function_return_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "str");
    }

    #[test]
    fn test_infer_python_function_return_type_list() {
        let code = "def get_items() -> List[int]: ";
        let ty = TreeSitterContextAnalyzer::infer_python_function_return_type(code);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty.name, "int");
        assert!(ty.is_array);
    }

    #[test]
    fn test_parse_rust_type_simple() {
        let ty = TreeSitterContextAnalyzer::parse_rust_type("String");
        assert_eq!(ty.name, "String");
        assert!(!ty.is_optional);
        assert!(!ty.is_array);
    }

    #[test]
    fn test_parse_rust_type_vec() {
        let ty = TreeSitterContextAnalyzer::parse_rust_type("Vec<i32>");
        assert_eq!(ty.name, "i32");
        assert!(ty.is_array);
    }

    #[test]
    fn test_parse_rust_type_option() {
        let ty = TreeSitterContextAnalyzer::parse_rust_type("Option<String>");
        assert_eq!(ty.name, "String");
        assert!(ty.is_optional);
    }

    #[test]
    fn test_parse_typescript_type_array() {
        let ty = TreeSitterContextAnalyzer::parse_typescript_type("string[]");
        assert_eq!(ty.name, "string");
        assert!(ty.is_array);
    }

    #[test]
    fn test_parse_typescript_type_array_generic() {
        let ty = TreeSitterContextAnalyzer::parse_typescript_type("Array<number>");
        assert_eq!(ty.name, "number");
        assert!(ty.is_array);
    }

    #[test]
    fn test_parse_python_type_list() {
        let ty = TreeSitterContextAnalyzer::parse_python_type("List[str]");
        assert_eq!(ty.name, "str");
        assert!(ty.is_array);
    }

    #[test]
    fn test_parse_python_type_optional() {
        let ty = TreeSitterContextAnalyzer::parse_python_type("Optional[int]");
        assert_eq!(ty.name, "int");
        assert!(ty.is_optional);
    }

    #[test]
    fn test_extract_rust_type_annotation() {
        let line = "let x: String = value;";
        let ty = TreeSitterContextAnalyzer::extract_rust_type_annotation(line);
        assert!(ty.is_some());
        assert_eq!(ty.unwrap(), "String");
    }

    #[test]
    fn test_extract_rust_type_annotation_complex() {
        let line = "let items: Vec<i32> = vec![];";
        let ty = TreeSitterContextAnalyzer::extract_rust_type_annotation(line);
        assert!(ty.is_some());
        assert_eq!(ty.unwrap(), "Vec<i32>");
    }

    #[test]
    fn test_extract_typescript_type_annotation() {
        let line = "let x: string = value;";
        let ty = TreeSitterContextAnalyzer::extract_typescript_type_annotation(line);
        assert!(ty.is_some());
        assert_eq!(ty.unwrap(), "string");
    }

    #[test]
    fn test_extract_python_type_annotation() {
        let line = "x: str = value";
        let ty = TreeSitterContextAnalyzer::extract_python_type_annotation(line);
        assert!(ty.is_some());
        assert_eq!(ty.unwrap(), "str");
    }
}
