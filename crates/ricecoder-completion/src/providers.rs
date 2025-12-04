/// Pluggable completion providers for language-specific behavior
use crate::types::*;
use async_trait::async_trait;

/// Helper function to convert symbol kind to completion item kind
fn symbol_kind_to_completion_kind(kind: SymbolKind) -> CompletionItemKind {
    match kind {
        SymbolKind::Variable => CompletionItemKind::Variable,
        SymbolKind::Function => CompletionItemKind::Function,
        SymbolKind::Type => CompletionItemKind::Class,
        SymbolKind::Constant => CompletionItemKind::Constant,
        SymbolKind::Module => CompletionItemKind::Module,
        SymbolKind::Class => CompletionItemKind::Class,
        SymbolKind::Struct => CompletionItemKind::Struct,
        SymbolKind::Enum => CompletionItemKind::Enum,
        SymbolKind::Interface => CompletionItemKind::Interface,
        SymbolKind::Trait => CompletionItemKind::Interface,
        SymbolKind::Method => CompletionItemKind::Method,
        SymbolKind::Property => CompletionItemKind::Property,
        SymbolKind::Field => CompletionItemKind::Field,
        SymbolKind::Parameter => CompletionItemKind::Variable,
        SymbolKind::Keyword => CompletionItemKind::Keyword,
    }
}

/// Helper function to create a completion item from a symbol
fn symbol_to_completion_item(symbol: &Symbol, score: f32) -> CompletionItem {
    let mut item = CompletionItem::new(
        symbol.name.clone(),
        symbol_kind_to_completion_kind(symbol.kind),
        symbol.name.clone(),
    )
    .with_score(score);

    // Add type information as detail
    if let Some(type_info) = &symbol.type_info {
        item = item.with_detail(type_info.clone());
    }

    // Add documentation
    if let Some(documentation) = &symbol.documentation {
        item = item.with_documentation(documentation.clone());
    }

    item
}

/// Helper function to create snippet completions
fn create_snippet_item(label: &str, template: &str, description: &str, score: f32) -> CompletionItem {
    CompletionItem::new(
        label.to_string(),
        CompletionItemKind::Snippet,
        template.to_string(),
    )
    .with_detail(description.to_string())
    .with_score(score)
}

/// Generic text-based completion provider (fallback for unconfigured languages)
pub struct GenericTextProvider;

#[async_trait]
impl crate::engine::CompletionProvider for GenericTextProvider {
    fn language(&self) -> &str {
        "generic"
    }

    async fn generate_completions(
        &self,
        _code: &str,
        _position: Position,
        context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>> {
        let mut completions = Vec::new();

        // Generate completions from available symbols
        for symbol in &context.available_symbols {
            let item = symbol_to_completion_item(symbol, 0.5);
            completions.push(item);
        }

        Ok(completions)
    }
}

/// Rust-specific completion provider
pub struct RustCompletionProvider;

#[async_trait]
impl crate::engine::CompletionProvider for RustCompletionProvider {
    fn language(&self) -> &str {
        "rust"
    }

    async fn generate_completions(
        &self,
        _code: &str,
        _position: Position,
        context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>> {
        let mut completions = Vec::new();

        // Add symbols from context first (higher priority)
        for symbol in &context.available_symbols {
            let mut item = CompletionItem::new(
                symbol.name.clone(),
                match symbol.kind {
                    SymbolKind::Variable => CompletionItemKind::Variable,
                    SymbolKind::Function => CompletionItemKind::Function,
                    SymbolKind::Type => CompletionItemKind::Struct,
                    SymbolKind::Constant => CompletionItemKind::Constant,
                    SymbolKind::Module => CompletionItemKind::Module,
                    SymbolKind::Class => CompletionItemKind::Struct,
                    SymbolKind::Struct => CompletionItemKind::Struct,
                    SymbolKind::Enum => CompletionItemKind::Enum,
                    SymbolKind::Interface => CompletionItemKind::Trait,
                    SymbolKind::Trait => CompletionItemKind::Trait,
                    SymbolKind::Method => CompletionItemKind::Method,
                    SymbolKind::Property => CompletionItemKind::Field,
                    SymbolKind::Field => CompletionItemKind::Field,
                    SymbolKind::Parameter => CompletionItemKind::Variable,
                    SymbolKind::Keyword => CompletionItemKind::Keyword,
                },
                symbol.name.clone(),
            )
            .with_score(0.8);

            // Add type information as detail
            if let Some(type_info) = &symbol.type_info {
                item = item.with_detail(type_info.clone());
            }

            // Add documentation
            if let Some(documentation) = &symbol.documentation {
                item = item.with_documentation(documentation.clone());
            }

            completions.push(item);
        }

        // Add Rust-specific keywords
        let rust_keywords = vec![
            ("fn", "Function declaration"),
            ("let", "Variable binding"),
            ("mut", "Mutable binding"),
            ("const", "Constant declaration"),
            ("static", "Static variable"),
            ("struct", "Struct definition"),
            ("enum", "Enum definition"),
            ("trait", "Trait definition"),
            ("impl", "Implementation block"),
            ("pub", "Public visibility"),
            ("mod", "Module declaration"),
            ("use", "Import statement"),
            ("match", "Pattern matching"),
            ("if", "Conditional"),
            ("else", "Else clause"),
            ("for", "For loop"),
            ("while", "While loop"),
            ("loop", "Infinite loop"),
            ("break", "Break statement"),
            ("continue", "Continue statement"),
            ("return", "Return statement"),
            ("async", "Async function"),
            ("await", "Await expression"),
            ("unsafe", "Unsafe block"),
            ("dyn", "Dynamic trait object"),
            ("where", "Where clause"),
            ("as", "Type casting"),
        ];

        for (keyword, description) in rust_keywords {
            let item = CompletionItem::new(
                keyword.to_string(),
                CompletionItemKind::Keyword,
                keyword.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.6);

            completions.push(item);
        }

        // Add Rust-specific snippets
        let rust_snippets = vec![
            ("fn_snippet", "fn ${1:name}(${2:args}) {\n    ${3:body}\n}", "Function template"),
            ("impl_snippet", "impl ${1:Type} {\n    ${2:methods}\n}", "Implementation block"),
            ("match_snippet", "match ${1:expr} {\n    ${2:pattern} => ${3:result},\n}", "Match expression"),
            ("for_snippet", "for ${1:item} in ${2:iter} {\n    ${3:body}\n}", "For loop"),
            ("while_snippet", "while ${1:condition} {\n    ${2:body}\n}", "While loop"),
            ("if_snippet", "if ${1:condition} {\n    ${2:then}\n} else {\n    ${3:else}\n}", "If-else block"),
            ("struct_snippet", "struct ${1:Name} {\n    ${2:fields}\n}", "Struct definition"),
            ("enum_snippet", "enum ${1:Name} {\n    ${2:variants}\n}", "Enum definition"),
            ("trait_snippet", "trait ${1:Name} {\n    ${2:methods}\n}", "Trait definition"),
            ("closure_snippet", "|${1:args}| ${2:body}", "Closure"),
        ];

        for (label, template, description) in rust_snippets {
            let item = create_snippet_item(label, template, description, 0.7);
            completions.push(item);
        }

        // Add Rust-specific traits and macros
        let rust_traits = vec![
            ("Debug", "Debug trait for formatting", CompletionItemKind::Trait),
            ("Clone", "Clone trait for copying", CompletionItemKind::Trait),
            ("Copy", "Copy trait for stack copying", CompletionItemKind::Trait),
            ("Default", "Default trait for default values", CompletionItemKind::Trait),
            ("Display", "Display trait for formatting", CompletionItemKind::Trait),
            ("Iterator", "Iterator trait for iteration", CompletionItemKind::Trait),
            ("IntoIterator", "IntoIterator trait for conversion", CompletionItemKind::Trait),
            ("From", "From trait for conversion", CompletionItemKind::Trait),
            ("Into", "Into trait for conversion", CompletionItemKind::Trait),
            ("AsRef", "AsRef trait for borrowing", CompletionItemKind::Trait),
            ("AsMut", "AsMut trait for mutable borrowing", CompletionItemKind::Trait),
            ("Deref", "Deref trait for dereferencing", CompletionItemKind::Trait),
            ("Drop", "Drop trait for cleanup", CompletionItemKind::Trait),
            ("Eq", "Eq trait for equality", CompletionItemKind::Trait),
            ("PartialEq", "PartialEq trait for partial equality", CompletionItemKind::Trait),
            ("Ord", "Ord trait for ordering", CompletionItemKind::Trait),
            ("PartialOrd", "PartialOrd trait for partial ordering", CompletionItemKind::Trait),
            ("Hash", "Hash trait for hashing", CompletionItemKind::Trait),
        ];

        for (name, description, kind) in rust_traits {
            let item = CompletionItem::new(
                name.to_string(),
                kind,
                name.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.75);
            completions.push(item);
        }

        // Add Rust-specific macros
        let rust_macros = vec![
            ("println!", "Print with newline"),
            ("print!", "Print without newline"),
            ("eprintln!", "Print to stderr with newline"),
            ("eprint!", "Print to stderr without newline"),
            ("format!", "Format string"),
            ("panic!", "Panic with message"),
            ("assert!", "Assert condition"),
            ("assert_eq!", "Assert equality"),
            ("assert_ne!", "Assert inequality"),
            ("debug_assert!", "Debug assert"),
            ("vec!", "Create vector"),
            ("map!", "Map macro"),
            ("include!", "Include file"),
            ("include_str!", "Include string"),
            ("include_bytes!", "Include bytes"),
            ("concat!", "Concatenate strings"),
            ("stringify!", "Stringify expression"),
            ("env!", "Get environment variable"),
            ("cfg!", "Check configuration"),
            ("compile_error!", "Compile error"),
        ];

        for (macro_name, description) in rust_macros {
            let item = CompletionItem::new(
                macro_name.to_string(),
                CompletionItemKind::Operator,
                macro_name.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.7);
            completions.push(item);
        }

        // Add Rust-specific derive attributes
        let rust_derives = vec![
            ("Debug", "Derive Debug trait"),
            ("Clone", "Derive Clone trait"),
            ("Copy", "Derive Copy trait"),
            ("Default", "Derive Default trait"),
            ("PartialEq", "Derive PartialEq trait"),
            ("Eq", "Derive Eq trait"),
            ("PartialOrd", "Derive PartialOrd trait"),
            ("Ord", "Derive Ord trait"),
            ("Hash", "Derive Hash trait"),
        ];

        for (derive_name, description) in rust_derives {
            let item = CompletionItem::new(
                format!("#[derive({})]", derive_name),
                CompletionItemKind::Keyword,
                format!("#[derive({})]", derive_name),
            )
            .with_detail(description.to_string())
            .with_score(0.65);
            completions.push(item);
        }

        Ok(completions)
    }
}

/// TypeScript-specific completion provider
pub struct TypeScriptCompletionProvider;

#[async_trait]
impl crate::engine::CompletionProvider for TypeScriptCompletionProvider {
    fn language(&self) -> &str {
        "typescript"
    }

    async fn generate_completions(
        &self,
        _code: &str,
        _position: Position,
        context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>> {
        let mut completions = Vec::new();

        // Add symbols from context first (higher priority)
        for symbol in &context.available_symbols {
            let mut item = CompletionItem::new(
                symbol.name.clone(),
                match symbol.kind {
                    SymbolKind::Variable => CompletionItemKind::Variable,
                    SymbolKind::Function => CompletionItemKind::Function,
                    SymbolKind::Type => CompletionItemKind::Interface,
                    SymbolKind::Constant => CompletionItemKind::Constant,
                    SymbolKind::Module => CompletionItemKind::Module,
                    SymbolKind::Class => CompletionItemKind::Class,
                    SymbolKind::Struct => CompletionItemKind::Class,
                    SymbolKind::Enum => CompletionItemKind::Enum,
                    SymbolKind::Interface => CompletionItemKind::Interface,
                    SymbolKind::Trait => CompletionItemKind::Interface,
                    SymbolKind::Method => CompletionItemKind::Method,
                    SymbolKind::Property => CompletionItemKind::Property,
                    SymbolKind::Field => CompletionItemKind::Field,
                    SymbolKind::Parameter => CompletionItemKind::Variable,
                    SymbolKind::Keyword => CompletionItemKind::Keyword,
                },
                symbol.name.clone(),
            )
            .with_score(0.8);

            // Add type information as detail
            if let Some(type_info) = &symbol.type_info {
                item = item.with_detail(type_info.clone());
            }

            // Add documentation
            if let Some(documentation) = &symbol.documentation {
                item = item.with_documentation(documentation.clone());
            }

            completions.push(item);
        }

        // Add TypeScript-specific keywords
        let ts_keywords = vec![
            ("function", "Function declaration"),
            ("const", "Constant declaration"),
            ("let", "Block-scoped variable"),
            ("var", "Function-scoped variable"),
            ("class", "Class definition"),
            ("interface", "Interface definition"),
            ("type", "Type alias"),
            ("enum", "Enum definition"),
            ("namespace", "Namespace"),
            ("module", "Module"),
            ("export", "Export declaration"),
            ("import", "Import statement"),
            ("async", "Async function"),
            ("await", "Await expression"),
            ("if", "Conditional"),
            ("else", "Else clause"),
            ("for", "For loop"),
            ("while", "While loop"),
            ("do", "Do-while loop"),
            ("switch", "Switch statement"),
            ("case", "Case clause"),
            ("default", "Default clause"),
            ("break", "Break statement"),
            ("continue", "Continue statement"),
            ("return", "Return statement"),
            ("throw", "Throw statement"),
            ("try", "Try block"),
            ("catch", "Catch block"),
            ("finally", "Finally block"),
            ("new", "New instance"),
            ("this", "This reference"),
            ("super", "Super reference"),
            ("extends", "Extends clause"),
            ("implements", "Implements clause"),
            ("public", "Public modifier"),
            ("private", "Private modifier"),
            ("protected", "Protected modifier"),
            ("readonly", "Readonly modifier"),
            ("static", "Static modifier"),
            ("abstract", "Abstract modifier"),
            ("declare", "Declare statement"),
            ("keyof", "Keyof operator"),
            ("typeof", "Typeof operator"),
            ("instanceof", "Instanceof operator"),
            ("in", "In operator"),
            ("of", "Of operator"),
        ];

        for (keyword, description) in ts_keywords {
            let item = CompletionItem::new(
                keyword.to_string(),
                CompletionItemKind::Keyword,
                keyword.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.6);

            completions.push(item);
        }

        // Add TypeScript-specific snippets
        let ts_snippets = vec![
            ("fn_snippet", "function ${1:name}(${2:args}): ${3:ReturnType} {\n    ${4:body}\n}", "Function template"),
            ("arrow_fn_snippet", "const ${1:name} = (${2:args}): ${3:ReturnType} => {\n    ${4:body}\n}", "Arrow function"),
            ("class_snippet", "class ${1:Name} {\n    constructor(${2:args}) {\n        ${3:init}\n    }\n    ${4:methods}\n}", "Class definition"),
            ("interface_snippet", "interface ${1:Name} {\n    ${2:properties}\n}", "Interface definition"),
            ("for_snippet", "for (let ${1:i} = 0; ${1:i} < ${2:length}; ${1:i}++) {\n    ${3:body}\n}", "For loop"),
            ("for_of_snippet", "for (const ${1:item} of ${2:iterable}) {\n    ${3:body}\n}", "For-of loop"),
            ("while_snippet", "while (${1:condition}) {\n    ${2:body}\n}", "While loop"),
            ("if_snippet", "if (${1:condition}) {\n    ${2:then}\n} else {\n    ${3:else}\n}", "If-else block"),
            ("try_snippet", "try {\n    ${1:body}\n} catch (${2:error}) {\n    ${3:handler}\n}", "Try-catch block"),
            ("async_fn_snippet", "async function ${1:name}(${2:args}): Promise<${3:Type}> {\n    ${4:body}\n}", "Async function"),
        ];

        for (label, template, description) in ts_snippets {
            let item = create_snippet_item(label, template, description, 0.7);
            completions.push(item);
        }

        // Add TypeScript-specific interfaces and types
        let ts_interfaces = vec![
            ("Record", "Record type for key-value pairs", CompletionItemKind::Interface),
            ("Partial", "Partial type for optional properties", CompletionItemKind::Interface),
            ("Required", "Required type for mandatory properties", CompletionItemKind::Interface),
            ("Readonly", "Readonly type for immutable properties", CompletionItemKind::Interface),
            ("Pick", "Pick type for selecting properties", CompletionItemKind::Interface),
            ("Omit", "Omit type for excluding properties", CompletionItemKind::Interface),
            ("Exclude", "Exclude type for union exclusion", CompletionItemKind::Interface),
            ("Extract", "Extract type for union extraction", CompletionItemKind::Interface),
            ("NonNullable", "NonNullable type for non-null values", CompletionItemKind::Interface),
            ("Parameters", "Parameters type for function parameters", CompletionItemKind::Interface),
            ("ReturnType", "ReturnType type for function return type", CompletionItemKind::Interface),
            ("InstanceType", "InstanceType type for class instances", CompletionItemKind::Interface),
            ("Awaited", "Awaited type for promise resolution", CompletionItemKind::Interface),
        ];

        for (name, description, kind) in ts_interfaces {
            let item = CompletionItem::new(
                name.to_string(),
                kind,
                name.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.75);
            completions.push(item);
        }

        // Add TypeScript-specific decorators
        let ts_decorators = vec![
            ("@deprecated", "Mark as deprecated"),
            ("@sealed", "Seal class"),
            ("@frozen", "Freeze class"),
            ("@readonly", "Mark property as readonly"),
            ("@validate", "Validate input"),
            ("@memoize", "Memoize function"),
            ("@throttle", "Throttle function"),
            ("@debounce", "Debounce function"),
            ("@observable", "Make observable"),
            ("@computed", "Computed property"),
        ];

        for (decorator_name, description) in ts_decorators {
            let item = CompletionItem::new(
                decorator_name.to_string(),
                CompletionItemKind::Keyword,
                decorator_name.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.65);
            completions.push(item);
        }

        // Add TypeScript generic patterns
        let ts_generics = vec![
            ("Array<T>", "Generic array type"),
            ("Promise<T>", "Generic promise type"),
            ("Map<K, V>", "Generic map type"),
            ("Set<T>", "Generic set type"),
            ("Record<K, V>", "Generic record type"),
            ("Tuple<T, U>", "Generic tuple type"),
        ];

        for (generic_name, description) in ts_generics {
            let item = CompletionItem::new(
                generic_name.to_string(),
                CompletionItemKind::TypeParameter,
                generic_name.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.7);
            completions.push(item);
        }

        Ok(completions)
    }
}

/// Python-specific completion provider
pub struct PythonCompletionProvider;

#[async_trait]
impl crate::engine::CompletionProvider for PythonCompletionProvider {
    fn language(&self) -> &str {
        "python"
    }

    async fn generate_completions(
        &self,
        _code: &str,
        _position: Position,
        context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>> {
        let mut completions = Vec::new();

        // Add symbols from context first (higher priority)
        for symbol in &context.available_symbols {
            let mut item = CompletionItem::new(
                symbol.name.clone(),
                match symbol.kind {
                    SymbolKind::Variable => CompletionItemKind::Variable,
                    SymbolKind::Function => CompletionItemKind::Function,
                    SymbolKind::Type => CompletionItemKind::Class,
                    SymbolKind::Constant => CompletionItemKind::Constant,
                    SymbolKind::Module => CompletionItemKind::Module,
                    SymbolKind::Class => CompletionItemKind::Class,
                    SymbolKind::Struct => CompletionItemKind::Class,
                    SymbolKind::Enum => CompletionItemKind::Enum,
                    SymbolKind::Interface => CompletionItemKind::Interface,
                    SymbolKind::Trait => CompletionItemKind::Interface,
                    SymbolKind::Method => CompletionItemKind::Method,
                    SymbolKind::Property => CompletionItemKind::Property,
                    SymbolKind::Field => CompletionItemKind::Field,
                    SymbolKind::Parameter => CompletionItemKind::Variable,
                    SymbolKind::Keyword => CompletionItemKind::Keyword,
                },
                symbol.name.clone(),
            )
            .with_score(0.8);

            // Add type information as detail
            if let Some(type_info) = &symbol.type_info {
                item = item.with_detail(type_info.clone());
            }

            // Add documentation
            if let Some(documentation) = &symbol.documentation {
                item = item.with_documentation(documentation.clone());
            }

            completions.push(item);
        }

        // Add Python-specific keywords
        let py_keywords = vec![
            ("def", "Function definition"),
            ("class", "Class definition"),
            ("if", "Conditional"),
            ("elif", "Else-if clause"),
            ("else", "Else clause"),
            ("for", "For loop"),
            ("while", "While loop"),
            ("break", "Break statement"),
            ("continue", "Continue statement"),
            ("return", "Return statement"),
            ("yield", "Yield statement"),
            ("import", "Import statement"),
            ("from", "From import"),
            ("as", "Alias"),
            ("try", "Try block"),
            ("except", "Except block"),
            ("finally", "Finally block"),
            ("raise", "Raise exception"),
            ("with", "Context manager"),
            ("assert", "Assert statement"),
            ("pass", "Pass statement"),
            ("del", "Delete statement"),
            ("lambda", "Lambda function"),
            ("and", "Logical and"),
            ("or", "Logical or"),
            ("not", "Logical not"),
            ("in", "In operator"),
            ("is", "Is operator"),
            ("None", "None value"),
            ("True", "True value"),
            ("False", "False value"),
            ("async", "Async function"),
            ("await", "Await expression"),
            ("global", "Global declaration"),
            ("nonlocal", "Nonlocal declaration"),
        ];

        for (keyword, description) in py_keywords {
            let item = CompletionItem::new(
                keyword.to_string(),
                CompletionItemKind::Keyword,
                keyword.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.6);

            completions.push(item);
        }

        // Add Python-specific snippets
        let py_snippets = vec![
            ("def_snippet", "def ${1:name}(${2:args}):\n    ${3:body}", "Function definition"),
            ("class_snippet", "class ${1:Name}:\n    def __init__(self, ${2:args}):\n        ${3:init}\n    ${4:methods}", "Class definition"),
            ("for_snippet", "for ${1:item} in ${2:iterable}:\n    ${3:body}", "For loop"),
            ("while_snippet", "while ${1:condition}:\n    ${2:body}", "While loop"),
            ("if_snippet", "if ${1:condition}:\n    ${2:then}\nelse:\n    ${3:else}", "If-else block"),
            ("try_snippet", "try:\n    ${1:body}\nexcept ${2:Exception}:\n    ${3:handler}", "Try-except block"),
            ("with_snippet", "with ${1:context} as ${2:var}:\n    ${3:body}", "Context manager"),
            ("lambda_snippet", "lambda ${1:args}: ${2:expr}", "Lambda function"),
            ("list_comp_snippet", "[${1:expr} for ${2:item} in ${3:iterable}]", "List comprehension"),
            ("dict_comp_snippet", "{${1:key}: ${2:value} for ${3:item} in ${4:iterable}}", "Dictionary comprehension"),
        ];

        for (label, template, description) in py_snippets {
            let item = create_snippet_item(label, template, description, 0.7);
            completions.push(item);
        }

        // Add Python-specific decorators
        let py_decorators = vec![
            ("@property", "Property decorator for getters"),
            ("@staticmethod", "Static method decorator"),
            ("@classmethod", "Class method decorator"),
            ("@abstractmethod", "Abstract method decorator"),
            ("@deprecated", "Deprecated decorator"),
            ("@lru_cache", "LRU cache decorator"),
            ("@wraps", "Wraps decorator for decorators"),
            ("@contextmanager", "Context manager decorator"),
            ("@dataclass", "Dataclass decorator"),
            ("@enum.unique", "Unique enum decorator"),
        ];

        for (decorator_name, description) in py_decorators {
            let item = CompletionItem::new(
                decorator_name.to_string(),
                CompletionItemKind::Keyword,
                decorator_name.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.65);
            completions.push(item);
        }

        // Add Python-specific type hints
        let py_type_hints = vec![
            ("List[T]", "List type hint"),
            ("Dict[K, V]", "Dictionary type hint"),
            ("Set[T]", "Set type hint"),
            ("Tuple[T, ...]", "Tuple type hint"),
            ("Optional[T]", "Optional type hint"),
            ("Union[T, U]", "Union type hint"),
            ("Any", "Any type hint"),
            ("Callable[[T], U]", "Callable type hint"),
            ("Iterator[T]", "Iterator type hint"),
            ("Generator[T, U, V]", "Generator type hint"),
            ("Iterable[T]", "Iterable type hint"),
            ("Sequence[T]", "Sequence type hint"),
            ("Mapping[K, V]", "Mapping type hint"),
            ("TypeVar", "Type variable"),
            ("Generic", "Generic base class"),
        ];

        for (type_hint, description) in py_type_hints {
            let item = CompletionItem::new(
                type_hint.to_string(),
                CompletionItemKind::TypeParameter,
                type_hint.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.7);
            completions.push(item);
        }

        // Add Python-specific context managers
        let py_context_managers = vec![
            ("open()", "File context manager"),
            ("lock", "Lock context manager"),
            ("pool", "Connection pool context manager"),
            ("transaction", "Database transaction context manager"),
            ("tempfile.TemporaryDirectory()", "Temporary directory context manager"),
            ("tempfile.NamedTemporaryFile()", "Temporary file context manager"),
            ("contextlib.suppress()", "Suppress exceptions context manager"),
            ("contextlib.redirect_stdout()", "Redirect stdout context manager"),
            ("contextlib.redirect_stderr()", "Redirect stderr context manager"),
        ];

        for (cm_name, description) in py_context_managers {
            let item = CompletionItem::new(
                cm_name.to_string(),
                CompletionItemKind::Function,
                cm_name.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.65);
            completions.push(item);
        }

        Ok(completions)
    }
}

/// Go-specific completion provider
pub struct GoCompletionProvider;

#[async_trait]
impl crate::engine::CompletionProvider for GoCompletionProvider {
    fn language(&self) -> &str {
        "go"
    }

    async fn generate_completions(
        &self,
        _code: &str,
        _position: Position,
        context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>> {
        let mut completions = Vec::new();

        // Add symbols from context
        for symbol in &context.available_symbols {
            let item = symbol_to_completion_item(symbol, 0.8);
            completions.push(item);
        }

        // Add Go-specific keywords
        let go_keywords = vec![
            ("package", "Package declaration"),
            ("import", "Import statement"),
            ("func", "Function declaration"),
            ("const", "Constant declaration"),
            ("var", "Variable declaration"),
            ("type", "Type declaration"),
            ("struct", "Struct definition"),
            ("interface", "Interface definition"),
            ("defer", "Defer statement"),
            ("go", "Goroutine"),
            ("chan", "Channel"),
            ("select", "Select statement"),
            ("case", "Case clause"),
            ("default", "Default clause"),
            ("if", "Conditional"),
            ("else", "Else clause"),
            ("for", "For loop"),
            ("range", "Range keyword"),
            ("break", "Break statement"),
            ("continue", "Continue statement"),
            ("return", "Return statement"),
            ("fallthrough", "Fallthrough statement"),
            ("switch", "Switch statement"),
            ("goto", "Goto statement"),
            ("map", "Map type"),
            ("make", "Make function"),
            ("new", "New function"),
            ("append", "Append function"),
            ("copy", "Copy function"),
            ("delete", "Delete function"),
            ("len", "Length function"),
            ("cap", "Capacity function"),
            ("close", "Close function"),
            ("panic", "Panic function"),
            ("recover", "Recover function"),
        ];

        for (keyword, description) in go_keywords {
            let item = CompletionItem::new(
                keyword.to_string(),
                CompletionItemKind::Keyword,
                keyword.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.6);
            completions.push(item);
        }

        // Add Go-specific snippets
        let go_snippets = vec![
            ("func_snippet", "func ${1:name}(${2:params}) ${3:returnType} {\n    ${4:body}\n}", "Function template"),
            ("interface_snippet", "type ${1:Name} interface {\n    ${2:methods}\n}", "Interface definition"),
            ("struct_snippet", "type ${1:Name} struct {\n    ${2:fields}\n}", "Struct definition"),
            ("for_snippet", "for ${1:i} := 0; ${1:i} < ${2:n}; ${1:i}++ {\n    ${3:body}\n}", "For loop"),
            ("for_range_snippet", "for ${1:key}, ${2:value} := range ${3:collection} {\n    ${4:body}\n}", "For-range loop"),
            ("if_err_snippet", "if err != nil {\n    ${1:handle error}\n}", "Error handling"),
            ("defer_snippet", "defer ${1:function}()", "Defer statement"),
            ("goroutine_snippet", "go ${1:function}()", "Goroutine"),
            ("channel_snippet", "ch := make(chan ${1:Type})\ngo func() {\n    ch <- ${2:value}\n}()\n${3:result} := <-ch", "Channel pattern"),
            ("select_snippet", "select {\ncase ${1:case1}:\n    ${2:body1}\ncase ${3:case2}:\n    ${4:body2}\ndefault:\n    ${5:default}\n}", "Select statement"),
        ];

        for (label, template, description) in go_snippets {
            let item = create_snippet_item(label, template, description, 0.7);
            completions.push(item);
        }

        Ok(completions)
    }
}

/// Java-specific completion provider
pub struct JavaCompletionProvider;

#[async_trait]
impl crate::engine::CompletionProvider for JavaCompletionProvider {
    fn language(&self) -> &str {
        "java"
    }

    async fn generate_completions(
        &self,
        _code: &str,
        _position: Position,
        context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>> {
        let mut completions = Vec::new();

        // Add symbols from context
        for symbol in &context.available_symbols {
            let item = symbol_to_completion_item(symbol, 0.8);
            completions.push(item);
        }

        // Add Java-specific keywords
        let java_keywords = vec![
            ("abstract", "Abstract modifier"),
            ("assert", "Assert statement"),
            ("boolean", "Boolean type"),
            ("break", "Break statement"),
            ("byte", "Byte type"),
            ("case", "Case clause"),
            ("catch", "Catch block"),
            ("char", "Char type"),
            ("class", "Class definition"),
            ("const", "Const keyword"),
            ("continue", "Continue statement"),
            ("default", "Default clause"),
            ("do", "Do-while loop"),
            ("double", "Double type"),
            ("else", "Else clause"),
            ("enum", "Enum definition"),
            ("extends", "Extends keyword"),
            ("final", "Final modifier"),
            ("finally", "Finally block"),
            ("float", "Float type"),
            ("for", "For loop"),
            ("if", "Conditional"),
            ("implements", "Implements keyword"),
            ("import", "Import statement"),
            ("instanceof", "Instanceof operator"),
            ("int", "Int type"),
            ("interface", "Interface definition"),
            ("long", "Long type"),
            ("native", "Native modifier"),
            ("new", "New keyword"),
            ("package", "Package declaration"),
            ("private", "Private modifier"),
            ("protected", "Protected modifier"),
            ("public", "Public modifier"),
            ("return", "Return statement"),
            ("short", "Short type"),
            ("static", "Static modifier"),
            ("strictfp", "Strictfp modifier"),
            ("super", "Super keyword"),
            ("switch", "Switch statement"),
            ("synchronized", "Synchronized modifier"),
            ("this", "This keyword"),
            ("throw", "Throw statement"),
            ("throws", "Throws clause"),
            ("transient", "Transient modifier"),
            ("try", "Try block"),
            ("void", "Void type"),
            ("volatile", "Volatile modifier"),
            ("while", "While loop"),
        ];

        for (keyword, description) in java_keywords {
            let item = CompletionItem::new(
                keyword.to_string(),
                CompletionItemKind::Keyword,
                keyword.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.6);
            completions.push(item);
        }

        // Add Java-specific snippets
        let java_snippets = vec![
            ("class_snippet", "public class ${1:ClassName} {\n    ${2:body}\n}", "Class declaration"),
            ("interface_snippet", "public interface ${1:InterfaceName} {\n    ${2:methods}\n}", "Interface definition"),
            ("method_snippet", "public ${1:returnType} ${2:methodName}(${3:params}) {\n    ${4:body}\n}", "Method declaration"),
            ("constructor_snippet", "public ${1:ClassName}(${2:params}) {\n    ${3:body}\n}", "Constructor"),
            ("for_snippet", "for (int ${1:i} = 0; ${1:i} < ${2:n}; ${1:i}++) {\n    ${3:body}\n}", "For loop"),
            ("for_each_snippet", "for (${1:Type} ${2:item} : ${3:collection}) {\n    ${4:body}\n}", "For-each loop"),
            ("try_catch_snippet", "try {\n    ${1:code}\n} catch (${2:Exception} e) {\n    ${3:handle}\n}", "Try-catch block"),
            ("if_snippet", "if (${1:condition}) {\n    ${2:then}\n} else {\n    ${3:else}\n}", "If-else statement"),
            ("switch_snippet", "switch (${1:expr}) {\n    case ${2:value1}:\n        ${3:body1}\n        break;\n    default:\n        ${4:default}\n}", "Switch statement"),
            ("while_snippet", "while (${1:condition}) {\n    ${2:body}\n}", "While loop"),
        ];

        for (label, template, description) in java_snippets {
            let item = create_snippet_item(label, template, description, 0.7);
            completions.push(item);
        }

        Ok(completions)
    }
}

/// Kotlin-specific completion provider
pub struct KotlinCompletionProvider;

#[async_trait]
impl crate::engine::CompletionProvider for KotlinCompletionProvider {
    fn language(&self) -> &str {
        "kotlin"
    }

    async fn generate_completions(
        &self,
        _code: &str,
        _position: Position,
        context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>> {
        let mut completions = Vec::new();

        // Add symbols from context
        for symbol in &context.available_symbols {
            let item = symbol_to_completion_item(symbol, 0.8);
            completions.push(item);
        }

        // Add Kotlin-specific keywords
        let kotlin_keywords = vec![
            ("fun", "Function declaration"),
            ("class", "Class definition"),
            ("interface", "Interface definition"),
            ("object", "Object declaration"),
            ("companion", "Companion object"),
            ("data", "Data class"),
            ("sealed", "Sealed class"),
            ("enum", "Enum definition"),
            ("val", "Immutable variable"),
            ("var", "Mutable variable"),
            ("const", "Constant declaration"),
            ("if", "Conditional"),
            ("else", "Else clause"),
            ("when", "When expression"),
            ("for", "For loop"),
            ("while", "While loop"),
            ("do", "Do-while loop"),
            ("break", "Break statement"),
            ("continue", "Continue statement"),
            ("return", "Return statement"),
            ("try", "Try block"),
            ("catch", "Catch block"),
            ("finally", "Finally block"),
            ("throw", "Throw statement"),
            ("as", "Type cast"),
            ("is", "Type check"),
            ("in", "In operator"),
            ("!in", "Not in operator"),
            ("by", "Delegation"),
            ("get", "Getter"),
            ("set", "Setter"),
            ("init", "Initializer block"),
            ("constructor", "Constructor"),
            ("private", "Private modifier"),
            ("protected", "Protected modifier"),
            ("public", "Public modifier"),
            ("internal", "Internal modifier"),
            ("abstract", "Abstract modifier"),
            ("final", "Final modifier"),
            ("open", "Open modifier"),
            ("override", "Override modifier"),
            ("suspend", "Suspend modifier"),
            ("async", "Async modifier"),
            ("await", "Await expression"),
            ("yield", "Yield expression"),
            ("lateinit", "Late initialization"),
            ("inline", "Inline modifier"),
            ("noinline", "No-inline modifier"),
            ("crossinline", "Cross-inline modifier"),
            ("reified", "Reified type parameter"),
            ("operator", "Operator overload"),
            ("infix", "Infix function"),
            ("tailrec", "Tail recursive"),
            ("external", "External declaration"),
            ("expect", "Expect declaration"),
            ("actual", "Actual declaration"),
        ];

        for (keyword, description) in kotlin_keywords {
            let item = CompletionItem::new(
                keyword.to_string(),
                CompletionItemKind::Keyword,
                keyword.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.6);
            completions.push(item);
        }

        // Add Kotlin-specific snippets
        let kotlin_snippets = vec![
            ("class_snippet", "class ${1:ClassName} {\n    ${2:body}\n}", "Class declaration"),
            ("data_class_snippet", "data class ${1:ClassName}(${2:properties})", "Data class"),
            ("fun_snippet", "fun ${1:functionName}(${2:params}): ${3:ReturnType} {\n    ${4:body}\n}", "Function declaration"),
            ("lambda_snippet", "{ ${1:params} -> ${2:body} }", "Lambda expression"),
            ("extension_snippet", "fun ${1:Type}.${2:functionName}(${3:params}): ${4:ReturnType} {\n    ${5:body}\n}", "Extension function"),
            ("for_snippet", "for (${1:item} in ${2:collection}) {\n    ${3:body}\n}", "For loop"),
            ("when_snippet", "when (${1:expr}) {\n    ${2:pattern1} -> ${3:result1}\n    ${4:pattern2} -> ${5:result2}\n    else -> ${6:default}\n}", "When expression"),
            ("try_catch_snippet", "try {\n    ${1:code}\n} catch (e: ${2:Exception}) {\n    ${3:handle}\n}", "Try-catch block"),
            ("if_snippet", "if (${1:condition}) {\n    ${2:then}\n} else {\n    ${3:else}\n}", "If-else statement"),
            ("interface_snippet", "interface ${1:InterfaceName} {\n    ${2:methods}\n}", "Interface definition"),
        ];

        for (label, template, description) in kotlin_snippets {
            let item = create_snippet_item(label, template, description, 0.7);
            completions.push(item);
        }

        Ok(completions)
    }
}

/// Dart-specific completion provider
pub struct DartCompletionProvider;

#[async_trait]
impl crate::engine::CompletionProvider for DartCompletionProvider {
    fn language(&self) -> &str {
        "dart"
    }

    async fn generate_completions(
        &self,
        _code: &str,
        _position: Position,
        context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>> {
        let mut completions = Vec::new();

        // Add symbols from context
        for symbol in &context.available_symbols {
            let item = symbol_to_completion_item(symbol, 0.8);
            completions.push(item);
        }

        // Add Dart-specific keywords
        let dart_keywords = vec![
            ("class", "Class definition"),
            ("abstract", "Abstract class"),
            ("interface", "Interface definition"),
            ("mixin", "Mixin definition"),
            ("enum", "Enum definition"),
            ("extension", "Extension definition"),
            ("typedef", "Type alias"),
            ("void", "Void type"),
            ("dynamic", "Dynamic type"),
            ("var", "Variable declaration"),
            ("final", "Final variable"),
            ("const", "Constant declaration"),
            ("late", "Late initialization"),
            ("required", "Required parameter"),
            ("covariant", "Covariant parameter"),
            ("factory", "Factory constructor"),
            ("get", "Getter"),
            ("set", "Setter"),
            ("operator", "Operator overload"),
            ("static", "Static member"),
            ("external", "External declaration"),
            ("if", "Conditional"),
            ("else", "Else clause"),
            ("for", "For loop"),
            ("while", "While loop"),
            ("do", "Do-while loop"),
            ("switch", "Switch statement"),
            ("case", "Case clause"),
            ("default", "Default clause"),
            ("break", "Break statement"),
            ("continue", "Continue statement"),
            ("return", "Return statement"),
            ("try", "Try block"),
            ("catch", "Catch block"),
            ("finally", "Finally block"),
            ("throw", "Throw statement"),
            ("rethrow", "Rethrow statement"),
            ("async", "Async function"),
            ("await", "Await expression"),
            ("yield", "Yield statement"),
            ("sync", "Sync generator"),
            ("as", "Type cast"),
            ("is", "Type check"),
            ("in", "In operator"),
            ("new", "New keyword"),
            ("this", "This keyword"),
            ("super", "Super keyword"),
            ("extends", "Extends keyword"),
            ("implements", "Implements keyword"),
            ("with", "With keyword"),
            ("on", "On keyword"),
            ("show", "Show keyword"),
            ("hide", "Hide keyword"),
            ("deferred", "Deferred import"),
            ("export", "Export statement"),
            ("import", "Import statement"),
            ("library", "Library declaration"),
            ("part", "Part declaration"),
            ("null", "Null value"),
            ("true", "True value"),
            ("false", "False value"),
        ];

        for (keyword, description) in dart_keywords {
            let item = CompletionItem::new(
                keyword.to_string(),
                CompletionItemKind::Keyword,
                keyword.to_string(),
            )
            .with_detail(description.to_string())
            .with_score(0.6);
            completions.push(item);
        }

        // Add Dart-specific snippets
        let dart_snippets = vec![
            ("class_snippet", "class ${1:ClassName} {\n    ${2:body}\n}", "Class declaration"),
            ("mixin_snippet", "mixin ${1:MixinName} {\n    ${2:methods}\n}", "Mixin definition"),
            ("abstract_class_snippet", "abstract class ${1:ClassName} {\n    ${2:abstract methods}\n}", "Abstract class"),
            ("method_snippet", "${1:returnType} ${2:methodName}(${3:params}) {\n    ${4:body}\n}", "Method declaration"),
            ("async_method_snippet", "Future<${1:Type}> ${2:methodName}(${3:params}) async {\n    ${4:body}\n}", "Async method"),
            ("stream_snippet", "Stream<${1:Type}> ${2:methodName}(${3:params}) async* {\n    ${4:yield statements}\n}", "Stream generator"),
            ("for_snippet", "for (var ${1:item} in ${2:collection}) {\n    ${3:body}\n}", "For loop"),
            ("for_each_snippet", "${1:collection}.forEach((${2:item}) {\n    ${3:body}\n});", "For-each loop"),
            ("if_snippet", "if (${1:condition}) {\n    ${2:then}\n} else {\n    ${3:else}\n}", "If-else statement"),
            ("switch_snippet", "switch (${1:expr}) {\n    case ${2:value1}:\n        ${3:body1}\n        break;\n    default:\n        ${4:default}\n}", "Switch statement"),
        ];

        for (label, template, description) in dart_snippets {
            let item = create_snippet_item(label, template, description, 0.7);
            completions.push(item);
        }

        Ok(completions)
    }
}

/// Completion provider factory for creating language-specific providers
pub struct CompletionProviderFactory;

impl CompletionProviderFactory {
    /// Create a completion provider for the given language
    pub fn create(language: crate::language::Language) -> Box<dyn crate::engine::CompletionProvider> {
        match language {
            crate::language::Language::Rust => Box::new(RustCompletionProvider),
            crate::language::Language::TypeScript => Box::new(TypeScriptCompletionProvider),
            crate::language::Language::Python => Box::new(PythonCompletionProvider),
            crate::language::Language::Go => Box::new(GoCompletionProvider),
            crate::language::Language::Java => Box::new(JavaCompletionProvider),
            crate::language::Language::Kotlin => Box::new(KotlinCompletionProvider),
            crate::language::Language::Dart => Box::new(DartCompletionProvider),
            crate::language::Language::Unknown => Box::new(GenericTextProvider),
        }
    }

    /// Create a completion provider from file path and content
    pub fn from_file(path: &std::path::Path, content: &str) -> Box<dyn crate::engine::CompletionProvider> {
        let language = crate::language::LanguageDetector::detect(path, content);
        Self::create(language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::CompletionProvider;

    #[test]
    fn test_generic_provider_language() {
        let provider = GenericTextProvider;
        assert_eq!(provider.language(), "generic");
    }

    #[test]
    fn test_rust_provider_language() {
        let provider = RustCompletionProvider;
        assert_eq!(provider.language(), "rust");
    }

    #[test]
    fn test_typescript_provider_language() {
        let provider = TypeScriptCompletionProvider;
        assert_eq!(provider.language(), "typescript");
    }

    #[test]
    fn test_python_provider_language() {
        let provider = PythonCompletionProvider;
        assert_eq!(provider.language(), "python");
    }

    #[test]
    fn test_go_provider_language() {
        let provider = GoCompletionProvider;
        assert_eq!(provider.language(), "go");
    }

    #[test]
    fn test_java_provider_language() {
        let provider = JavaCompletionProvider;
        assert_eq!(provider.language(), "java");
    }

    #[test]
    fn test_kotlin_provider_language() {
        let provider = KotlinCompletionProvider;
        assert_eq!(provider.language(), "kotlin");
    }

    #[test]
    fn test_dart_provider_language() {
        let provider = DartCompletionProvider;
        assert_eq!(provider.language(), "dart");
    }

    #[test]
    fn test_provider_factory_rust() {
        let provider = CompletionProviderFactory::create(crate::language::Language::Rust);
        assert_eq!(provider.language(), "rust");
    }

    #[test]
    fn test_provider_factory_go() {
        let provider = CompletionProviderFactory::create(crate::language::Language::Go);
        assert_eq!(provider.language(), "go");
    }

    #[test]
    fn test_provider_factory_java() {
        let provider = CompletionProviderFactory::create(crate::language::Language::Java);
        assert_eq!(provider.language(), "java");
    }

    #[test]
    fn test_provider_factory_kotlin() {
        let provider = CompletionProviderFactory::create(crate::language::Language::Kotlin);
        assert_eq!(provider.language(), "kotlin");
    }

    #[test]
    fn test_provider_factory_dart() {
        let provider = CompletionProviderFactory::create(crate::language::Language::Dart);
        assert_eq!(provider.language(), "dart");
    }

    #[test]
    fn test_provider_factory_unknown() {
        let provider = CompletionProviderFactory::create(crate::language::Language::Unknown);
        assert_eq!(provider.language(), "generic");
    }
}
