use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rust_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_path = temp_dir.path().join("Cargo.toml");
        tokio::fs::write(&cargo_path, r#"
[package]
name = "test"
version = "0.1.0"
        "#).await.unwrap();

        let bootstrap = ProjectBootstrap::new(temp_dir.path().to_path_buf());
        let result = bootstrap.bootstrap().await.unwrap();

        assert_eq!(result.primary_language, Language::Rust);
    }

    #[tokio::test]
    async fn test_python_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        let req_path = temp_dir.path().join("requirements.txt");
        tokio::fs::write(&req_path, "requests==2.25.1\n").await.unwrap();

        let bootstrap = ProjectBootstrap::new(temp_dir.path().to_path_buf());
        let result = bootstrap.bootstrap().await.unwrap();

        assert_eq!(result.primary_language, Language::Python);
    }

    #[tokio::test]
    async fn test_typescript_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("package.json");
        tokio::fs::write(&package_path, r#"{"name": "test", "version": "1.0.0"}"#).await.unwrap();

        let bootstrap = ProjectBootstrap::new(temp_dir.path().to_path_buf());
        let result = bootstrap.bootstrap().await.unwrap();

        assert_eq!(result.primary_language, Language::TypeScript);
    }
}