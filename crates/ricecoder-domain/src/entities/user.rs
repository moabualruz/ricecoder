//! User entity representing a system user

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::{DomainError, DomainResult};

/// User entity representing a system user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: String,
    username: String,
    email: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    metadata: HashMap<String, String>,
}

impl User {
    /// Create a new user
    pub fn new(id: String, username: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            username,
            email: None,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Create a user with email validation
    pub fn with_email(id: String, username: String, email: String) -> DomainResult<Self> {
        let user = Self {
            id,
            username,
            email: Some(email),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        user.validate_email()?;
        Ok(user)
    }

    /// Get user ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get username
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Get email
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    /// Get created timestamp
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get updated timestamp
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Get metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Update username
    pub fn update_username(&mut self, username: String) {
        self.username = username;
        self.updated_at = Utc::now();
    }

    /// Set email
    pub fn set_email(&mut self, email: Option<String>) {
        self.email = email;
        self.updated_at = Utc::now();
    }

    /// Update user profile
    pub fn update_profile(&mut self, email: Option<String>, metadata: HashMap<String, String>) {
        self.email = email;
        self.metadata = metadata;
        self.updated_at = Utc::now();
    }

    /// Validate email format
    pub fn validate_email(&self) -> DomainResult<()> {
        if let Some(email) = &self.email {
            if !email.contains('@') || !email.contains('.') {
                return Err(DomainError::ValidationError {
                    field: "email".to_string(),
                    reason: "Invalid email format".to_string(),
                });
            }
        }
        Ok(())
    }
}
