# ricecoder-images

**Purpose**: Centralized image handling with format validation, AI analysis, smart caching, and terminal display for RiceCoder

## Overview

`ricecoder-images` provides comprehensive image processing and management capabilities for RiceCoder, enabling drag-and-drop image support with AI-powered analysis, intelligent caching, and beautiful terminal display. It supports multiple image formats with automatic optimization and provides seamless integration with AI providers for image understanding.

## Features

- **Multi-Format Support**: PNG, JPG, GIF, WebP with automatic format detection
- **AI-Powered Analysis**: Integration with OpenAI, Anthropic, and other vision-capable providers
- **Smart Caching**: LRU-based caching with configurable TTL and size limits
- **Terminal Display**: Beautiful image rendering with ASCII fallback for unsupported terminals
- **Drag-and-Drop Support**: Seamless image inclusion in chat sessions
- **Automatic Optimization**: Large image compression and size optimization
- **Session Integration**: Persistent image storage and management in conversations
- **Audit Logging**: Comprehensive logging of image operations and analysis
- **Token Counting**: Accurate token calculation for image-based requests

## Architecture

### Responsibilities
- Image format validation and metadata extraction
- AI provider integration for image analysis
- Cache management with intelligent eviction policies
- Terminal display rendering with fallback support
- Session-based image storage and retrieval
- Audit logging and compliance tracking
- Token counting for billing and rate limiting

### Dependencies
- **Image Processing**: `image` crate for format handling
- **HTTP Client**: `reqwest` for provider API calls
- **Async Runtime**: `tokio` for concurrent operations
- **Caching**: Custom LRU implementation
- **Storage**: `ricecoder-storage` for persistence
- **Sessions**: `ricecoder-sessions` for session integration

### Integration Points
- **TUI**: Drag-and-drop interface and terminal display
- **Providers**: AI analysis integration with vision-capable providers
- **Sessions**: Image storage and retrieval in conversation context
- **Storage**: Cache persistence and metadata storage
- **Commands**: Image processing and analysis commands

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-images = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_images::{ImageHandler, ImageConfig};

// Create image handler
let config = ImageConfig::default();
let handler = ImageHandler::new(config).await?;

// Process image from file
let result = handler.process_image_from_path("screenshot.png").await?;
println!("Analysis: {}", result.analysis);
```

### AI Analysis Integration

```rust
use ricecoder_images::analyzer::ImageAnalyzer;

// Create analyzer with provider integration
let analyzer = ImageAnalyzer::new(provider_config);

// Analyze image with AI
let analysis = analyzer.analyze_image(image_data, "What's in this image?").await?;
println!("Description: {}", analysis.description);
println!("Objects: {:?}", analysis.objects);
```

### Terminal Display

```rust
use ricecoder_images::display::ImageDisplay;

// Create display handler
let display = ImageDisplay::new(terminal_config);

// Display image in terminal
display.display_image(&image_data, DisplayConfig::default()).await?;
```

### Session Integration

```rust
use ricecoder_images::session_manager::SessionImageManager;

// Create session image manager
let session_manager = SessionImageManager::new(session_id);

// Add image to session
let image_id = session_manager.add_image(image_data, metadata).await?;

// Retrieve image from session
let stored_image = session_manager.get_image(&image_id).await?;
```

## Configuration

Image handling configuration via YAML:

```yaml
images:
  # Supported formats
  formats:
    - png
    - jpg
    - jpeg
    - gif
    - webp

  # Display settings
  display:
    max_width: 80
    max_height: 30
    placeholder_char: "█"
    color_depth: "truecolor"

  # Cache settings
  cache:
    enabled: true
    max_size_mb: 100
    ttl_seconds: 86400  # 24 hours
    lru_eviction: true

  # Analysis settings
  analysis:
    timeout_seconds: 30
    max_image_size_mb: 10
    optimize_large_images: true
    retry_attempts: 3

  # Provider integration
  providers:
    openai:
      enabled: true
      model: "gpt-4-vision-preview"
    anthropic:
      enabled: true
      model: "claude-3-sonnet-20240229"
```

## API Reference

### Key Types

- **`ImageHandler`**: Main image processing coordinator
- **`ImageAnalyzer`**: AI-powered image analysis
- **`ImageCache`**: Smart caching with LRU eviction
- **`ImageDisplay`**: Terminal display rendering
- **`SessionImageManager`**: Session-based image management

### Key Functions

- **`process_image()`**: Complete image processing pipeline
- **`analyze_image()`**: AI-powered image analysis
- **`display_image()`**: Terminal display with fallbacks
- **`cache_image()`**: Intelligent caching with TTL
- **`add_to_session()`**: Session integration for persistence

## Error Handling

```rust
use ricecoder_images::ImageError;

match handler.process_image(image_data).await {
    Ok(result) => println!("Image processed: {}", result.metadata.format),
    Err(ImageError::UnsupportedFormat) => eprintln!("Unsupported image format"),
    Err(ImageError::AnalysisFailed(msg)) => eprintln!("Analysis failed: {}", msg),
    Err(ImageError::CacheFull) => eprintln!("Image cache is full"),
}
```

## Testing

Run comprehensive image processing tests:

```bash
# Run all tests
cargo test -p ricecoder-images

# Run property tests for image handling
cargo test -p ricecoder-images property

# Test AI analysis integration
cargo test -p ricecoder-images analysis

# Test caching behavior
cargo test -p ricecoder-images cache
```

Key test areas:
- Image format validation accuracy
- AI analysis result correctness
- Cache eviction and persistence
- Terminal display rendering
- Session integration reliability

## Performance

- **Format Validation**: < 10ms for supported formats
- **Image Processing**: < 100ms for typical images (< 5MB)
- **AI Analysis**: Variable based on provider (2-30 seconds)
- **Cache Lookup**: < 5ms for cached images
- **Terminal Display**: < 200ms for rendering
- **Session Storage**: < 50ms for image persistence

## Contributing

When working with `ricecoder-images`:

1. **Format Safety**: Validate all image formats before processing
2. **Privacy**: Handle image data securely and respect user privacy
3. **Performance**: Optimize for common image sizes and use cases
4. **Fallbacks**: Provide graceful degradation when features unavailable
5. **Testing**: Test with various image formats and edge cases

## DDD Layer

**Layer**: Presentation (User Interface) / Infrastructure (External Integration)

### Responsibilities
- Image format validation and metadata extraction
- AI provider integration for image analysis
- Cache management with LRU eviction
- Terminal display rendering with fallbacks
- Session-based image storage
- Token counting for billing

### SOLID Analysis
- **SRP**: Separate handlers for processing, analysis, display, caching ✅
- **OCP**: Extensible via custom analyzers and display renderers ✅
- **LSP**: ImageHandler abstraction supports different image sources ✅
- **ISP**: Separate interfaces for Handler, Analyzer, Cache, Display ✅
- **DIP**: Depends on provider abstraction for AI analysis ✅

### Integration Points
| Dependency | Direction | Purpose |
|------------|-----------|---------|
| ricecoder-storage | Inbound | Cache persistence |
| ricecoder-sessions | Inbound | Session image management |
| ricecoder-providers | Inbound | AI vision analysis |
| ricecoder-tui | Outbound | Drag-and-drop and display |
| image | External | Format handling |

## License

MIT