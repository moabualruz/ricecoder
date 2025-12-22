use std::{collections::HashMap, sync::Arc, time::Instant};

use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    #[tokio::test]
    async fn api_key_rate_limit_blocked() {
        let config = AuthConfig {
            api_keys: vec!["key".into()],
            oauth_providers: Vec::new(),
            jwt_secret: None,
            rate_limit: RateLimitConfig {
                requests_per_minute: 2,
                window_seconds: 60,
                endpoint_limits: HashMap::new(),
            },
        };
        let manager = AuthManager::new(config, AuthMethod::ApiKey);
        let credentials = AuthCredentials::with_api_key("key");

        assert!(manager
            .authenticate("search", credentials.clone())
            .await
            .is_ok());
        assert!(manager
            .authenticate("search", credentials.clone())
            .await
            .is_ok());
        let blocked = manager.authenticate("search", credentials).await;
        assert!(blocked.is_err());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub api_keys: Vec<String>,
    pub oauth_providers: Vec<String>,
    pub jwt_secret: Option<String>,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: usize,
    pub window_seconds: u64,
    pub endpoint_limits: HashMap<String, usize>,
}

impl RateLimitConfig {
    pub fn limit_for(&self, endpoint: &str) -> usize {
        self.endpoint_limits
            .get(endpoint)
            .cloned()
            .unwrap_or(self.requests_per_minute)
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 120,
            window_seconds: 60,
            endpoint_limits: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthMethod {
    ApiKey,
    OAuth2,
    Jwt,
}

#[derive(Debug, Clone)]
pub struct AuthCredentials {
    pub api_key: Option<String>,
    pub oauth_token: Option<String>,
    pub jwt: Option<String>,
}

impl AuthCredentials {
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let api_key = headers
            .get("x-api-key")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string());
        Self {
            api_key,
            oauth_token: None,
            jwt: None,
        }
    }

    pub fn with_api_key(key: impl Into<String>) -> Self {
        Self {
            api_key: Some(key.into()),
            oauth_token: None,
            jwt: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub client_id: String,
    pub method: AuthMethod,
}

#[derive(Debug, Clone)]
struct RateLimitState {
    window_start: Instant,
    count: usize,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            window_start: Instant::now(),
            count: 0,
        }
    }
}

#[derive(Clone)]
pub struct AuthManager {
    config: AuthConfig,
    method: AuthMethod,
    rate_limits: Arc<Mutex<HashMap<String, RateLimitState>>>,
}

impl AuthManager {
    pub fn new(config: AuthConfig, method: AuthMethod) -> Self {
        Self {
            config,
            method,
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn authenticate(
        &self,
        endpoint: &str,
        credentials: AuthCredentials,
    ) -> Result<AuthContext, String> {
        if let Some(api_key) = credentials.api_key {
            self.authenticate_api_key(endpoint, api_key).await
        } else {
            Err("missing API key".to_string())
        }
    }

    async fn authenticate_api_key(
        &self,
        endpoint: &str,
        api_key: String,
    ) -> Result<AuthContext, String> {
        if !self.config.api_keys.contains(&api_key) {
            return Err("invalid API key".to_string());
        }

        self.enforce_rate_limit(&api_key, endpoint).await?;

        Ok(AuthContext {
            client_id: api_key,
            method: AuthMethod::ApiKey,
        })
    }

    async fn enforce_rate_limit(&self, client: &str, endpoint: &str) -> Result<(), String> {
        let limit = self.config.rate_limit.limit_for(endpoint);
        let key = format!("{client}:{endpoint}");
        let mut guard = self.rate_limits.lock().await;
        let now = Instant::now();
        let state = guard.entry(key).or_insert_with(RateLimitState::default);
        if now.duration_since(state.window_start).as_secs() >= self.config.rate_limit.window_seconds
        {
            state.window_start = now;
            state.count = 0;
        }

        state.count += 1;
        if state.count > limit {
            return Err(format!("rate limit exceeded for {endpoint}"));
        }

        Ok(())
    }
}
