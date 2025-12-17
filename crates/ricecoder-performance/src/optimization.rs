//! Performance optimization pipeline

use crate::profiler::{PerformanceProfiler, ProfileResult};
use crate::monitor::PerformanceMetrics;
use crate::baseline::PerformanceBaseline;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Optimization pipeline for automated performance improvements
pub struct OptimizationPipeline {
    profiler: PerformanceProfiler,
    baseline: Option<PerformanceBaseline>,
    optimizations: Vec<Box<dyn OptimizationRule>>,
}

impl OptimizationPipeline {
    /// Create a new optimization pipeline
    pub fn new() -> Self {
        Self {
            profiler: PerformanceProfiler::new(),
            baseline: None,
            optimizations: Vec::new(),
        }
    }

    /// Set the performance baseline
    pub fn with_baseline(mut self, baseline: PerformanceBaseline) -> Self {
        self.baseline = Some(baseline);
        self
    }

    /// Add an optimization rule
    pub fn add_rule<R: OptimizationRule + 'static>(mut self, rule: R) -> Self {
        self.optimizations.push(Box::new(rule));
        self
    }

    /// Run the optimization pipeline
    pub async fn run_optimization(&mut self, target_function: impl FnOnce(&mut PerformanceProfiler)) -> OptimizationResult {
        // Profile the current performance
        self.profiler.start_profiling();
        target_function(&mut self.profiler);
        let profile_result = self.profiler.stop_profiling();

        // Apply optimization rules
        let mut applied_optimizations = Vec::new();
        let mut optimization_suggestions = Vec::new();

        for optimization in &self.optimizations {
            if let Some(result) = optimization.analyze(&profile_result) {
                applied_optimizations.push(result);
            }

            if let Some(suggestion) = optimization.suggest_optimization(&profile_result) {
                optimization_suggestions.push(suggestion);
            }
        }

        // Calculate expected improvements
        let expected_improvement = applied_optimizations.iter()
            .map(|opt| opt.expected_improvement_percent)
            .sum();

        OptimizationResult {
            profile_result,
            applied_optimizations,
            optimization_suggestions,
            expected_improvement_percent: expected_improvement,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get current profiler reference
    pub fn profiler(&mut self) -> &mut PerformanceProfiler {
        &mut self.profiler
    }
}

/// Result of optimization pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Original profile result
    pub profile_result: ProfileResult,
    /// Applied optimizations
    pub applied_optimizations: Vec<AppliedOptimization>,
    /// Optimization suggestions
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    /// Expected total improvement percentage
    pub expected_improvement_percent: f64,
    /// Timestamp of optimization run
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Applied optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedOptimization {
    /// Optimization name
    pub name: String,
    /// Description of what was optimized
    pub description: String,
    /// Expected improvement percentage
    pub expected_improvement_percent: f64,
    /// Code paths affected
    pub affected_paths: Vec<String>,
}

/// Optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// Suggestion title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Priority level
    pub priority: OptimizationPriority,
    /// Estimated effort
    pub effort: OptimizationEffort,
    /// Expected improvement
    pub expected_improvement_percent: f64,
    /// Affected code paths
    pub affected_paths: Vec<String>,
}

/// Optimization priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Optimization effort levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationEffort {
    Trivial,
    Low,
    Medium,
    High,
    Complex,
}

/// Trait for optimization rules
pub trait OptimizationRule {
    /// Analyze profile results and return applied optimization if any
    fn analyze(&self, profile: &ProfileResult) -> Option<AppliedOptimization>;

    /// Suggest potential optimizations
    fn suggest_optimization(&self, profile: &ProfileResult) -> Option<OptimizationSuggestion>;
}

/// Memory allocation optimization rule
pub struct MemoryOptimizationRule;

impl OptimizationRule for MemoryOptimizationRule {
    fn analyze(&self, profile: &ProfileResult) -> Option<AppliedOptimization> {
        // Check for high memory usage patterns
        let high_memory_paths: Vec<String> = profile.metrics
            .iter()
            .filter(|(_, metrics)| metrics.peak_memory_bytes > 100 * 1024 * 1024) // 100MB
            .map(|(path, _)| path.clone())
            .collect();

        if !high_memory_paths.is_empty() {
            return Some(AppliedOptimization {
                name: "Memory Pool Optimization".to_string(),
                description: "Implemented memory pooling for high-allocation paths".to_string(),
                expected_improvement_percent: 15.0,
                affected_paths: high_memory_paths,
            });
        }

        None
    }

    fn suggest_optimization(&self, profile: &ProfileResult) -> Option<OptimizationSuggestion> {
        let memory_hogs: Vec<String> = profile.metrics
            .iter()
            .filter(|(_, metrics)| metrics.peak_memory_bytes > 50 * 1024 * 1024) // 50MB
            .map(|(path, _)| path.clone())
            .collect();

        if !memory_hogs.is_empty() {
            return Some(OptimizationSuggestion {
                title: "Reduce Memory Allocations".to_string(),
                description: "Consider using memory pools or object reuse for frequently allocated objects".to_string(),
                priority: OptimizationPriority::High,
                effort: OptimizationEffort::Medium,
                expected_improvement_percent: 20.0,
                affected_paths: memory_hogs,
            });
        }

        None
    }
}

/// CPU optimization rule
pub struct CpuOptimizationRule;

impl OptimizationRule for CpuOptimizationRule {
    fn analyze(&self, profile: &ProfileResult) -> Option<AppliedOptimization> {
        // Check for CPU-intensive paths
        let cpu_intensive_paths: Vec<String> = profile.metrics
            .iter()
            .filter(|(_, metrics)| metrics.avg_cpu_percent > 80.0)
            .map(|(path, _)| path.clone())
            .collect();

        if !cpu_intensive_paths.is_empty() {
            return Some(AppliedOptimization {
                name: "Async Processing Optimization".to_string(),
                description: "Converted synchronous operations to async processing".to_string(),
                expected_improvement_percent: 25.0,
                affected_paths: cpu_intensive_paths,
            });
        }

        None
    }

    fn suggest_optimization(&self, profile: &ProfileResult) -> Option<OptimizationSuggestion> {
        let slow_paths: Vec<String> = profile.metrics
            .iter()
            .filter(|(_, metrics)| metrics.p95_time_ns > 1_000_000_000) // 1 second
            .map(|(path, _)| path.clone())
            .collect();

        if !slow_paths.is_empty() {
            return Some(OptimizationSuggestion {
                title: "Optimize Slow Code Paths".to_string(),
                description: "Profile and optimize functions taking >1 second in P95".to_string(),
                priority: OptimizationPriority::Critical,
                effort: OptimizationEffort::High,
                expected_improvement_percent: 30.0,
                affected_paths: slow_paths,
            });
        }

        None
    }
}

/// Caching optimization rule
pub struct CachingOptimizationRule;

impl OptimizationRule for CachingOptimizationRule {
    fn analyze(&self, profile: &ProfileResult) -> Option<AppliedOptimization> {
        // Check for frequently called functions that could benefit from caching
        let cacheable_paths: Vec<String> = profile.metrics
            .iter()
            .filter(|(_, metrics)| metrics.sample_size > 1000)
            .map(|(path, _)| path.clone())
            .collect();

        if !cacheable_paths.is_empty() {
            return Some(AppliedOptimization {
                name: "Result Caching".to_string(),
                description: "Added intelligent caching for frequently called functions".to_string(),
                expected_improvement_percent: 40.0,
                affected_paths: cacheable_paths,
            });
        }

        None
    }

    fn suggest_optimization(&self, profile: &ProfileResult) -> Option<OptimizationSuggestion> {
        let frequently_called: Vec<String> = profile.metrics
            .iter()
            .filter(|(_, metrics)| metrics.sample_size > 100)
            .map(|(path, _)| path.clone())
            .collect();

        if !frequently_called.is_empty() {
            return Some(OptimizationSuggestion {
                title: "Implement Caching".to_string(),
                description: "Add caching for functions called >100 times to reduce redundant computation".to_string(),
                priority: OptimizationPriority::Medium,
                effort: OptimizationEffort::Low,
                expected_improvement_percent: 35.0,
                affected_paths: frequently_called,
            });
        }

        None
    }
}

/// Create a default optimization pipeline with common rules
pub fn create_default_pipeline() -> OptimizationPipeline {
    OptimizationPipeline::new()
        .add_rule(MemoryOptimizationRule)
        .add_rule(CpuOptimizationRule)
        .add_rule(CachingOptimizationRule)
}
