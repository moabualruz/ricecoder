//! Built-in language providers implementation
//!
//! This module implements the IdeProvider trait for built-in language support
//! (Rust, TypeScript, Python) using basic pattern matching and analysis.

use async_trait::async_trait;
use tracing::debug;

use crate::{error::IdeResult, provider::IdeProvider, types::*};

/// Built-in provider for Rust
pub struct RustProvider;

/// Built-in provider for TypeScript/JavaScript
pub struct TypeScriptProvider;

/// Built-in provider for Python
pub struct PythonProvider;

#[async_trait]
impl IdeProvider for RustProvider {
    async fn get_completions(&self, params: &CompletionParams) -> IdeResult<Vec<CompletionItem>> {
        debug!("Getting completions from built-in Rust provider");

        let mut completions = Vec::new();

        // Basic Rust keyword completions
        if params.context.contains("fn ") {
            completions.push(CompletionItem {
                label: "fn".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("function declaration".to_string()),
                documentation: Some("Declare a function".to_string()),
                insert_text: "fn ${1:name}(${2:args}) {\n    ${3:body}\n}".to_string(),
            });
        }

        if params.context.contains("let ") {
            completions.push(CompletionItem {
                label: "let".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("variable binding".to_string()),
                documentation: Some("Bind a variable".to_string()),
                insert_text: "let ${1:name} = ${2:value};".to_string(),
            });
        }

        if params.context.contains("struct ") {
            completions.push(CompletionItem {
                label: "struct".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("struct definition".to_string()),
                documentation: Some("Define a struct".to_string()),
                insert_text: "struct ${1:Name} {\n    ${2:fields}\n}".to_string(),
            });
        }

        if params.context.contains("impl ") {
            completions.push(CompletionItem {
                label: "impl".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("implementation block".to_string()),
                documentation: Some("Implement methods for a type".to_string()),
                insert_text: "impl ${1:Type} {\n    ${2:methods}\n}".to_string(),
            });
        }

        Ok(completions)
    }

    async fn get_diagnostics(&self, params: &DiagnosticsParams) -> IdeResult<Vec<Diagnostic>> {
        debug!("Getting diagnostics from built-in Rust provider");

        let mut diagnostics = Vec::new();

        // Basic Rust diagnostics
        if params.source.contains("let ") && !params.source.contains("let _") {
            // Check for unused variables
            if let Some(var_name) = Self::extract_variable_name(&params.source) {
                if !params.source.contains(&format!("_{}", var_name))
                    && !params.source.contains(&var_name[1..])
                {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 0,
                                character: 10,
                            },
                        },
                        severity: DiagnosticSeverity::Warning,
                        message: format!("unused variable: `{}`", var_name),
                        source: "rust-builtin".to_string(),
                    });
                }
            }
        }

        Ok(diagnostics)
    }

    async fn get_hover(&self, _params: &HoverParams) -> IdeResult<Option<Hover>> {
        debug!("Getting hover from built-in Rust provider");
        Ok(None)
    }

    async fn get_definition(&self, _params: &DefinitionParams) -> IdeResult<Option<Location>> {
        debug!("Getting definition from built-in Rust provider");
        Ok(None)
    }

    fn is_available(&self, language: &str) -> bool {
        language == "rust"
    }

    fn name(&self) -> &str {
        "rust-builtin"
    }
}

impl RustProvider {
    /// Extract variable name from let binding
    fn extract_variable_name(source: &str) -> Option<String> {
        if let Some(start) = source.find("let ") {
            let after_let = &source[start + 4..];
            if let Some(end) = after_let.find([' ', '=', ':']) {
                return Some(after_let[..end].trim().to_string());
            }
        }
        None
    }
}

#[async_trait]
impl IdeProvider for TypeScriptProvider {
    async fn get_completions(&self, params: &CompletionParams) -> IdeResult<Vec<CompletionItem>> {
        debug!("Getting completions from built-in TypeScript provider");

        let mut completions = Vec::new();

        // Basic TypeScript keyword completions
        if params.context.contains("function ") || params.context.contains("const ") {
            completions.push(CompletionItem {
                label: "function".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("function declaration".to_string()),
                documentation: Some("Declare a function".to_string()),
                insert_text: "function ${1:name}(${2:args}) {\n    ${3:body}\n}".to_string(),
            });
        }

        if params.context.contains("const ") {
            completions.push(CompletionItem {
                label: "const".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("constant binding".to_string()),
                documentation: Some("Declare a constant".to_string()),
                insert_text: "const ${1:name} = ${2:value};".to_string(),
            });
        }

        if params.context.contains("interface ") {
            completions.push(CompletionItem {
                label: "interface".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("interface definition".to_string()),
                documentation: Some("Define an interface".to_string()),
                insert_text: "interface ${1:Name} {\n    ${2:properties}\n}".to_string(),
            });
        }

        if params.context.contains("class ") {
            completions.push(CompletionItem {
                label: "class".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("class definition".to_string()),
                documentation: Some("Define a class".to_string()),
                insert_text: "class ${1:Name} {\n    ${2:members}\n}".to_string(),
            });
        }

        Ok(completions)
    }

    async fn get_diagnostics(&self, params: &DiagnosticsParams) -> IdeResult<Vec<Diagnostic>> {
        debug!("Getting diagnostics from built-in TypeScript provider");

        let mut diagnostics = Vec::new();

        // Basic TypeScript diagnostics
        if params.source.contains("const ") && !params.source.contains("const _") {
            // Check for unused variables
            if let Some(var_name) = Self::extract_variable_name(&params.source) {
                if !params.source.contains(&format!("_{}", var_name))
                    && !params.source.contains(&var_name[1..])
                {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 0,
                                character: 10,
                            },
                        },
                        severity: DiagnosticSeverity::Warning,
                        message: format!("'{}' is declared but its value is never used", var_name),
                        source: "typescript-builtin".to_string(),
                    });
                }
            }
        }

        Ok(diagnostics)
    }

    async fn get_hover(&self, _params: &HoverParams) -> IdeResult<Option<Hover>> {
        debug!("Getting hover from built-in TypeScript provider");
        Ok(None)
    }

    async fn get_definition(&self, _params: &DefinitionParams) -> IdeResult<Option<Location>> {
        debug!("Getting definition from built-in TypeScript provider");
        Ok(None)
    }

    fn is_available(&self, language: &str) -> bool {
        language == "typescript" || language == "javascript"
    }

    fn name(&self) -> &str {
        "typescript-builtin"
    }
}

impl TypeScriptProvider {
    /// Extract variable name from const binding
    fn extract_variable_name(source: &str) -> Option<String> {
        if let Some(start) = source.find("const ") {
            let after_const = &source[start + 6..];
            if let Some(end) = after_const.find([' ', '=', ':']) {
                return Some(after_const[..end].trim().to_string());
            }
        }
        None
    }
}

#[async_trait]
impl IdeProvider for PythonProvider {
    async fn get_completions(&self, params: &CompletionParams) -> IdeResult<Vec<CompletionItem>> {
        debug!("Getting completions from built-in Python provider");

        let mut completions = Vec::new();

        // Basic Python keyword completions
        if params.context.contains("def ") {
            completions.push(CompletionItem {
                label: "def".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("function definition".to_string()),
                documentation: Some("Define a function".to_string()),
                insert_text: "def ${1:name}(${2:args}):\n    ${3:body}".to_string(),
            });
        }

        if params.context.contains("class ") {
            completions.push(CompletionItem {
                label: "class".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("class definition".to_string()),
                documentation: Some("Define a class".to_string()),
                insert_text: "class ${1:Name}:\n    ${2:body}".to_string(),
            });
        }

        if params.context.contains("if ") {
            completions.push(CompletionItem {
                label: "if".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("conditional statement".to_string()),
                documentation: Some("Conditional execution".to_string()),
                insert_text: "if ${1:condition}:\n    ${2:body}".to_string(),
            });
        }

        if params.context.contains("for ") {
            completions.push(CompletionItem {
                label: "for".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("loop statement".to_string()),
                documentation: Some("Iterate over a sequence".to_string()),
                insert_text: "for ${1:item} in ${2:sequence}:\n    ${3:body}".to_string(),
            });
        }

        Ok(completions)
    }

    async fn get_diagnostics(&self, params: &DiagnosticsParams) -> IdeResult<Vec<Diagnostic>> {
        debug!("Getting diagnostics from built-in Python provider");

        let mut diagnostics = Vec::new();

        // Basic Python diagnostics
        if params.source.contains("import ") {
            // Check for unused imports
            if let Some(module_name) = Self::extract_import_name(&params.source) {
                if !params.source.contains(&module_name)
                    || params.source.matches(&module_name).count() == 1
                {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 0,
                                character: 10,
                            },
                        },
                        severity: DiagnosticSeverity::Warning,
                        message: format!("'{}' imported but unused", module_name),
                        source: "python-builtin".to_string(),
                    });
                }
            }
        }

        Ok(diagnostics)
    }

    async fn get_hover(&self, _params: &HoverParams) -> IdeResult<Option<Hover>> {
        debug!("Getting hover from built-in Python provider");
        Ok(None)
    }

    async fn get_definition(&self, _params: &DefinitionParams) -> IdeResult<Option<Location>> {
        debug!("Getting definition from built-in Python provider");
        Ok(None)
    }

    fn is_available(&self, language: &str) -> bool {
        language == "python"
    }

    fn name(&self) -> &str {
        "python-builtin"
    }
}

impl PythonProvider {
    /// Extract module name from import statement
    fn extract_import_name(source: &str) -> Option<String> {
        if let Some(start) = source.find("import ") {
            let after_import = &source[start + 7..];
            let end = after_import
                .find([' ', '\n', ';'])
                .unwrap_or(after_import.len());
            if end > 0 {
                return Some(after_import[..end].trim().to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rust_provider_completions() {
        let provider = RustProvider;
        let params = CompletionParams {
            language: "rust".to_string(),
            file_path: "src/main.rs".to_string(),
            position: Position {
                line: 10,
                character: 5,
            },
            context: "fn test".to_string(),
        };

        let result = provider.get_completions(&params).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_rust_provider_is_available() {
        let provider = RustProvider;
        assert!(provider.is_available("rust"));
        assert!(!provider.is_available("typescript"));
    }

    #[tokio::test]
    async fn test_typescript_provider_completions() {
        let provider = TypeScriptProvider;
        let params = CompletionParams {
            language: "typescript".to_string(),
            file_path: "src/main.ts".to_string(),
            position: Position {
                line: 10,
                character: 5,
            },
            context: "const test".to_string(),
        };

        let result = provider.get_completions(&params).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_typescript_provider_is_available() {
        let provider = TypeScriptProvider;
        assert!(provider.is_available("typescript"));
        assert!(provider.is_available("javascript"));
        assert!(!provider.is_available("rust"));
    }

    #[tokio::test]
    async fn test_python_provider_completions() {
        let provider = PythonProvider;
        let params = CompletionParams {
            language: "python".to_string(),
            file_path: "main.py".to_string(),
            position: Position {
                line: 10,
                character: 5,
            },
            context: "def test".to_string(),
        };

        let result = provider.get_completions(&params).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_python_provider_is_available() {
        let provider = PythonProvider;
        assert!(provider.is_available("python"));
        assert!(!provider.is_available("rust"));
    }

    #[test]
    fn test_rust_extract_variable_name() {
        let source = "let my_var = 5;";
        let name = RustProvider::extract_variable_name(source);
        assert_eq!(name, Some("my_var".to_string()));
    }

    #[test]
    fn test_typescript_extract_variable_name() {
        let source = "const my_var = 5;";
        let name = TypeScriptProvider::extract_variable_name(source);
        assert_eq!(name, Some("my_var".to_string()));
    }

    #[test]
    fn test_python_extract_import_name() {
        let source = "import os";
        let name = PythonProvider::extract_import_name(source);
        assert_eq!(name, Some("os".to_string()));
    }
}
