# ricecoder-generation

**Purpose**: Code generation engine with template support, AI enhancement, and validation for automated code creation in RiceCoder

## Features

- **Template-Based Generation**: Flexible template system for code generation with variable substitution
- **AI-Enhanced Generation**: Integration with AI providers for intelligent code suggestions and improvements
- **Multi-Language Support**: Code generation for Rust, TypeScript, Python, Go, and other languages
- **Validation Framework**: Built-in validation to ensure generated code correctness and style compliance
- **Context-Aware Generation**: Generation that adapts to project structure and existing codebase patterns

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-generation = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_generation::{CodeGenerator, TemplateEngine};

// Create code generator
let generator = CodeGenerator::new();

// Load a template
let template = TemplateEngine::load_template("rust/struct.rs.hbs")?;

// Generate code from template
let context = serde_json::json!({
    "name": "User",
    "fields": [
        {"name": "id", "type": "u64"},
        {"name": "name", "type": "String"}
    ]
});

let generated_code = generator.generate_from_template(&template, &context)?;
println!("Generated code:\n{}", generated_code);
```

## Documentation

For more information, see the [documentation](https://docs.rs/ricecoder-generation).

## License

MIT
