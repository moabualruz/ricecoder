//! Integration tests for LocalModelManager

use ricecoder_local_models::{LocalModelManager, LocalModelError};

#[test]
fn test_local_model_manager_creation() {
    let manager = LocalModelManager::new("http://localhost:11434".to_string());
    assert!(manager.is_ok());
}

#[test]
fn test_local_model_manager_creation_empty_url() {
    let manager = LocalModelManager::new("".to_string());
    assert!(manager.is_err());
    
    match manager {
        Err(LocalModelError::ConfigError(msg)) => {
            assert!(msg.contains("base URL is required"));
        }
        _ => panic!("Expected ConfigError"),
    }
}

#[test]
fn test_local_model_manager_default_endpoint() {
    let manager = LocalModelManager::with_default_endpoint();
    assert!(manager.is_ok());
}

#[tokio::test]
async fn test_pull_model_validation() {
    let manager = LocalModelManager::new("http://localhost:11434".to_string()).unwrap();
    
    // Empty model name should fail
    let result = manager.pull_model("").await;
    assert!(result.is_err());
    
    match result {
        Err(LocalModelError::InvalidModelName(msg)) => {
            assert!(msg.contains("cannot be empty"));
        }
        _ => panic!("Expected InvalidModelName error"),
    }
}

#[tokio::test]
async fn test_remove_model_validation() {
    let manager = LocalModelManager::new("http://localhost:11434".to_string()).unwrap();
    
    // Empty model name should fail
    let result = manager.remove_model("").await;
    assert!(result.is_err());
    
    match result {
        Err(LocalModelError::InvalidModelName(msg)) => {
            assert!(msg.contains("cannot be empty"));
        }
        _ => panic!("Expected InvalidModelName error"),
    }
}

#[tokio::test]
async fn test_update_model_validation() {
    let manager = LocalModelManager::new("http://localhost:11434".to_string()).unwrap();
    
    // Empty model name should fail
    let result = manager.update_model("").await;
    assert!(result.is_err());
    
    match result {
        Err(LocalModelError::InvalidModelName(msg)) => {
            assert!(msg.contains("cannot be empty"));
        }
        _ => panic!("Expected InvalidModelName error"),
    }
}

#[tokio::test]
async fn test_get_model_info_validation() {
    let manager = LocalModelManager::new("http://localhost:11434".to_string()).unwrap();
    
    // Empty model name should fail
    let result = manager.get_model_info("").await;
    assert!(result.is_err());
    
    match result {
        Err(LocalModelError::InvalidModelName(msg)) => {
            assert!(msg.contains("cannot be empty"));
        }
        _ => panic!("Expected InvalidModelName error"),
    }
}

#[tokio::test]
async fn test_model_exists_validation() {
    let manager = LocalModelManager::new("http://localhost:11434".to_string()).unwrap();
    
    // Empty model name should fail
    let result = manager.model_exists("").await;
    assert!(result.is_err());
}

#[test]
fn test_multiple_managers_independent() {
    let manager1 = LocalModelManager::new("http://localhost:11434".to_string()).unwrap();
    let manager2 = LocalModelManager::new("http://localhost:11434".to_string()).unwrap();
    
    // Both managers should be independent instances
    assert_eq!(manager1.base_url(), manager2.base_url());
}

#[test]
fn test_manager_with_custom_url() {
    let manager = LocalModelManager::new("http://custom-host:11434".to_string()).unwrap();
    assert_eq!(manager.base_url(), "http://custom-host:11434");
}

#[test]
fn test_manager_with_https_url() {
    let manager = LocalModelManager::new("https://secure-ollama:11434".to_string()).unwrap();
    assert_eq!(manager.base_url(), "https://secure-ollama:11434");
}
