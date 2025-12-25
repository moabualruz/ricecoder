# ricecoder-http

**Purpose**: Centralized, mockable HTTP client wrapper for all RiceCoder crates

## Overview

`ricecoder-http` provides a unified HTTP client with configurable timeouts, retry logic, proxy support, and connection pooling. It replaces scattered `reqwest::Client::new()` calls across 31 crates with a consistent, testable interface.

## Features

- **Trait-based Design**: Mockable via `HttpClientTrait`
- **Configurable**: Timeouts, retries, proxy, user-agent
- **Retry Middleware**: Exponential backoff with configurable attempts
- **Connection Pooling**: Managed by underlying reqwest client
- **Testing Support**: Easy mocking with wiremock
- **Proxy Support**: HTTP/HTTPS proxy configuration
- **Error Handling**: Comprehensive error types with retry classification

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-http = { path = "../ricecoder-http", version = "0.1" }
```

## Usage

### Basic Usage

```rust
use ricecoder_http::{HttpClient, HttpConfig};

// Create client with defaults
let client = HttpClient::with_defaults()?;

// Make a GET request
let response = client.get("https://api.example.com/data").await?;
let body = response.text().await?;
```

### Custom Configuration

```rust
use ricecoder_http::{HttpClient, HttpConfig};
use std::time::Duration;

// Create custom config
let config = HttpConfig::default()
    .with_timeout(Duration::from_secs(15))
    .with_retry_count(5)
    .with_proxy("http://proxy.example.com:8080")
    .with_user_agent("MyApp/1.0");

let client = HttpClient::new(config)?;
```

### Preset Configurations

```rust
// Fast operations (5s timeout, no retries)
let client = HttpClient::new(HttpConfig::fast())?;

// Long operations (60s timeout, 3 retries)
let client = HttpClient::new(HttpConfig::long())?;

// AI provider operations (30s timeout, 3 retries)
let client = HttpClient::new(HttpConfig::ai_provider())?;
```

### POST/PUT/DELETE Requests

```rust
use reqwest::Body;

// POST with JSON body
let body = Body::from(serde_json::to_vec(&data)?);
let response = client.post("https://api.example.com/create", body).await?;

// PUT request
let response = client.put("https://api.example.com/update", body).await?;

// DELETE request
let response = client.delete("https://api.example.com/resource/123").await?;
```

### Shared Client (Arc-wrapped)

```rust
use ricecoder_http::{shared_client, HttpConfig};
use std::sync::Arc;

// Create shared client for multiple consumers
let client = shared_client(HttpConfig::default())?;

// Clone and use across threads/modules
let client_clone = Arc::clone(&client);
tokio::spawn(async move {
    client_clone.get("https://example.com").await
});
```

### Testing with Mocks

```rust
use async_trait::async_trait;
use ricecoder_http::{HttpClientTrait, Result, Response};
use reqwest::{Body, Method};

// Implement mock client
struct MockHttpClient {
    responses: Vec<Response>,
}

#[async_trait]
impl HttpClientTrait for MockHttpClient {
    async fn get(&self, url: &str) -> Result<Response> {
        Ok(self.responses[0].clone())
    }

    // ... implement other methods
}

// Use in tests
let mock = MockHttpClient {
    responses: vec![/* mock responses */],
};
let result = mock.get("https://test.com").await?;
```

### Integration with wiremock

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_api_call() {
    // Start mock server
    let server = MockServer::start().await;

    // Configure mock response
    Mock::given(method("GET"))
        .and(path("/api/data"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok"})))
        .mount(&server)
        .await;

    // Create client and make request
    let client = HttpClient::with_defaults().unwrap();
    let url = format!("{}/api/data", server.uri());
    let response = client.get(&url).await.unwrap();

    assert_eq!(response.status(), 200);
}
```

## Configuration

### HttpConfig Fields

| Field | Default | Description |
|-------|---------|-------------|
| `timeout` | 30s | Overall request timeout |
| `connect_timeout` | 10s | Connection establishment timeout |
| `retry_count` | 3 | Maximum retry attempts |
| `retry_delay` | 500ms | Initial retry delay (exponential backoff) |
| `proxy` | None | HTTP/HTTPS proxy URL |
| `user_agent` | `RiceCoder/{version}` | Custom user agent string |
| `max_redirects` | 10 | Maximum redirects to follow |
| `pool_enabled` | true | Enable connection pooling |
| `pool_idle_timeout` | 90s | Pool idle connection timeout |

### Retry Behavior

Retries are automatically applied to:
- Network timeouts
- Connection errors
- 5xx server errors
- 429 Too Many Requests

Exponential backoff formula:
```
delay = initial_delay * backoff_multiplier^attempt
```

Default: 500ms, 1s, 2s (max 30s)

## Error Handling

```rust
use ricecoder_http::HttpError;

match client.get(url).await {
    Ok(response) => println!("Success: {}", response.status()),
    Err(HttpError::Timeout(duration)) => eprintln!("Timed out after {:?}", duration),
    Err(HttpError::HttpStatus { status, message }) => {
        eprintln!("HTTP {}: {}", status, message)
    }
    Err(HttpError::RetryLimitExceeded { attempts }) => {
        eprintln!("Failed after {} attempts", attempts)
    }
    Err(e) => eprintln!("Request failed: {}", e),
}
```

## Migration Guide

### Before (scattered reqwest usage)

```rust
// Each crate creates its own client
let client = reqwest::Client::new();
let response = client.get(url).send().await?;
```

### After (centralized HTTP client)

```rust
use ricecoder_http::{HttpClient, HttpConfig};

// Use shared HTTP client
let client = HttpClient::new(HttpConfig::ai_provider())?;
let response = client.get(url).await?;
```

### Benefits

- ✅ Consistent timeouts across all crates
- ✅ Centralized retry logic
- ✅ Easy proxy configuration
- ✅ Mockable for testing
- ✅ Connection pooling optimization
- ✅ Reduced code duplication

## Performance

- **Request overhead**: < 1ms (trait dispatch)
- **Retry overhead**: Exponential backoff (500ms → 2s)
- **Connection pooling**: Reuses connections within 90s idle timeout
- **Memory**: Minimal (single client instance per config)

## Contributing

When working with `ricecoder-http`:

1. **Keep trait simple**: Don't add every reqwest method
2. **Maintain testability**: Ensure HttpClientTrait is easily mockable
3. **Document config changes**: Update README when adding config options
4. **Test retry logic**: Verify exponential backoff behavior
5. **Benchmark changes**: Performance tests for client overhead

## License

MIT
