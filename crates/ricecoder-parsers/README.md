# ricecoder-parsers

**Purpose**: AST parsing and syntax tree analysis for multiple programming languages

## DDD Layer

**Infrastructure** - Provides parsing infrastructure for code analysis across the RiceCoder ecosystem.

## Overview

`ricecoder-parsers` provides unified interfaces for parsing, traversing, and analyzing code across Rust, Python, TypeScript, and Go. It delivers consistent AST access and traversal utilities for all supported languages.

## Features

- **Multi-language Support**: AST parsing for Rust, Python, TypeScript, and Go
- **Unified API**: Consistent interfaces across all supported languages
- **Syntax Tree Traversal**: Powerful utilities for code analysis
- **Performance Optimized**: Caching and incremental parsing
- **Extensible Architecture**: Easy to add support for new languages

## Architecture

### Core Components

| Component | Responsibility |
|-----------|----------------|
| `CodeParser` | Main parsing interface for all languages |
| `Language` | Language detection and configuration |
| `TreeWalker` | AST traversal utilities |
| `NodeVisitor` | Visitor pattern for AST nodes |
| `SyntaxTree` | Parsed AST representation |

### Dependencies

#### External Libraries
- `tree-sitter`: Core AST parsing engine
- `tree-sitter-rust`: Rust language grammar
- `tree-sitter-python`: Python language grammar  
- `tree-sitter-typescript`: TypeScript language grammar
- `tree-sitter-go`: Go language grammar

### Integration Points
- **LSP**: Provides parsed ASTs for semantic analysis
- **Completion**: Supplies syntax context for code completion
- **Refactoring**: Enables AST-based refactoring operations
- **Diagnostics**: Supports syntax-aware diagnostic generation

## Usage

### Basic Parsing

```rust
use ricecoder_parsers::{CodeParser, Language, ParserConfig};

// Create parser
let parser = CodeParser::new(ParserConfig::default());

// Parse code
let result = parser.parse("fn main() { }", Language::Rust)?;
println!("Root node: {:?}", result.tree.root());
```

### Tree Traversal

```rust
use ricecoder_parsers::{TreeWalker, NodeVisitor, VisitorResult};

struct FunctionFinder;

impl NodeVisitor for FunctionFinder {
    fn visit(&mut self, node: &ASTNode) -> VisitorResult {
        if node.node_type == NodeType::Function {
            println!("Found function: {}", node.name);
        }
        VisitorResult::Continue
    }
}

let walker = TreeWalker::new(syntax_tree);
walker.walk(&mut FunctionFinder);
```

## Key Types

- **`CodeParser`**: Main parsing interface
- **`ParseResult`**: Result of parsing operation
- **`SyntaxTree`**: Parsed AST representation
- **`ASTNode`**: Individual AST node
- **`Language`**: Supported language enum
- **`TreeWalker`**: AST traversal utility
- **`NodeVisitor`**: Visitor trait for AST nodes
- **`Position`**: Source code position
- **`Range`**: Source code range

## Error Handling

```rust
use ricecoder_parsers::{ParserError, ParserResult};

match parser.parse(code, language) {
    Ok(result) => println!("Parsed successfully"),
    Err(ParserError::UnsupportedLanguage(lang)) => {
        eprintln!("Language {:?} not supported", lang)
    }
    Err(ParserError::ParseError(msg)) => {
        eprintln!("Parse error: {}", msg)
    }
}
```

## Performance

- **Parse Time**: < 50ms for typical files (< 1000 lines)
- **Memory**: Efficient tree representation with node pooling
- **Caching**: Incremental parsing for unchanged regions
- **Concurrent**: Thread-safe parsing operations

## Testing

```bash
# Run all tests
cargo test -p ricecoder-parsers
```

## License

MIT
