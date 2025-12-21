//! Command-line interface for the benchmark

use crate::error::BenchmarkError;
use crate::runner::BenchmarkRunner;
use clap::{Parser, Subcommand};
use ricecoder_providers::{ProviderManager, ProviderRegistry};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "ricecoder-benchmark")]
#[command(about = "Aider polyglot test suite for LLM evaluation")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the benchmark suite
    Run {
        /// Model to benchmark (e.g., "openai/gpt-4", "anthropic/claude-3")
        #[arg(short, long)]
        model: String,

        /// Languages to test (comma-separated, default: all)
        #[arg(short, long)]
        languages: Option<String>,

        /// Maximum number of attempts per exercise
        #[arg(short, long, default_value = "2")]
        max_attempts: usize,

        /// Number of exercises to run (default: all)
        #[arg(short = 'n', long)]
        num_exercises: Option<usize>,

        /// Maximum concurrent exercises
        #[arg(short, long, default_value = "10")]
        concurrent: usize,

        /// Directory containing exercises
        #[arg(short, long, default_value = "exercises")]
        exercises_dir: PathBuf,

        /// Directory to store results
        #[arg(short, long, default_value = "results")]
        results_dir: PathBuf,
    },

    /// List available exercises
    List {
        /// Languages to list (comma-separated, default: all)
        #[arg(short, long)]
        languages: Option<String>,

        /// Directory containing exercises
        #[arg(short, long, default_value = "exercises")]
        exercises_dir: PathBuf,
    },

    /// Show results summary
    Summary {
        /// Results directory
        #[arg(short, long, default_value = "results")]
        results_dir: PathBuf,

        /// Specific run ID
        #[arg(short, long)]
        run_id: Option<String>,
    },
}

pub async fn run_cli(cli: Cli) -> Result<(), BenchmarkError> {
    match cli.command {
        Commands::Run {
            model,
            languages,
            max_attempts,
            num_exercises,
            concurrent,
            exercises_dir,
            results_dir,
        } => {
            run_benchmark(
                model,
                languages,
                max_attempts,
                num_exercises,
                concurrent,
                exercises_dir,
                results_dir,
            )
            .await
        }

        Commands::List {
            languages,
            exercises_dir,
        } => list_exercises(languages, exercises_dir),

        Commands::Summary {
            results_dir,
            run_id,
        } => show_summary(results_dir, run_id),
    }
}

async fn run_benchmark(
    model: String,
    languages: Option<String>,
    max_attempts: usize,
    num_exercises: Option<usize>,
    concurrent: usize,
    exercises_dir: PathBuf,
    results_dir: PathBuf,
) -> Result<(), BenchmarkError> {
    // Initialize provider manager
    // TODO: Proper provider manager initialization with DI
    // For now, create a basic registry and manager
    let registry = ProviderRegistry::new();
    let provider_manager = Arc::new(ProviderManager::new(registry, "default".to_string()));

    let languages = languages.map(|l| l.split(',').map(|s| s.trim().to_string()).collect());

    let runner = BenchmarkRunner::new(exercises_dir, results_dir, provider_manager, concurrent);

    let results = runner
        .run_benchmark(&model, languages, max_attempts, num_exercises)
        .await?;

    println!("{}", results.summary());

    Ok(())
}

fn list_exercises(languages: Option<String>, exercises_dir: PathBuf) -> Result<(), BenchmarkError> {
    use crate::exercise::Exercise;
    use walkdir::WalkDir;

    let languages: Option<Vec<String>> =
        languages.map(|l| l.split(',').map(|s| s.trim().to_string()).collect());

    println!("Available exercises:");
    println!("====================");

    for entry in WalkDir::new(&exercises_dir) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            let path = entry.path();
            if path.join(".meta/config.json").exists() {
                let exercise = Exercise::load_from_path(path)?;
                if let Some(ref langs) = languages {
                    if langs.contains(&exercise.language) {
                        println!("{} ({})", exercise.name, exercise.language);
                    }
                } else {
                    println!("{} ({})", exercise.name, exercise.language);
                }
            }
        }
    }

    Ok(())
}

fn show_summary(results_dir: PathBuf, run_id: Option<String>) -> Result<(), BenchmarkError> {
    use crate::results::BenchmarkResults;

    let results_file = if let Some(run_id) = run_id {
        results_dir.join(run_id).join("results.json")
    } else {
        // Find latest results
        let mut latest_time = 0;
        let mut latest_file = None;

        for entry in walkdir::WalkDir::new(&results_dir) {
            let entry = entry?;
            if entry.file_name() == "results.json" {
                let metadata = entry.metadata()?;
                let modified = metadata
                    .modified()?
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs();
                if modified > latest_time {
                    latest_time = modified;
                    latest_file = Some(entry.path().to_path_buf());
                }
            }
        }

        latest_file.ok_or_else(|| BenchmarkError::Config("No results found".to_string()))?
    };

    let content = std::fs::read_to_string(results_file)?;
    let results: BenchmarkResults = serde_json::from_str(&content)?;

    println!("{}", results.summary());

    Ok(())
}
