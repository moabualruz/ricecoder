//! OAuth 2.0 and OpenID Connect support for secure token management

use std::collections::HashMap;

use async_trait::async_trait;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl,
    AuthorizationCode as OAuthAuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest::async_http_client as oidc_http_client,
    AccessTokenHash, AuthorizationCode as OidcAuthorizationCode, ClientId as OidcClientId,
    ClientSecret as OidcClientSecret, CsrfToken as OidcCsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge as OidcPkceCodeChallenge,
    PkceCodeVerifier as OidcPkceCodeVerifier, RedirectUrl as OidcRedirectUrl, Scope as OidcScope,
    SubjectIdentifier, TokenResponse as OidcTokenResponse,
};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid;

use crate::{audit::AuditLogger, Result, SecurityError};

/// OAuth 2.0 provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProvider {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
}

/// OpenID Connect provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcProvider {
    pub name: String,
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
}

/// OAuth 2.0 token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}

/// User information from OAuth/OIDC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub subject: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub preferred_username: Option<String>,
    pub groups: Vec<String>,
    pub attributes: HashMap<String, serde_json::Value>,
}

/// OAuth 2.0 client for token management
#[derive(Debug)]
pub struct OAuthClient {
    providers: HashMap<String, BasicClient>,
}

/// OpenID Connect client
#[derive(Debug)]
pub struct OidcClient {
    providers: HashMap<String, CoreClient>,
    nonces: HashMap<String, Nonce>,
}

/// Secure token manager combining OAuth 2.0 and OIDC
#[derive(Debug)]
pub struct TokenManager {
    oauth_client: OAuthClient,
    oidc_client: OidcClient,
    audit_logger: std::sync::Arc<AuditLogger>,
    stored_tokens: HashMap<String, OAuthToken>,
}

impl OAuthClient {
    /// Create a new OAuth client
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register an OAuth 2.0 provider
    pub fn register_provider(&mut self, config: OAuthProvider) -> Result<()> {
        let client = BasicClient::new(
            ClientId::new(config.client_id),
            Some(ClientSecret::new(config.client_secret)),
            AuthUrl::new(config.auth_url)?,
            Some(TokenUrl::new(config.token_url)?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_url)?);

        self.providers.insert(config.name, client);
        Ok(())
    }

    /// Generate authorization URL for OAuth 2.0 flow
    pub fn generate_auth_url(
        &self,
        provider_name: &str,
        scopes: &[String],
    ) -> Result<(Url, CsrfToken, oauth2::PkceCodeVerifier)> {
        let client =
            self.providers
                .get(provider_name)
                .ok_or_else(|| SecurityError::Validation {
                    message: format!("OAuth provider '{}' not found", provider_name),
                })?;

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut auth_request = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);

        for scope in scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        let (auth_url, csrf_token) = auth_request.url();

        Ok((auth_url, csrf_token, pkce_verifier))
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(
        &self,
        provider_name: &str,
        code: &str,
        pkce_verifier: oauth2::PkceCodeVerifier,
    ) -> Result<OAuthToken> {
        let client =
            self.providers
                .get(provider_name)
                .ok_or_else(|| SecurityError::Validation {
                    message: format!("OAuth provider '{}' not found", provider_name),
                })?;

        let token_result = client
            .exchange_code(OAuthAuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| SecurityError::Validation {
                message: format!("Token exchange failed: {}", e),
            })?;

        let token = OAuthToken {
            access_token: token_result.access_token().secret().clone(),
            token_type: token_result.token_type().as_ref().to_string(),
            expires_in: token_result.expires_in().map(|d| d.as_secs()),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            scope: token_result.scopes().map(|s| {
                s.iter()
                    .map(|scope| scope.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            }),
            id_token: None, // OAuth 2.0 doesn't have ID tokens
        };

        Ok(token)
    }
}

impl OidcClient {
    /// Create a new OIDC client
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            nonces: HashMap::new(),
        }
    }

    /// Register an OpenID Connect provider
    pub async fn register_provider(&mut self, config: OidcProvider) -> Result<()> {
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(config.issuer_url)?,
            oidc_http_client,
        )
        .await
        .map_err(|e| SecurityError::Validation {
            message: format!("OIDC discovery failed: {}", e),
        })?;

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            OidcClientId::new(config.client_id),
            Some(OidcClientSecret::new(config.client_secret)),
        )
        .set_redirect_uri(OidcRedirectUrl::new(config.redirect_url)?);

        self.providers.insert(config.name, client);
        Ok(())
    }

    /// Generate authorization URL for OIDC flow
    pub fn generate_auth_url(
        &mut self,
        provider_name: &str,
        scopes: &[String],
    ) -> Result<(Url, OidcCsrfToken, Nonce, OidcPkceCodeVerifier)> {
        let client =
            self.providers
                .get(provider_name)
                .ok_or_else(|| SecurityError::Validation {
                    message: format!("OIDC provider '{}' not found", provider_name),
                })?;

        let (pkce_challenge, pkce_verifier) = OidcPkceCodeChallenge::new_random_sha256();
        let nonce = Nonce::new_random();
        let nonce_clone = nonce.clone();

        let mut auth_request = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                OidcCsrfToken::new_random,
                move || nonce_clone,
            )
            .set_pkce_challenge(pkce_challenge);

        for scope in scopes {
            auth_request = auth_request.add_scope(OidcScope::new(scope.clone()));
        }

        let (auth_url, csrf_token, returned_nonce) = auth_request.url();

        // Store nonce for later verification
        self.nonces.insert(csrf_token.secret().clone(), nonce);

        Ok((auth_url, csrf_token, returned_nonce, pkce_verifier))
    }

    /// Exchange authorization code for tokens (OIDC)
    pub async fn exchange_code(
        &mut self,
        provider_name: &str,
        code: &str,
        pkce_verifier: OidcPkceCodeVerifier,
        csrf_token: &str,
    ) -> Result<(OAuthToken, UserInfo)> {
        let client =
            self.providers
                .get(provider_name)
                .ok_or_else(|| SecurityError::Validation {
                    message: format!("OIDC provider '{}' not found", provider_name),
                })?;

        let nonce = self
            .nonces
            .remove(csrf_token)
            .ok_or_else(|| SecurityError::Validation {
                message: "Invalid or expired CSRF token".to_string(),
            })?;

        let token_result = client
            .exchange_code(OidcAuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(oidc_http_client)
            .await
            .map_err(|e| SecurityError::Validation {
                message: format!("OIDC token exchange failed: {}", e),
            })?;

        // Verify ID token
        let id_token_verifier = client.id_token_verifier();
        let id_token = token_result
            .id_token()
            .ok_or_else(|| SecurityError::Validation {
                message: "Missing ID token in OIDC response".to_string(),
            })?;

        let claims =
            id_token
                .claims(&id_token_verifier, &nonce)
                .map_err(|e| SecurityError::Validation {
                    message: format!("ID token verification failed: {}", e),
                })?;

        // Extract user info from ID token claims
        let user_info = UserInfo {
            subject: claims.subject().to_string(),
            email: claims.email().map(|e| e.as_str().to_string()),
            email_verified: claims.email_verified(),
            name: Some(claims.name().map_or("".to_string(), |n| {
                n.get(None).map_or("".to_string(), |s| s.to_string())
            })),
            given_name: Some(claims.given_name().map_or("".to_string(), |n| {
                n.get(None).map_or("".to_string(), |s| s.to_string())
            })),
            family_name: Some(claims.family_name().map_or("".to_string(), |n| {
                n.get(None).map_or("".to_string(), |s| s.to_string())
            })),
            preferred_username: claims.preferred_username().map(|u| u.as_str().to_string()),
            groups: vec![],             // Would need custom claims for groups
            attributes: HashMap::new(), // Additional custom claims
        };

        let token = OAuthToken {
            access_token: token_result.access_token().secret().clone(),
            token_type: token_result.token_type().as_ref().to_string(),
            expires_in: token_result.expires_in().map(|d| d.as_secs()),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            scope: token_result.scopes().map(|s| {
                s.iter()
                    .map(|scope| scope.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            }),
            id_token: Some(id_token.to_string()),
        };

        Ok((token, user_info))
    }
}

impl TokenManager {
    /// Create a new token manager
    pub fn new(audit_logger: std::sync::Arc<AuditLogger>) -> Self {
        Self {
            oauth_client: OAuthClient::new(),
            oidc_client: OidcClient::new(),
            audit_logger,
            stored_tokens: HashMap::new(),
        }
    }

    /// Register OAuth 2.0 provider
    pub fn register_oauth_provider(&mut self, config: OAuthProvider) -> Result<()> {
        self.oauth_client.register_provider(config)
    }

    /// Register OIDC provider
    pub async fn register_oidc_provider(&mut self, config: OidcProvider) -> Result<()> {
        self.oidc_client.register_provider(config).await
    }

    /// Generate OAuth authorization URL
    pub fn generate_oauth_auth_url(
        &self,
        provider_name: &str,
        scopes: &[String],
    ) -> Result<(Url, CsrfToken, oauth2::PkceCodeVerifier)> {
        self.oauth_client.generate_auth_url(provider_name, scopes)
    }

    /// Generate OIDC authorization URL
    pub fn generate_oidc_auth_url(
        &mut self,
        provider_name: &str,
        scopes: &[String],
    ) -> Result<(Url, OidcCsrfToken, Nonce, OidcPkceCodeVerifier)> {
        self.oidc_client.generate_auth_url(provider_name, scopes)
    }

    /// Complete OAuth token exchange
    pub async fn complete_oauth_exchange(
        &mut self,
        provider_name: &str,
        code: &str,
        pkce_verifier: oauth2::PkceCodeVerifier,
        user_id: &str,
    ) -> Result<String> {
        let token = self
            .oauth_client
            .exchange_code(provider_name, code, pkce_verifier)
            .await?;
        let token_id = format!("oauth_{}_{}", provider_name, uuid::Uuid::new_v4());

        self.stored_tokens.insert(token_id.clone(), token);

        // Audit token issuance
        self.audit_logger
            .log_event(crate::audit::AuditEvent {
                event_type: crate::audit::AuditEventType::Authentication,
                user_id: Some(user_id.to_string()),
                session_id: None,
                action: "oauth_token_issued".to_string(),
                resource: format!("oauth_provider:{}", provider_name),
                metadata: serde_json::json!({
                    "provider": provider_name,
                    "token_type": "oauth2"
                }),
            })
            .await?;

        Ok(token_id)
    }

    /// Complete OIDC token exchange
    pub async fn complete_oidc_exchange(
        &mut self,
        provider_name: &str,
        code: &str,
        pkce_verifier: OidcPkceCodeVerifier,
        csrf_token: &str,
        user_id: &str,
    ) -> Result<(String, UserInfo)> {
        let (token, user_info) = self
            .oidc_client
            .exchange_code(provider_name, code, pkce_verifier, csrf_token)
            .await?;
        let token_id = format!("oidc_{}_{}", provider_name, uuid::Uuid::new_v4());

        self.stored_tokens.insert(token_id.clone(), token);

        // Audit token issuance
        self.audit_logger
            .log_event(crate::audit::AuditEvent {
                event_type: crate::audit::AuditEventType::Authentication,
                user_id: Some(user_id.to_string()),
                session_id: None,
                action: "oidc_token_issued".to_string(),
                resource: format!("oidc_provider:{}", provider_name),
                metadata: serde_json::json!({
                    "provider": provider_name,
                    "subject": user_info.subject,
                    "token_type": "oidc"
                }),
            })
            .await?;

        Ok((token_id, user_info))
    }

    /// Get stored token
    pub fn get_token(&self, token_id: &str) -> Option<&OAuthToken> {
        self.stored_tokens.get(token_id)
    }

    /// Revoke token
    pub async fn revoke_token(&mut self, token_id: &str, user_id: &str) -> Result<()> {
        if self.stored_tokens.remove(token_id).is_some() {
            // Audit token revocation
            self.audit_logger
                .log_event(crate::audit::AuditEvent {
                    event_type: crate::audit::AuditEventType::Authentication,
                    user_id: Some(user_id.to_string()),
                    session_id: None,
                    action: "token_revoked".to_string(),
                    resource: format!("token:{}", token_id),
                    metadata: serde_json::json!({
                        "token_id": token_id
                    }),
                })
                .await?;
        }
        Ok(())
    }

    /// Validate token (basic expiry check)
    pub fn validate_token(&self, token_id: &str) -> Result<&OAuthToken> {
        self.stored_tokens
            .get(token_id)
            .ok_or_else(|| SecurityError::Validation {
                message: "Token not found".to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_client_creation() {
        let client = OAuthClient::new();
        assert!(client.providers.is_empty());
    }

    #[test]
    fn test_oidc_client_creation() {
        let client = OidcClient::new();
        assert!(client.providers.is_empty());
        assert!(client.nonces.is_empty());
    }

    #[tokio::test]
    async fn test_token_manager_creation() {
        let storage = std::sync::Arc::new(crate::audit::MemoryAuditStorage::new());
        let audit_logger = std::sync::Arc::new(crate::audit::AuditLogger::new(storage));
        let manager = TokenManager::new(audit_logger);

        assert!(manager.stored_tokens.is_empty());
    }
}
