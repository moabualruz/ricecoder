//! HTTP client configuration

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// HTTP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Request timeout
    #[serde(default = "default_timeout")]
    pub timeout: Duration,

    /// Connection timeout
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: Duration,

    /// Maximum retry attempts
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,

    /// Initial retry delay (exponential backoff)
    #[serde(default = "default_retry_delay")]
    pub retry_delay: Duration,

    /// HTTP/HTTPS proxy URL
    #[serde(default)]
    pub proxy: Option<String>,

    /// Custom user agent
    #[serde(default = "default_user_agent")]
    pub user_agent: String,

    /// Maximum redirects to follow (0 = no redirects)
    #[serde(default = "default_max_redirects")]
    pub max_redirects: usize,

    /// Enable connection pooling
    #[serde(default = "default_pool_enabled")]
    pub pool_enabled: bool,

    /// Pool idle timeout
    #[serde(default = "default_pool_idle_timeout")]
    pub pool_idle_timeout: Duration,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            connect_timeout: default_connect_timeout(),
            retry_count: default_retry_count(),
            retry_delay: default_retry_delay(),
            proxy: None,
            user_agent: default_user_agent(),
            max_redirects: default_max_redirects(),
            pool_enabled: default_pool_enabled(),
            pool_idle_timeout: default_pool_idle_timeout(),
        }
    }
}

impl HttpConfig {
    /// Create a new HTTP config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create config for fast operations (5s timeout, no retries)
    pub fn fast() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            connect_timeout: Duration::from_secs(2),
            retry_count: 0,
            ..Default::default()
        }
    }

    /// Create config for long operations (60s timeout, 3 retries)
    pub fn long() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            connect_timeout: Duration::from_secs(10),
            retry_count: 3,
            retry_delay: Duration::from_secs(2),
            ..Default::default()
        }
    }

    /// Create config for AI provider operations (30s timeout, 3 retries)
    pub fn ai_provider() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(5),
            retry_count: 3,
            retry_delay: Duration::from_millis(500),
            user_agent: format!("RiceCoder/{}", env!("CARGO_PKG_VERSION")),
            ..Default::default()
        }
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set retry count
    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    /// Set proxy URL
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }
}

// Default value functions for serde
fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_connect_timeout() -> Duration {
    Duration::from_secs(10)
}

fn default_retry_count() -> u32 {
    3
}

fn default_retry_delay() -> Duration {
    Duration::from_millis(500)
}

fn default_user_agent() -> String {
    format!("RiceCoder/{}", env!("CARGO_PKG_VERSION"))
}

fn default_max_redirects() -> usize {
    10
}

fn default_pool_enabled() -> bool {
    true
}

fn default_pool_idle_timeout() -> Duration {
    Duration::from_secs(90)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HttpConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.retry_count, 3);
        assert!(config.pool_enabled);
    }

    #[test]
    fn test_fast_config() {
        let config = HttpConfig::fast();
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.retry_count, 0);
    }

    #[test]
    fn test_long_config() {
        let config = HttpConfig::long();
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.retry_count, 3);
    }

    #[test]
    fn test_builder_pattern() {
        let config = HttpConfig::new()
            .with_timeout(Duration::from_secs(15))
            .with_retry_count(5)
            .with_proxy("http://proxy.example.com:8080");

        assert_eq!(config.timeout, Duration::from_secs(15));
        assert_eq!(config.retry_count, 5);
        assert_eq!(config.proxy, Some("http://proxy.example.com:8080".to_string()));
    }
}
