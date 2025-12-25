# ricecoder-research

**Purpose**: Research and analysis utilities providing code understanding, project analysis, and intelligent code insights for RiceCoder

## DDD Layer

**Application** - Code research and analysis orchestration as an application service.

## Features

- **Code Analysis**: Deep analysis of codebases for patterns, anti-patterns, and optimization opportunities
- **Project Understanding**: Comprehensive project structure analysis and documentation generation
- **Intelligent Insights**: AI-powered insights into code quality, maintainability, and architecture
- **Research Integration**: Connection to external research and best practices databases
- **Automated Reporting**: Generated reports on code health, complexity, and improvement recommendations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-research = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_research::{CodeAnalyzer, ProjectAnalyzer};

// Analyze a codebase
let analyzer = CodeAnalyzer::new();
let analysis = analyzer.analyze_codebase("./src")?;

// Generate insights report
let insights = analyzer.generate_insights(&analysis)?;
println!("Code quality score: {}", insights.quality_score);
```

## Documentation

For more information, see the [documentation](https://docs.rs/ricecoder-research).

## License

MIT
