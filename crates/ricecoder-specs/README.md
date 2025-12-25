# ricecoder-specs

**Purpose**: Specification parsing and management system for processing YAML/JSON specs and managing development workflows in RiceCoder

## DDD Layer

**Application** - Specification management as an application service layer.

## Features

- **YAML/JSON Spec Parsing**: Robust parsing of specification files with validation
- **Spec Validation**: Comprehensive validation against schemas and business rules
- **Workflow Management**: Specification-driven development workflow orchestration
- **Template Processing**: Dynamic spec template processing with variable substitution
- **Integration APIs**: APIs for other crates to consume and utilize specifications

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-specs = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_specs::{SpecParser, SpecValidator};

// Parse a YAML specification
let parser = SpecParser::new();
let spec = parser.parse_yaml(include_str!("example_spec.yaml"))?;

// Validate the specification
let validator = SpecValidator::new();
validator.validate(&spec)?;
```

## Documentation

For more information, see the [documentation](https://docs.rs/ricecoder-specs).

## License

MIT
