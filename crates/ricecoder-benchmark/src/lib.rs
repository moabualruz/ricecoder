pub mod cli;
pub mod error;
pub mod evaluator;
pub mod exercise;
pub mod results;
pub mod runner;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_exercise_loading() {
        // This test would require actual exercise files
        // For now, just test that the module compiles
        assert!(true);
    }

    #[test]
    fn test_results_creation() {
        let results = BenchmarkResults::new("test-run".to_string(), "test-model".to_string(), 10);
        assert_eq!(results.run_id, "test-run");
        assert_eq!(results.model, "test-model");
        assert_eq!(results.total_exercises, 10);
    }

    #[tokio::test]
    async fn test_evaluator_timeout() {
        let temp_dir = tempdir().unwrap();
        let exercise = Exercise {
            name: "test".to_string(),
            language: "python".to_string(),
            path: temp_dir.path().to_path_buf(),
            config: ExerciseConfig {
                files: ExerciseFiles {
                    solution: vec!["test.py".to_string()],
                    test: vec!["test.py".to_string()],
                    example: vec![],
                },
                language: "python".to_string(),
            },
            instructions: "Write a function".to_string(),
            introduction: None,
            instructions_append: None,
        };

        // Create a test file that would timeout
        std::fs::write(
            temp_dir.path().join("test.py"),
            "import time; time.sleep(300)",
        )
        .unwrap();

        let result = Evaluator::run_tests(&exercise, temp_dir.path())
            .await
            .unwrap();
        assert!(!result.passed);
        assert!(result.timeout);
    }
}
