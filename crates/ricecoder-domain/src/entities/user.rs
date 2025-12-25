//! User entity representing a system user

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User entity representing a system user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
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
}
