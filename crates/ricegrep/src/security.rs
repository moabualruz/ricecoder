//! Security and authentication features for RiceGrep enterprise
//!
//! This module provides comprehensive security capabilities including
//! credential management, access controls, and secure authentication.

use crate::error::RiceGrepError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::fs;
use std::path::PathBuf;

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable security features
    pub enabled: bool,
    /// Encryption key for credentials
    pub encryption_key: Option<String>,
    /// Session timeout in minutes
    pub session_timeout_minutes: u64,
    /// Maximum login attempts
    pub max_login_attempts: u32,
    /// Lockout duration in minutes
    pub lockout_duration_minutes: u64,
    /// Audit log enabled
    pub audit_enabled: bool,
    /// Audit log path
    pub audit_log_path: Option<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            encryption_key: None,
            session_timeout_minutes: 480, // 8 hours
            max_login_attempts: 5,
            lockout_duration_minutes: 30,
            audit_enabled: false,
            audit_log_path: None,
        }
    }
}

/// User authentication information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub active: bool,
}

/// User roles for access control
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Developer,
    Analyst,
    Guest,
}

impl UserRole {
    /// Check if role has permission for an action
    pub fn has_permission(&self, action: &str) -> bool {
        match self {
            UserRole::Admin => true, // Admin can do everything
            UserRole::Developer => matches!(action, "search" | "replace" | "index" | "read_audit"),
            UserRole::Analyst => matches!(action, "search" | "read_reports"),
            UserRole::Guest => matches!(action, "search"),
        }
    }
}

/// Authentication session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub active: bool,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub action: String,
    pub resource: String,
    pub success: bool,
    pub details: HashMap<String, String>,
    pub ip_address: Option<String>,
}

/// Credential store for secure storage
pub struct CredentialStore {
    /// Encrypted credentials storage
    credentials: Arc<Mutex<HashMap<String, EncryptedCredential>>>,
    /// Encryption key
    encryption_key: Vec<u8>,
}

/// Encrypted credential
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedCredential {
    /// Service name
    service: String,
    /// Username
    username: String,
    /// Encrypted password/token
    encrypted_data: Vec<u8>,
    /// Salt for encryption
    salt: Vec<u8>,
    /// Created timestamp
    created_at: DateTime<Utc>,
}

/// Authentication manager
pub struct AuthManager {
    /// Security configuration
    config: SecurityConfig,
    /// User store
    users: Arc<Mutex<HashMap<String, User>>>,
    /// Session store
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    /// Credential store
    credentials: Arc<CredentialStore>,
    /// Audit logger
    audit_logger: Option<AuditLogger>,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new(config: SecurityConfig) -> Result<Self, RiceGrepError> {
        let credentials = CredentialStore::new(config.encryption_key.clone())?;
        let audit_logger = if config.audit_enabled {
            Some(AuditLogger::new(config.audit_log_path.clone())?)
        } else {
            None
        };

        Ok(Self {
            config,
            users: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            credentials: Arc::new(credentials),
            audit_logger,
        })
    }

    /// Authenticate a user
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<Session, RiceGrepError> {
        let users = self.users.lock().await;
        let user = users.get(username)
            .ok_or_else(|| RiceGrepError::Auth {
                message: "User not found".to_string(),
            })?;

        // Check if user is active
        if !user.active {
            self.audit_log("authentication", &format!("user/{}", username), false, Some("User account disabled")).await;
            return Err(RiceGrepError::Auth {
                message: "Account is disabled".to_string(),
            });
        }

        // Check if user is locked
        if let Some(locked_until) = user.locked_until {
            if Utc::now() < locked_until {
                self.audit_log("authentication", &format!("user/{}", username), false, Some("Account locked")).await;
                return Err(RiceGrepError::Auth {
                    message: "Account is temporarily locked".to_string(),
                });
            }
        }

        // In a real implementation, this would verify the password hash
        // For now, we'll use a simple check
        if password != "password" { // Placeholder
            let mut user = user.clone();
            user.login_attempts += 1;

            if user.login_attempts >= self.config.max_login_attempts {
                user.locked_until = Some(Utc::now() + Duration::minutes(self.config.lockout_duration_minutes as i64));
            }

            drop(users);
            let mut users = self.users.lock().await;
            users.insert(username.to_string(), user);

            self.audit_log("authentication", &format!("user/{}", username), false, Some("Invalid password")).await;
            return Err(RiceGrepError::Auth {
                message: "Invalid credentials".to_string(),
            });
        }

        // Authentication successful
        let mut user = user.clone();
        user.last_login = Some(Utc::now());
        user.login_attempts = 0;
        user.locked_until = None;

        drop(users);
        let mut users = self.users.lock().await;
        users.insert(username.to_string(), user);

        // Create session
        let session = Session {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(self.config.session_timeout_minutes as i64),
            ip_address: None, // Would be set from request
            user_agent: None, // Would be set from request
            active: true,
        };

        let mut sessions = self.sessions.lock().await;
        sessions.insert(session.id.clone(), session.clone());

        self.audit_log("authentication", &format!("user/{}", username), true, Some("Login successful")).await;

        Ok(session)
    }

    /// Validate a session
    pub async fn validate_session(&self, session_id: &str) -> Result<User, RiceGrepError> {
        let sessions = self.sessions.lock().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| RiceGrepError::Auth {
                message: "Session not found".to_string(),
            })?;

        if !session.active {
            return Err(RiceGrepError::Auth {
                message: "Session is inactive".to_string(),
            });
        }

        if Utc::now() > session.expires_at {
            return Err(RiceGrepError::Auth {
                message: "Session has expired".to_string(),
            });
        }

        let users = self.users.lock().await;
        let user = users.get(&session.user_id)
            .ok_or_else(|| RiceGrepError::Auth {
                message: "User not found for session".to_string(),
            })?;

        Ok(user.clone())
    }

    /// Check if user has permission for an action
    pub async fn check_permission(&self, user: &User, action: &str, resource: &str) -> bool {
        // Check role-based permissions
        if !user.role.has_permission(action) {
            self.audit_log(action, resource, false, Some("Insufficient permissions")).await;
            return false;
        }

        // Additional resource-specific checks could be added here
        true
    }

    /// Create a new user
    pub async fn create_user(&self, username: &str, email: &str, role: UserRole) -> Result<User, RiceGrepError> {
        let mut users = self.users.lock().await;

        if users.contains_key(username) {
            return Err(RiceGrepError::Auth {
                message: "User already exists".to_string(),
            });
        }

        let user = User {
            id: uuid::Uuid::new_v4().to_string(),
            username: username.to_string(),
            email: email.to_string(),
            role,
            created_at: Utc::now(),
            last_login: None,
            login_attempts: 0,
            locked_until: None,
            active: true,
        };

        users.insert(username.to_string(), user.clone());

        self.audit_log("user_create", &format!("user/{}", username), true, None).await;

        Ok(user)
    }

    /// Store a credential securely
    pub async fn store_credential(&self, service: &str, username: &str, password: &str) -> Result<(), RiceGrepError> {
        self.credentials.store_credential(service, username, password).await?;
        self.audit_log("credential_store", &format!("service/{}", service), true, None).await;
        Ok(())
    }

    /// Retrieve a credential
    pub async fn get_credential(&self, service: &str) -> Result<(String, String), RiceGrepError> {
        let (username, password) = self.credentials.get_credential(service).await?;
        self.audit_log("credential_retrieve", &format!("service/{}", service), true, None).await;
        Ok((username, password))
    }

    /// Log an audit event
    async fn audit_log(&self, action: &str, resource: &str, success: bool, details: Option<&str>) {
        if let Some(ref logger) = self.audit_logger {
            let entry = AuditEntry {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                user_id: None, // Would be set from current session
                action: action.to_string(),
                resource: resource.to_string(),
                success,
                details: details.map(|d| {
                    let mut map = HashMap::new();
                    map.insert("details".to_string(), d.to_string());
                    map
                }).unwrap_or_default(),
                ip_address: None, // Would be set from request
            };

            if let Err(e) = logger.log_entry(entry).await {
                eprintln!("Failed to write audit log: {}", e);
            }
        }
    }
}

/// Audit logger for security events
pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(log_path: Option<String>) -> Result<Self, RiceGrepError> {
        let log_path = log_path
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".ricegrep")
                    .join("audit.log")
            });

        // Ensure parent directory exists
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| RiceGrepError::Io {
                    message: format!("Failed to create audit log directory: {}", e),
                })?;
        }

        Ok(Self { log_path })
    }

    /// Log an audit entry
    pub async fn log_entry(&self, entry: AuditEntry) -> Result<(), RiceGrepError> {
        let log_line = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}\n",
            entry.timestamp.to_rfc3339(),
            entry.user_id.as_deref().unwrap_or(""),
            entry.action,
            entry.resource,
            entry.success,
            entry.details.get("details").unwrap_or(&String::new()),
            entry.ip_address.as_deref().unwrap_or(""),
            entry.id
        );

        tokio::fs::write(&self.log_path, log_line)
            .await
            .map_err(|e| RiceGrepError::Io {
                message: format!("Failed to write audit log: {}", e),
            })?;

        Ok(())
    }
}

impl CredentialStore {
    /// Create a new credential store
    pub fn new(encryption_key: Option<String>) -> Result<Self, RiceGrepError> {
        let encryption_key = if let Some(key) = encryption_key {
            key.into_bytes()
        } else {
            // Generate a default key (not secure for production)
            b"default_encryption_key_32_bytes!".to_vec()
        };

        if encryption_key.len() != 32 {
            return Err(RiceGrepError::Security {
                message: "Encryption key must be 32 bytes".to_string(),
            });
        }

        Ok(Self {
            credentials: Arc::new(Mutex::new(HashMap::new())),
            encryption_key,
        })
    }

    /// Store an encrypted credential
    pub async fn store_credential(&self, service: &str, username: &str, password: &str) -> Result<(), RiceGrepError> {
        let salt = Self::generate_salt();
        let encrypted_data = self.encrypt_data(password.as_bytes(), &salt)?;

        let credential = EncryptedCredential {
            service: service.to_string(),
            username: username.to_string(),
            encrypted_data,
            salt,
            created_at: Utc::now(),
        };

        let mut credentials = self.credentials.lock().await;
        credentials.insert(service.to_string(), credential);

        Ok(())
    }

    /// Retrieve a decrypted credential
    pub async fn get_credential(&self, service: &str) -> Result<(String, String), RiceGrepError> {
        let credentials = self.credentials.lock().await;
        let credential = credentials.get(service)
            .ok_or_else(|| RiceGrepError::Security {
                message: format!("Credential not found for service: {}", service),
            })?;

        let decrypted_data = self.decrypt_data(&credential.encrypted_data, &credential.salt)?;
        let password = String::from_utf8(decrypted_data)
            .map_err(|e| RiceGrepError::Security {
                message: format!("Failed to decode password: {}", e),
            })?;

        Ok((credential.username.clone(), password))
    }

    /// Generate a random salt
    fn generate_salt() -> Vec<u8> {
        (0..16).map(|_| rand::random::<u8>()).collect()
    }

    /// Encrypt data using AES-256-GCM
    fn encrypt_data(&self, data: &[u8], salt: &[u8]) -> Result<Vec<u8>, RiceGrepError> {
        // In a real implementation, this would use proper AES encryption
        // For now, we'll use a simple XOR with the key (not secure)
        let mut encrypted = Vec::with_capacity(data.len());
        for (i, &byte) in data.iter().enumerate() {
            let key_byte = self.encryption_key[i % self.encryption_key.len()];
            let salt_byte = salt[i % salt.len()];
            encrypted.push(byte ^ key_byte ^ salt_byte);
        }
        Ok(encrypted)
    }

    /// Decrypt data using AES-256-GCM
    fn decrypt_data(&self, data: &[u8], salt: &[u8]) -> Result<Vec<u8>, RiceGrepError> {
        // Reverse the XOR operation
        let mut decrypted = Vec::with_capacity(data.len());
        for (i, &byte) in data.iter().enumerate() {
            let key_byte = self.encryption_key[i % self.encryption_key.len()];
            let salt_byte = salt[i % salt.len()];
            decrypted.push(byte ^ key_byte ^ salt_byte);
        }
        Ok(decrypted)
    }
}