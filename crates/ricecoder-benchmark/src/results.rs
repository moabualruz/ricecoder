//! Benchmark results and statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub run_id: String,
    pub model: String,
    pub total_exercises: usize,
    pub completed_exercises: usize,
    pub pass_rates: Vec<f64>, // Pass rate after each attempt (1-indexed)
    pub total_cost: f64,
    pub total_duration: f64,
    pub average_duration_per_exercise: f64,
    pub cost_per_exercise: f64,
    pub error_outputs: usize,
    pub malformed_responses: usize,
    pub syntax_errors: usize,
    pub timeouts: usize,
    pub exercise_results: Vec<ExerciseResult>,
    pub metadata: BenchmarkMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseResult {
    pub exercise_name: String,
    pub language: String,
    pub passed: bool,
    pub attempts: usize,
    pub cost: f64,
    pub duration: f64,
    pub error_output: bool,
    pub malformed_response: bool,
    pub syntax_errors: usize,
    pub timeout: bool,
    pub test_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub commit_hash: String,
    pub version: String,
    pub languages: Vec<String>,
}

impl BenchmarkResults {
    pub fn new(run_id: String, model: String, total_exercises: usize) -> Self {
        Self {
            run_id,
            model,
            total_exercises,
            completed_exercises: 0,
            pass_rates: vec![],
            total_cost: 0.0,
            total_duration: 0.0,
            average_duration_per_exercise: 0.0,
            cost_per_exercise: 0.0,
            error_outputs: 0,
            malformed_responses: 0,
            syntax_errors: 0,
            timeouts: 0,
            exercise_results: vec![],
            metadata: BenchmarkMetadata {
                start_time: chrono::Utc::now(),
                end_time: chrono::Utc::now(),
                commit_hash: String::new(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                languages: vec![],
            },
        }
    }

    pub fn add_exercise_result(&mut self, result: ExerciseResult) {
        self.completed_exercises += 1;
        self.total_cost += result.cost;
        self.total_duration += result.duration;

        if result.error_output {
            self.error_outputs += 1;
        }
        if result.malformed_response {
            self.malformed_responses += 1;
        }
        self.syntax_errors += result.syntax_errors;
        if result.timeout {
            self.timeouts += 1;
        }

        self.exercise_results.push(result);
        self.update_pass_rates();
        self.update_averages();
    }

    pub fn finalize(&mut self) {
        self.metadata.end_time = chrono::Utc::now();
        self.update_pass_rates();
        self.update_averages();
    }

    fn update_pass_rates(&mut self) {
        if self.exercise_results.is_empty() {
            return;
        }

        // Find maximum attempts
        let max_attempts = self.exercise_results.iter()
            .map(|r| r.attempts)
            .max()
            .unwrap_or(0);

        self.pass_rates = vec![];
        for attempt in 1..=max_attempts {
            let passed_count = self.exercise_results.iter()
                .filter(|r| r.passed && r.attempts <= attempt)
                .count();
            let pass_rate = (passed_count as f64 / self.completed_exercises as f64) * 100.0;
            self.pass_rates.push(pass_rate);
        }
    }

    fn update_averages(&mut self) {
        if self.completed_exercises == 0 {
            return;
        }

        self.average_duration_per_exercise = self.total_duration / self.completed_exercises as f64;
        self.cost_per_exercise = self.total_cost / self.completed_exercises as f64;
    }

    pub fn summary(&self) -> String {
        format!(
            "Benchmark Results for {}:\n\
             Total Exercises: {}\n\
             Completed: {}\n\
             Pass Rates: {:.1}% (1st attempt), {:.1}% (2nd attempt)\n\
             Total Cost: ${:.4f}\n\
             Average Duration: {:.1f}s per exercise\n\
             Error Outputs: {}\n\
             Malformed Responses: {}\n\
             Syntax Errors: {}\n\
             Timeouts: {}",
            self.model,
            self.total_exercises,
            self.completed_exercises,
            self.pass_rates.get(0).copied().unwrap_or(0.0),
            self.pass_rates.get(1).copied().unwrap_or(0.0),
            self.total_cost,
            self.average_duration_per_exercise,
            self.error_outputs,
            self.malformed_responses,
            self.syntax_errors,
            self.timeouts
        )
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}