//! Built-in language configurations for LSP
//!
//! This module provides built-in language configurations that are embedded
//! in the ricecoder-storage crate and available as fallback when no
//! user or project configurations are found.

/// Get all built-in language configurations
pub fn get_builtin_language_configs() -> Vec<(&'static str, &'static str)> {
    vec![
        ("rust", include_str!("rust.yaml")),
        ("typescript", include_str!("typescript.yaml")),
        ("python", include_str!("python.yaml")),
        ("go", include_str!("go.yaml")),
        ("java", include_str!("java.yaml")),
        ("kotlin", include_str!("kotlin.yaml")),
        ("dart", include_str!("dart.yaml")),
    ]
}

/// Get a specific built-in language configuration
pub fn get_language_config(language: &str) -> Option<&'static str> {
    match language {
        "rust" => Some(include_str!("rust.yaml")),
        "typescript" | "ts" | "tsx" => Some(include_str!("typescript.yaml")),
        "python" | "py" => Some(include_str!("python.yaml")),
        "go" => Some(include_str!("go.yaml")),
        "java" => Some(include_str!("java.yaml")),
        "kotlin" | "kt" | "kts" => Some(include_str!("kotlin.yaml")),
        "dart" => Some(include_str!("dart.yaml")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_builtin_language_configs() {
        let configs = get_builtin_language_configs();
        assert_eq!(configs.len(), 3);
        assert!(configs.iter().any(|(lang, _)| *lang == "rust"));
        assert!(configs.iter().any(|(lang, _)| *lang == "typescript"));
        assert!(configs.iter().any(|(lang, _)| *lang == "python"));
    }

    #[test]
    fn test_get_language_config_rust() {
        let config = get_language_config("rust");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_language_config_typescript() {
        let config = get_language_config("typescript");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_language_config_python() {
        let config = get_language_config("python");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_language_config_aliases() {
        assert!(get_language_config("ts").is_some());
        assert!(get_language_config("tsx").is_some());
        assert!(get_language_config("py").is_some());
    }

    #[test]
    fn test_get_language_config_unknown() {
        let config = get_language_config("unknown");
        assert!(config.is_none());
    }
}
