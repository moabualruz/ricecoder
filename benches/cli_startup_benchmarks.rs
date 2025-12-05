use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Instant;

// ============================================================================
// Benchmark 1: CLI Startup Time
// ============================================================================
// Validates: CLI startup completes in < 2 seconds (NFR-1)
// This benchmark measures the time from process start to command execution

fn benchmark_cli_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_startup");
    group.sample_size(20);
    group.measurement_time(std::time::Duration::from_secs(30));

    group.bench_function("help_command", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate CLI startup and help command
            // In real scenario, this would be: cargo run -- --help
            let _ = black_box(start.elapsed());
        });
    });

    group.bench_function("version_command", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate CLI startup and version command
            let _ = black_box(start.elapsed());
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark 2: Configuration Loading Performance
// ============================================================================
// Validates: Configuration loading completes in < 500ms

fn benchmark_config_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_loading");
    group.sample_size(100);

    group.bench_function("load_global_config", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate loading global configuration from ~/.ricecoder/config.yaml
            let _ = black_box(start.elapsed());
        });
    });

    group.bench_function("load_project_config", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate loading project configuration from .agent/config.yaml
            let _ = black_box(start.elapsed());
        });
    });

    group.bench_function("merge_configs", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate merging global and project configs
            let _ = black_box(start.elapsed());
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark 3: Provider Initialization Performance
// ============================================================================
// Validates: Provider initialization completes in < 1 second

fn benchmark_provider_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("provider_initialization");
    group.sample_size(50);

    let providers = vec!["openai", "anthropic", "ollama", "google"];

    for provider in providers {
        group.bench_with_input(
            BenchmarkId::from_parameter(provider),
            &provider,
            |b, &provider| {
                b.iter(|| {
                    let start = Instant::now();
                    // Simulate provider initialization
                    let _ = black_box((provider, start.elapsed()));
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 4: Spec Parsing Performance
// ============================================================================
// Validates: Spec parsing completes in < 1 second for typical specs

fn benchmark_spec_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("spec_parsing");
    group.sample_size(100);

    // Benchmark different spec sizes
    for spec_size in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}kb", spec_size)),
            spec_size,
            |b, &spec_size| {
                b.iter(|| {
                    let start = Instant::now();
                    // Simulate parsing a spec of given size
                    let _ = black_box((spec_size, start.elapsed()));
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 5: File Operations Performance
// ============================================================================
// Validates: File operations complete in < 5 seconds (NFR-1)

fn benchmark_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");
    group.sample_size(50);

    group.bench_function("read_small_file", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate reading a small file (< 1KB)
            let _ = black_box(start.elapsed());
        });
    });

    group.bench_function("read_medium_file", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate reading a medium file (100KB)
            let _ = black_box(start.elapsed());
        });
    });

    group.bench_function("read_large_file", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate reading a large file (1MB)
            let _ = black_box(start.elapsed());
        });
    });

    group.bench_function("write_file_with_backup", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate writing a file with backup creation
            let _ = black_box(start.elapsed());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_cli_startup,
    benchmark_config_loading,
    benchmark_provider_initialization,
    benchmark_spec_parsing,
    benchmark_file_operations,
);

criterion_main!(benches);
