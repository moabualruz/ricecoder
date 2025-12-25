# ricecoder-patterns

**Purpose**: Code pattern detection and architectural analysis for identifying design patterns, architectural styles, and coding conventions

## DDD Layer

**Infrastructure** - Provides pattern detection infrastructure for code analysis across RiceCoder.

## Overview

`ricecoder-patterns` enables detection of architectural patterns, design patterns, and coding conventions in software projects. It provides consistent pattern identification to help understand and maintain codebase structure.

## Features

- **Architectural Pattern Detection**: Identify layered, microservices, event-driven, and monolithic architectures
- **Design Pattern Detection**: Detect common design patterns (factory, observer, repository, etc.)
- **Coding Convention Analysis**: Analyze naming conventions, documentation styles, import organization
- **Pattern Stability**: Consistent pattern detection across multiple runs
- **Extensible Framework**: Easy to add new pattern detectors

## Architecture

### Core Components

| Component | Responsibility |
|-----------|----------------|
| `PatternDetector` | Main pattern detection interface |
| `ArchitecturalPatternDetector` | Detects architectural patterns |
| `CodingPatternDetector` | Detects coding conventions |

### Dependencies

- Tree-sitter for code parsing
- Pattern matching algorithms

### Integration Points
- **LSP**: Provides pattern context for code intelligence
- **Refactoring**: Informs refactoring decisions based on detected patterns
- **Agents**: Supplies architectural context to AI agents

## Usage

### Basic Usage

```rust
use ricecoder_patterns::{PatternDetector, ArchitecturalPatternDetector};

// Create detector
let detector = ArchitecturalPatternDetector::new();

// Detect patterns in codebase
let patterns = detector.detect(&project_path)?;

for pattern in patterns {
    println!("Found: {} (confidence: {})", pattern.name, pattern.confidence);
}
```

### Coding Conventions

```rust
use ricecoder_patterns::CodingPatternDetector;

let detector = CodingPatternDetector::new();
let conventions = detector.analyze(&source_code)?;

println!("Naming style: {:?}", conventions.naming_style);
println!("Import organization: {:?}", conventions.import_style);
```

## Key Types

- **`PatternDetector`**: Trait for pattern detection
- **`ArchitecturalPatternDetector`**: Architectural pattern detection
- **`CodingPatternDetector`**: Coding convention detection
- **`DetectedPattern`**: Detected pattern with confidence score

## Error Handling

```rust
use ricecoder_patterns::{PatternError, PatternResult};

match detector.detect(path) {
    Ok(patterns) => process_patterns(patterns),
    Err(PatternError::ParseError(msg)) => eprintln!("Parse error: {}", msg),
    Err(PatternError::UnsupportedLanguage(lang)) => eprintln!("Unsupported: {}", lang),
}
```

## Testing

```bash
cargo test -p ricecoder-patterns
```

## License

MIT
