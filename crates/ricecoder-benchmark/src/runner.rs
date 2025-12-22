//! Benchmark runner that orchestrates the evaluation

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use ricecoder_providers::ProviderManager;
use tokio::{sync::Semaphore, task};
use walkdir::WalkDir;

use crate::{
    error::BenchmarkError,
    evaluator::{Evaluator, TestResult},
    exercise::Exercise,
    results::{BenchmarkResults, ExerciseResult},
};

pub struct BenchmarkRunner {
    exercises_dir: PathBuf,
    results_dir: PathBuf,
    provider_manager: Arc<ProviderManager>,
    max_concurrent: usize,
}

impl BenchmarkRunner {
    pub fn new(
        exercises_dir: PathBuf,
        results_dir: PathBuf,
        provider_manager: Arc<ProviderManager>,
        max_concurrent: usize,
    ) -> Self {
        Self {
            exercises_dir,
            results_dir,
            provider_manager,
            max_concurrent,
        }
    }

    pub async fn run_benchmark(
        &self,
        model: &str,
        languages: Option<Vec<String>>,
        max_attempts: usize,
        num_exercises: Option<usize>,
    ) -> Result<BenchmarkResults, BenchmarkError> {
        // Load exercises
        let exercises = self.load_exercises(languages)?;
        let exercises = if let Some(limit) = num_exercises {
            exercises.into_iter().take(limit).collect()
        } else {
            exercises
        };

        let run_id = format!(
            "{}-{}",
            model.replace("/", "-"),
            chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S")
        );
        let mut results = BenchmarkResults::new(run_id.clone(), model.to_string(), exercises.len());

        // Create results directory
        let run_dir = self.results_dir.join(&run_id);
        std::fs::create_dir_all(&run_dir)?;

        // Run exercises concurrently
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut tasks = vec![];

        for exercise in exercises {
            let semaphore = semaphore.clone();
            let provider_manager = self.provider_manager.clone();
            let run_dir = run_dir.clone();
            let model = model.to_string();

            let task = task::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                Self::run_single_exercise(
                    exercise,
                    &provider_manager,
                    &model,
                    max_attempts,
                    &run_dir,
                )
                .await
            });
            tasks.push(task);
        }

        // Collect results
        for task in tasks {
            match task.await {
                Ok(Ok(exercise_result)) => {
                    results.add_exercise_result(exercise_result);
                }
                Ok(Err(e)) => {
                    eprintln!("Exercise failed: {}", e);
                }
                Err(e) => {
                    eprintln!("Task panicked: {}", e);
                }
            }
        }

        results.finalize();

        // Save results
        let results_file = run_dir.join("results.json");
        std::fs::write(&results_file, results.to_json()?)?;

        Ok(results)
    }

    async fn run_single_exercise(
        exercise: Exercise,
        provider_manager: &ProviderManager,
        model: &str,
        max_attempts: usize,
        run_dir: &Path,
    ) -> Result<ExerciseResult, BenchmarkError> {
        let exercise_dir = run_dir.join(&exercise.name);
        std::fs::create_dir_all(&exercise_dir)?;

        // Copy solution files
        for solution_file in &exercise.get_solution_files() {
            if solution_file.exists() {
                let dest = exercise_dir.join(solution_file.file_name().unwrap());
                std::fs::create_dir_all(dest.parent().unwrap())?;
                std::fs::copy(solution_file, dest)?;
            }
        }

        let mut attempts = 0;
        let mut passed = false;
        let mut test_output = String::new();
        let mut timeout = false;
        let start_time = std::time::Instant::now();

        // Try up to max_attempts
        while attempts < max_attempts && !passed {
            attempts += 1;

            // Get LLM response
            let instructions = exercise.get_full_instructions();
            let response = Self::get_llm_response(provider_manager, model, &instructions).await?;

            // Apply the response (simplified - in real aider this would parse and apply diffs)
            Self::apply_llm_response(&response, &exercise_dir, &exercise)?;

            // Run tests
            let test_result = Evaluator::run_tests(&exercise, &exercise_dir).await?;
            passed = test_result.passed;
            test_output = test_result.output.clone();
            timeout = test_result.timeout;

            if !passed && attempts < max_attempts {
                // Add error feedback for next attempt
                let error_instructions = format!(
                    "The tests failed with the following output:\n\n{}\n\nPlease fix the implementation.",
                    test_result.output
                );
                // In a real implementation, we'd append this to instructions
            }
        }

        let duration = start_time.elapsed().as_secs_f64();

        // For now, simplified metrics
        let cost = 0.0; // Would need to track from provider
        let error_output = test_output.contains("error") || test_output.contains("Error");
        let malformed_response = false; // Would need to detect
        let syntax_errors = test_output
            .lines()
            .filter(|line| line.contains("SyntaxError") || line.contains("syntax error"))
            .count();

        Ok(ExerciseResult {
            exercise_name: exercise.name,
            language: exercise.language,
            passed,
            attempts,
            cost,
            duration,
            error_output,
            malformed_response,
            syntax_errors,
            timeout,
            test_output,
        })
    }

    async fn get_llm_response(
        _provider_manager: &ProviderManager,
        _model: &str,
        instructions: &str,
    ) -> Result<String, BenchmarkError> {
        // TODO: Implement proper LLM integration with ricecoder-providers
        // For now, return a placeholder response
        Ok(format!(
            "// TODO: Implement based on instructions:\n{}",
            instructions
        ))
    }

    fn apply_llm_response(
        response: &str,
        exercise_dir: &Path,
        exercise: &Exercise,
    ) -> Result<(), BenchmarkError> {
        // Simplified - in real implementation, this would parse the LLM response
        // and apply code changes to the appropriate files
        // For now, just create a placeholder implementation

        for solution_file in &exercise.get_solution_files() {
            if !exercise
                .get_ignore_files()
                .contains(&solution_file.to_string_lossy().to_string())
            {
                let file_path = exercise_dir.join(solution_file.file_name().unwrap());
                if !file_path.exists() {
                    // Create a basic implementation
                    let content = match exercise.language.as_str() {
                        "rust" => "fn solution() {\n    // TODO: Implement\n}\n",
                        "python" => "def solution():\n    # TODO: Implement\n    pass\n",
                        "javascript" => "function solution() {\n    // TODO: Implement\n}\n",
                        "go" => "func solution() {\n    // TODO: Implement\n}\n",
                        "java" => "public class Solution {\n    // TODO: Implement\n}\n",
                        "cpp" => "void solution() {\n    // TODO: Implement\n}\n",
                        _ => "// TODO: Implement\n",
                    };
                    std::fs::write(&file_path, content)?;
                }
            }
        }

        Ok(())
    }

    fn load_exercises(
        &self,
        languages: Option<Vec<String>>,
    ) -> Result<Vec<Exercise>, BenchmarkError> {
        let mut exercises = vec![];

        for entry in WalkDir::new(&self.exercises_dir) {
            let entry = entry?;
            if entry.file_type().is_dir() {
                let path = entry.path();
                if path.join(".meta/config.json").exists() {
                    let exercise = Exercise::load_from_path(path)?;
                    if let Some(ref langs) = languages {
                        if langs.contains(&exercise.language) {
                            exercises.push(exercise);
                        }
                    } else {
                        exercises.push(exercise);
                    }
                }
            }
        }

        Ok(exercises)
    }
}
