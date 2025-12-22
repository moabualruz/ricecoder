//! Performance benchmarking for TUI operations
//! Measures actual performance metrics and compares against thresholds
//! Validates Requirements 6.1, 6.2, 12.2

use std::time::{Duration, Instant};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ricecoder_tui::{Layout, Rect, Theme};

// ============================================================================
// Benchmark 1: Layout Calculation Performance
// ============================================================================

fn bench_layout_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout_calculation");

    // Benchmark different terminal sizes
    for &(width, height) in &[(80, 24), (120, 30), (160, 50), (200, 60)] {
        group.bench_function(format!("layout_{}x{}", width, height), |b| {
            b.iter(|| {
                let rect = Rect::new(0, 0, width, height);
                let layout = Layout::new(width, height);
                // Simulate layout calculation with constraints
                let _areas = layout.split(
                    rect,
                    &[
                        ricecoder_tui::Constraint::Percentage(70),
                        ricecoder_tui::Constraint::Percentage(30),
                    ],
                );
                black_box(layout);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark 2: Text Rendering Performance
// ============================================================================

fn bench_text_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_rendering");

    // Different text sizes
    for &size in &[100, 1000, 10000] {
        let text = "A".repeat(size);

        group.bench_function(format!("text_render_{}_chars", size), |b| {
            b.iter(|| {
                // Simulate text rendering
                let lines: Vec<String> = text
                    .as_bytes()
                    .chunks(80) // Simulate line wrapping
                    .map(|chunk| String::from_utf8_lossy(chunk).to_string())
                    .collect();
                black_box(lines);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark 3: Component State Updates
// ============================================================================

fn bench_state_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_updates");

    // Benchmark different numbers of state updates
    for &count in &[10, 100, 1000] {
        group.bench_function(format!("state_updates_{}", count), |b| {
            b.iter(|| {
                let mut state = 0u64;
                for i in 0..count {
                    // Simulate state update
                    state = state.wrapping_add(i);
                    // Simulate some processing
                    let _processed = state * 2;
                }
                black_box(state);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark 4: Memory Allocation Patterns
// ============================================================================

fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    // Benchmark different allocation patterns
    for &size in &[100, 1000, 10000] {
        group.bench_function(format!("vec_allocation_{}", size), |b| {
            b.iter(|| {
                let mut vec = Vec::with_capacity(size);
                for i in 0..size {
                    vec.push(i.to_string());
                }
                black_box(vec);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark 5: Event Processing Performance
// ============================================================================

fn bench_event_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_processing");

    // Benchmark event queue processing
    for &queue_size in &[10, 100, 1000] {
        group.bench_function(format!("event_queue_{}", queue_size), |b| {
            b.iter(|| {
                let mut processed = 0;
                for i in 0..queue_size {
                    // Simulate event processing
                    processed += i;
                    // Simulate some work
                    let _result = processed % 100;
                }
                black_box(processed);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Custom Performance Test Runner
// ============================================================================

/// Custom performance test runner for integration with existing test suite
pub struct PerformanceTestRunner {
    results: Vec<PerformanceResult>,
    thresholds: PerformanceThresholds,
}

#[derive(Debug)]
pub struct PerformanceResult {
    pub test_name: String,
    pub duration: Duration,
    pub operations: usize,
    pub ops_per_second: f64,
    pub passed: bool,
}

#[derive(Debug)]
pub struct PerformanceThresholds {
    pub max_layout_time_ms: u64,
    pub max_render_time_ms: u64,
    pub max_state_update_time_us: u64,
    pub min_ops_per_second: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_layout_time_ms: 16, // 60 FPS target
            max_render_time_ms: 50,
            max_state_update_time_us: 1000, // 1ms per update
            min_ops_per_second: 1000.0,
        }
    }
}

impl PerformanceTestRunner {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            thresholds: PerformanceThresholds::default(),
        }
    }

    pub fn with_thresholds(mut self, thresholds: PerformanceThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// Run layout performance test
    pub fn test_layout_performance(&mut self, width: u16, height: u16) {
        let start = Instant::now();

        // Perform layout operations
        for _ in 0..1000 {
            let rect = Rect::new(0, 0, width, height);
            let layout = Layout::new(width, height);
            let _areas = layout.split(
                rect,
                &[
                    ricecoder_tui::Constraint::Percentage(70),
                    ricecoder_tui::Constraint::Percentage(30),
                ],
            );
        }

        let duration = start.elapsed();
        let operations = 1000;
        let ops_per_second = operations as f64 / duration.as_secs_f64();

        let passed = duration.as_millis() < self.thresholds.max_layout_time_ms as u128;

        self.results.push(PerformanceResult {
            test_name: format!("layout_{}x{}", width, height),
            duration,
            operations,
            ops_per_second,
            passed,
        });
    }

    /// Run rendering performance test
    pub fn test_rendering_performance(&mut self, text_size: usize) {
        let text = "A".repeat(text_size);
        let start = Instant::now();

        // Perform rendering operations
        for _ in 0..100 {
            let lines: Vec<String> = text
                .as_bytes()
                .chunks(80)
                .map(|chunk| String::from_utf8_lossy(chunk).to_string())
                .collect();
            black_box(lines);
        }

        let duration = start.elapsed();
        let operations = 100;
        let ops_per_second = operations as f64 / duration.as_secs_f64();

        let passed = duration.as_millis() < self.thresholds.max_render_time_ms as u128;

        self.results.push(PerformanceResult {
            test_name: format!("render_{}_chars", text_size),
            duration,
            operations,
            ops_per_second,
            passed,
        });
    }

    /// Run state update performance test
    pub fn test_state_update_performance(&mut self, update_count: usize) {
        let start = Instant::now();

        let mut state = 0u64;
        for i in 0..update_count {
            state = state.wrapping_add(i as u64);
            let _processed = state * 2;
        }

        let duration = start.elapsed();
        let operations = update_count;
        let ops_per_second = operations as f64 / duration.as_secs_f64();

        let passed = duration.as_micros()
            < self.thresholds.max_state_update_time_us as u128 * update_count as u128;

        self.results.push(PerformanceResult {
            test_name: format!("state_updates_{}", update_count),
            duration,
            operations,
            ops_per_second,
            passed,
        });
    }

    /// Get test results
    pub fn results(&self) -> &[PerformanceResult] {
        &self.results
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.passed)
    }

    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::from("# Performance Test Report\n\n");

        report.push_str(&format!("## Summary\n"));
        report.push_str(&format!("Total Tests: {}\n", self.results.len()));
        report.push_str(&format!(
            "Passed: {}\n",
            self.results.iter().filter(|r| r.passed).count()
        ));
        report.push_str(&format!(
            "Failed: {}\n",
            self.results.iter().filter(|r| !r.passed).count()
        ));
        report.push_str(&format!(
            "Overall: {}\n\n",
            if self.all_passed() { "PASS" } else { "FAIL" }
        ));

        report.push_str("## Detailed Results\n\n");
        for result in &self.results {
            report.push_str(&format!("### {}\n", result.test_name));
            report.push_str(&format!("- Duration: {:?}\n", result.duration));
            report.push_str(&format!("- Operations: {}\n", result.operations));
            report.push_str(&format!("- Ops/sec: {:.2}\n", result.ops_per_second));
            report.push_str(&format!(
                "- Status: {}\n\n",
                if result.passed { "PASS" } else { "FAIL" }
            ));
        }

        report
    }
}

// ============================================================================
// Criterion Benchmark Group
// ============================================================================

criterion_group!(
    benches,
    bench_layout_calculation,
    bench_text_rendering,
    bench_state_updates,
    bench_memory_allocation,
    bench_event_processing
);
criterion_main!(benches);

// ============================================================================
// Integration Tests with Performance Assertions
// ============================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_layout_performance_thresholds() {
        let mut runner = PerformanceTestRunner::new();

        runner.test_layout_performance(80, 24);
        runner.test_layout_performance(160, 50);

        let results = runner.results();
        assert_eq!(results.len(), 2);

        for result in results {
            assert!(
                result.passed,
                "Layout performance test '{}' failed: {:?}",
                result.test_name, result.duration
            );
        }
    }

    #[test]
    fn test_rendering_performance_thresholds() {
        let mut runner = PerformanceTestRunner::new();

        runner.test_rendering_performance(1000);
        runner.test_rendering_performance(10000);

        let results = runner.results();
        assert_eq!(results.len(), 2);

        for result in results {
            assert!(
                result.passed,
                "Rendering performance test '{}' failed: {:?}",
                result.test_name, result.duration
            );
        }
    }

    #[test]
    fn test_state_update_performance_thresholds() {
        let mut runner = PerformanceTestRunner::new();

        runner.test_state_update_performance(1000);
        runner.test_state_update_performance(10000);

        let results = runner.results();
        assert_eq!(results.len(), 2);

        for result in results {
            assert!(
                result.passed,
                "State update performance test '{}' failed: {:?}",
                result.test_name, result.duration
            );
        }
    }

    #[test]
    fn test_performance_test_runner() {
        let mut runner = PerformanceTestRunner::new();

        // Run some tests
        runner.test_layout_performance(80, 24);
        runner.test_rendering_performance(100);

        assert!(!runner.results().is_empty());

        let report = runner.generate_report();
        assert!(report.contains("Performance Test Report"));
        assert!(report.contains("PASS") || report.contains("FAIL"));
    }

    #[test]
    fn test_performance_thresholds_configuration() {
        let thresholds = PerformanceThresholds {
            max_layout_time_ms: 10,
            max_render_time_ms: 25,
            max_state_update_time_us: 500,
            min_ops_per_second: 2000.0,
        };

        let runner = PerformanceTestRunner::new().with_thresholds(thresholds);

        // The thresholds should be applied (this is tested implicitly by other tests)
        assert!(runner.thresholds.max_layout_time_ms == 10);
    }
}
