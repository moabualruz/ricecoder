//! Built-in language configurations for refactoring
//!
//! This module provides built-in refactoring configurations that are embedded
//! in the ricecoder-storage crate and available as fallback when no
//! user or project configurations are found.

/// Get all built-in refactoring configurations
pub fn get_builtin_refactoring_configs() -> Vec<(&'static str, &'static str)> {
    vec![
        ("rust", include_str!("rust.yaml")),
        ("typescript", include_str!("typescript.yaml")),
        ("python", include_str!("python.yaml")),
    ]
}

/// Get a specific built-in refactoring configuration
pub fn get_refactoring_config(language: &str) -> Option<&'static str> {
    match language {
        "rust" => Some(include_str!("rust.yaml")),
        "typescript" | "ts" | "tsx" => Some(include_str!("typescript.yaml")),
        "python" | "py" => Some(include_str!("python.yaml")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_builtin_refactoring_configs() {
        let configs = get_builtin_refactoring_configs();
        assert_eq!(configs.len(), 3);
        assert!(configs.iter().any(|(lang, _)| *lang == "rust"));
        assert!(configs.iter().any(|(lang, _)| *lang == "typescript"));
        assert!(configs.iter().any(|(lang, _)| *lang == "python"));
    }

    #[test]
    fn test_get_refactoring_config_rust() {
        let config = get_refactoring_config("rust");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_refactoring_config_typescript() {
        let config = get_refactoring_config("typescript");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_refactoring_config_python() {
        let config = get_refactoring_config("python");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_refactoring_config_aliases() {
        assert!(get_refactoring_config("ts").is_some());
        assert!(get_refactoring_config("tsx").is_some());
        assert!(get_refactoring_config("py").is_some());
    }

    #[test]
    fn test_get_refactoring_config_unknown() {
        let config = get_refactoring_config("unknown");
        assert!(config.is_none());
    }
}
