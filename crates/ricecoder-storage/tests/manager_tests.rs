use ricecoder_storage::*;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_global_path_with_env_override() {
        // Set RICECODER_HOME environment variable
        std::env::set_var("RICECODER_HOME", "/tmp/ricecoder-test");
        let path = PathResolver::resolve_global_path().expect("Should resolve path");
        assert_eq!(path, PathBuf::from("/tmp/ricecoder-test"));
        std::env::remove_var("RICECODER_HOME");
    }

    #[test]
    fn test_resolve_global_path_without_env() {
        // Ensure RICECODER_HOME is not set
        std::env::remove_var("RICECODER_HOME");
        let path = PathResolver::resolve_global_path().expect("Should resolve path");
        // Should be either Documents/.ricecoder or ~/.ricecoder
        assert!(path.to_string_lossy().contains(".ricecoder"));
    }

    #[test]
    fn test_resolve_project_path() {
        let path = PathResolver::resolve_project_path();
        assert_eq!(path, PathBuf::from(".agent"));
    }

    #[test]
    fn test_expand_home_with_tilde() {
        let path = PathBuf::from("~/.ricecoder");
        let expanded = PathResolver::expand_home(&path).expect("Should expand");
        assert!(!expanded.to_string_lossy().contains("~"));
    }

    #[test]
    fn test_expand_home_without_tilde() {
        let path = PathBuf::from("/tmp/ricecoder");
        let expanded = PathResolver::expand_home(&path).expect("Should expand");
        assert_eq!(expanded, path);
    }
}
