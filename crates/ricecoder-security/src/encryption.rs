//! API key encryption and secure storage

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm,
};
use argon2::{Argon2, Params};
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

use crate::{SecurityError, Result};

/// Encrypted data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Base64-encoded salt
    pub salt: String,
    /// Base64-encoded nonce
    pub nonce: String,
    /// Base64-encoded encrypted data
    pub ciphertext: String,
}

/// Key manager for API key encryption/decryption
pub struct KeyManager {
    master_key: [u8; 32],
}

impl KeyManager {
    /// Create a new key manager with a master password
    pub fn new(master_password: &str) -> Result<Self> {
        let master_key = Self::derive_key(master_password, b"ricecoder-master-key")?;
        Ok(Self { master_key })
    }

    /// Encrypt API key data
    pub fn encrypt_api_key(&self, api_key: &str) -> Result<EncryptedData> {
        let salt = Self::generate_salt();
        let nonce = Self::generate_nonce();

        // Derive encryption key from master key and salt
        let encryption_key = Self::derive_key_from_bytes(&self.master_key, &salt)?;

        let cipher = Aes256Gcm::new(&encryption_key.into());
        let nonce_gcm = aes_gcm::Nonce::from_slice(&nonce);
        let ciphertext = cipher
            .encrypt(nonce_gcm, api_key.as_bytes())
            .map_err(|e| SecurityError::Encryption {
                message: e.to_string(),
            })?;

        Ok(EncryptedData {
            salt: general_purpose::STANDARD.encode(salt),
            nonce: general_purpose::STANDARD.encode(nonce),
            ciphertext: general_purpose::STANDARD.encode(ciphertext),
        })
    }

    /// Decrypt API key data
    pub fn decrypt_api_key(&self, encrypted: &EncryptedData) -> Result<String> {
        let salt = general_purpose::STANDARD.decode(&encrypted.salt)?;
        let nonce_bytes = general_purpose::STANDARD.decode(&encrypted.nonce)?;
        let ciphertext = general_purpose::STANDARD.decode(&encrypted.ciphertext)?;

        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        // Derive encryption key from master key and salt
        let encryption_key = Self::derive_key_from_bytes(&self.master_key, &salt)?;

        let cipher = Aes256Gcm::new(&encryption_key.into());
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| SecurityError::Decryption {
                message: e.to_string(),
            })?;

        String::from_utf8(plaintext).map_err(Into::into)
    }

    /// Save encrypted data to file
    pub async fn save_to_file(&self, encrypted: &EncryptedData, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(encrypted)?;
        fs::write(path, json).await?;
        Ok(())
    }

    /// Load encrypted data from file
    pub async fn load_from_file(path: &Path) -> Result<EncryptedData> {
        let json = fs::read_to_string(path).await?;
        let encrypted: EncryptedData = serde_json::from_str(&json)?;
        Ok(encrypted)
    }

    /// Derive key from password using Argon2
    fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
        let mut key = [0u8; 32];
        let argon2 = Argon2::default();

        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| SecurityError::KeyDerivation {
                message: e.to_string(),
            })?;

        Ok(key)
    }

    /// Derive key from existing key bytes and salt
    fn derive_key_from_bytes(key: &[u8; 32], salt: &[u8]) -> Result<[u8; 32]> {
        let mut derived_key = [0u8; 32];
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(65536, 3, 4, None).unwrap(),
        );

        argon2
            .hash_password_into(key, salt, &mut derived_key)
            .map_err(|e| SecurityError::KeyDerivation {
                message: e.to_string(),
            })?;

        Ok(derived_key)
    }

    /// Generate random salt
    fn generate_salt() -> [u8; 32] {
        rand::thread_rng().gen()
    }

    /// Generate random nonce
    fn generate_nonce() -> [u8; 12] {
        let mut bytes = [0u8; 12];
        rand::thread_rng().fill(&mut bytes);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt_api_key() {
        let key_manager = KeyManager::new("test-password").unwrap();
        let api_key = "sk-test12345678901234567890123456789012";

        let encrypted = key_manager.encrypt_api_key(api_key).unwrap();
        let decrypted = key_manager.decrypt_api_key(&encrypted).unwrap();

        assert_eq!(api_key, decrypted);
    }

    #[tokio::test]
    async fn test_save_load_encrypted_data() {
        let key_manager = KeyManager::new("test-password").unwrap();
        let api_key = "sk-test12345678901234567890123456789012";

        let encrypted = key_manager.encrypt_api_key(api_key).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        key_manager.save_to_file(&encrypted, &path).await.unwrap();
        let loaded = KeyManager::load_from_file(&path).await.unwrap();

        let decrypted = key_manager.decrypt_api_key(&loaded).unwrap();
        assert_eq!(api_key, decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let key_manager1 = KeyManager::new("password1").unwrap();
        let key_manager2 = KeyManager::new("password2").unwrap();
        let api_key = "sk-test12345678901234567890123456789012";

        let encrypted = key_manager1.encrypt_api_key(api_key).unwrap();
        let result = key_manager2.decrypt_api_key(&encrypted);

        assert!(result.is_err());
    }
}