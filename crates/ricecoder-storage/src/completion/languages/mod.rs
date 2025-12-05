//! Built-in language configurations for code completion
//!
//! This module provides built-in language configurations that are embedded
//! in the ricecoder-storage crate and available as fallback when no
//! user or project configurations are found.
//!
//! Supported languages:
//! - Rust (rs)
//! - TypeScript (ts, tsx, js, jsx)
//! - Python (py)
//! - Go (go)
//! - Java (java)
//! - Kotlin (kt, kts)
//! - Dart (dart)

/// Get all built-in completion language configurations
pub fn get_builtin_completion_configs() -> Vec<(&'static str, &'static str)> {
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

/// Get a specific built-in completion language configuration
pub fn get_completion_config(language: &str) -> Option<&'static str> {
    match language {
        "rust" => Some(include_str!("rust.yaml")),
        "typescript" | "ts" | "tsx" | "js" | "jsx" => Some(include_str!("typescript.yaml")),
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
    fn test_get_builtin_completion_configs() {
        let configs = get_builtin_completion_configs();
        assert_eq!(configs.len(), 7);
        assert!(configs.iter().any(|(lang, _)| *lang == "rust"));
        assert!(configs.iter().any(|(lang, _)| *lang == "typescript"));
        assert!(configs.iter().any(|(lang, _)| *lang == "python"));
        assert!(configs.iter().any(|(lang, _)| *lang == "go"));
        assert!(configs.iter().any(|(lang, _)| *lang == "java"));
        assert!(configs.iter().any(|(lang, _)| *lang == "kotlin"));
        assert!(configs.iter().any(|(lang, _)| *lang == "dart"));
    }

    #[test]
    fn test_get_completion_config_rust() {
        let config = get_completion_config("rust");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_completion_config_typescript() {
        let config = get_completion_config("typescript");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_completion_config_python() {
        let config = get_completion_config("python");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_completion_config_go() {
        let config = get_completion_config("go");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_completion_config_java() {
        let config = get_completion_config("java");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_completion_config_kotlin() {
        let config = get_completion_config("kotlin");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_completion_config_dart() {
        let config = get_completion_config("dart");
        assert!(config.is_some());
        assert!(!config.unwrap().is_empty());
    }

    #[test]
    fn test_get_completion_config_aliases() {
        assert!(get_completion_config("ts").is_some());
        assert!(get_completion_config("tsx").is_some());
        assert!(get_completion_config("js").is_some());
        assert!(get_completion_config("jsx").is_some());
        assert!(get_completion_config("py").is_some());
        assert!(get_completion_config("kt").is_some());
        assert!(get_completion_config("kts").is_some());
    }

    #[test]
    fn test_get_completion_config_unknown() {
        let config = get_completion_config("unknown");
        assert!(config.is_none());
    }
}
