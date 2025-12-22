//! OAuth authentication and enterprise authentication flows

use std::collections::HashMap;

use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use url::Url;

use crate::error::{IndustryError, IndustryResult};

/// OAuth configuration for enterprise integrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// OAuth client ID
    pub client_id: String,
    /// OAuth client secret
    pub client_secret: String,
    /// Authorization URL
    pub auth_url: String,
    /// Token URL
    pub token_url: String,
    /// Redirect URL for OAuth flow
    pub redirect_url: String,
    /// Requested scopes
    pub scopes: Vec<String>,
    /// Additional parameters
    pub extra_params: HashMap<String, String>,
}

/// OAuth token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    /// Access token
    pub access_token: String,
    /// Token type (usually "Bearer")
    pub token_type: String,
    /// Refresh token (optional)
    pub refresh_token: Option<String>,
    /// Token expiration time (Unix timestamp)
    pub expires_at: Option<i64>,
    /// Granted scopes
    pub scopes: Vec<String>,
}

/// OAuth authentication flow state
#[derive(Debug)]
pub struct OAuthFlow {
    client: BasicClient,
    config: OAuthConfig,
    http_client: Client,
}

/// OAuth client for managing enterprise integrations
#[derive(Debug)]
pub struct OAuthClient {
    flows: RwLock<HashMap<String, OAuthFlow>>,
    tokens: RwLock<HashMap<String, OAuthToken>>,
}

impl OAuthConfig {
    /// Create a new OAuth configuration
    pub fn new(
        client_id: String,
        client_secret: String,
        auth_url: String,
        token_url: String,
        redirect_url: String,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            auth_url,
            token_url,
            redirect_url,
            scopes: Vec::new(),
            extra_params: HashMap::new(),
        }
    }

    /// Add scopes to the configuration
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Add extra parameters
    pub fn with_extra_params(mut self, params: HashMap<String, String>) -> Self {
        self.extra_params = params;
        self
    }
}

impl OAuthFlow {
    /// Create a new OAuth flow
    pub fn new(config: OAuthConfig) -> IndustryResult<Self> {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())?,
            Some(TokenUrl::new(config.token_url.clone())?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone())?);

        Ok(Self {
            client,
            config,
            http_client: Client::new(),
        })
    }

    /// Generate authorization URL for OAuth flow
    pub fn get_authorization_url(&self) -> (Url, CsrfToken, PkceCodeChallenge) {
        let (pkce_challenge, _pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut auth_request = self.client.authorize_url(CsrfToken::new_random);

        // Add scopes
        for scope in &self.config.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        // Add extra parameters
        for (key, value) in &self.config.extra_params {
            auth_request = auth_request.add_extra_param(key, value);
        }

        let (auth_url, csrf_token) = auth_request
            .set_pkce_challenge(pkce_challenge.clone())
            .url();

        (auth_url, csrf_token, pkce_challenge)
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(
        &self,
        code: String,
        pkce_verifier: oauth2::PkceCodeVerifier,
    ) -> IndustryResult<OAuthToken> {
        let token_result = self
            .client
            .exchange_code(oauth2::AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| IndustryError::OAuthError {
                message: format!("Token exchange failed: {}", e),
            })?;

        let token = OAuthToken {
            access_token: token_result.access_token().secret().clone(),
            token_type: "Bearer".to_string(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            expires_at: token_result
                .expires_in()
                .map(|d| chrono::Utc::now().timestamp() + d.as_secs() as i64),
            scopes: token_result
                .scopes()
                .map(|scopes| scopes.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        };

        Ok(token)
    }

    /// Refresh an expired token
    pub async fn refresh_token(&self, refresh_token: String) -> IndustryResult<OAuthToken> {
        let token_result = self
            .client
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await
            .map_err(|e| IndustryError::OAuthError {
                message: format!("Token refresh failed: {}", e),
            })?;

        let token = OAuthToken {
            access_token: token_result.access_token().secret().clone(),
            token_type: "Bearer".to_string(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            expires_at: token_result
                .expires_in()
                .map(|d| chrono::Utc::now().timestamp() + d.as_secs() as i64),
            scopes: token_result
                .scopes()
                .map(|scopes| scopes.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        };

        Ok(token)
    }
}

impl OAuthClient {
    /// Create a new OAuth client
    pub fn new() -> Self {
        Self {
            flows: RwLock::new(HashMap::new()),
            tokens: RwLock::new(HashMap::new()),
        }
    }

    /// Register an OAuth flow for a provider
    pub async fn register_flow(&self, provider: String, config: OAuthConfig) -> IndustryResult<()> {
        let flow = OAuthFlow::new(config)?;
        self.flows.write().await.insert(provider, flow);
        Ok(())
    }

    /// Get authorization URL for a provider
    pub async fn get_authorization_url(
        &self,
        provider: &str,
    ) -> IndustryResult<(Url, String, String)> {
        let flows = self.flows.read().await;
        let flow = flows
            .get(provider)
            .ok_or_else(|| IndustryError::OAuthError {
                message: format!("No OAuth flow registered for provider: {}", provider),
            })?;

        let (url, csrf_token, pkce_challenge) = flow.get_authorization_url();
        Ok((
            url,
            csrf_token.secret().clone(),
            pkce_challenge.as_str().to_string(),
        ))
    }

    /// Complete OAuth flow and store token
    pub async fn complete_flow(
        &self,
        provider: &str,
        code: String,
        pkce_verifier: String,
    ) -> IndustryResult<()> {
        let flows = self.flows.read().await;
        let flow = flows
            .get(provider)
            .ok_or_else(|| IndustryError::OAuthError {
                message: format!("No OAuth flow registered for provider: {}", provider),
            })?;

        let pkce_verifier = oauth2::PkceCodeVerifier::new(pkce_verifier);
        let token = flow.exchange_code(code, pkce_verifier).await?;

        self.tokens
            .write()
            .await
            .insert(provider.to_string(), token);
        Ok(())
    }

    /// Get stored token for a provider
    pub async fn get_token(&self, provider: &str) -> Option<OAuthToken> {
        self.tokens.read().await.get(provider).cloned()
    }

    /// Check if token is expired
    pub fn is_token_expired(token: &OAuthToken) -> bool {
        token
            .expires_at
            .map(|exp| chrono::Utc::now().timestamp() > exp)
            .unwrap_or(false)
    }

    /// Refresh token for a provider
    pub async fn refresh_token(&self, provider: &str) -> IndustryResult<()> {
        let mut tokens = self.tokens.write().await;
        let current_token = tokens
            .get(provider)
            .ok_or_else(|| IndustryError::OAuthError {
                message: format!("No token found for provider: {}", provider),
            })?;

        let refresh_token =
            current_token
                .refresh_token
                .clone()
                .ok_or_else(|| IndustryError::OAuthError {
                    message: format!("No refresh token available for provider: {}", provider),
                })?;

        let flows = self.flows.read().await;
        let flow = flows
            .get(provider)
            .ok_or_else(|| IndustryError::OAuthError {
                message: format!("No OAuth flow registered for provider: {}", provider),
            })?;

        let new_token = flow.refresh_token(refresh_token).await?;
        tokens.insert(provider.to_string(), new_token);

        Ok(())
    }
}

impl Default for OAuthClient {
    fn default() -> Self {
        Self::new()
    }
}
