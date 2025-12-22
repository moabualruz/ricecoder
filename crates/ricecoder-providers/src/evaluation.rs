//! Automated provider evaluation and benchmarking system
//!
//! This module provides automated evaluation of LLM providers using
//! standardized benchmarks and performance metrics.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    error::ProviderError,
    models::{ChatRequest, Message, ModelInfo},
    performance_monitor::ProviderPerformanceMonitor,
    provider::Provider,
};

/// Evaluation result for a single benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub benchmark_name: String,
    /// Score (0.0 to 1.0)
    pub score: f64,
    /// Total tests run
    pub total_tests: usize,
    /// Tests passed
    pub passed_tests: usize,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// Total tokens used
    pub total_tokens: usize,
    /// Cost incurred
    pub cost: f64,
    /// Timestamp
    pub timestamp: SystemTime,
}

/// Comprehensive evaluation result for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEvaluation {
    /// Provider ID
    pub provider_id: String,
    /// Model evaluated
    pub model: String,
    /// Overall score (0.0 to 1.0)
    pub overall_score: f64,
    /// Individual benchmark results
    pub benchmark_results: Vec<BenchmarkResult>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Reliability score
    pub reliability_score: f64,
    /// Cost efficiency score
    pub cost_efficiency_score: f64,
    /// Timestamp
    pub timestamp: SystemTime,
}

/// Performance metrics from evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// 95th percentile response time
    pub p95_response_time_ms: f64,
    /// Requests per second
    pub requests_per_second: f64,
    /// Total tokens processed
    pub total_tokens: usize,
    /// Total cost
    pub total_cost: f64,
}

/// Automated provider evaluator
#[derive(Clone)]
pub struct ProviderEvaluator {
    benchmarks: Vec<Benchmark>,
    performance_monitor: Arc<ProviderPerformanceMonitor>,
}

impl ProviderEvaluator {
    /// Create a new provider evaluator
    pub fn new(performance_monitor: Arc<ProviderPerformanceMonitor>) -> Self {
        Self {
            benchmarks: Self::default_benchmarks(),
            performance_monitor,
        }
    }

    /// Evaluate a provider with all benchmarks
    pub async fn evaluate_provider(
        &self,
        provider: &Arc<dyn Provider>,
        model: &str,
    ) -> Result<ProviderEvaluation, ProviderError> {
        let mut benchmark_results = Vec::new();
        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        for benchmark in &self.benchmarks {
            let result = self.run_benchmark(provider, model, benchmark).await?;
            benchmark_results.push(result.clone());
            total_score += result.score * benchmark.weight;
            total_weight += benchmark.weight;
        }

        let overall_score = if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0
        };

        // Calculate performance metrics
        let performance_metrics = self.calculate_performance_metrics(&benchmark_results);

        // Calculate reliability and cost efficiency scores
        let reliability_score = self.calculate_reliability_score(provider, model).await?;
        let cost_efficiency_score = self.calculate_cost_efficiency_score(&performance_metrics);

        Ok(ProviderEvaluation {
            provider_id: provider.id().to_string(),
            model: model.to_string(),
            overall_score,
            benchmark_results,
            performance_metrics,
            reliability_score,
            cost_efficiency_score,
            timestamp: SystemTime::now(),
        })
    }

    /// Run a single benchmark
    async fn run_benchmark(
        &self,
        provider: &Arc<dyn Provider>,
        model: &str,
        benchmark: &Benchmark,
    ) -> Result<BenchmarkResult, ProviderError> {
        let mut total_response_time = 0.0;
        let mut total_tokens = 0;
        let mut total_cost = 0.0;
        let mut passed_tests = 0;

        for test_case in &benchmark.test_cases {
            let start_time = std::time::Instant::now();

            let request = ChatRequest {
                messages: vec![Message {
                    role: "user".to_string(),
                    content: test_case.prompt.clone(),
                }],
                model: model.to_string(),
                max_tokens: Some(512),
                stream: false,
                temperature: Some(0.7),
            };

            match provider.chat(request).await {
                Ok(response) => {
                    let response_time = start_time.elapsed().as_millis() as f64;
                    total_response_time += response_time;
                    total_tokens += response.usage.total_tokens;
                    total_cost += self.calculate_cost(provider, &response.usage);

                    // Evaluate response quality
                    if self.evaluate_response(&response.content, &test_case.expected_patterns) {
                        passed_tests += 1;
                    }
                }
                Err(_) => {
                    // Failed request
                    total_response_time += 30000.0; // 30 second penalty for failures
                }
            }

            // Small delay between requests to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let score = passed_tests as f64 / benchmark.test_cases.len() as f64;
        let avg_response_time = total_response_time / benchmark.test_cases.len() as f64;

        Ok(BenchmarkResult {
            benchmark_name: benchmark.name.clone(),
            score,
            total_tests: benchmark.test_cases.len(),
            passed_tests,
            avg_response_time_ms: avg_response_time,
            total_tokens,
            cost: total_cost,
            timestamp: SystemTime::now(),
        })
    }

    /// Calculate performance metrics from benchmark results
    fn calculate_performance_metrics(&self, results: &[BenchmarkResult]) -> PerformanceMetrics {
        if results.is_empty() {
            return PerformanceMetrics {
                avg_response_time_ms: 0.0,
                p95_response_time_ms: 0.0,
                requests_per_second: 0.0,
                total_tokens: 0,
                total_cost: 0.0,
            };
        }

        let mut response_times = Vec::new();
        let mut total_tokens = 0;
        let mut total_cost = 0.0;

        for result in results {
            response_times.push(result.avg_response_time_ms);
            total_tokens += result.total_tokens;
            total_cost += result.cost;
        }

        response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let avg_response_time = response_times.iter().sum::<f64>() / response_times.len() as f64;

        let p95_index = (response_times.len() as f64 * 0.95) as usize;
        let p95_response_time = response_times
            .get(p95_index)
            .copied()
            .unwrap_or(avg_response_time);

        let total_time_seconds = response_times.iter().sum::<f64>() / 1000.0;
        let total_requests = results.iter().map(|r| r.total_tests).sum::<usize>();
        let requests_per_second = total_requests as f64 / total_time_seconds.max(1.0);

        PerformanceMetrics {
            avg_response_time_ms: avg_response_time,
            p95_response_time_ms: p95_response_time,
            requests_per_second,
            total_tokens,
            total_cost,
        }
    }

    /// Calculate reliability score for a provider
    async fn calculate_reliability_score(
        &self,
        provider: &Arc<dyn Provider>,
        model: &str,
    ) -> Result<f64, ProviderError> {
        // Simple reliability test: make several requests and measure success rate
        let mut successful_requests = 0;
        let total_requests = 5;

        for i in 0..total_requests {
            let request = ChatRequest {
                messages: vec![Message {
                    role: "user".to_string(),
                    content: format!("Hello, this is test request number {}", i),
                }],
                model: model.to_string(),
                max_tokens: Some(50),
                stream: false,
                temperature: Some(0.1),
            };

            if provider.chat(request).await.is_ok() {
                successful_requests += 1;
            }

            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        Ok(successful_requests as f64 / total_requests as f64)
    }

    /// Calculate cost efficiency score
    fn calculate_cost_efficiency_score(&self, metrics: &PerformanceMetrics) -> f64 {
        if metrics.total_cost == 0.0 {
            return 1.0; // Free is perfectly efficient
        }

        // Score based on cost per token and performance
        let cost_per_token = metrics.total_cost / metrics.total_tokens.max(1) as f64;
        let performance_factor = 1.0 / (1.0 + metrics.avg_response_time_ms / 1000.0);

        // Lower cost_per_token and higher performance = higher score
        let cost_score = 1.0 / (1.0 + cost_per_token * 1000.0); // Normalize cost
        (cost_score + performance_factor) / 2.0
    }

    /// Evaluate if a response matches expected patterns
    fn evaluate_response(&self, response: &str, expected_patterns: &[String]) -> bool {
        if expected_patterns.is_empty() {
            // If no specific patterns expected, just check if response is non-empty
            !response.trim().is_empty()
        } else {
            expected_patterns
                .iter()
                .any(|pattern| response.to_lowercase().contains(&pattern.to_lowercase()))
        }
    }

    /// Calculate cost for token usage
    fn calculate_cost(
        &self,
        provider: &Arc<dyn Provider>,
        usage: &crate::models::TokenUsage,
    ) -> f64 {
        // Get model pricing info
        let models = provider.models();
        if let Some(model) = models.first() {
            if let Some(pricing) = &model.pricing {
                let input_cost =
                    (usage.prompt_tokens as f64 / 1000.0) * pricing.input_per_1k_tokens;
                let output_cost =
                    (usage.completion_tokens as f64 / 1000.0) * pricing.output_per_1k_tokens;
                return input_cost + output_cost;
            }
        }
        0.0
    }

    /// Get default benchmarks
    fn default_benchmarks() -> Vec<Benchmark> {
        vec![
            Benchmark {
                name: "basic_chat".to_string(),
                description: "Basic conversational capabilities".to_string(),
                weight: 1.0,
                test_cases: vec![
                    TestCase {
                        prompt: "Hello, how are you?".to_string(),
                        expected_patterns: vec!["hello".to_string(), "hi".to_string()],
                    },
                    TestCase {
                        prompt: "What is 2 + 2?".to_string(),
                        expected_patterns: vec!["4".to_string(), "four".to_string()],
                    },
                ],
            },
            Benchmark {
                name: "code_generation".to_string(),
                description: "Code generation capabilities".to_string(),
                weight: 1.5,
                test_cases: vec![TestCase {
                    prompt: "Write a Python function to calculate fibonacci numbers".to_string(),
                    expected_patterns: vec!["def".to_string(), "fibonacci".to_string()],
                }],
            },
            Benchmark {
                name: "reasoning".to_string(),
                description: "Logical reasoning capabilities".to_string(),
                weight: 1.2,
                test_cases: vec![TestCase {
                    prompt: "If all cats are mammals and some mammals are pets, are all cats pets?"
                        .to_string(),
                    expected_patterns: vec!["no".to_string(), "not necessarily".to_string()],
                }],
            },
        ]
    }
}

/// Benchmark definition
#[derive(Debug, Clone)]
struct Benchmark {
    name: String,
    description: String,
    weight: f64,
    test_cases: Vec<TestCase>,
}

/// Test case for a benchmark
#[derive(Debug, Clone)]
struct TestCase {
    prompt: String,
    expected_patterns: Vec<String>,
}

/// Continuous evaluation scheduler
#[derive(Clone)]
pub struct ContinuousEvaluator {
    evaluators: HashMap<String, ProviderEvaluator>,
    evaluations: Arc<RwLock<HashMap<String, Vec<ProviderEvaluation>>>>,
    evaluation_interval: Duration,
}

impl ContinuousEvaluator {
    /// Create a new continuous evaluator
    pub fn new(evaluation_interval: Duration) -> Self {
        Self {
            evaluators: HashMap::new(),
            evaluations: Arc::new(RwLock::new(HashMap::new())),
            evaluation_interval,
        }
    }

    /// Add a provider to continuous evaluation
    pub fn add_provider(&mut self, provider_id: &str, evaluator: ProviderEvaluator) {
        self.evaluators.insert(provider_id.to_string(), evaluator);
    }

    /// Start continuous evaluation
    pub async fn start_evaluation(&self) {
        let evaluators = self.evaluators.clone();
        let evaluations = self.evaluations.clone();
        let interval = self.evaluation_interval;

        tokio::spawn(async move {
            loop {
                for (provider_id, evaluator) in &evaluators {
                    // For now, just evaluate the first model of each provider
                    // In practice, you'd want to evaluate all models
                    if let Some(model) = Self::get_provider_models(provider_id).first() {
                        match evaluator
                            .evaluate_provider(&Self::get_provider(provider_id), model)
                            .await
                        {
                            Ok(evaluation) => {
                                let mut evals = evaluations.write().await;
                                evals
                                    .entry(provider_id.to_string())
                                    .or_insert_with(Vec::new)
                                    .push(evaluation);
                            }
                            Err(e) => {
                                eprintln!("Failed to evaluate provider {}: {}", provider_id, e);
                            }
                        }
                    }
                }

                tokio::time::sleep(interval).await;
            }
        });
    }

    /// Get latest evaluation for a provider
    pub async fn get_latest_evaluation(&self, provider_id: &str) -> Option<ProviderEvaluation> {
        let evaluations = self.evaluations.read().await;
        evaluations.get(provider_id)?.last().cloned()
    }

    /// Get evaluation history for a provider
    pub async fn get_evaluation_history(&self, provider_id: &str) -> Vec<ProviderEvaluation> {
        let evaluations = self.evaluations.read().await;
        evaluations.get(provider_id).cloned().unwrap_or_default()
    }

    // Helper methods (would need to be implemented based on your provider registry)
    fn get_provider(_provider_id: &str) -> Arc<dyn Provider> {
        // This would need to be implemented to get the actual provider instance
        unimplemented!("Provider retrieval not implemented")
    }

    fn get_provider_models(_provider_id: &str) -> Vec<String> {
        // This would need to be implemented to get available models
        vec!["gpt-4".to_string()] // Placeholder
    }
}
